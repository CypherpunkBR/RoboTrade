# Coleta de Dados de Mercado

Este documento descreve as regras de negócio para coleta e gerenciamento de dados de mercado.

## Visão Geral

O módulo `market_data` é responsável por:
- Coletar dados de preço de múltiplas exchanges
- Coletar indicadores de sentimento (Fear & Greed)
- Normalizar dados de diferentes fontes
- Persistir dados para análise histórica
- Fornecer dados em tempo real para outros módulos

## Fontes de Dados

### Exchanges Suportadas

| Exchange | Tipo | Dados Coletados | Status |
|----------|------|-----------------|--------|
| Binance Futures | CEX | Candles, Ticker, Orderbook | Implementado |
| Binance Spot | CEX | Candles, Ticker | Planejado |
| Kraken Futures | CEX | Candles, Ticker | Planejado |
| Bybit | CEX | Candles, Ticker | Planejado |
| OKX | CEX | Candles, Ticker | Planejado |

### Indicadores de Sentimento

| Fonte | Dados | Frequência |
|-------|-------|------------|
| Alternative.me | Fear & Greed Index | Diário |
| CoinMarketCap | Global Market Cap | Horário |

## Providers

### Interface do Provider

```rust
#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    /// Identificador único do provider
    fn provider_id(&self) -> &str;

    /// Exchange associada
    fn exchange(&self) -> ExchangeId;

    /// Símbolos suportados
    fn supported_symbols(&self) -> Vec<String>;

    /// Timeframes suportados
    fn supported_timeframes(&self) -> Vec<TimeFrame>;

    /// Busca candles históricos
    async fn fetch_candles(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
        limit: usize,
    ) -> MarketDataResult<Vec<Candle>>;

    /// Busca candles em período específico
    async fn fetch_candles_range(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> MarketDataResult<Vec<Candle>>;

    /// Busca ticker atual
    async fn fetch_ticker(&self, symbol: &str) -> MarketDataResult<Ticker>;

    /// Busca múltiplos tickers
    async fn fetch_tickers(&self, symbols: &[String]) -> MarketDataResult<Vec<Ticker>>;

    /// Health check
    async fn health_check(&self) -> MarketDataResult<bool>;
}
```

### Fear & Greed Provider

```rust
#[async_trait]
pub trait FearGreedProvider: Send + Sync {
    fn provider_id(&self) -> &str;

    /// Busca índice atual
    async fn fetch_current(&self) -> MarketDataResult<FearGreedData>;

    /// Busca histórico
    async fn fetch_history(&self, days: u32) -> MarketDataResult<Vec<FearGreedData>>;

    async fn health_check(&self) -> MarketDataResult<bool>;
}
```

## Regras de Coleta

### Frequência de Coleta

| Timeframe | Intervalo de Coleta | Atraso Máximo |
|-----------|---------------------|---------------|
| 1m | 10 segundos | 30 segundos |
| 5m | 30 segundos | 1 minuto |
| 15m | 1 minuto | 2 minutos |
| 1h | 5 minutos | 10 minutos |
| 4h | 15 minutos | 30 minutos |
| 1d | 1 hora | 2 horas |

### Rate Limiting

Cada provider deve respeitar os limites da API:

```rust
pub struct RateLimiter {
    requests_per_minute: u32,
    weight_per_minute: u32,
    current_requests: AtomicU32,
    current_weight: AtomicU32,
    last_reset: RwLock<Instant>,
}

impl RateLimiter {
    pub async fn acquire(&self, weight: u32) -> bool {
        // Reset contadores se passou 1 minuto
        {
            let mut last_reset = self.last_reset.write();
            if last_reset.elapsed() > Duration::from_secs(60) {
                self.current_requests.store(0, Ordering::SeqCst);
                self.current_weight.store(0, Ordering::SeqCst);
                *last_reset = Instant::now();
            }
        }

        // Verificar limites
        let current_requests = self.current_requests.load(Ordering::SeqCst);
        let current_weight = self.current_weight.load(Ordering::SeqCst);

        if current_requests >= self.requests_per_minute {
            return false;
        }

        if current_weight + weight > self.weight_per_minute {
            return false;
        }

        // Incrementar contadores
        self.current_requests.fetch_add(1, Ordering::SeqCst);
        self.current_weight.fetch_add(weight, Ordering::SeqCst);

        true
    }
}
```

### Limites por Exchange

| Exchange | Requests/min | Weight/min |
|----------|-------------|------------|
| Binance Futures | 2400 | 2400 |
| Binance Spot | 1200 | 1200 |
| Kraken | 60 | - |
| Bybit | 120 | - |

### Retry Policy

```rust
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}
```

Erros que permitem retry:
- Timeout
- Connection reset
- Rate limit exceeded (com delay apropriado)
- 5xx errors
- Network errors

Erros que NÃO permitem retry:
- 4xx errors (exceto 429)
- Invalid symbol
- Authentication errors

## Normalização de Dados

### Candle

Todos os candles devem ser normalizados para o formato padrão:

```rust
pub struct Candle {
    pub symbol: String,           // Sempre uppercase: "BTCUSDT"
    pub timeframe: TimeFrame,     // Enum padronizado
    pub open_time: DateTime<Utc>, // Sempre UTC
    pub close_time: DateTime<Utc>,
    pub open: Decimal,            // Sempre Decimal para precisão
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,          // Volume em base asset
    pub quote_volume: Decimal,    // Volume em quote asset
    pub trades_count: u32,
}
```

### Regras de Normalização

1. **Símbolo**: Sempre uppercase, sem separadores
   - "BTC/USDT" → "BTCUSDT"
   - "btc_usdt" → "BTCUSDT"

2. **Timestamps**: Sempre UTC
   - Converter de milliseconds se necessário
   - Validar que open_time < close_time

3. **Preços**: Usar Decimal
   - Converter de string para evitar perda de precisão
   - Validar: open, high, low, close > 0
   - Validar: low <= open, close <= high

4. **Volume**: Nunca negativo
   - Converter para Decimal
   - quote_volume = volume * average_price (aproximado)

### Validação

```rust
impl Candle {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Preços positivos
        if self.open <= Decimal::ZERO
            || self.high <= Decimal::ZERO
            || self.low <= Decimal::ZERO
            || self.close <= Decimal::ZERO
        {
            return Err(ValidationError::InvalidPrice);
        }

        // High é o maior
        if self.high < self.open || self.high < self.close {
            return Err(ValidationError::InvalidHighPrice);
        }

        // Low é o menor
        if self.low > self.open || self.low > self.close {
            return Err(ValidationError::InvalidLowPrice);
        }

        // Timestamps válidos
        if self.open_time >= self.close_time {
            return Err(ValidationError::InvalidTimestamps);
        }

        // Volume não negativo
        if self.volume < Decimal::ZERO || self.quote_volume < Decimal::ZERO {
            return Err(ValidationError::InvalidVolume);
        }

        Ok(())
    }
}
```

## Scheduler de Coleta

### Configuração

```toml
[data_collection]
enabled = true
interval_seconds = 60

# Símbolos a coletar
symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"]

# Timeframes a coletar
timeframes = ["1h", "4h", "1d"]

# Providers habilitados
providers = ["binance_futures", "fear_greed"]

# Backfill automático
backfill_enabled = true
backfill_days = 365
```

### Fluxo de Coleta

```
┌─────────────────────────────────────────────────────────────────┐
│                    DATA COLLECTION SCHEDULER                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Verificar se coleta está habilitada                         │
│     └─ Se não, aguardar próximo ciclo                           │
│                                                                  │
│  2. Para cada provider habilitado:                               │
│     └─ Verificar health                                          │
│     └─ Se unhealthy, logar e pular                              │
│                                                                  │
│  3. Para cada símbolo configurado:                               │
│     └─ Verificar rate limit                                      │
│     └─ Se rate limited, aguardar                                │
│                                                                  │
│  4. Para cada timeframe configurado:                             │
│     └─ Buscar último candle salvo                               │
│     └─ Calcular range a coletar                                 │
│     └─ Fetch candles do provider                                │
│     └─ Normalizar dados                                         │
│     └─ Validar dados                                            │
│     └─ Salvar no banco                                          │
│                                                                  │
│  5. Emitir evento "market-data-updated"                         │
│                                                                  │
│  6. Atualizar métricas                                          │
│                                                                  │
│  7. Aguardar próximo ciclo                                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Implementação

```rust
pub struct DataCollectionScheduler {
    providers: HashMap<String, Box<dyn MarketDataProvider>>,
    fear_greed_provider: Box<dyn FearGreedProvider>,
    candle_repo: Arc<dyn CandleRepository>,
    fear_greed_repo: Arc<dyn FearGreedRepository>,
    config: DataCollectionConfig,
    metrics: Arc<MetricsRegistry>,
    running: AtomicBool,
}

impl DataCollectionScheduler {
    pub async fn start(&self) {
        self.running.store(true, Ordering::SeqCst);

        tracing::info!(
            interval = self.config.interval_seconds,
            symbols = ?self.config.symbols,
            timeframes = ?self.config.timeframes,
            "Data collection scheduler started"
        );

        while self.running.load(Ordering::SeqCst) {
            let cycle_start = Instant::now();

            // Coletar dados de mercado
            for (provider_id, provider) in &self.providers {
                if let Err(e) = self.collect_from_provider(provider_id, provider.as_ref()).await {
                    tracing::error!(
                        provider = provider_id,
                        error = %e,
                        "Collection failed"
                    );
                    self.metrics.market_data().fetch_errors.inc();
                }
            }

            // Coletar Fear & Greed (menos frequente)
            if self.should_collect_fear_greed() {
                if let Err(e) = self.collect_fear_greed().await {
                    tracing::error!(error = %e, "Fear & Greed collection failed");
                }
            }

            // Aguardar próximo ciclo
            let elapsed = cycle_start.elapsed();
            let wait_time = Duration::from_secs(self.config.interval_seconds)
                .saturating_sub(elapsed);

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }

        tracing::info!("Data collection scheduler stopped");
    }

    async fn collect_from_provider(
        &self,
        provider_id: &str,
        provider: &dyn MarketDataProvider,
    ) -> MarketDataResult<()> {
        // Health check
        if !provider.health_check().await? {
            return Err(MarketDataError::ProviderUnavailable {
                provider: provider_id.to_string(),
                reason: "Health check failed".to_string(),
            });
        }

        for symbol in &self.config.symbols {
            for timeframe in &self.config.timeframes {
                let tf = TimeFrame::from_str(timeframe)?;

                // Buscar último candle salvo
                let last_candle = self.candle_repo.get_last_candle(symbol, tf).await?;

                // Calcular quantos candles buscar
                let limit = match &last_candle {
                    Some(c) => self.calculate_missing_candles(&c.close_time, tf),
                    None => 1000, // Backfill inicial
                };

                if limit == 0 {
                    continue; // Dados atualizados
                }

                let start = Instant::now();

                // Fetch candles
                let candles = provider.fetch_candles(symbol, tf, limit).await?;

                self.metrics.market_data().fetch_latency.observe(
                    start.elapsed().as_millis() as f64
                );

                // Filtrar apenas candles novos
                let new_candles: Vec<Candle> = match &last_candle {
                    Some(last) => candles
                        .into_iter()
                        .filter(|c| c.open_time > last.close_time)
                        .collect(),
                    None => candles,
                };

                if new_candles.is_empty() {
                    continue;
                }

                // Validar e salvar
                for candle in &new_candles {
                    candle.validate()?;
                }

                self.candle_repo.save_candles(symbol, tf, &new_candles).await?;

                self.metrics.market_data().candles_processed.add(new_candles.len() as u64);
                self.metrics.market_data().fetch_success.inc();

                tracing::debug!(
                    symbol = symbol,
                    timeframe = %tf,
                    count = new_candles.len(),
                    "Candles saved"
                );
            }
        }

        Ok(())
    }

    fn calculate_missing_candles(&self, last_close: &DateTime<Utc>, timeframe: TimeFrame) -> usize {
        let now = Utc::now();
        let duration_since = now - *last_close;
        let timeframe_duration = timeframe.duration();

        let missing = duration_since.num_seconds() / timeframe_duration.num_seconds();
        missing.max(0) as usize
    }
}
```

## Backfill de Dados Históricos

### Estratégia de Backfill

```rust
pub struct BackfillService {
    provider: Box<dyn MarketDataProvider>,
    candle_repo: Arc<dyn CandleRepository>,
    config: BackfillConfig,
}

pub struct BackfillConfig {
    pub enabled: bool,
    pub days: u32,
    pub batch_size: usize,
    pub delay_between_batches_ms: u64,
}

impl BackfillService {
    pub async fn backfill(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
    ) -> MarketDataResult<BackfillResult> {
        let end = Utc::now();
        let start = end - Duration::days(self.config.days as i64);

        tracing::info!(
            symbol = symbol,
            timeframe = %timeframe,
            start = %start,
            end = %end,
            "Starting backfill"
        );

        let mut total_candles = 0;
        let mut current_end = end;

        while current_end > start {
            let batch_start = (current_end - timeframe.duration() * self.config.batch_size as i32)
                .max(start);

            let candles = self.provider
                .fetch_candles_range(symbol, timeframe, batch_start, current_end)
                .await?;

            if candles.is_empty() {
                break;
            }

            self.candle_repo.save_candles(symbol, timeframe, &candles).await?;

            total_candles += candles.len();
            current_end = batch_start;

            // Rate limiting
            tokio::time::sleep(Duration::from_millis(
                self.config.delay_between_batches_ms
            )).await;

            tracing::debug!(
                symbol = symbol,
                progress = format!("{:.1}%", (1.0 - (current_end - start).num_seconds() as f64 / (end - start).num_seconds() as f64) * 100.0),
                "Backfill progress"
            );
        }

        Ok(BackfillResult {
            symbol: symbol.to_string(),
            timeframe,
            total_candles,
            start_date: start,
            end_date: end,
        })
    }
}
```

## Armazenamento

### Tabelas

Ver [database-schema.md](../architecture/database-schema.md) para detalhes.

### Índices Otimizados

```sql
-- Busca rápida de candles por símbolo e timeframe
CREATE INDEX idx_candles_lookup ON candles(symbol, timeframe, open_time DESC);

-- Busca por período
CREATE INDEX idx_candles_period ON candles(symbol, timeframe, open_time, close_time);

-- Fear & Greed por data
CREATE INDEX idx_fear_greed_date ON fear_greed_index(date DESC);
```

### Retenção de Dados

| Timeframe | Retenção |
|-----------|----------|
| 1m | 7 dias |
| 5m | 30 dias |
| 15m | 90 dias |
| 1h | 365 dias |
| 4h | 730 dias (2 anos) |
| 1d | Permanente |

```rust
pub async fn cleanup_old_data(&self) -> InfraResult<CleanupResult> {
    let mut deleted = 0;

    // 1m - 7 dias
    deleted += self.delete_old_candles(TimeFrame::M1, 7).await?;

    // 5m - 30 dias
    deleted += self.delete_old_candles(TimeFrame::M5, 30).await?;

    // 15m - 90 dias
    deleted += self.delete_old_candles(TimeFrame::M15, 90).await?;

    // 1h - 365 dias
    deleted += self.delete_old_candles(TimeFrame::H1, 365).await?;

    // 4h - 730 dias
    deleted += self.delete_old_candles(TimeFrame::H4, 730).await?;

    // 1d - Nunca deletar

    Ok(CleanupResult { deleted_candles: deleted })
}
```

---

**Próximo**: [Motor de Estratégias](./strategy-engine.md)
