# Detalhamento dos Módulos

Este documento detalha cada módulo do RoboTrade, suas responsabilidades, estrutura interna e interfaces.

## robotrade-core

### Propósito

Crate fundamental que define o vocabulário do domínio. Contém entidades, traits, erros e DTOs que são utilizados por todos os outros módulos.

### Estrutura

```
crates/core/src/
├── lib.rs              # Re-exports públicos
├── entities/
│   ├── mod.rs          # Re-exports de entidades
│   ├── asset.rs        # Asset, Balance
│   ├── candle.rs       # Candle, TimeFrame
│   ├── order.rs        # Order, OrderType, OrderSide, OrderStatus
│   ├── position.rs     # Position, PositionSide
│   ├── signal.rs       # Signal, SignalType, SignalStatus
│   ├── trade.rs        # Trade, TradeSide
│   └── fear_greed.rs   # FearGreedData, FearGreedClassification
├── traits.rs           # Traits fundamentais
├── error.rs            # Tipos de erro
└── dto/
    └── mod.rs          # DTOs para serialização
```

### Entidades Principais

#### Candle

Representa um candlestick de preço:

```rust
pub struct Candle {
    pub symbol: String,
    pub timeframe: TimeFrame,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub trades_count: u32,
}
```

#### Order

Representa uma ordem de trading:

```rust
pub struct Order {
    pub id: OrderId,
    pub exchange: ExchangeId,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub status: OrderStatus,
    pub filled_quantity: Decimal,
    pub average_price: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Signal

Representa um sinal de trading gerado por uma estratégia:

```rust
pub struct Signal {
    pub id: SignalId,
    pub strategy_id: String,
    pub symbol: String,
    pub signal_type: SignalType,
    pub side: OrderSide,
    pub entry_price: Decimal,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub quantity: Option<Decimal>,
    pub confidence: Decimal,
    pub status: SignalStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
```

### Traits Principais

#### ExchangeGateway

Interface para comunicação com exchanges:

```rust
#[async_trait]
pub trait ExchangeGateway: Send + Sync {
    fn exchange_id(&self) -> ExchangeId;
    fn is_paper_trading(&self) -> bool;

    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order>;
    async fn cancel_order(&self, order_id: &str) -> ExchangeResult<bool>;
    async fn get_order_status(&self, order_id: &str) -> ExchangeResult<Order>;
    async fn get_open_orders(&self, symbol: Option<&str>) -> ExchangeResult<Vec<Order>>;
    async fn get_positions(&self) -> ExchangeResult<Vec<Position>>;
    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>>;
    async fn set_leverage(&self, symbol: &str, leverage: u32) -> ExchangeResult<()>;
    async fn ping(&self) -> ExchangeResult<()>;
}
```

#### Strategy

Interface para estratégias de trading:

```rust
#[async_trait]
pub trait Strategy: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn symbols(&self) -> &[String];
    fn required_timeframes(&self) -> &[TimeFrame];
    fn min_candles_required(&self) -> usize;

    async fn evaluate(&self, context: &StrategyContext) -> Option<Signal>;
    async fn should_close_position(&self, context: &StrategyContext) -> Option<Signal>;
}
```

#### Repository

Interface genérica para repositórios:

```rust
#[async_trait]
pub trait Repository<T, Id>: Send + Sync {
    async fn find_by_id(&self, id: &Id) -> InfraResult<Option<T>>;
    async fn save(&self, entity: &T) -> InfraResult<()>;
    async fn delete(&self, id: &Id) -> InfraResult<bool>;
    async fn find_all(&self) -> InfraResult<Vec<T>>;
}
```

---

## robotrade-infra

### Propósito

Fornece infraestrutura técnica: configuração, banco de dados, logging e implementações de repositórios.

### Estrutura

```
crates/infra/src/
├── lib.rs              # Re-exports públicos
├── config/
│   └── mod.rs          # AppConfig, carregamento TOML
├── database/
│   ├── mod.rs          # Pool, init, migrations
│   └── schema.rs       # Definições de schema
├── logging/
│   └── mod.rs          # Tracing setup
└── repositories/
    ├── mod.rs          # Re-exports
    ├── candle.rs       # SqliteCandleRepository
    └── fear_greed.rs   # SqliteFearGreedRepository
```

### Configuração

Estrutura de configuração carregada de `config.toml`:

```rust
pub struct AppConfig {
    pub general: GeneralConfig,
    pub trading: TradingConfig,
    pub exchanges: HashMap<String, ExchangeConfig>,
    pub data_collection: DataCollectionConfig,
    pub logging: LoggingConfig,
    pub notifications: NotificationConfig,
}

pub struct GeneralConfig {
    pub app_name: String,
    pub environment: Environment,  // Development, Production
    pub data_dir: PathBuf,
}

pub struct TradingConfig {
    pub mode: TradingMode,         // Paper, Live
    pub default_leverage: u32,
    pub max_position_size: Decimal,
    pub risk_per_trade: Decimal,
}

pub struct ExchangeConfig {
    pub enabled: bool,
    pub testnet: bool,
    pub api_key_id: String,        // ID para buscar no keyring
}
```

### Database

Inicialização e pool de conexões:

```rust
pub type DbPool = SqlitePool;

pub async fn init_database(config: &DatabaseConfig) -> InfraResult<DbPool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
```

### Logging

Setup de tracing com múltiplos outputs:

```rust
pub fn init_logging(config: &LoggingConfig) -> LoggingGuard {
    let file_appender = tracing_appender::rolling::daily(&config.dir, "robotrade.log");

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().json().with_writer(file_appender));

    tracing::subscriber::set_global_default(subscriber).unwrap();

    LoggingGuard { _guard: guard }
}
```

---

## robotrade-market-data

### Propósito

Coleta dados de mercado de fontes externas, normaliza e disponibiliza para persistência.

### Estrutura

```
crates/market_data/src/
├── lib.rs              # Re-exports
├── providers/
│   ├── mod.rs          # Registry de providers
│   ├── fear_greed.rs   # AlternativeMe Fear & Greed
│   ├── binance.rs      # Binance market data (planejado)
│   └── mock.rs         # Provider para testes
├── scheduler/
│   └── mod.rs          # Scheduler de coleta
└── normalizer/
    └── mod.rs          # Normalização de dados
```

### Providers

Implementação do Fear & Greed provider:

```rust
pub struct AlternativeMeFearGreedProvider {
    client: reqwest::Client,
    base_url: String,
}

#[async_trait]
impl FearGreedProvider for AlternativeMeFearGreedProvider {
    fn provider_id(&self) -> &str {
        "alternative_me_fear_greed"
    }

    async fn fetch_current(&self) -> MarketDataResult<FearGreedData> {
        let response: ApiResponse = self.client
            .get(&format!("{}/fng/", self.base_url))
            .send()
            .await?
            .json()
            .await?;

        self.normalize(response)
    }

    async fn fetch_history(&self, days: u32) -> MarketDataResult<Vec<FearGreedData>> {
        let response: ApiResponse = self.client
            .get(&format!("{}/fng/?limit={}", self.base_url, days))
            .send()
            .await?
            .json()
            .await?;

        response.data.iter().map(|d| self.normalize(d)).collect()
    }
}
```

### Scheduler

Loop de coleta configurável:

```rust
pub struct DataCollectionScheduler {
    providers: Vec<Box<dyn MarketDataProvider>>,
    repositories: Arc<Repositories>,
    config: DataCollectionConfig,
    running: AtomicBool,
}

impl DataCollectionScheduler {
    pub async fn start(&self) {
        self.running.store(true, Ordering::SeqCst);

        while self.running.load(Ordering::SeqCst) {
            for provider in &self.providers {
                if let Err(e) = self.collect_from(provider).await {
                    tracing::error!(provider = provider.provider_id(), error = %e, "Collection failed");
                }
            }

            tokio::time::sleep(self.config.interval).await;
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
```

---

## robotrade-analytics

### Propósito

Análise de dados de mercado: indicadores técnicos, estratégias, sinais e backtesting.

### Estrutura

```
crates/analytics/src/
├── lib.rs              # Re-exports
├── indicators/
│   ├── mod.rs          # Registry de indicadores
│   ├── sma.rs          # Simple Moving Average
│   ├── ema.rs          # Exponential Moving Average
│   ├── rsi.rs          # Relative Strength Index
│   ├── macd.rs         # MACD
│   ├── bollinger.rs    # Bollinger Bands
│   └── atr.rs          # Average True Range
├── strategies/
│   ├── mod.rs          # Registry de estratégias
│   ├── fear_greed.rs   # Fear & Greed contrarian
│   ├── rsi_oversold.rs # RSI oversold/overbought
│   └── dca.rs          # Dollar Cost Averaging
├── signals/
│   └── mod.rs          # Geração e gestão de sinais
└── backtest/
    ├── mod.rs          # Motor de backtest
    ├── engine.rs       # BacktestEngine
    └── metrics.rs      # Cálculo de métricas
```

### Indicadores

Exemplo de implementação do RSI:

```rust
pub fn calculate_rsi(candles: &[Candle], period: usize) -> Vec<Decimal> {
    if candles.len() < period + 1 {
        return vec![];
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..candles.len() {
        let change = candles[i].close - candles[i - 1].close;
        if change > Decimal::ZERO {
            gains.push(change);
            losses.push(Decimal::ZERO);
        } else {
            gains.push(Decimal::ZERO);
            losses.push(change.abs());
        }
    }

    let mut rsi_values = Vec::new();
    let mut avg_gain = gains[..period].iter().sum::<Decimal>() / Decimal::from(period);
    let mut avg_loss = losses[..period].iter().sum::<Decimal>() / Decimal::from(period);

    for i in period..gains.len() {
        avg_gain = (avg_gain * Decimal::from(period - 1) + gains[i]) / Decimal::from(period);
        avg_loss = (avg_loss * Decimal::from(period - 1) + losses[i]) / Decimal::from(period);

        let rs = if avg_loss == Decimal::ZERO {
            Decimal::from(100)
        } else {
            avg_gain / avg_loss
        };

        let rsi = Decimal::from(100) - (Decimal::from(100) / (Decimal::ONE + rs));
        rsi_values.push(rsi);
    }

    rsi_values
}
```

### Estratégias

Exemplo de estratégia Fear & Greed contrarian:

```rust
pub struct FearGreedStrategy {
    id: String,
    config: FearGreedConfig,
}

pub struct FearGreedConfig {
    pub buy_threshold: u32,      // Compra quando Fear & Greed <= threshold
    pub sell_threshold: u32,     // Vende quando Fear & Greed >= threshold
    pub symbols: Vec<String>,
    pub position_size_pct: Decimal,
}

#[async_trait]
impl Strategy for FearGreedStrategy {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Fear & Greed Contrarian" }

    fn description(&self) -> &str {
        "Compra quando o mercado está em medo extremo, vende em ganância extrema"
    }

    fn symbols(&self) -> &[String] { &self.config.symbols }
    fn required_timeframes(&self) -> &[TimeFrame] { &[TimeFrame::D1] }
    fn min_candles_required(&self) -> usize { 1 }

    async fn evaluate(&self, context: &StrategyContext) -> Option<Signal> {
        let fg = context.fear_greed.as_ref()?;

        if fg.value <= self.config.buy_threshold && context.current_position.is_none() {
            Some(Signal::new_buy(
                &self.id,
                &self.config.symbols[0],
                context.ticker?.last_price,
                self.config.position_size_pct,
            ))
        } else if fg.value >= self.config.sell_threshold && context.current_position.is_some() {
            Some(Signal::new_sell(
                &self.id,
                &self.config.symbols[0],
                context.ticker?.last_price,
            ))
        } else {
            None
        }
    }
}
```

### Backtesting

Motor de backtest:

```rust
pub struct BacktestEngine {
    candle_repo: Arc<dyn CandleRepository>,
    fear_greed_repo: Arc<dyn FearGreedRepository>,
}

impl BacktestEngine {
    pub async fn run(
        &self,
        strategy: &dyn Strategy,
        symbol: &str,
        timeframe: TimeFrame,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        initial_capital: Decimal,
    ) -> AnalyticsResult<BacktestResult> {
        let candles = self.candle_repo
            .find_candles_in_period(symbol, timeframe, start, end)
            .await?;

        let mut capital = initial_capital;
        let mut position: Option<SimulatedPosition> = None;
        let mut trades: Vec<Trade> = Vec::new();
        let mut equity_curve: Vec<Decimal> = Vec::new();

        for (i, candle) in candles.iter().enumerate() {
            let context = self.build_context(&candles[..=i], &position).await?;

            if let Some(signal) = strategy.evaluate(&context).await {
                if let Some(trade) = self.execute_signal(&signal, &mut position, &mut capital, candle) {
                    trades.push(trade);
                }
            }

            equity_curve.push(self.calculate_equity(capital, &position, candle));
        }

        Ok(self.calculate_metrics(initial_capital, capital, &trades, &equity_curve))
    }
}
```

---

## robotrade-exchange-gateways

### Propósito

Comunicação com exchanges via APIs REST e WebSocket.

### Estrutura

```
crates/exchange_gateways/src/
├── lib.rs              # Re-exports
├── binance/
│   ├── mod.rs          # BinanceFuturesClient
│   ├── client.rs       # HTTP client
│   ├── signer.rs       # HMAC-SHA256 signing
│   ├── models.rs       # Request/Response models
│   └── websocket.rs    # WebSocket streams (planejado)
├── paper/
│   └── mod.rs          # Paper trading gateway
└── factory.rs          # Gateway factory
```

### Binance Client

Cliente para Binance Futures:

```rust
pub struct BinanceFuturesClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    signer: BinanceSigner,
    testnet: bool,
}

impl BinanceFuturesClient {
    pub fn new(api_key: String, secret_key: String, testnet: bool) -> Self {
        let base_url = if testnet {
            "https://testnet.binancefuture.com"
        } else {
            "https://fapi.binance.com"
        };

        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            api_key,
            signer: BinanceSigner::new(secret_key),
            testnet,
        }
    }
}

#[async_trait]
impl ExchangeGateway for BinanceFuturesClient {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::BinanceFutures
    }

    fn is_paper_trading(&self) -> bool {
        self.testnet
    }

    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order> {
        let mut params = vec![
            ("symbol", request.symbol.clone()),
            ("side", request.side.to_string()),
            ("type", request.order_type.to_string()),
            ("quantity", request.quantity.to_string()),
        ];

        if let Some(price) = request.price {
            params.push(("price", price.to_string()));
            params.push(("timeInForce", "GTC".to_string()));
        }

        let response = self.signed_post("/fapi/v1/order", &params).await?;
        self.parse_order_response(response)
    }

    // ... outras implementações
}
```

### Signer

Assinatura HMAC-SHA256:

```rust
pub struct BinanceSigner {
    secret_key: String,
}

impl BinanceSigner {
    pub fn sign(&self, message: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    pub fn sign_params(&self, params: &[(String, String)]) -> String {
        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let message = format!("{}&timestamp={}", query, timestamp);
        let signature = self.sign(&message);

        format!("{}&signature={}", message, signature)
    }
}
```

### Paper Trading

Gateway simulado:

```rust
pub struct PaperTradingGateway {
    positions: RwLock<HashMap<String, Position>>,
    orders: RwLock<HashMap<String, Order>>,
    balances: RwLock<HashMap<String, Balance>>,
    order_counter: AtomicU64,
}

#[async_trait]
impl ExchangeGateway for PaperTradingGateway {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Paper
    }

    fn is_paper_trading(&self) -> bool {
        true
    }

    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order> {
        let order_id = self.order_counter.fetch_add(1, Ordering::SeqCst);

        let order = Order {
            id: OrderId::new(format!("PAPER-{}", order_id)),
            exchange: ExchangeId::Paper,
            symbol: request.symbol,
            side: request.side,
            order_type: request.order_type,
            quantity: request.quantity,
            price: request.price,
            status: OrderStatus::Filled, // Paper sempre preenche
            filled_quantity: request.quantity,
            average_price: request.price,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.orders.write().await.insert(order.id.to_string(), order.clone());
        self.update_position(&order).await;

        Ok(order)
    }
}
```

---

## robotrade-trading-worker

### Propósito

Execução assíncrona de operações de trading via sistema de filas.

### Estrutura

```
crates/trading_worker/src/
├── lib.rs              # Re-exports
├── worker.rs           # Worker principal
├── queue.rs            # Sistema de filas
├── jobs/
│   ├── mod.rs          # Enum de jobs
│   ├── place_order.rs  # PlaceOrderJob
│   ├── cancel_order.rs # CancelOrderJob
│   ├── sync_balance.rs # SyncBalancesJob
│   └── health_check.rs # HealthCheckJob
└── executor.rs         # Executor de jobs
```

### Worker

```rust
pub struct TradingWorker<G: ExchangeGateway> {
    gateway: Arc<G>,
    job_repo: Arc<dyn JobRepository>,
    running: AtomicBool,
    config: WorkerConfig,
}

impl<G: ExchangeGateway> TradingWorker<G> {
    pub async fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        tracing::info!("Trading worker started");

        while self.running.load(Ordering::SeqCst) {
            match self.process_next_job().await {
                Ok(Some(job)) => {
                    tracing::info!(job_id = %job.id, "Job completed");
                }
                Ok(None) => {
                    tokio::time::sleep(self.config.poll_interval).await;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Job processing failed");
                }
            }
        }

        tracing::info!("Trading worker stopped");
    }

    async fn process_next_job(&self) -> WorkerResult<Option<Job>> {
        let job = self.job_repo
            .fetch_next_pending(self.config.max_retries)
            .await?;

        if let Some(mut job) = job {
            job.status = JobStatus::Running;
            self.job_repo.update(&job).await?;

            match self.execute(&job).await {
                Ok(_) => {
                    job.status = JobStatus::Completed;
                }
                Err(e) => {
                    job.retries += 1;
                    job.last_error = Some(e.to_string());
                    job.status = if job.retries >= self.config.max_retries {
                        JobStatus::Failed
                    } else {
                        JobStatus::Pending
                    };
                }
            }

            self.job_repo.update(&job).await?;
            return Ok(Some(job));
        }

        Ok(None)
    }
}
```

### Jobs

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobPayload {
    PlaceOrder(PlaceOrderPayload),
    CancelOrder(CancelOrderPayload),
    SyncBalances,
    SyncPositions,
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOrderPayload {
    pub signal_id: SignalId,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub payload: JobPayload,
    pub priority: JobPriority,
    pub status: JobStatus,
    pub retries: u32,
    pub last_error: Option<String>,
    pub scheduled_for: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## robotrade-app (src-tauri)

### Propósito

Aplicação Tauri que integra todos os módulos e expõe comandos para o frontend.

### Estrutura

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── src/
│   ├── main.rs         # Entry point
│   ├── commands/
│   │   ├── mod.rs      # Re-exports
│   │   ├── market.rs   # Comandos de market data
│   │   ├── analytics.rs # Comandos de analytics
│   │   ├── worker.rs   # Comandos do worker
│   │   └── config.rs   # Comandos de configuração
│   ├── state.rs        # Estado da aplicação
│   └── events.rs       # Eventos emitidos
└── icons/              # Ícones do app
```

### State

```rust
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db_pool: DbPool,
    pub candle_repo: Arc<SqliteCandleRepository>,
    pub fear_greed_repo: Arc<SqliteFearGreedRepository>,
    pub gateway: Arc<dyn ExchangeGateway>,
    pub worker: Arc<TradingWorker<dyn ExchangeGateway>>,
    pub scheduler: Arc<DataCollectionScheduler>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self, AppError> {
        let db_pool = init_database(&config.database).await?;

        let candle_repo = Arc::new(SqliteCandleRepository::new(db_pool.clone()));
        let fear_greed_repo = Arc::new(SqliteFearGreedRepository::new(db_pool.clone()));

        let gateway: Arc<dyn ExchangeGateway> = if config.trading.mode == TradingMode::Paper {
            Arc::new(PaperTradingGateway::new())
        } else {
            Arc::new(BinanceFuturesClient::new(/* ... */))
        };

        // ... inicialização de outros componentes

        Ok(Self { /* ... */ })
    }
}
```

### Comandos

```rust
#[tauri::command]
pub async fn fetch_fear_greed(
    state: tauri::State<'_, AppState>,
) -> Result<FearGreedDto, String> {
    let data = state.fear_greed_repo
        .get_latest()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No data available")?;

    Ok(FearGreedDto::from(data))
}

#[tauri::command]
pub async fn run_backtest(
    state: tauri::State<'_, AppState>,
    request: BacktestRequest,
) -> Result<BacktestResultDto, String> {
    let strategy = state.strategy_registry
        .get(&request.strategy_id)
        .ok_or("Strategy not found")?;

    let result = state.backtest_engine
        .run(
            strategy.as_ref(),
            &request.symbol,
            request.timeframe,
            request.start_date,
            request.end_date,
            request.initial_capital,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(BacktestResultDto::from(result))
}

#[tauri::command]
pub async fn enqueue_order(
    state: tauri::State<'_, AppState>,
    request: OrderRequest,
) -> Result<JobId, String> {
    let job_id = state.worker
        .enqueue(JobPayload::PlaceOrder(PlaceOrderPayload::from(request)))
        .await
        .map_err(|e| e.to_string())?;

    Ok(job_id)
}
```

---

**Próximo**: [Fluxo de Dados](./data-flow.md)
