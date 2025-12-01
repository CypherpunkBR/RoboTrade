//! Mensagens para comunicação entre actors
//!
//! Este módulo define os tipos de mensagens usados no sistema de actors.
//! As mensagens são categorizadas por domínio:
//!
//! - `MarketMessage`: Dados de mercado (candles, tickers, orderbook)
//! - `TradingMessage`: Execução de ordens, posições
//! - `SignalMessage`: Sinais de trading, alertas
//! - `SystemMessage`: Comandos de sistema (shutdown, health check)

mod market;
mod signal;
mod system;
mod trading;

pub use market::*;
pub use signal::*;
pub use system::*;
pub use trading::*;
