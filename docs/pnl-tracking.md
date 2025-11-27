# Sistema de P&L (Profit & Loss) - Rastreamento e Relatórios

Este documento descreve o sistema de rastreamento de lucros e perdas (P&L) no RoboTrade, incluindo capacidades atuais, limitações e roadmap para funcionalidades futuras.

## Capacidades Atuais ✅

### 1. Rastreamento de Trades Completo

Cada trade armazenado no banco contém:

```sql
CREATE TABLE trades (
    id INTEGER PRIMARY KEY,
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,  -- 'Long' ou 'Short'
    
    -- Entrada
    entry_order_id INTEGER,
    entry_price REAL NOT NULL,
    entry_time TEXT NOT NULL,
    
    -- Saída
    exit_order_id INTEGER,
    exit_price REAL NOT NULL,
    exit_time TEXT NOT NULL,
    
    -- Quantities
    quantity REAL NOT NULL,
    leverage INTEGER DEFAULT 1,
    
    -- P&L
    gross_pnl REAL NOT NULL,      -- Antes das fees
    net_pnl REAL NOT NULL,         -- Depois das fees
    pnl_percentage REAL NOT NULL,  -- % do valor investido
    roi_percentage REAL NOT NULL,  -- % considerando leverage
    
    -- Fees
    entry_fee REAL DEFAULT 0,
    exit_fee REAL DEFAULT 0,
    total_fees REAL DEFAULT 0,
    
    -- Metadata
    close_reason TEXT,  -- StopLoss, TakeProfit, Signal, Manual, etc.
    signal_id INTEGER,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

**Exemplo de Query**:
```rust
use robotrade_core::entities::Trade;
use sqlx::SqlitePool;

// Buscar todos os trades de um símbolo
let trades = sqlx::query_as!(
    Trade,
    "SELECT * FROM trades WHERE symbol = ? ORDER BY exit_time DESC",
    symbol
)
.fetch_all(&pool)
.await?;
```

### 2. TradeStats - Agregação de Métricas

```rust
pub struct TradeStats {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: Decimal,              // % de trades vencedoras
    
    pub gross_profit: Decimal,          // Soma de todos os lucros
    pub gross_loss: Decimal,            // Soma de todas as perdas
    pub net_profit: Decimal,            // Lucro líquido total
    pub profit_factor: Decimal,         // gross_profit / gross_loss
    
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    
    pub max_consecutive_wins: u32,
    pub max_consecutive_losses: u32,
    pub current_streak: i32,
    
    pub total_fees: Decimal,
    pub average_trade_duration: Duration,
}

// Calcular stats de uma lista de trades
let stats = TradeStats::from_trades(&trades);

println!("Win Rate: {:.2}%", stats.win_rate);
println!("Profit Factor: {:.2}", stats.profit_factor);
println!("Net Profit: ${:.2}", stats.net_profit);
```

### 3. Income History (Binance)

A API da Binance fornece histórico detalhado de P&L:

```rust
use robotrade_exchange_gateways::binance::BinanceFuturesClient;

let client = BinanceFuturesClient::new(api_key, secret_key, false);

// Buscar income history (últimos 30 dias)
let income = client.get_income_history(
    Some("BTCUSDT"),
    Some(start_time),
    Some(end_time),
    Some(1000),
).await?;

// Tipos de income:
// - REALIZED_PNL: P&L realizado em fechamento de posição
// - FUNDING_FEE: Taxas de funding
// - COMMISSION: Comissões de trading
// - TRANSFER: Transferências
for item in income {
    println!("{}: {} {}", 
        item.income_type, 
        item.income, 
        item.asset
    );
}
```

### 4. Realized P&L Aggregation (Binance)

```rust
// Calcular P&L total realizado por símbolo
let pnl_summary = client.get_all_realized_pnl().await?;

for (symbol, pnl) in pnl_summary {
    println!("{}: ${:.2}", symbol, pnl);
}
```

### 5. Cálculo em Posições Abertas

```rust
pub struct Position {
    pub symbol: String,
    pub quantity: Decimal,
    pub entry_price: Decimal,
    pub current_price: Decimal,
    pub leverage: i32,
    
    // P&L não realizado (posição ainda aberta)
    pub unrealized_pnl: Decimal,
    pub unrealized_pnl_pct: Decimal,
    
    // P&L realizado (closures parciais)
    pub realized_pnl: Decimal,
    
    // ROI considerando leverage
    pub roi_pct: Decimal,
}

impl Position {
    pub fn calculate_unrealized_pnl(&self) -> Decimal {
        let price_diff = if self.side == PositionSide::Long {
            self.current_price - self.entry_price
        } else {
            self.entry_price - self.current_price
        };
        
        price_diff * self.quantity
    }
}
```

---

## Limitações Atuais ⚠️

### 1. ❌ Sem Agregação Mensal Automática

**Problema**: Não existe tabela de agregação por mês/moeda.

**Impacto**: Para gerar relatório mensal, precisa:
```sql
-- Query manual (lenta se muitos trades)
SELECT 
    strftime('%Y-%m', exit_time) as month,
    SUM(net_pnl) as total_pnl,
    COUNT(*) as trades_count
FROM trades
WHERE symbol = 'BTCUSDT'
GROUP BY month
ORDER BY month DESC;
```

**Solução Planejada**: Criar tabela `monthly_pnl`.

### 2. ❌ Sem Rastreamento por Moeda/Asset

**Problema**: P&L é calculado em quote asset (USDT), mas não separado por base asset.

**Impacto**: Difícil responder "quanto lucrei com BTC vs ETH?"

**Workaround Atual**:
```sql
SELECT 
    SUBSTR(symbol, 1, LENGTH(symbol) - 4) as base_asset,  -- Remove 'USDT'
    SUM(net_pnl) as total_pnl
FROM trades
WHERE symbol LIKE '%USDT'
GROUP BY base_asset;
```

**Solução Planejada**: Adicionar campo `base_asset` e `quote_asset` em trades.

### 3. ❌ Sem Account-Level Snapshots

**Problema**: Não há registro histórico de equity total da conta.

**Impacto**: Não dá para plotar curva de equity ao longo do tempo.

**Solução Planejada**: Criar tabela `account_snapshots`.

### 4. ❌ Repositories Não Implementados

**Problema**: Traits definidos mas sem implementação SQLite.

**Afetado**:
- `TradeRepository` ❌
- `PositionRepository` ❌
- `OrderRepository` ❌

**Impacto**: Queries manuais com sqlx ao invés de interface limpa.

**Exemplo do que DEVERIA existir**:
```rust
// Isso ainda não funciona! (TODO)
let trade_repo = SqliteTradeRepository::new(pool.clone());

// Buscar trades por mês
let trades = trade_repo.find_by_month(2024, 11).await?;

// Buscar P&L por moeda
let btc_pnl = trade_repo.aggregate_pnl_by_asset("BTC", start, end).await?;
```

---

## Queries Úteis (Enquanto Repositories Não Existem)

### P&L Total por Símbolo
```sql
SELECT 
    symbol,
    COUNT(*) as trades,
    SUM(net_pnl) as total_pnl,
    AVG(net_pnl) as avg_pnl,
    SUM(CASE WHEN net_pnl > 0 THEN 1 ELSE 0 END) as wins,
    SUM(CASE WHEN net_pnl < 0 THEN 1 ELSE 0 END) as losses
FROM trades
GROUP BY symbol
ORDER BY total_pnl DESC;
```

### P&L Mensal
```sql
SELECT 
    strftime('%Y-%m', exit_time) as month,
    COUNT(*) as trades,
    SUM(net_pnl) as pnl,
    SUM(total_fees) as fees
FROM trades
GROUP BY month
ORDER BY month DESC;
```

### Top 10 Melhores Trades
```sql
SELECT 
    symbol,
    entry_time,
    exit_time,
    net_pnl,
    pnl_percentage,
    close_reason
FROM trades
ORDER BY net_pnl DESC
LIMIT 10;
```

### P&L por Exchange
```sql
SELECT 
    exchange,
    COUNT(*) as trades,
    SUM(net_pnl) as total_pnl
FROM trades
GROUP BY exchange;
```

---

## Roadmap de Implementação

### Fase 1: Monthly P&L Aggregation ⏳

**Criar tabela**:
```sql
CREATE TABLE monthly_pnl (
    id INTEGER PRIMARY KEY,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL,
    symbol TEXT NOT NULL,
    base_asset TEXT NOT NULL,
    quote_asset TEXT NOT NULL,
    
    total_trades INTEGER DEFAULT 0,
    winning_trades INTEGER DEFAULT 0,
    losing_trades INTEGER DEFAULT 0,
    
    gross_profit REAL DEFAULT 0,
    gross_loss REAL DEFAULT 0,
    net_pnl REAL DEFAULT 0,
    total_fees REAL DEFAULT 0,
    
    win_rate REAL DEFAULT 0,
    profit_factor REAL DEFAULT 0,
    
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(year, month, symbol)
);
```

**Trigger para atualizar automaticamente**:
```sql
CREATE TRIGGER update_monthly_pnl AFTER INSERT ON trades
BEGIN
    INSERT INTO monthly_pnl (year, month, symbol, base_asset, quote_asset, ...)
    VALUES (
        strftime('%Y', NEW.exit_time),
        strftime('%m', NEW.exit_time),
        NEW.symbol,
        ...
    )
    ON CONFLICT(year, month, symbol) DO UPDATE SET
        total_trades = total_trades + 1,
        net_pnl = net_pnl + NEW.net_pnl,
        ...;
END;
```

### Fase 2: TradeRepository Implementation ⏳

```rust
#[async_trait]
pub trait TradeRepository: Repository<Trade, i64> {
    async fn find_by_symbol(&self, symbol: &str) -> Result<Vec<Trade>>;
    async fn find_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Trade>>;
    async fn find_by_month(&self, year: i32, month: u32) -> Result<Vec<Trade>>;
    async fn aggregate_pnl_by_symbol(&self, symbol: &str) -> Result<TradeStats>;
    async fn aggregate_pnl_by_month(&self, year: i32, month: u32) -> Result<HashMap<String, Decimal>>;
}

pub struct SqliteTradeRepository {
    pool: SqlitePool,
}

#[async_trait]
impl TradeRepository for SqliteTradeRepository {
    async fn find_by_month(&self, year: i32, month: u32) -> Result<Vec<Trade>> {
        let trades = sqlx::query_as!(
            Trade,
            "SELECT * FROM trades 
             WHERE strftime('%Y', exit_time) = ? 
             AND strftime('%m', exit_time) = ?
             ORDER BY exit_time DESC",
            year.to_string(),
            format!("{:02}", month)
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(trades)
    }
    
    // ... outros métodos
}
```

### Fase 3: Account Snapshots ⏳

```sql
CREATE TABLE account_snapshots (
    id INTEGER PRIMARY KEY,
    timestamp TEXT NOT NULL,
    exchange TEXT NOT NULL,
    
    total_equity REAL NOT NULL,
    available_balance REAL NOT NULL,
    margin_used REAL NOT NULL,
    unrealized_pnl REAL NOT NULL,
    
    -- Breakdown por asset
    balances_json TEXT,  -- JSON: [{"asset": "USDT", "free": 1000, ...}]
    positions_json TEXT, -- JSON: [{"symbol": "BTCUSDT", "pnl": 100, ...}]
    
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_snapshots_timestamp ON account_snapshots(timestamp);
CREATE INDEX idx_snapshots_exchange ON account_snapshots(exchange);
```

### Fase 4: P&L por Moeda ⏳

**Adicionar campos em trades**:
```sql
ALTER TABLE trades ADD COLUMN base_asset TEXT;
ALTER TABLE trades ADD COLUMN quote_asset TEXT;

-- Popular retroativamente
UPDATE trades SET 
    base_asset = SUBSTR(symbol, 1, LENGTH(symbol) - 4),
    quote_asset = 'USDT'
WHERE symbol LIKE '%USDT';
```

---

## Como Usar (Atualmente)

### Calcular P&L Mensal Manual

```rust
use sqlx::SqlitePool;
use chrono::{Utc, Datelike};

pub async fn calculate_monthly_pnl(
    pool: &SqlitePool,
    year: i32,
    month: u32,
) -> Result<HashMap<String, Decimal>> {
    let start = format!("{}-{:02}-01", year, month);
    let end = if month == 12 {
        format!("{}-01-01", year + 1)
    } else {
        format!("{}-{:02}-01", year, month + 1)
    };
    
    let rows = sqlx::query!(
        "SELECT symbol, SUM(net_pnl) as total_pnl
         FROM trades
         WHERE exit_time >= ? AND exit_time < ?
         GROUP BY symbol",
        start,
        end
    )
    .fetch_all(pool)
    .await?;
    
    let mut pnl_map = HashMap::new();
    for row in rows {
        pnl_map.insert(
            row.symbol,
            Decimal::from_str(&row.total_pnl.unwrap().to_string())?
        );
    }
    
    Ok(pnl_map)
}
```

### Gerar Relatório de P&L

```rust
pub async fn generate_pnl_report(pool: &SqlitePool) -> Result<String> {
    let now = Utc::now();
    let current_month_pnl = calculate_monthly_pnl(
        pool,
        now.year(),
        now.month(),
    ).await?;
    
    let mut report = String::new();
    report.push_str(&format!("📊 P&L Report - {}/{}\n\n", now.month(), now.year()));
    
    for (symbol, pnl) in current_month_pnl {
        report.push_str(&format!("{}: ${:.2}\n", symbol, pnl));
    }
    
    Ok(report)
}
```

---

## Próximos Passos

1. ✅ **Documentar estado atual** (este documento)
2. ⏳ **Implementar monthly_pnl table** com triggers
3. ⏳ **Implementar TradeRepository, PositionRepository, OrderRepository**
4. ⏳ **Adicionar account_snapshots para equity curve**
5. ⏳ **Adicionar base_asset/quote_asset separation**
6. ⏳ **Criar dashboard/UI para visualizar P&L**

---

## Referências

- [crates/core/src/entities/trade.rs](../crates/core/src/entities/trade.rs)
- [crates/infra/src/database/schema.rs](../crates/infra/src/database/schema.rs)
- [docs/architecture/database-schema.md](./architecture/database-schema.md)
- [Binance Income History API](https://binance-docs.github.io/apidocs/futures/en/#get-income-history-user_data)
