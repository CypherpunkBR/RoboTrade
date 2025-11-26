# Módulo Core

O crate `robotrade-core` é a fundação do sistema, contendo todas as entidades, traits, erros e DTOs utilizados pelos demais módulos. Não possui dependências internas, garantindo que seja a base estável do projeto.

## Estrutura de Diretórios

```
crates/core/src/
├── lib.rs              # Exportações públicas
├── entities/           # Entidades de domínio
│   ├── mod.rs
│   ├── asset.rs        # Asset, TradingPair, ExchangeId
│   ├── candle.rs       # Candle, TimeFrame, Ticker
│   ├── order.rs        # Order, OrderRequest, OrderStatus, OrderType
│   ├── position.rs     # Position, PositionSide, PositionStatus
│   ├── signal.rs       # Signal, SignalStrength, TradeDirection
│   ├── trade.rs        # Trade, TradeCloseReason, TradeStats
│   └── fear_greed.rs   # FearGreedData, FearGreedClassification
├── error/              # Hierarquia de erros
│   └── mod.rs
├── traits/             # Traits (interfaces)
│   └── mod.rs
└── dto/                # Data Transfer Objects
    └── mod.rs
```

## Entidades Principais

### Asset e TradingPair

```rust
/// Representa um ativo (moeda/token)
pub struct Asset {
    pub symbol: String,  // Ex: "BTC", "ETH"
}

/// Par de trading (base/quote)
pub struct TradingPair {
    pub base: Asset,           // Ex: BTC
    pub quote: Asset,          // Ex: USDT
    pub exchange_symbol: String, // Ex: "BTCUSDT"
}

/// Identificador de exchange
pub enum ExchangeId {
    BinanceFutures,
    BinanceSpot,
    Okx,
    Bybit,
    Paper,  // Simulação
}
```

### Candle (Vela OHLCV)

```rust
/// Vela com dados OHLCV
pub struct Candle {
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub quote_volume: Option<Decimal>,
    pub trade_count: Option<u32>,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
}

/// Timeframes suportados (15 intervalos)
pub enum TimeFrame {
    M1, M3, M5, M15, M30,     // Minutos
    H1, H2, H4, H6, H8, H12,  // Horas
    D1, D3,                    // Dias
    W1,                        // Semana
    Mo1,                       // Mês
}
```

### Order (Ordem)

```rust
/// Ordem de trading
pub struct Order {
    pub id: OrderId,
    pub exchange: ExchangeId,
    pub exchange_order_id: Option<String>,
    pub symbol: String,
    pub side: OrderSide,        // Buy, Sell
    pub order_type: OrderType,  // Market, Limit, StopLoss, etc.
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub filled_quantity: Decimal,
    pub avg_fill_price: Option<Decimal>,
    pub status: OrderStatus,    // 8 estados possíveis
    // ...
}

/// Status da ordem
pub enum OrderStatus {
    Pending,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
    Failed,
}

/// Tipos de ordem
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    TrailingStop,
}
```

### Position (Posição)

```rust
/// Posição aberta
pub struct Position {
    pub id: PositionId,
    pub exchange: ExchangeId,
    pub symbol: String,
    pub side: PositionSide,     // Long, Short
    pub quantity: Decimal,
    pub entry_price: Decimal,
    pub leverage: u32,
    pub margin: Decimal,
    pub unrealized_pnl: Decimal,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub status: PositionStatus,
    // ...
}

pub enum PositionSide {
    Long,
    Short,
}
```

### Signal (Sinal de Trading)

```rust
/// Sinal gerado por estratégia
pub struct Signal {
    pub id: SignalId,
    pub strategy_id: String,
    pub strategy_name: String,
    pub symbol: String,
    pub signal_type: SignalType,    // Entry, Exit, StopLoss, etc.
    pub direction: TradeDirection,   // Long, Short
    pub strength: SignalStrength,    // Weak, Moderate, Strong, VeryStrong
    pub trigger_price: Decimal,
    pub confidence: Option<Decimal>,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub reason: String,
    // ...
}
```

### Fear & Greed Index

```rust
/// Dados do índice Fear & Greed
pub struct FearGreedData {
    pub value: u8,                          // 0-100
    pub classification: FearGreedClassification,
    pub timestamp: DateTime<Utc>,
}

/// Classificação do índice
pub enum FearGreedClassification {
    ExtremeFear,   // 0-24
    Fear,          // 25-44
    Neutral,       // 45-55
    Greed,         // 56-75
    ExtremeGreed,  // 76-100
}
```

## Hierarquia de Erros

O sistema possui uma hierarquia de erros tipados para cada domínio:

```rust
/// Erro principal (wrapper)
pub enum RoboTradeError {
    Core(CoreError),
    MarketData(MarketDataError),
    Exchange(ExchangeError),
    Trading(TradingError),
    Infra(InfraError),
    Analytics(AnalyticsError),
}

/// Erros de domínio específicos
pub enum CoreError {
    Validation { field: String, reason: String },
    InvalidState { expected: String, actual: String },
    NotFound { entity: String, id: String },
    // ...
}

pub enum ExchangeError {
    ConnectionFailed { exchange: ExchangeId, reason: String },
    RateLimited { exchange: ExchangeId, retry_after: Duration },
    InvalidCredentials { exchange: ExchangeId },
    InsufficientFunds { symbol: String, required: Decimal },
    // ...
}
```

## Traits (Interfaces)

### Repository Pattern

```rust
/// Repositório genérico
#[async_trait]
pub trait Repository<T, Id>: Send + Sync {
    async fn save(&self, entity: &T) -> CoreResult<()>;
    async fn find_by_id(&self, id: &Id) -> CoreResult<Option<T>>;
    async fn delete(&self, id: &Id) -> CoreResult<bool>;
}

/// Repositório de Candles
#[async_trait]
pub trait CandleRepository: Send + Sync {
    async fn save(&self, candle: &Candle, symbol: &str, timeframe: TimeFrame) -> CoreResult<()>;
    async fn get_range(...) -> CoreResult<Vec<Candle>>;
    async fn get_latest(...) -> CoreResult<Option<Candle>>;
}
```

### Provider Pattern

```rust
/// Provider de dados Fear & Greed
#[async_trait]
pub trait FearGreedProvider: Send + Sync {
    async fn fetch_current(&self) -> MarketDataResult<FearGreedData>;
    async fn fetch_history(&self, days: u32) -> MarketDataResult<Vec<FearGreedData>>;
    async fn health_check(&self) -> bool;
}

/// Provider de dados de mercado
#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn get_ticker(&self, symbol: &str) -> MarketDataResult<Ticker>;
    async fn get_candles(...) -> MarketDataResult<Vec<Candle>>;
}
```

### Gateway Pattern

```rust
/// Gateway de exchange
#[async_trait]
pub trait ExchangeGateway: Send + Sync {
    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order>;
    async fn cancel_order(&self, order_id: &OrderId) -> ExchangeResult<bool>;
    async fn get_order(&self, order_id: &OrderId) -> ExchangeResult<Option<Order>>;
    async fn get_positions(&self) -> ExchangeResult<Vec<Position>>;
    async fn get_balances(&self) -> ExchangeResult<HashMap<String, Decimal>>;
}
```

### Strategy Pattern

```rust
/// Estratégia de trading
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn generate_signal(&self, context: &StrategyContext) -> Option<Signal>;
    fn is_ready(&self, context: &StrategyContext) -> bool;
}
```

## DTOs

```rust
/// Resumo do dashboard
pub struct DashboardSummary {
    pub trading_mode: TradingMode,
    pub connection_status: ServiceStatus,
    pub active_positions: u32,
    pub daily_pnl: Decimal,
    pub daily_pnl_pct: Decimal,
    pub total_balance: Decimal,
    pub fear_greed_current: Option<FearGreedData>,
}

/// Modo de trading
pub enum TradingMode {
    Paper,  // Simulação
    Live,   // Produção
}
```

## Uso

```rust
use robotrade_core::{
    entities::{Asset, TradingPair, Candle, TimeFrame},
    traits::{ExchangeGateway, Strategy},
    error::RoboTradeResult,
};

// Criar par de trading
let pair = TradingPair::new("BTC", "USDT");

// Verificar se candle é bullish
if candle.is_bullish() {
    println!("Vela de alta!");
}

// Parsear timeframe
let tf: TimeFrame = "4h".parse()?;
```
