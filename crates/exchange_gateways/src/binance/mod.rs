//! Gateway para Binance Futures
//!
//! Suporta tanto testnet quanto produção.
//! Documentação: https://binance-docs.github.io/apidocs/futures/en/

mod client;
mod models;
mod signer;

pub use client::BinanceFuturesClient;
pub use models::*;
