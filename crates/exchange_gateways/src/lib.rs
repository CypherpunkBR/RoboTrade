//! # robotrade-exchange-gateways
//!
//! Gateways para comunicação com exchanges:
//! - Binance Futures (testnet e produção)
//! - Kraken Pro Futures (planejado)
//! - Paper trading (simulação local)
//!
//! Cada gateway implementa a trait `ExchangeGateway` do core.

pub mod binance;

pub use binance::BinanceFuturesClient;
