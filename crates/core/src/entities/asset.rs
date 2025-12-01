//! Definições de ativos e pares de trading

use serde::{Deserialize, Serialize};
use std::fmt;

/// Representa um ativo (moeda/token)
/// Exemplo: BTC, ETH, USDT
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Asset {
  /// Símbolo do ativo (ex: "BTC", "ETH")
  pub symbol: String,
}

impl Asset {
  /// Cria um novo ativo
  pub fn new(symbol: impl Into<String>) -> Self {
    Self {
      symbol: symbol.into().to_uppercase(),
    }
  }
}

impl fmt::Display for Asset {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.symbol)
  }
}

/// Par de trading (base/quote)
/// Exemplo: BTC/USDT onde BTC é base e USDT é quote
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradingPair {
  /// Ativo base (o que você compra/vende)
  pub base: Asset,
  /// Ativo de cotação (o que você usa para pagar)
  pub quote: Asset,
  /// Símbolo como usado na exchange (ex: "BTCUSDT")
  pub exchange_symbol: String,
}

impl TradingPair {
  /// Cria um novo par de trading
  pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
    let base = Asset::new(base);
    let quote = Asset::new(quote);
    let exchange_symbol = format!("{}{}", base.symbol, quote.symbol);

    Self {
      base,
      quote,
      exchange_symbol,
    }
  }

  /// Cria um par a partir do símbolo da exchange
  /// Nota: Assume que os últimos 4 caracteres são o quote (USDT, BUSD)
  pub fn from_exchange_symbol(symbol: &str) -> Option<Self> {
    if symbol.len() < 5 {
      return None;
    }

    // Tenta detectar o quote asset comum
    let quote_assets = ["USDT", "BUSD", "USDC", "BTC", "ETH"];

    for quote in quote_assets {
      if let Some(base) = symbol.strip_suffix(quote) {
        if !base.is_empty() {
          return Some(Self {
            base: Asset::new(base),
            quote: Asset::new(quote),
            exchange_symbol: symbol.to_string(),
          });
        }
      }
    }

    None
  }
}

impl fmt::Display for TradingPair {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}/{}", self.base, self.quote)
  }
}

/// Identificador de exchange
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExchangeId {
  /// Binance Futures
  BinanceFutures,
  /// Binance Spot
  BinanceSpot,
  /// Kraken Futures
  KrakenFutures,
  /// OKX
  Okx,
  /// Bybit
  Bybit,
  /// Paper trading (simulação)
  Paper,
}

impl fmt::Display for ExchangeId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ExchangeId::BinanceFutures => write!(f, "binance_futures"),
      ExchangeId::BinanceSpot => write!(f, "binance_spot"),
      ExchangeId::KrakenFutures => write!(f, "kraken_futures"),
      ExchangeId::Okx => write!(f, "okx"),
      ExchangeId::Bybit => write!(f, "bybit"),
      ExchangeId::Paper => write!(f, "paper"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_asset_creation() {
    let btc = Asset::new("btc");
    assert_eq!(btc.symbol, "BTC");
  }

  #[test]
  fn test_trading_pair_creation() {
    let pair = TradingPair::new("btc", "usdt");
    assert_eq!(pair.base.symbol, "BTC");
    assert_eq!(pair.quote.symbol, "USDT");
    assert_eq!(pair.exchange_symbol, "BTCUSDT");
  }

  #[test]
  fn test_trading_pair_from_symbol() {
    let pair = TradingPair::from_exchange_symbol("BTCUSDT").unwrap();
    assert_eq!(pair.base.symbol, "BTC");
    assert_eq!(pair.quote.symbol, "USDT");
  }
}
