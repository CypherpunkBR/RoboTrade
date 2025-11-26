//! # robotrade-analytics
//!
//! Motor de análise técnica e backtest:
//! - Indicadores técnicos (RSI, MA, Bollinger, etc.)
//! - Estratégias de trading
//! - Motor de backtest
//! - Métricas de performance

pub mod backtest;
pub mod strategies;

// Re-exports
pub use backtest::{BacktestConfig, BacktestEngine};
pub use strategies::{fear_greed::FearGreedStrategyConfig, FearGreedStrategy, Strategy};

// TODO: Implementar módulos adicionais
// pub mod indicators;
// pub mod strategies;
// pub mod backtest;
// pub mod metrics;
