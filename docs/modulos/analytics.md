# Módulo Analytics

O crate `robotrade-analytics` fornece ferramentas de análise, backtesting e estratégias de trading.

## Estrutura de Diretórios

```
crates/analytics/src/
├── lib.rs              # Exportações públicas
├── backtest/           # Engine de backtest
│   ├── mod.rs
│   └── engine.rs       # BacktestEngine
└── strategies/         # Estratégias de trading
    ├── mod.rs
    └── fear_greed.rs   # FearGreedStrategy
```

## Engine de Backtest

O motor de backtest simula a execução de estratégias em dados históricos.

### Configuração

```rust
use robotrade_analytics::backtest::{BacktestEngine, BacktestConfig};
use rust_decimal_macros::dec;

let config = BacktestConfig {
    initial_capital: dec!(10000),      // Capital inicial
    position_size_pct: dec!(10),       // % do capital por posição
    trading_fee_pct: dec!(0.04),       // Taxa de trading (0.04%)
    slippage_pct: dec!(0.05),          // Slippage estimado
    leverage: 10,                       // Alavancagem
    default_stop_loss_pct: Some(dec!(2)),   // Stop loss padrão
    default_take_profit_pct: Some(dec!(4)), // Take profit padrão
};

let engine = BacktestEngine::new(config);
```

### Executando Backtest

```rust
use robotrade_analytics::strategies::FearGreedStrategy;
use robotrade_core::entities::TimeFrame;

let strategy = FearGreedStrategy::default();
let candles = vec![/* candles históricos */];
let fear_greed_data = vec![/* dados F&G históricos */];

let result = engine.run(
    &strategy,
    &candles,
    &fear_greed_data,
    "BTCUSDT",
    TimeFrame::H4,
).await?;

// Analisar resultados
println!("Total de trades: {}", result.total_trades);
println!("Win rate: {}%", result.win_rate_pct);
println!("Retorno total: {}%", result.total_return_pct);
println!("Max drawdown: {}%", result.max_drawdown_pct);
println!("Profit factor: {}", result.profit_factor);
println!("Sharpe ratio: {}", result.sharpe_ratio);
```

### Métricas Calculadas

```rust
pub struct BacktestResultDto {
    pub strategy_name: String,
    pub symbol: String,
    pub timeframe: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,

    // Métricas de performance
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate_pct: Decimal,

    // P&L
    pub initial_capital: Decimal,
    pub final_capital: Decimal,
    pub total_return_pct: Decimal,
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,

    // Risco
    pub max_drawdown_pct: Decimal,
    pub profit_factor: Decimal,
    pub sharpe_ratio: Decimal,
    pub avg_trade_pnl: Decimal,

    // Detalhes
    pub trades: Vec<SimulatedTrade>,
    pub equity_curve: Vec<Decimal>,
}
```

### Simulação de Posições

O engine simula:
- **Entrada**: Com slippage no preço
- **Taxas**: Na entrada e saída
- **Stop Loss/Take Profit**: Verificados a cada candle
- **P&L**: Calculado com alavancagem

```rust
struct SimulatedPosition {
    side: PositionSide,
    entry_price: Decimal,
    quantity: Decimal,
    stop_loss: Option<Decimal>,
    take_profit: Option<Decimal>,
    opened_at: DateTime<Utc>,
    entry_fee: Decimal,
}

struct SimulatedTrade {
    side: PositionSide,
    entry_price: Decimal,
    exit_price: Decimal,
    pnl_pct: Decimal,
    net_pnl: Decimal,
    opened_at: DateTime<Utc>,
    closed_at: DateTime<Utc>,
}
```

## Estratégia Fear & Greed

Estratégia contrarian baseada no índice Fear & Greed.

### Lógica

- **Extreme Fear (0-24)**: Sinal de compra (Long)
- **Extreme Greed (76-100)**: Sinal de venda (Short)

### Configuração

```rust
use robotrade_analytics::strategies::{FearGreedStrategy, FearGreedStrategyConfig};
use rust_decimal_macros::dec;

let config = FearGreedStrategyConfig {
    fear_threshold: 25,              // Abaixo = sinal de compra
    greed_threshold: 75,             // Acima = sinal de venda
    min_candles_for_signal: 10,      // Mínimo de candles para operar
    position_size_pct: dec!(10),     // Tamanho da posição
    stop_loss_pct: Some(dec!(5)),    // Stop loss
    take_profit_pct: Some(dec!(10)), // Take profit
};

let strategy = FearGreedStrategy::new(config);
```

### Implementação do Trait Strategy

```rust
impl Strategy for FearGreedStrategy {
    fn name(&self) -> &str {
        "Fear & Greed Contrarian"
    }

    fn description(&self) -> &str {
        "Estratégia contrarian baseada no índice Fear & Greed"
    }

    fn generate_signal(&self, context: &StrategyContext) -> Option<Signal> {
        let fear_greed = context.fear_greed_data.last()?;

        // Medo extremo = comprar
        if fear_greed.value <= self.config.fear_threshold {
            return Some(Signal::new(
                "fear_greed",
                self.name(),
                &context.symbol,
                SignalType::Entry,
                TradeDirection::Long,
                SignalStrength::Strong,
                context.candles.last()?.close,
                format!("Fear & Greed em {}: medo extremo", fear_greed.value),
            ));
        }

        // Ganância extrema = vender
        if fear_greed.value >= self.config.greed_threshold {
            return Some(Signal::new(
                "fear_greed",
                self.name(),
                &context.symbol,
                SignalType::Entry,
                TradeDirection::Short,
                SignalStrength::Strong,
                context.candles.last()?.close,
                format!("Fear & Greed em {}: ganância extrema", fear_greed.value),
            ));
        }

        None
    }

    fn is_ready(&self, context: &StrategyContext) -> bool {
        context.candles.len() >= self.config.min_candles_for_signal
            && !context.fear_greed_data.is_empty()
    }
}
```

### Contexto da Estratégia

```rust
pub struct StrategyContext {
    pub symbol: String,
    pub timeframe: TimeFrame,
    pub candles: Vec<Candle>,
    pub fear_greed_data: Vec<FearGreedData>,
    pub current_position: Option<Position>,
    pub account_balance: Decimal,
}
```

## Estratégias Planejadas

| Estratégia | Descrição | Status |
|------------|-----------|--------|
| Fear & Greed | Contrarian F&G | ✅ Implementado |
| RSI Oversold/Overbought | RSI extremos | 📋 Planejado |
| MA Crossover | Cruzamento de médias | 📋 Planejado |
| Bollinger Breakout | Rompimento de bandas | 📋 Planejado |
| DCA Automatizado | Dollar Cost Averaging | 📋 Planejado |
| Grid Trading | Grid de ordens | 📋 Planejado |

## Indicadores Técnicos Planejados

```rust
// Exemplos de indicadores futuros
fn rsi(candles: &[Candle], period: usize) -> Decimal;
fn sma(candles: &[Candle], period: usize) -> Decimal;
fn ema(candles: &[Candle], period: usize) -> Decimal;
fn bollinger_bands(candles: &[Candle], period: usize, std_dev: Decimal)
    -> (Decimal, Decimal, Decimal); // (upper, middle, lower)
fn macd(candles: &[Candle]) -> (Decimal, Decimal, Decimal); // (macd, signal, histogram)
fn atr(candles: &[Candle], period: usize) -> Decimal;
```

## Exemplo Completo

```rust
use robotrade_analytics::{
    backtest::{BacktestEngine, BacktestConfig},
    strategies::{FearGreedStrategy, FearGreedStrategyConfig},
};
use robotrade_core::entities::TimeFrame;
use rust_decimal_macros::dec;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurar backtest
    let backtest_config = BacktestConfig {
        initial_capital: dec!(10000),
        position_size_pct: dec!(20),
        trading_fee_pct: dec!(0.04),
        slippage_pct: dec!(0.05),
        leverage: 5,
        default_stop_loss_pct: Some(dec!(3)),
        default_take_profit_pct: Some(dec!(6)),
    };

    // Configurar estratégia
    let strategy_config = FearGreedStrategyConfig {
        fear_threshold: 20,
        greed_threshold: 80,
        min_candles_for_signal: 24,
        position_size_pct: dec!(20),
        stop_loss_pct: Some(dec!(3)),
        take_profit_pct: Some(dec!(6)),
    };

    let engine = BacktestEngine::new(backtest_config);
    let strategy = FearGreedStrategy::new(strategy_config);

    // Carregar dados históricos (exemplo)
    let candles = load_historical_candles("BTCUSDT", TimeFrame::H4).await?;
    let fear_greed = load_fear_greed_history(365).await?;

    // Executar backtest
    let result = engine.run(
        &strategy,
        &candles,
        &fear_greed,
        "BTCUSDT",
        TimeFrame::H4,
    ).await?;

    // Imprimir resultados
    println!("=== Resultado do Backtest ===");
    println!("Estratégia: {}", result.strategy_name);
    println!("Período: {} - {}", result.start_date, result.end_date);
    println!("Total trades: {}", result.total_trades);
    println!("Win rate: {:.2}%", result.win_rate_pct);
    println!("Retorno: {:.2}%", result.total_return_pct);
    println!("Max Drawdown: {:.2}%", result.max_drawdown_pct);
    println!("Profit Factor: {:.2}", result.profit_factor);
    println!("Sharpe Ratio: {:.2}", result.sharpe_ratio);

    Ok(())
}
```
