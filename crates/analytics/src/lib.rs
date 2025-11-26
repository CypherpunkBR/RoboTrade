//! # robotrade-analytics
//!
//! Motor de análise técnica e backtest:
//! - Indicadores técnicos (RSI, MA, Bollinger, etc.)
//! - Estratégias de trading
//! - Motor de backtest
//! - Métricas de performance

pub mod strategies;
pub mod backtest;

// Re-exports
pub use strategies::{Strategy, FearGreedStrategy, fear_greed::FearGreedStrategyConfig};
pub use backtest::{BacktestEngine, BacktestConfig};

// TODO: Implementar módulos adicionais
// pub mod indicators;
// pub mod strategies;
// pub mod backtest;
// pub mod metrics;
