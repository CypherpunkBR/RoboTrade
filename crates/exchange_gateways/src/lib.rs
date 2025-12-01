//! # robotrade-exchange-gateways
//!
//! Gateways para comunicação com exchanges:
//! - Binance Futures (testnet e produção)
//! - Kraken Futures (demo e produção)
//! - Paper trading (simulação local)
//!
//! Cada gateway implementa a trait `ExchangeGateway` do core.

pub mod binance;
pub mod kraken;
pub mod paper;

pub use binance::BinanceFuturesClient;
pub use kraken::KrakenFuturesClient;
pub use paper::PaperTradingClient;
