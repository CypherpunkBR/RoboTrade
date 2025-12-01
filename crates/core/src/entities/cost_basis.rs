//! Entidades relacionadas ao Cost Basis (base de custo)
//!
//! Rastreamento de lotes de aquisição para cálculos FIFO/LIFO/Average Cost.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// ID único de um lote de custo
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CostBasisLotId(pub String);

impl CostBasisLotId {
  pub fn new() -> Self {
    Self(uuid::Uuid::new_v4().to_string())
  }

  pub fn from_string(s: String) -> Self {
    Self(s)
  }
}

impl Default for CostBasisLotId {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for CostBasisLotId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl AsRef<str> for CostBasisLotId {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

/// Método de cálculo do cost basis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CostBasisMethod {
  /// First In, First Out - Lotes mais antigos são vendidos primeiro
  Fifo,
  /// Last In, First Out - Lotes mais recentes são vendidos primeiro
  Lifo,
  /// Average Cost - Custo médio ponderado
  Avg,
}

impl fmt::Display for CostBasisMethod {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CostBasisMethod::Fifo => write!(f, "FIFO"),
      CostBasisMethod::Lifo => write!(f, "LIFO"),
      CostBasisMethod::Avg => write!(f, "AVG"),
    }
  }
}

impl std::str::FromStr for CostBasisMethod {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_uppercase().as_str() {
      "FIFO" => Ok(CostBasisMethod::Fifo),
      "LIFO" => Ok(CostBasisMethod::Lifo),
      "AVG" | "AVERAGE" | "AVERAGE_COST" => Ok(CostBasisMethod::Avg),
      _ => Err(format!("Unknown cost basis method: {}", s)),
    }
  }
}

/// Tipo de aquisição do ativo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AcquisitionType {
  /// Compra direta
  Buy,
  /// Transferência recebida
  TransferIn,
  /// Depósito
  Deposit,
  /// Airdrop
  Airdrop,
  /// Fork de blockchain
  Fork,
  /// Mineração
  Mining,
  /// Recompensa de staking
  StakingReward,
}

impl fmt::Display for AcquisitionType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      AcquisitionType::Buy => write!(f, "BUY"),
      AcquisitionType::TransferIn => write!(f, "TRANSFER_IN"),
      AcquisitionType::Deposit => write!(f, "DEPOSIT"),
      AcquisitionType::Airdrop => write!(f, "AIRDROP"),
      AcquisitionType::Fork => write!(f, "FORK"),
      AcquisitionType::Mining => write!(f, "MINING"),
      AcquisitionType::StakingReward => write!(f, "STAKING_REWARD"),
    }
  }
}

impl std::str::FromStr for AcquisitionType {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_uppercase().as_str() {
      "BUY" => Ok(AcquisitionType::Buy),
      "TRANSFER_IN" => Ok(AcquisitionType::TransferIn),
      "DEPOSIT" => Ok(AcquisitionType::Deposit),
      "AIRDROP" => Ok(AcquisitionType::Airdrop),
      "FORK" => Ok(AcquisitionType::Fork),
      "MINING" => Ok(AcquisitionType::Mining),
      "STAKING_REWARD" => Ok(AcquisitionType::StakingReward),
      _ => Err(format!("Unknown acquisition type: {}", s)),
    }
  }
}

/// Lote de cost basis - representa uma aquisição específica de um ativo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBasisLot {
  /// ID único do lote
  pub id: CostBasisLotId,
  /// ID da exchange
  pub exchange_id: String,
  /// Ativo (ex: "BTC", "ETH")
  pub asset: String,
  /// Par de trading se de trade (ex: "BTCUSDT")
  pub symbol: Option<String>,
  /// Quantidade original adquirida
  pub quantity: Decimal,
  /// Quantidade restante (não vendida)
  pub remaining_quantity: Decimal,
  /// Custo por unidade na moeda de cotação
  pub cost_per_unit: Decimal,
  /// Custo total do lote
  pub total_cost: Decimal,
  /// Taxa incluída no cost basis
  pub fee_included: Decimal,
  /// Método de cost basis usado
  pub cost_basis_method: CostBasisMethod,
  /// Tipo de aquisição
  pub acquisition_type: AcquisitionType,
  /// Data de aquisição
  pub acquisition_date: DateTime<Utc>,
  /// ID de referência (trade_id, deposit_id, etc)
  pub reference_id: Option<String>,
  /// ID da entrada do ledger relacionada
  pub ledger_entry_id: Option<String>,
  /// Se o lote foi completamente vendido
  pub is_closed: bool,
  /// Quando o lote foi fechado
  pub closed_at: Option<DateTime<Utc>>,
  /// ID da referência de venda
  pub disposal_reference_id: Option<String>,
  /// Criado em
  pub created_at: DateTime<Utc>,
  /// Atualizado em
  pub updated_at: DateTime<Utc>,
}

impl CostBasisLot {
  /// Cria um novo lote de cost basis
  pub fn new(
    exchange_id: String,
    asset: String,
    quantity: Decimal,
    cost_per_unit: Decimal,
    cost_basis_method: CostBasisMethod,
    acquisition_type: AcquisitionType,
    acquisition_date: DateTime<Utc>,
  ) -> Self {
    let total_cost = quantity * cost_per_unit;
    let now = Utc::now();

    Self {
      id: CostBasisLotId::new(),
      exchange_id,
      asset,
      symbol: None,
      quantity,
      remaining_quantity: quantity,
      cost_per_unit,
      total_cost,
      fee_included: Decimal::ZERO,
      cost_basis_method,
      acquisition_type,
      acquisition_date,
      reference_id: None,
      ledger_entry_id: None,
      is_closed: false,
      closed_at: None,
      disposal_reference_id: None,
      created_at: now,
      updated_at: now,
    }
  }

  /// Builder: define símbolo
  pub fn with_symbol(mut self, symbol: String) -> Self {
    self.symbol = Some(symbol);
    self
  }

  /// Builder: define taxa incluída
  pub fn with_fee(mut self, fee: Decimal) -> Self {
    self.fee_included = fee;
    self.total_cost = self.quantity * self.cost_per_unit + fee;
    self
  }

  /// Builder: define referência
  pub fn with_reference(mut self, reference_id: String) -> Self {
    self.reference_id = Some(reference_id);
    self
  }

  /// Builder: define entrada do ledger
  pub fn with_ledger_entry(mut self, ledger_entry_id: String) -> Self {
    self.ledger_entry_id = Some(ledger_entry_id);
    self
  }

  /// Calcula o custo médio por unidade (incluindo taxas)
  pub fn average_cost_per_unit(&self) -> Decimal {
    if self.quantity.is_zero() {
      Decimal::ZERO
    } else {
      self.total_cost / self.quantity
    }
  }

  /// Calcula o custo proporcional da quantidade restante
  pub fn remaining_cost(&self) -> Decimal {
    if self.quantity.is_zero() {
      Decimal::ZERO
    } else {
      self.total_cost * (self.remaining_quantity / self.quantity)
    }
  }

  /// Consome uma quantidade do lote e retorna o custo correspondente
  pub fn consume(&mut self, quantity: Decimal) -> ConsumedLot {
    let consume_qty = quantity.min(self.remaining_quantity);
    let cost = self.average_cost_per_unit() * consume_qty;

    self.remaining_quantity -= consume_qty;
    self.updated_at = Utc::now();

    if self.remaining_quantity.is_zero() {
      self.is_closed = true;
      self.closed_at = Some(Utc::now());
    }

    ConsumedLot {
      lot_id: self.id.clone(),
      quantity: consume_qty,
      cost_basis: cost,
      cost_per_unit: self.average_cost_per_unit(),
      acquisition_date: self.acquisition_date,
      acquisition_type: self.acquisition_type,
    }
  }

  /// Calcula o período de holding até uma data
  pub fn holding_period(&self, sale_date: DateTime<Utc>) -> Duration {
    sale_date - self.acquisition_date
  }

  /// Verifica se é ganho de longo prazo (>= 1 ano de holding)
  pub fn is_long_term(&self, sale_date: DateTime<Utc>) -> bool {
    self.holding_period(sale_date) >= Duration::days(365)
  }
}

/// Resultado do consumo de um lote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumedLot {
  /// ID do lote consumido
  pub lot_id: CostBasisLotId,
  /// Quantidade consumida
  pub quantity: Decimal,
  /// Cost basis da quantidade consumida
  pub cost_basis: Decimal,
  /// Custo por unidade
  pub cost_per_unit: Decimal,
  /// Data de aquisição original
  pub acquisition_date: DateTime<Utc>,
  /// Tipo de aquisição
  pub acquisition_type: AcquisitionType,
}

impl ConsumedLot {
  /// Verifica se é ganho de longo prazo
  pub fn is_long_term(&self, sale_date: DateTime<Utc>) -> bool {
    (sale_date - self.acquisition_date) >= Duration::days(365)
  }
}

// Nota: Os tipos RealizedPnL, UnrealizedPnL e PnLSummary foram movidos para
// crates/core/src/entities/pnl.rs como tipos consolidados que incluem informações
// de exchange, símbolo, e suportam tanto spot quanto futures.

#[cfg(test)]
mod tests {
  use super::*;
  use rust_decimal_macros::dec;

  #[test]
  fn test_cost_basis_lot_creation() {
    let lot = CostBasisLot::new(
      "binance".to_string(),
      "BTC".to_string(),
      dec!(1),
      dec!(50000),
      CostBasisMethod::Fifo,
      AcquisitionType::Buy,
      Utc::now(),
    );

    assert_eq!(lot.quantity, dec!(1));
    assert_eq!(lot.remaining_quantity, dec!(1));
    assert_eq!(lot.total_cost, dec!(50000));
    assert!(!lot.is_closed);
  }

  #[test]
  fn test_lot_with_fee() {
    let lot = CostBasisLot::new(
      "binance".to_string(),
      "BTC".to_string(),
      dec!(1),
      dec!(50000),
      CostBasisMethod::Fifo,
      AcquisitionType::Buy,
      Utc::now(),
    )
    .with_fee(dec!(50));

    assert_eq!(lot.total_cost, dec!(50050));
    assert_eq!(lot.average_cost_per_unit(), dec!(50050));
  }

  #[test]
  fn test_lot_consume() {
    let mut lot = CostBasisLot::new(
      "binance".to_string(),
      "BTC".to_string(),
      dec!(2),
      dec!(50000),
      CostBasisMethod::Fifo,
      AcquisitionType::Buy,
      Utc::now(),
    );

    let consumed = lot.consume(dec!(1));

    assert_eq!(consumed.quantity, dec!(1));
    assert_eq!(consumed.cost_basis, dec!(50000));
    assert_eq!(lot.remaining_quantity, dec!(1));
    assert!(!lot.is_closed);

    let consumed2 = lot.consume(dec!(1));
    assert_eq!(consumed2.quantity, dec!(1));
    assert_eq!(lot.remaining_quantity, dec!(0));
    assert!(lot.is_closed);
  }

  // Nota: Testes de RealizedPnL e UnrealizedPnL movidos para pnl.rs
}
