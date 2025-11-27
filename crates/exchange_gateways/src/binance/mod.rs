//! Gateway para Binance Futures
//!
//! Suporta tanto testnet quanto produção.
//! Documentação: https://binance-docs.github.io/apidocs/futures/en/

mod client;
mod models;
mod signer;
mod websocket;

pub use client::BinanceFuturesClient;
pub use models::*;
pub use websocket::*;
