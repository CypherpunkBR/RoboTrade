//! Entidade Exchange e Symbol
//!
//! Modelos para exchanges e símbolos de trading.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::ExchangeId;

/// Status de uma exchange
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExchangeStatus {
  /// Exchange ativa e operacional
  Active,
  /// Exchange em manutenção
  Maintenance,
  /// Exchange desabilitada
  Disabled,
}

impl fmt::Display for ExchangeStatus {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ExchangeStatus::Active => write!(f, "active"),
      ExchangeStatus::Maintenance => write!(f, "maintenance"),
      ExchangeStatus::Disabled => write!(f, "disabled"),
    }
  }
}

impl std::str::FromStr for ExchangeStatus {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "active" => Ok(ExchangeStatus::Active),
      "maintenance" => Ok(ExchangeStatus::Maintenance),
      "disabled" => Ok(ExchangeStatus::Disabled),
      _ => Err(format!("Status de exchange inválido: {}", s)),
    }
  }
}

/// Tipo de exchange
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExchangeType {
  /// Exchange centralizada
  Cex,
  /// Exchange descentralizada
  Dex,
  /// Paper trading (simulação)
  Paper,
}

impl fmt::Display for ExchangeType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ExchangeType::Cex => write!(f, "cex"),
      ExchangeType::Dex => write!(f, "dex"),
      ExchangeType::Paper => write!(f, "paper"),
    }
  }
}

impl std::str::FromStr for ExchangeType {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "cex" => Ok(ExchangeType::Cex),
      "dex" => Ok(ExchangeType::Dex),
      "paper" => Ok(ExchangeType::Paper),
      _ => Err(format!("Tipo de exchange inválido: {}", s)),
    }
  }
}

/// Configuração de uma exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exchange {
  /// ID único da exchange
  pub id: ExchangeId,
  /// Nome da exchange
  pub name: String,
  /// Tipo de exchange
  pub exchange_type: ExchangeType,
  /// URL base da API REST
  pub api_base_url: Option<String>,
  /// URL base do WebSocket
  pub ws_base_url: Option<String>,
  /// Status atual
  pub status: ExchangeStatus,
  /// Limite de requisições por minuto
  pub rate_limit_requests: u32,
  /// Limite de ordens por segundo
  pub rate_limit_orders: u32,
  /// Features suportadas (spot, futures, margin)
  pub supported_features: Vec<String>,
  /// Metadados adicionais
  pub metadata: Option<serde_json::Value>,
  /// Data de criação
  pub created_at: DateTime<Utc>,
  /// Última atualização
  pub updated_at: DateTime<Utc>,
}

impl Exchange {
  /// Cria uma nova exchange
  pub fn new(id: ExchangeId, name: impl Into<String>, exchange_type: ExchangeType) -> Self {
    let now = Utc::now();
    Self {
      id,
      name: name.into(),
      exchange_type,
      api_base_url: None,
      ws_base_url: None,
      status: ExchangeStatus::Active,
      rate_limit_requests: 1200,
      rate_limit_orders: 10,
      supported_features: vec![],
      metadata: None,
      created_at: now,
      updated_at: now,
    }
  }

  /// Verifica se a exchange está ativa
  pub fn is_active(&self) -> bool {
    self.status == ExchangeStatus::Active
  }

  /// Verifica se suporta uma feature específica
  pub fn supports_feature(&self, feature: &str) -> bool {
    self
      .supported_features
      .iter()
      .any(|f| f.eq_ignore_ascii_case(feature))
  }
}

/// Tipo de símbolo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolType {
  /// Mercado spot
  Spot,
  /// Contrato perpétuo
  Perpetual,
  /// Contrato com vencimento
  Delivery,
  /// Opção
  Option,
}

impl fmt::Display for SymbolType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SymbolType::Spot => write!(f, "spot"),
      SymbolType::Perpetual => write!(f, "perpetual"),
      SymbolType::Delivery => write!(f, "delivery"),
      SymbolType::Option => write!(f, "option"),
    }
  }
}

impl std::str::FromStr for SymbolType {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "spot" => Ok(SymbolType::Spot),
      "perpetual" => Ok(SymbolType::Perpetual),
      "delivery" => Ok(SymbolType::Delivery),
      "option" => Ok(SymbolType::Option),
      _ => Err(format!("Tipo de símbolo inválido: {}", s)),
    }
  }
}

/// Status de um símbolo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolStatus {
  /// Símbolo ativo para trading
  Active,
  /// Símbolo pausado
  Paused,
  /// Símbolo deslistado
  Delisted,
}

impl fmt::Display for SymbolStatus {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SymbolStatus::Active => write!(f, "active"),
      SymbolStatus::Paused => write!(f, "paused"),
      SymbolStatus::Delisted => write!(f, "delisted"),
    }
  }
}

impl std::str::FromStr for SymbolStatus {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "active" => Ok(SymbolStatus::Active),
      "paused" => Ok(SymbolStatus::Paused),
      "delisted" => Ok(SymbolStatus::Delisted),
      _ => Err(format!("Status de símbolo inválido: {}", s)),
    }
  }
}

/// Símbolo de trading (par de ativos)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
  /// ID único (auto-incrementado)
  pub id: i64,
  /// ID da exchange
  pub exchange_id: ExchangeId,
  /// Símbolo (ex: BTCUSDT)
  pub symbol: String,
  /// Ativo base (ex: BTC)
  pub base_asset: String,
  /// Ativo de cotação (ex: USDT)
  pub quote_asset: String,
  /// Tipo de símbolo
  pub symbol_type: SymbolType,
  /// Status
  pub status: SymbolStatus,

  // Precisão
  /// Casas decimais do preço
  pub price_precision: u8,
  /// Casas decimais da quantidade
  pub quantity_precision: u8,
  /// Casas decimais do quote
  pub quote_precision: u8,

  // Limites de trading
  /// Quantidade mínima
  pub min_quantity: Decimal,
  /// Quantidade máxima
  pub max_quantity: Option<Decimal>,
  /// Valor mínimo em quote (notional)
  pub min_notional: Decimal,
  /// Valor máximo em quote
  pub max_notional: Option<Decimal>,
  /// Incremento mínimo de preço
  pub tick_size: Decimal,
  /// Incremento mínimo de quantidade
  pub step_size: Decimal,

  // Específico de futuros
  /// Tipo de contrato
  pub contract_type: Option<String>,
  /// Tamanho do contrato
  pub contract_size: Option<Decimal>,
  /// Ativo de margem
  pub margin_asset: Option<String>,
  /// Taxa de margem de manutenção
  pub maintenance_margin_rate: Option<Decimal>,
  /// Alavancagem máxima
  pub max_leverage: Option<u32>,

  // Taxas
  /// Taxa maker
  pub maker_fee: Decimal,
  /// Taxa taker
  pub taker_fee: Decimal,

  /// Metadados adicionais
  pub metadata: Option<serde_json::Value>,
  /// Data de criação
  pub created_at: DateTime<Utc>,
  /// Última atualização
  pub updated_at: DateTime<Utc>,
}

impl Symbol {
  /// Cria um novo símbolo
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    exchange_id: ExchangeId,
    symbol: impl Into<String>,
    base_asset: impl Into<String>,
    quote_asset: impl Into<String>,
    symbol_type: SymbolType,
    price_precision: u8,
    quantity_precision: u8,
    min_quantity: Decimal,
    min_notional: Decimal,
    tick_size: Decimal,
    step_size: Decimal,
  ) -> Self {
    let now = Utc::now();
    Self {
      id: 0, // será definido pelo banco
      exchange_id,
      symbol: symbol.into(),
      base_asset: base_asset.into(),
      quote_asset: quote_asset.into(),
      symbol_type,
      status: SymbolStatus::Active,
      price_precision,
      quantity_precision,
      quote_precision: 8,
      min_quantity,
      max_quantity: None,
      min_notional,
      max_notional: None,
      tick_size,
      step_size,
      contract_type: None,
      contract_size: None,
      margin_asset: None,
      maintenance_margin_rate: None,
      max_leverage: None,
      maker_fee: Decimal::new(1, 3), // 0.001 = 0.1%
      taker_fee: Decimal::new(1, 3),
      metadata: None,
      created_at: now,
      updated_at: now,
    }
  }

  /// Verifica se o símbolo está ativo
  pub fn is_active(&self) -> bool {
    self.status == SymbolStatus::Active
  }

  /// Verifica se é um contrato de futuros
  pub fn is_futures(&self) -> bool {
    matches!(
      self.symbol_type,
      SymbolType::Perpetual | SymbolType::Delivery
    )
  }

  /// Arredonda preço para a precisão correta
  pub fn round_price(&self, price: Decimal) -> Decimal {
    let scale = self.price_precision as u32;
    price.round_dp(scale)
  }

  /// Arredonda quantidade para a precisão correta
  pub fn round_quantity(&self, quantity: Decimal) -> Decimal {
    let scale = self.quantity_precision as u32;
    quantity.round_dp(scale)
  }

  /// Valida se uma quantidade está dentro dos limites
  pub fn validate_quantity(&self, quantity: Decimal) -> Result<(), String> {
    if quantity < self.min_quantity {
      return Err(format!(
        "Quantidade {} abaixo do mínimo {}",
        quantity, self.min_quantity
      ));
    }
    if let Some(max) = self.max_quantity {
      if quantity > max {
        return Err(format!("Quantidade {} acima do máximo {}", quantity, max));
      }
    }
    Ok(())
  }

  /// Valida se um valor notional está dentro dos limites
  pub fn validate_notional(&self, notional: Decimal) -> Result<(), String> {
    if notional < self.min_notional {
      return Err(format!(
        "Valor {} abaixo do mínimo {}",
        notional, self.min_notional
      ));
    }
    if let Some(max) = self.max_notional {
      if notional > max {
        return Err(format!("Valor {} acima do máximo {}", notional, max));
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rust_decimal_macros::dec;

  #[test]
  fn test_exchange_creation() {
    let exchange = Exchange::new(
      ExchangeId::BinanceFutures,
      "Binance Futures",
      ExchangeType::Cex,
    );

    assert_eq!(exchange.name, "Binance Futures");
    assert!(exchange.is_active());
  }

  #[test]
  fn test_symbol_validation() {
    let symbol = Symbol::new(
      ExchangeId::BinanceFutures,
      "BTCUSDT",
      "BTC",
      "USDT",
      SymbolType::Perpetual,
      2,
      3,
      dec!(0.001),
      dec!(5),
      dec!(0.01),
      dec!(0.001),
    );

    assert!(symbol.validate_quantity(dec!(0.01)).is_ok());
    assert!(symbol.validate_quantity(dec!(0.0001)).is_err());
    assert!(symbol.validate_notional(dec!(10)).is_ok());
    assert!(symbol.validate_notional(dec!(1)).is_err());
  }

  #[test]
  fn test_symbol_rounding() {
    let symbol = Symbol::new(
      ExchangeId::BinanceFutures,
      "BTCUSDT",
      "BTC",
      "USDT",
      SymbolType::Perpetual,
      2,
      3,
      dec!(0.001),
      dec!(5),
      dec!(0.01),
      dec!(0.001),
    );

    assert_eq!(symbol.round_price(dec!(42000.123456)), dec!(42000.12));
    assert_eq!(symbol.round_quantity(dec!(0.123456)), dec!(0.123));
  }
}
