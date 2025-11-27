//! Kraken Futures exchange gateway
//!
//! Este módulo fornece integração com a API REST da Kraken Futures,
//! permitindo trading de contratos perpétuos e futuros.

mod client;
mod models;
mod signer;
mod websocket;

pub use client::KrakenFuturesClient;
pub use models::*;
pub use signer::KrakenSigner;
pub use websocket::*;
