//! # robotrade-analytics
//!
//! Motor de análise técnica e backtest:
//! - Indicadores técnicos (RSI, MA, Bollinger, etc.)
//! - Estratégias de trading
//! - Motor de backtest
//! - Métricas de performance

pub mod backtest;
pub mod indicators;
pub mod strategies;

// Re-exports - Backtest
pub use backtest::{BacktestConfig, BacktestEngine};

// Re-exports - Strategies
pub use strategies::{fear_greed::FearGreedStrategyConfig, FearGreedStrategy, Strategy};

// Re-exports - Indicators
pub use indicators::{
    // Traits
    Indicator, CandleIndicator,
    // Moving Averages
    SMA, EMA,
    // Oscillators
    RSI, RSILevels,
    MACD, MACDResult,
    // Volatility
    BollingerBands, BollingerResult,
    ATR, ATRMultipliers,
    // Utils
    extract_close_prices, extract_high_prices, extract_low_prices,
};
