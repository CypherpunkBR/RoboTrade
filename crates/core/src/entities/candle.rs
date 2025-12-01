//! Entidade Candle (vela) para dados OHLCV

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Vela (Candle) representando dados OHLCV
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candle {
  /// Preço de abertura
  pub open: Decimal,
  /// Preço máximo
  pub high: Decimal,
  /// Preço mínimo
  pub low: Decimal,
  /// Preço de fechamento
  pub close: Decimal,
  /// Volume negociado
  pub volume: Decimal,
  /// Volume em quote asset (opcional)
  pub quote_volume: Option<Decimal>,
  /// Número de trades (opcional)
  pub trade_count: Option<u32>,
  /// Timestamp de abertura da vela
  pub open_time: DateTime<Utc>,
  /// Timestamp de fechamento da vela
  pub close_time: DateTime<Utc>,
}

impl Candle {
  /// Cria uma nova vela
  pub fn new(
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    open_time: DateTime<Utc>,
    close_time: DateTime<Utc>,
  ) -> Self {
    Self {
      open,
      high,
      low,
      close,
      volume,
      quote_volume: None,
      trade_count: None,
      open_time,
      close_time,
    }
  }

  /// Retorna o tamanho do corpo da vela (diferença entre open e close)
  pub fn body_size(&self) -> Decimal {
    (self.close - self.open).abs()
  }

  /// Retorna o range total da vela (high - low)
  pub fn range(&self) -> Decimal {
    self.high - self.low
  }

  /// Verifica se a vela é de alta (bullish)
  pub fn is_bullish(&self) -> bool {
    self.close > self.open
  }

  /// Verifica se a vela é de baixa (bearish)
  pub fn is_bearish(&self) -> bool {
    self.close < self.open
  }

  /// Verifica se a vela é um doji (corpo muito pequeno)
  pub fn is_doji(&self, threshold: Decimal) -> bool {
    let body_pct = if self.open != Decimal::ZERO {
      self.body_size() / self.open * Decimal::from(100)
    } else {
      Decimal::ZERO
    };
    body_pct < threshold
  }

  /// Retorna o preço médio (típico) da vela
  pub fn typical_price(&self) -> Decimal {
    (self.high + self.low + self.close) / Decimal::from(3)
  }

  /// Retorna a sombra superior (upper shadow)
  pub fn upper_shadow(&self) -> Decimal {
    self.high - self.close.max(self.open)
  }

  /// Retorna a sombra inferior (lower shadow)
  pub fn lower_shadow(&self) -> Decimal {
    self.close.min(self.open) - self.low
  }
}

/// Intervalo de tempo das velas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeFrame {
  /// 1 minuto
  M1,
  /// 3 minutos
  M3,
  /// 5 minutos
  M5,
  /// 15 minutos
  M15,
  /// 30 minutos
  M30,
  /// 1 hora
  H1,
  /// 2 horas
  H2,
  /// 4 horas
  H4,
  /// 6 horas
  H6,
  /// 8 horas
  H8,
  /// 12 horas
  H12,
  /// 1 dia
  D1,
  /// 3 dias
  D3,
  /// 1 semana
  W1,
  /// 1 mês
  Mo1,
}

impl TimeFrame {
  /// Retorna a duração em minutos
  pub fn as_minutes(&self) -> u32 {
    match self {
      TimeFrame::M1 => 1,
      TimeFrame::M3 => 3,
      TimeFrame::M5 => 5,
      TimeFrame::M15 => 15,
      TimeFrame::M30 => 30,
      TimeFrame::H1 => 60,
      TimeFrame::H2 => 120,
      TimeFrame::H4 => 240,
      TimeFrame::H6 => 360,
      TimeFrame::H8 => 480,
      TimeFrame::H12 => 720,
      TimeFrame::D1 => 1440,
      TimeFrame::D3 => 4320,
      TimeFrame::W1 => 10080,
      TimeFrame::Mo1 => 43200, // Aproximado
    }
  }

  /// Converte para string no formato da Binance
  pub fn to_binance_interval(&self) -> &'static str {
    match self {
      TimeFrame::M1 => "1m",
      TimeFrame::M3 => "3m",
      TimeFrame::M5 => "5m",
      TimeFrame::M15 => "15m",
      TimeFrame::M30 => "30m",
      TimeFrame::H1 => "1h",
      TimeFrame::H2 => "2h",
      TimeFrame::H4 => "4h",
      TimeFrame::H6 => "6h",
      TimeFrame::H8 => "8h",
      TimeFrame::H12 => "12h",
      TimeFrame::D1 => "1d",
      TimeFrame::D3 => "3d",
      TimeFrame::W1 => "1w",
      TimeFrame::Mo1 => "1M",
    }
  }

  /// Cria TimeFrame a partir de string
  pub fn parse(s: &str) -> Option<Self> {
    match s.to_lowercase().as_str() {
      "1m" => Some(TimeFrame::M1),
      "3m" => Some(TimeFrame::M3),
      "5m" => Some(TimeFrame::M5),
      "15m" => Some(TimeFrame::M15),
      "30m" => Some(TimeFrame::M30),
      "1h" => Some(TimeFrame::H1),
      "2h" => Some(TimeFrame::H2),
      "4h" => Some(TimeFrame::H4),
      "6h" => Some(TimeFrame::H6),
      "8h" => Some(TimeFrame::H8),
      "12h" => Some(TimeFrame::H12),
      "1d" => Some(TimeFrame::D1),
      "3d" => Some(TimeFrame::D3),
      "1w" => Some(TimeFrame::W1),
      "1mo" | "1month" => Some(TimeFrame::Mo1),
      _ => None,
    }
  }
}

impl std::str::FromStr for TimeFrame {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s).ok_or_else(|| format!("TimeFrame inválido: {}", s))
  }
}

impl fmt::Display for TimeFrame {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_binance_interval())
  }
}

/// Ticker com preço atual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
  /// Par de trading
  pub symbol: String,
  /// Preço atual
  pub price: Decimal,
  /// Melhor bid (compra)
  pub bid: Decimal,
  /// Melhor ask (venda)
  pub ask: Decimal,
  /// Volume 24h
  pub volume_24h: Decimal,
  /// Variação 24h em percentual
  pub change_24h_pct: Decimal,
  /// Timestamp da exchange
  pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use rust_decimal_macros::dec;

  #[test]
  fn test_candle_bullish() {
    let candle = Candle::new(
      dec!(100),
      dec!(110),
      dec!(95),
      dec!(105),
      dec!(1000),
      Utc::now(),
      Utc::now(),
    );

    assert!(candle.is_bullish());
    assert!(!candle.is_bearish());
  }

  #[test]
  fn test_candle_body_size() {
    let candle = Candle::new(
      dec!(100),
      dec!(110),
      dec!(95),
      dec!(105),
      dec!(1000),
      Utc::now(),
      Utc::now(),
    );

    assert_eq!(candle.body_size(), dec!(5));
    assert_eq!(candle.range(), dec!(15));
  }

  #[test]
  fn test_timeframe_conversion() {
    assert_eq!(TimeFrame::H4.as_minutes(), 240);
    assert_eq!(TimeFrame::H4.to_binance_interval(), "4h");
  }
}
