//! Mensagens relacionadas a dados de mercado

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::entities::{Candle, Ticker, TimeFrame};

/// Mensagens de dados de mercado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketMessage {
  /// Novo candle recebido
  CandleUpdate {
    symbol: String,
    timeframe: TimeFrame,
    candle: Candle,
  },

  /// Atualização de ticker
  TickerUpdate { symbol: String, ticker: Ticker },

  /// Atualização do orderbook
  OrderBookUpdate {
    symbol: String,
    bids: Vec<(Decimal, Decimal)>,
    asks: Vec<(Decimal, Decimal)>,
  },

  /// Novo trade público executado
  PublicTrade {
    symbol: String,
    price: Decimal,
    quantity: Decimal,
    is_buyer_maker: bool,
    timestamp: i64,
  },

  /// Subscrever a um símbolo
  Subscribe {
    symbol: String,
    timeframe: Option<TimeFrame>,
  },

  /// Cancelar subscrição
  Unsubscribe {
    symbol: String,
    timeframe: Option<TimeFrame>,
  },

  /// Reconexão necessária
  Reconnect { reason: String },

  /// Conexão perdida
  Disconnected { reason: String },

  /// Conexão estabelecida
  Connected,
}

/// Comando de request-response para market data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketRequest {
  /// Buscar candles históricos
  GetCandles {
    symbol: String,
    timeframe: TimeFrame,
    limit: usize,
  },

  /// Buscar ticker atual
  GetTicker { symbol: String },

  /// Buscar orderbook
  GetOrderBook { symbol: String, depth: usize },

  /// Listar símbolos disponíveis
  ListSymbols,
}

/// Resposta para requests de market data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketResponse {
  /// Candles retornados
  Candles {
    symbol: String,
    timeframe: TimeFrame,
    candles: Vec<Candle>,
  },

  /// Ticker retornado
  Ticker { symbol: String, ticker: Ticker },

  /// Orderbook retornado
  OrderBook {
    symbol: String,
    bids: Vec<(Decimal, Decimal)>,
    asks: Vec<(Decimal, Decimal)>,
  },

  /// Lista de símbolos
  Symbols { symbols: Vec<String> },

  /// Erro na requisição
  Error { message: String },
}
