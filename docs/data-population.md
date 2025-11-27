# Workflow de População de Dados

Este documento descreve como buscar e popular dados das exchanges para o banco de dados local SQLite do RoboTrade.

## Tipos de Dados

### 1. Market Data (OHLCV Candles)
- **Fonte**: APIs REST das exchanges (Binance, Kraken)
- **Destino**: Tabela `candles`
- **Frequência**: Histórico (backfill) + Updates periódicos
- **Uso**: Backtesting, estratégias, visualizações

### 2. Trade History (Usuário)
- **Fonte**: APIs autenticadas (Binance `/fapi/v1/userTrades`)
- **Destino**: Tabela `trades`
- **Frequência**: Após cada trade executado
- **Uso**: P&L tracking, estatísticas

### 3. Position History
- **Fonte**: APIs de positions (Binance `/fapi/v2/positionRisk`)
- **Destino**: Tabela `positions`
- **Frequência**: Snapshots periódicos (a cada hora)
- **Uso**: Tracking de posições abertas/fechadas

### 4. Balance & Account
- **Fonte**: APIs de account (Binance `/fapi/v2/balance`)
- **Destino**: Tabela `account_snapshots` (planejado)
- **Frequência**: Diária ou após trades
- **Uso**: Equity curve, capital management

### 5. Fear & Greed Index
- **Fonte**: Alternative.me API
- **Destino**: Tabela `fear_greed_data`
- **Frequência**: Diária
- **Uso**: Estratégia Fear & Greed

---

## Workflow 1: Backfill Historical Candles

### Objetivo
Popular banco com candles históricos para backtesting.

### Implementação

```rust
use robotrade_exchange_gateways::binance::BinanceFuturesClient;
use robotrade_infra::repositories::SqliteCandleRepository;
use chrono::{Utc, Duration};

pub async fn backfill_candles(
    symbol: &str,
    interval: &str,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    pool: SqlitePool,
) -> Result<()> {
    let client = BinanceFuturesClient::new(
        api_key.clone(),
        secret_key.clone(),
        false,
    );
    
    let repository = SqliteCandleRepository::new(pool);
    
    // Binance limita a 1000 candles por request
    let mut current_start = start_date;
    let interval_duration = parse_interval_duration(interval)?;
    let chunk_size = Duration::hours(1000); // 1000 candles de 1h
    
    while current_start < end_date {
        let current_end = (current_start + chunk_size).min(end_date);
        
        info!("Fetching candles {} {} from {} to {}",
            symbol, interval, current_start, current_end
        );
        
        // Buscar candles da exchange
        let candles = client.get_candles(
            symbol,
            interval,
            Some(current_start),
            Some(current_end),
            Some(1000),
        ).await?;
        
        // Salvar no banco
        for candle in candles {
            repository.save(&candle).await?;
        }
        
        info!("Saved {} candles", candles.len());
        
        current_start = current_end;
        
        // Respeitar rate limit
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    Ok(())
}
```

### Uso via CLI

```bash
cargo run --bin backfill -- \
    --symbol BTCUSDT \
    --interval 1h \
    --start 2023-01-01 \
    --end 2024-01-01
```

---

## Workflow 2: Sync Recent Candles

### Objetivo
Manter candles atualizados com dados recentes.

### Implementação

```rust
use tokio::time::{interval, Duration};

pub async fn sync_candles_continuously(
    symbols: Vec<String>,
    interval_str: &str,
    pool: SqlitePool,
) -> Result<()> {
    let client = BinanceFuturesClient::new(api_key, secret_key, false);
    let repository = SqliteCandleRepository::new(pool);
    
    // Sync a cada 1 minuto
    let mut ticker = interval(Duration::from_secs(60));
    
    loop {
        ticker.tick().await;
        
        for symbol in &symbols {
            // Buscar última candle no banco
            let last_candle = repository
                .find_latest(symbol, interval_str)
                .await?;
            
            let start_time = if let Some(candle) = last_candle {
                candle.close_time + Duration::seconds(1)
            } else {
                Utc::now() - Duration::days(1)
            };
            
            // Buscar candles novos
            let new_candles = client.get_candles(
                symbol,
                interval_str,
                Some(start_time),
                None,
                Some(100),
            ).await?;
            
            // Salvar
            for candle in new_candles {
                repository.save(&candle).await?;
            }
            
            info!("Synced {} new candles for {}", 
                new_candles.len(), symbol
            );
        }
    }
}
```

### Uso

```bash
cargo run --bin sync-candles -- \
    --symbols BTCUSDT,ETHUSDT \
    --interval 1h
```

---

## Workflow 3: Fetch User Trades

### Objetivo
Sincronizar trades executados pelo usuário.

### Implementação

```rust
pub async fn sync_user_trades(
    symbol: &str,
    pool: SqlitePool,
) -> Result<()> {
    let client = BinanceFuturesClient::new(api_key, secret_key, false);
    
    // Buscar último trade no banco
    let last_trade_time = sqlx::query!(
        "SELECT MAX(time) as last_time FROM user_trades WHERE symbol = ?",
        symbol
    )
    .fetch_one(&pool)
    .await?
    .last_time;
    
    let start_time = if let Some(time) = last_trade_time {
        DateTime::parse_from_rfc3339(&time)?.with_timezone(&Utc)
    } else {
        Utc::now() - Duration::days(30)
    };
    
    // Buscar trades da API
    let trades = client.get_user_trades(symbol, Some(1000)).await?;
    
    // Filtrar apenas trades novos
    let new_trades: Vec<_> = trades
        .into_iter()
        .filter(|t| t.time > start_time)
        .collect();
    
    // Salvar no banco
    for trade in new_trades {
        sqlx::query!(
            "INSERT INTO user_trades (symbol, order_id, price, qty, commission, time, side, realized_pnl)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            trade.symbol,
            trade.order_id,
            trade.price.to_string(),
            trade.qty.to_string(),
            trade.commission.to_string(),
            trade.time.to_rfc3339(),
            trade.side.to_string(),
            trade.realized_pnl.to_string()
        )
        .execute(&pool)
        .await?;
    }
    
    info!("Synced {} new user trades for {}", new_trades.len(), symbol);
    
    Ok(())
}
```

---

## Workflow 4: Position Snapshots

### Objetivo
Registrar estado de posições periodicamente.

### Implementação

```rust
use tokio_cron_scheduler::{JobScheduler, Job};

pub async fn start_position_snapshot_job(pool: SqlitePool) -> Result<()> {
    let scheduler = JobScheduler::new().await?;
    
    // Executar a cada hora
    scheduler.add(
        Job::new_async("0 0 * * * *", move |_uuid, _lock| {
            let pool = pool.clone();
            Box::pin(async move {
                if let Err(e) = take_position_snapshot(&pool).await {
                    error!("Failed to take position snapshot: {}", e);
                }
            })
        })?
    ).await?;
    
    scheduler.start().await?;
    
    Ok(())
}

async fn take_position_snapshot(pool: &SqlitePool) -> Result<()> {
    let client = BinanceFuturesClient::new(api_key, secret_key, false);
    
    // Buscar posições atuais
    let positions = client.get_positions().await?;
    
    let timestamp = Utc::now();
    
    for position in positions {
        sqlx::query!(
            "INSERT INTO position_snapshots 
             (timestamp, symbol, quantity, entry_price, current_price, unrealized_pnl, leverage)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            timestamp.to_rfc3339(),
            position.symbol,
            position.quantity.to_string(),
            position.entry_price.to_string(),
            position.current_price.to_string(),
            position.unrealized_pnl.to_string(),
            position.leverage
        )
        .execute(pool)
        .await?;
    }
    
    info!("Took snapshot of {} positions", positions.len());
    
    Ok(())
}
```

---

## Workflow 5: Daily Balance Snapshot

### Objetivo
Registrar equity total diariamente para tracking.

### Implementação

```rust
pub async fn daily_balance_snapshot(pool: SqlitePool) -> Result<()> {
    let client = BinanceFuturesClient::new(api_key, secret_key, false);
    
    // Buscar balances
    let balances = client.get_balance().await?;
    let account_info = client.get_account_info().await?;
    
    let total_equity: Decimal = balances
        .iter()
        .map(|b| b.wallet_balance)
        .sum();
    
    let timestamp = Utc::now();
    
    sqlx::query!(
        "INSERT INTO account_snapshots 
         (timestamp, exchange, total_equity, available_balance, margin_used, unrealized_pnl)
         VALUES (?, ?, ?, ?, ?, ?)",
        timestamp.to_rfc3339(),
        "BinanceFutures",
        total_equity.to_string(),
        account_info.available_balance.to_string(),
        account_info.total_margin_balance.to_string(),
        account_info.total_unrealized_profit.to_string()
    )
    .execute(&pool)
    .await?;
    
    info!("Daily balance snapshot: ${}", total_equity);
    
    Ok(())
}
```

---

## Workflow 6: Fear & Greed Index

### Objetivo
Manter Fear & Greed Index atualizado diariamente.

### Implementação

```rust
use robotrade_market_data::providers::AlternativeMeFearGreedProvider;
use robotrade_infra::repositories::SqliteFearGreedRepository;

pub async fn sync_fear_greed_index(pool: SqlitePool) -> Result<()> {
    let provider = AlternativeMeFearGreedProvider::new();
    let repository = SqliteFearGreedRepository::new(pool);
    
    // Buscar último valor no banco
    let last_entry = repository.find_latest().await?;
    
    // Buscar histórico da API
    let history = provider.get_history(Some(30), None).await?;
    
    // Salvar apenas novos valores
    for data in history {
        if let Some(ref last) = last_entry {
            if data.timestamp <= last.timestamp {
                continue;
            }
        }
        
        repository.save(&data).await?;
    }
    
    info!("Synced Fear & Greed Index");
    
    Ok(())
}
```

---

## Scheduled Jobs (Completo)

### Usando tokio-cron-scheduler

```rust
use tokio_cron_scheduler::{JobScheduler, Job};

pub async fn start_all_sync_jobs(pool: SqlitePool) -> Result<()> {
    let scheduler = JobScheduler::new().await?;
    
    // 1. Sync candles a cada 1 minuto
    scheduler.add(
        Job::new_async("0 * * * * *", move |_uuid, _lock| {
            let pool = pool.clone();
            Box::pin(async move {
                sync_recent_candles(&pool).await.ok();
            })
        })?
    ).await?;
    
    // 2. Sync user trades a cada 5 minutos
    scheduler.add(
        Job::new_async("0 */5 * * * *", move |_uuid, _lock| {
            let pool = pool.clone();
            Box::pin(async move {
                sync_all_user_trades(&pool).await.ok();
            })
        })?
    ).await?;
    
    // 3. Position snapshots a cada hora
    scheduler.add(
        Job::new_async("0 0 * * * *", move |_uuid, _lock| {
            let pool = pool.clone();
            Box::pin(async move {
                take_position_snapshot(&pool).await.ok();
            })
        })?
    ).await?;
    
    // 4. Balance snapshot diário (às 00:00 UTC)
    scheduler.add(
        Job::new_async("0 0 0 * * *", move |_uuid, _lock| {
            let pool = pool.clone();
            Box::pin(async move {
                daily_balance_snapshot(pool.clone()).await.ok();
            })
        })?
    ).await?;
    
    // 5. Fear & Greed Index diário (às 12:00 UTC)
    scheduler.add(
        Job::new_async("0 0 12 * * *", move |_uuid, _lock| {
            let pool = pool.clone();
            Box::pin(async move {
                sync_fear_greed_index(pool.clone()).await.ok();
            })
        })?
    ).await?;
    
    scheduler.start().await?;
    
    info!("All sync jobs started successfully");
    
    Ok(())
}
```

---

## Validação de Dados

### Checagem de Integridade

```rust
pub async fn validate_candle_data(pool: &SqlitePool) -> Result<()> {
    // 1. Verificar gaps temporais
    let gaps = sqlx::query!(
        "SELECT symbol, interval, 
                LAG(close_time) OVER (PARTITION BY symbol, interval ORDER BY close_time) as prev_close,
                open_time
         FROM candles
         WHERE open_time - prev_close > interval_duration(interval)"
    )
    .fetch_all(pool)
    .await?;
    
    if !gaps.is_empty() {
        warn!("Found {} gaps in candle data", gaps.len());
    }
    
    // 2. Verificar preços anormais (possíveis erros)
    let anomalies = sqlx::query!(
        "SELECT symbol, open_time, high, low
         FROM candles
         WHERE high / low > 2.0  -- Variação > 100% em um candle
         OR low = 0
         OR high = 0"
    )
    .fetch_all(pool)
    .await?;
    
    if !anomalies.is_empty() {
        warn!("Found {} price anomalies", anomalies.len());
    }
    
    Ok(())
}
```

---

## Comandos CLI Úteis

```bash
# Backfill histórico
cargo run --bin backfill -- \
    --symbol BTCUSDT \
    --interval 1h \
    --start 2023-01-01 \
    --end 2024-01-01

# Sync contínuo
cargo run --bin sync-daemon -- \
    --symbols BTCUSDT,ETHUSDT,BNBUSDT \
    --intervals 1m,5m,1h

# Validar dados
cargo run --bin validate-data

# Estatísticas do banco
cargo run --bin db-stats
```

---

## Próximos Passos

1. ✅ Documentar workflows (este documento)
2. ⏳ Implementar CLIs de backfill e sync
3. ⏳ Adicionar daemon de sync automático
4. ⏳ Implementar validação de dados
5. ⏳ Adicionar retry logic robusto
6. ⏳ Criar dashboard para monitorar sync status

---

## Referências

- [Binance Futures Market Data API](https://binance-docs.github.io/apidocs/futures/en/#market-data-endpoints)
- [crates/infra/src/repositories/](../crates/infra/src/repositories/)
- [docs/exchange-integration.md](./exchange-integration.md)
- [Alternative.me Fear & Greed API](https://alternative.me/crypto/fear-and-greed-index/)
