//! # robotrade-market-data
//!
//! Crate responsável pela coleta e gerenciamento de dados de mercado:
//! - Provedores de dados (Fear & Greed Index, Binance)
//! - WebSocket para dados em tempo real
//! - Scheduler de coleta

pub mod providers;

pub use providers::AlternativeMeFearGreedProvider;
