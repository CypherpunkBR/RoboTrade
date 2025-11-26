# Fluxo de Dados

Este documento descreve como os dados fluem através do sistema RoboTrade, desde a coleta até a execução de ordens.

## Visão Geral dos Fluxos

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FONTES EXTERNAS                                    │
│                                                                              │
│   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐               │
│   │ Binance  │   │   OKX    │   │  Bybit   │   │Fear&Greed│               │
│   │  API     │   │   API    │   │   API    │   │   API    │               │
│   └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘               │
│        │              │              │              │                       │
└────────┼──────────────┼──────────────┼──────────────┼───────────────────────┘
         │              │              │              │
         └──────────────┴──────────────┴──────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          MARKET DATA MODULE                                  │
│                                                                              │
│   ┌────────────────────────────────────────────────────────────────────┐   │
│   │                        Scheduler (Tokio)                            │   │
│   │                                                                      │   │
│   │   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐            │   │
│   │   │   Fetch     │───►│  Normalize  │───►│   Persist   │            │   │
│   │   │   Data      │    │    Data     │    │    Data     │            │   │
│   │   └─────────────┘    └─────────────┘    └─────────────┘            │   │
│   │                                                                      │   │
│   └────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                             SQLite                                           │
│                                                                              │
│   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐               │
│   │ candles  │   │ symbols  │   │fear_greed│   │  market  │               │
│   │          │   │          │   │  _index  │   │ _stats   │               │
│   └──────────┘   └──────────┘   └──────────┘   └──────────┘               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ANALYTICS MODULE                                     │
│                                                                              │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                 │
│   │  Indicators  │───►│  Strategies  │───►│   Signals    │                 │
│   │  Calculation │    │  Evaluation  │    │  Generation  │                 │
│   └──────────────┘    └──────────────┘    └──────────────┘                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TRADING WORKER                                       │
│                                                                              │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                 │
│   │   Signal     │───►│    Job       │───►│   Executor   │                 │
│   │   Handler    │    │    Queue     │    │              │                 │
│   └──────────────┘    └──────────────┘    └──────────────┘                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       EXCHANGE GATEWAYS                                      │
│                                                                              │
│   ┌──────────────────────────────────────────────────────────────────────┐ │
│   │                    ExchangeGateway Trait                              │ │
│   │                                                                        │ │
│   │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                │ │
│   │  │   Binance    │  │    Paper     │  │   (Kraken)   │                │ │
│   │  │   Futures    │  │   Trading    │  │   Futures    │                │ │
│   │  └──────────────┘  └──────────────┘  └──────────────┘                │ │
│   │                                                                        │ │
│   └──────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Fluxo 1: Coleta de Dados de Mercado

### Descrição

O sistema coleta periodicamente dados de mercado de múltiplas fontes, normaliza e persiste no banco de dados.

### Sequência Detalhada

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│Scheduler │     │ Provider │     │Normalizer│     │Repository│     │ SQLite   │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │                │
     │ tick()         │                │                │                │
     │───────────────►│                │                │                │
     │                │                │                │                │
     │                │ fetch_candles()│                │                │
     │                │───────────────►│                │                │
     │                │                │                │                │
     │                │◄───────────────│                │                │
     │                │  raw_candles   │                │                │
     │                │                │                │                │
     │                │                │ normalize()    │                │
     │                │                │───────────────►│                │
     │                │                │                │                │
     │                │                │◄───────────────│                │
     │                │                │  Candle[]      │                │
     │                │                │                │                │
     │                │                │                │ save_candles() │
     │                │                │                │───────────────►│
     │                │                │                │                │
     │                │                │                │◄───────────────│
     │                │                │                │      Ok        │
     │                │                │                │                │
     │ emit("market-data-updated")     │                │                │
     │─────────────────────────────────────────────────────────────────►│
     │                │                │                │                │
```

### Dados Envolvidos

**Input** (API Externa):
```json
{
  "symbol": "BTCUSDT",
  "interval": "1h",
  "openTime": 1703548800000,
  "open": "42150.50",
  "high": "42300.00",
  "low": "42100.00",
  "close": "42250.00",
  "volume": "1234.567",
  "closeTime": 1703552399999,
  "quoteAssetVolume": "52000000.00",
  "numberOfTrades": 15000
}
```

**Output** (Candle normalizado):
```rust
Candle {
    symbol: "BTCUSDT",
    timeframe: TimeFrame::H1,
    open_time: "2023-12-26T00:00:00Z",
    close_time: "2023-12-26T00:59:59Z",
    open: Decimal::from_str("42150.50"),
    high: Decimal::from_str("42300.00"),
    low: Decimal::from_str("42100.00"),
    close: Decimal::from_str("42250.00"),
    volume: Decimal::from_str("1234.567"),
    quote_volume: Decimal::from_str("52000000.00"),
    trades_count: 15000,
}
```

### Configuração

```toml
[data_collection]
enabled = true
interval_seconds = 60
symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"]
timeframes = ["1m", "5m", "15m", "1h", "4h", "1d"]
max_retries = 3
retry_delay_ms = 1000
```

---

## Fluxo 2: Cálculo de Indicadores

### Descrição

Após novos dados serem persistidos, o sistema pode calcular indicadores técnicos sob demanda ou automaticamente.

### Sequência

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│Analytics │     │CandelRepo│     │Indicators│     │IndicRepo │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ calculate_indicators(symbol, timeframe, indicators)
     │───────────────►│                │                │
     │                │                │                │
     │                │ find_candles() │                │
     │                │───────────────►│                │
     │                │                │                │
     │                │◄───────────────│                │
     │                │   Candle[]     │                │
     │                │                │                │
     │                │                │ calculate()    │
     │                │                │───────────────►│
     │                │                │                │
     │                │                │◄───────────────│
     │                │                │  IndicatorValue[]
     │                │                │                │
     │                │                │                │ save()
     │                │                │                │──────►│
     │                │                │                │       │
     │◄───────────────────────────────────────────────────────│
     │                         Ok                       │
```

### Indicadores Suportados

| Indicador | Parâmetros | Descrição |
|-----------|------------|-----------|
| SMA | period | Simple Moving Average |
| EMA | period | Exponential Moving Average |
| RSI | period | Relative Strength Index |
| MACD | fast, slow, signal | Moving Average Convergence Divergence |
| BB | period, std_dev | Bollinger Bands |
| ATR | period | Average True Range |

### Estrutura de Saída

```rust
pub struct IndicatorValue {
    pub symbol: String,
    pub timeframe: TimeFrame,
    pub indicator: IndicatorType,
    pub timestamp: DateTime<Utc>,
    pub value: Decimal,
    pub metadata: Option<serde_json::Value>,
}

// Exemplo para Bollinger Bands (metadata)
{
    "upper": "43500.00",
    "middle": "42500.00",
    "lower": "41500.00"
}
```

---

## Fluxo 3: Avaliação de Estratégia e Geração de Sinais

### Descrição

Estratégias avaliam o contexto de mercado e geram sinais de entrada/saída.

### Sequência

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ Strategy │     │ Context  │     │  Signal  │     │SignalRepo│     │  Event   │
│ Engine   │     │ Builder  │     │Generator │     │          │     │ Emitter  │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │                │
     │ evaluate_all() │                │                │                │
     │───────────────►│                │                │                │
     │                │                │                │                │
     │                │ build_context()│                │                │
     │                │───────────────►│                │                │
     │                │                │                │                │
     │                │◄───────────────│                │                │
     │                │  StrategyContext                │                │
     │                │                │                │                │
     │ strategy.evaluate(context)      │                │                │
     │────────────────────────────────►│                │                │
     │                │                │                │                │
     │◄────────────────────────────────│                │                │
     │            Option<Signal>       │                │                │
     │                │                │                │                │
     │ [if signal]    │                │                │                │
     │                │                │ save(signal)   │                │
     │                │                │───────────────►│                │
     │                │                │                │                │
     │                │                │                │ emit("signal") │
     │                │                │                │───────────────►│
     │                │                │                │                │
```

### Contexto de Estratégia

```rust
pub struct StrategyContext {
    /// Candles recentes (mais antigo primeiro)
    pub candles: Vec<Candle>,

    /// Ticker atual do símbolo
    pub ticker: Option<Ticker>,

    /// Posição atual (se houver)
    pub current_position: Option<Position>,

    /// Dados de Fear & Greed
    pub fear_greed: Option<FearGreedData>,

    /// Indicadores pré-calculados
    pub indicators: HashMap<IndicatorType, Vec<Decimal>>,

    /// Timestamp da avaliação
    pub timestamp: DateTime<Utc>,
}
```

### Estrutura do Sinal

```rust
pub struct Signal {
    pub id: SignalId,
    pub strategy_id: String,
    pub strategy_version: String,
    pub symbol: String,
    pub signal_type: SignalType,      // Entry, Exit, Adjustment
    pub side: OrderSide,               // Buy, Sell
    pub entry_price: Decimal,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub quantity: Option<Decimal>,
    pub risk_reward_ratio: Option<Decimal>,
    pub confidence: Decimal,           // 0.0 - 1.0
    pub status: SignalStatus,          // Pending, Executed, Expired, Cancelled
    pub reason: String,                // Explicação do sinal
    pub metadata: serde_json::Value,   // Dados extras
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
```

---

## Fluxo 4: Execução de Ordem via Worker

### Descrição

Sinais aprovados são transformados em jobs que são executados pelo worker.

### Sequência

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  Signal  │     │  Worker  │     │  Queue   │     │ Executor │     │ Gateway  │
│ Handler  │     │          │     │          │     │          │     │          │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │                │
     │ handle_signal()│                │                │                │
     │───────────────►│                │                │                │
     │                │                │                │                │
     │                │ create_job()   │                │                │
     │                │───────────────►│                │                │
     │                │                │                │                │
     │                │◄───────────────│                │                │
     │                │    JobId       │                │                │
     │                │                │                │                │
     │ [worker loop]  │                │                │                │
     │                │ fetch_next()   │                │                │
     │                │───────────────►│                │                │
     │                │                │                │                │
     │                │◄───────────────│                │                │
     │                │     Job        │                │                │
     │                │                │                │                │
     │                │                │ execute(job)   │                │
     │                │                │───────────────►│                │
     │                │                │                │                │
     │                │                │                │ submit_order() │
     │                │                │                │───────────────►│
     │                │                │                │                │
     │                │                │                │◄───────────────│
     │                │                │                │     Order      │
     │                │                │                │                │
     │                │                │◄───────────────│                │
     │                │                │ ExecutionResult│                │
     │                │                │                │                │
     │                │ update_job()   │                │                │
     │                │───────────────►│                │                │
     │                │                │                │                │
```

### Estrutura do Job

```rust
pub struct Job {
    pub id: JobId,
    pub payload: JobPayload,
    pub priority: JobPriority,
    pub status: JobStatus,
    pub retries: u32,
    pub max_retries: u32,
    pub last_error: Option<String>,
    pub scheduled_for: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum JobPriority {
    Critical = 0,   // Ordens de stop loss
    High = 1,       // Ordens normais
    Medium = 2,     // Sincronização
    Low = 3,        // Health checks
}

pub enum JobStatus {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

### Payload de Ordem

```rust
pub struct PlaceOrderPayload {
    pub signal_id: SignalId,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_loss: Option<StopLossConfig>,
    pub take_profit: Option<TakeProfitConfig>,
    pub reduce_only: bool,
    pub client_order_id: Option<String>,
}

pub struct StopLossConfig {
    pub price: Decimal,
    pub order_type: OrderType,  // StopMarket, StopLimit
}
```

---

## Fluxo 5: Backtest

### Descrição

Simula a execução de uma estratégia usando dados históricos.

### Sequência

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ Backtest │     │CandelRepo│     │ Strategy │     │Simulator │     │ Metrics  │
│ Engine   │     │          │     │          │     │          │     │Calculator│
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │                │
     │ run(params)    │                │                │                │
     │───────────────►│                │                │                │
     │                │                │                │                │
     │                │ find_candles() │                │                │
     │                │───────────────►│                │                │
     │                │                │                │                │
     │                │◄───────────────│                │                │
     │                │  candles[]     │                │                │
     │                │                │                │                │
     │ [for each candle]               │                │                │
     │                │                │                │                │
     │                │                │ evaluate()     │                │
     │                │                │───────────────►│                │
     │                │                │                │                │
     │                │                │◄───────────────│                │
     │                │                │ Option<Signal> │                │
     │                │                │                │                │
     │                │                │                │ simulate_trade()
     │                │                │                │───────────────►│
     │                │                │                │                │
     │                │                │                │◄───────────────│
     │                │                │                │   TradeResult  │
     │                │                │                │                │
     │ [end loop]     │                │                │                │
     │                │                │                │                │
     │                │                │                │ calculate()    │
     │                │                │                │───────────────►│
     │                │                │                │                │
     │◄───────────────────────────────────────────────────────────────────│
     │                         BacktestResult                             │
```

### Parâmetros do Backtest

```rust
pub struct BacktestParams {
    pub strategy_id: String,
    pub strategy_config: serde_json::Value,
    pub symbol: String,
    pub timeframe: TimeFrame,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub initial_capital: Decimal,
    pub position_size_pct: Decimal,    // % do capital por trade
    pub commission_rate: Decimal,      // Taxa por trade
    pub slippage_rate: Decimal,        // Slippage simulado
    pub use_leverage: bool,
    pub max_leverage: u32,
}
```

### Resultado do Backtest

```rust
pub struct BacktestResult {
    pub id: String,
    pub strategy_id: String,
    pub params: BacktestParams,

    // Capital
    pub initial_capital: Decimal,
    pub final_capital: Decimal,
    pub total_return: Decimal,         // %
    pub total_return_annualized: Decimal,

    // Trades
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: Decimal,

    // Métricas de risco
    pub profit_factor: Decimal,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub max_drawdown: Decimal,
    pub max_drawdown_duration: Duration,
    pub calmar_ratio: Decimal,

    // Detalhes
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub average_trade_duration: Duration,
    pub exposure_time_pct: Decimal,

    // Dados para gráficos
    pub equity_curve: Vec<EquityPoint>,
    pub drawdown_curve: Vec<DrawdownPoint>,
    pub trades: Vec<BacktestTrade>,

    pub executed_at: DateTime<Utc>,
    pub execution_time_ms: u64,
}
```

---

## Fluxo 6: Eventos e Notificações

### Eventos do Sistema

```rust
pub enum AppEvent {
    // Market Data
    MarketDataUpdated { symbol: String, timeframe: TimeFrame },
    FearGreedUpdated { value: u32, classification: String },

    // Analytics
    IndicatorCalculated { symbol: String, indicator: String },
    SignalGenerated { signal_id: String, symbol: String, side: String },

    // Worker
    JobEnqueued { job_id: String, job_type: String },
    JobCompleted { job_id: String, result: String },
    JobFailed { job_id: String, error: String },

    // Trading
    OrderSubmitted { order_id: String, symbol: String },
    OrderFilled { order_id: String, symbol: String, price: Decimal },
    OrderCancelled { order_id: String },
    PositionOpened { position_id: String, symbol: String },
    PositionClosed { position_id: String, pnl: Decimal },

    // System
    WorkerStarted,
    WorkerStopped,
    SchedulerStarted,
    SchedulerStopped,
    ConfigReloaded,
    Error { component: String, message: String },
}
```

### Emissão de Eventos (Tauri)

```rust
// Backend
app_handle.emit_all("signal-generated", SignalEventPayload {
    signal_id: signal.id.to_string(),
    symbol: signal.symbol.clone(),
    side: signal.side.to_string(),
    entry_price: signal.entry_price.to_string(),
    confidence: signal.confidence.to_string(),
})?;

// Frontend
import { listen } from '@tauri-apps/api/event';

listen('signal-generated', (event) => {
    const signal = event.payload;
    showNotification(`New ${signal.side} signal for ${signal.symbol}`);
    updateSignalsList();
});
```

---

## Transformações de Dados

### API → Candle (Normalização)

```rust
impl From<BinanceKline> for Candle {
    fn from(kline: BinanceKline) -> Self {
        Candle {
            symbol: kline.symbol,
            timeframe: TimeFrame::from_str(&kline.interval).unwrap(),
            open_time: DateTime::from_timestamp_millis(kline.open_time).unwrap(),
            close_time: DateTime::from_timestamp_millis(kline.close_time).unwrap(),
            open: Decimal::from_str(&kline.open).unwrap(),
            high: Decimal::from_str(&kline.high).unwrap(),
            low: Decimal::from_str(&kline.low).unwrap(),
            close: Decimal::from_str(&kline.close).unwrap(),
            volume: Decimal::from_str(&kline.volume).unwrap(),
            quote_volume: Decimal::from_str(&kline.quote_volume).unwrap(),
            trades_count: kline.trades as u32,
        }
    }
}
```

### Signal → Job (Conversão)

```rust
impl From<&Signal> for PlaceOrderPayload {
    fn from(signal: &Signal) -> Self {
        PlaceOrderPayload {
            signal_id: signal.id.clone(),
            symbol: signal.symbol.clone(),
            side: signal.side,
            order_type: OrderType::Market,
            quantity: signal.quantity.unwrap_or_default(),
            price: None,
            stop_loss: signal.stop_loss.map(|sl| StopLossConfig {
                price: sl,
                order_type: OrderType::StopMarket,
            }),
            take_profit: signal.take_profit.map(|tp| TakeProfitConfig {
                price: tp,
                order_type: OrderType::TakeProfitMarket,
            }),
            reduce_only: false,
            client_order_id: Some(format!("signal-{}", signal.id)),
        }
    }
}
```

### Candle → DTO (Serialização)

```rust
#[derive(Serialize, Deserialize)]
pub struct CandleDto {
    pub symbol: String,
    pub timeframe: String,
    pub open_time: String,
    pub close_time: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub quote_volume: String,
    pub trades_count: u32,
}

impl From<Candle> for CandleDto {
    fn from(candle: Candle) -> Self {
        CandleDto {
            symbol: candle.symbol,
            timeframe: candle.timeframe.to_string(),
            open_time: candle.open_time.to_rfc3339(),
            close_time: candle.close_time.to_rfc3339(),
            open: candle.open.to_string(),
            high: candle.high.to_string(),
            low: candle.low.to_string(),
            close: candle.close.to_string(),
            volume: candle.volume.to_string(),
            quote_volume: candle.quote_volume.to_string(),
            trades_count: candle.trades_count,
        }
    }
}
```

---

**Próximo**: [Schema do Banco de Dados](./database-schema.md)
