//! Entidades relacionadas ao Ledger financeiro
//!
//! Sistema de registro de todas as movimentações financeiras.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// ID único de uma entrada no ledger
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LedgerEntryId(pub String);

impl LedgerEntryId {
  pub fn new() -> Self {
    Self(uuid::Uuid::new_v4().to_string())
  }

  pub fn from_string(s: String) -> Self {
    Self(s)
  }
}

impl Default for LedgerEntryId {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for LedgerEntryId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl AsRef<str> for LedgerEntryId {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

/// Tipo de entrada no ledger
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LedgerEntryType {
  /// Depósito de fundos
  Deposit,
  /// Retirada de fundos
  Withdrawal,
  /// Lucro/prejuízo realizado de trade
  TradePnl,
  /// Taxa cobrada
  Fee,
  /// Pagamento de funding (perpetual futures)
  FundingPayment,
  /// Transferência recebida (entre contas)
  TransferIn,
  /// Transferência enviada (entre contas)
  TransferOut,
  /// Ajuste manual ou correção
  Adjustment,
  /// Liquidação forçada
  Liquidation,
  /// Rebate de comissão
  CommissionRebate,
}

impl fmt::Display for LedgerEntryType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      LedgerEntryType::Deposit => write!(f, "DEPOSIT"),
      LedgerEntryType::Withdrawal => write!(f, "WITHDRAWAL"),
      LedgerEntryType::TradePnl => write!(f, "TRADE_PNL"),
      LedgerEntryType::Fee => write!(f, "FEE"),
      LedgerEntryType::FundingPayment => write!(f, "FUNDING_PAYMENT"),
      LedgerEntryType::TransferIn => write!(f, "TRANSFER_IN"),
      LedgerEntryType::TransferOut => write!(f, "TRANSFER_OUT"),
      LedgerEntryType::Adjustment => write!(f, "ADJUSTMENT"),
      LedgerEntryType::Liquidation => write!(f, "LIQUIDATION"),
      LedgerEntryType::CommissionRebate => write!(f, "COMMISSION_REBATE"),
    }
  }
}

impl std::str::FromStr for LedgerEntryType {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_uppercase().as_str() {
      "DEPOSIT" => Ok(LedgerEntryType::Deposit),
      "WITHDRAWAL" => Ok(LedgerEntryType::Withdrawal),
      "TRADE_PNL" => Ok(LedgerEntryType::TradePnl),
      "FEE" => Ok(LedgerEntryType::Fee),
      "FUNDING_PAYMENT" => Ok(LedgerEntryType::FundingPayment),
      "TRANSFER_IN" => Ok(LedgerEntryType::TransferIn),
      "TRANSFER_OUT" => Ok(LedgerEntryType::TransferOut),
      "ADJUSTMENT" => Ok(LedgerEntryType::Adjustment),
      "LIQUIDATION" => Ok(LedgerEntryType::Liquidation),
      "COMMISSION_REBATE" => Ok(LedgerEntryType::CommissionRebate),
      _ => Err(format!("Unknown ledger entry type: {}", s)),
    }
  }
}

/// Tipo de referência para a entrada
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceType {
  Trade,
  Order,
  Funding,
  Deposit,
  Withdrawal,
  Transfer,
  Liquidation,
  Manual,
}

impl fmt::Display for ReferenceType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ReferenceType::Trade => write!(f, "trade"),
      ReferenceType::Order => write!(f, "order"),
      ReferenceType::Funding => write!(f, "funding"),
      ReferenceType::Deposit => write!(f, "deposit"),
      ReferenceType::Withdrawal => write!(f, "withdrawal"),
      ReferenceType::Transfer => write!(f, "transfer"),
      ReferenceType::Liquidation => write!(f, "liquidation"),
      ReferenceType::Manual => write!(f, "manual"),
    }
  }
}

impl std::str::FromStr for ReferenceType {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "trade" => Ok(ReferenceType::Trade),
      "order" => Ok(ReferenceType::Order),
      "funding" => Ok(ReferenceType::Funding),
      "deposit" => Ok(ReferenceType::Deposit),
      "withdrawal" => Ok(ReferenceType::Withdrawal),
      "transfer" => Ok(ReferenceType::Transfer),
      "liquidation" => Ok(ReferenceType::Liquidation),
      "manual" => Ok(ReferenceType::Manual),
      _ => Err(format!("Unknown reference type: {}", s)),
    }
  }
}

/// Entrada no ledger financeiro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
  /// ID único da entrada
  pub id: LedgerEntryId,
  /// ID da exchange
  pub exchange_id: String,
  /// ID da conta (opcional, para sub-accounts)
  pub account_id: Option<String>,
  /// Tipo da entrada
  pub entry_type: LedgerEntryType,
  /// Ativo (ex: "USDT", "BTC")
  pub asset: String,
  /// Valor (positivo = crédito, negativo = débito)
  pub amount: Decimal,
  /// Saldo após esta entrada
  pub balance_after: Decimal,
  /// Tipo de referência
  pub reference_type: Option<ReferenceType>,
  /// ID da referência (trade_id, order_id, etc)
  pub reference_id: Option<String>,
  /// ID externo da exchange (para deduplicação)
  pub external_id: Option<String>,
  /// Descrição legível
  pub description: Option<String>,
  /// Metadados adicionais (JSON)
  pub metadata: Option<String>,
  /// Quando o evento ocorreu na exchange
  pub timestamp: DateTime<Utc>,
  /// Quando foi criado localmente
  pub created_at: DateTime<Utc>,
}

impl LedgerEntry {
  /// Cria uma nova entrada de ledger
  pub fn new(
    exchange_id: String,
    entry_type: LedgerEntryType,
    asset: String,
    amount: Decimal,
    balance_after: Decimal,
    timestamp: DateTime<Utc>,
  ) -> Self {
    Self {
      id: LedgerEntryId::new(),
      exchange_id,
      account_id: None,
      entry_type,
      asset,
      amount,
      balance_after,
      reference_type: None,
      reference_id: None,
      external_id: None,
      description: None,
      metadata: None,
      timestamp,
      created_at: Utc::now(),
    }
  }

  /// Builder: define ID da conta
  pub fn with_account_id(mut self, account_id: String) -> Self {
    self.account_id = Some(account_id);
    self
  }

  /// Builder: define referência
  pub fn with_reference(mut self, ref_type: ReferenceType, ref_id: String) -> Self {
    self.reference_type = Some(ref_type);
    self.reference_id = Some(ref_id);
    self
  }

  /// Builder: define ID externo
  pub fn with_external_id(mut self, external_id: String) -> Self {
    self.external_id = Some(external_id);
    self
  }

  /// Builder: define descrição
  pub fn with_description(mut self, description: String) -> Self {
    self.description = Some(description);
    self
  }

  /// Builder: define metadados
  pub fn with_metadata(mut self, metadata: String) -> Self {
    self.metadata = Some(metadata);
    self
  }

  /// Verifica se é um crédito (valor positivo)
  pub fn is_credit(&self) -> bool {
    self.amount > Decimal::ZERO
  }

  /// Verifica se é um débito (valor negativo)
  pub fn is_debit(&self) -> bool {
    self.amount < Decimal::ZERO
  }
}

/// Resumo de saldo para um ativo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalanceSummary {
  /// Ativo
  pub asset: String,
  /// Saldo atual calculado
  pub balance: Decimal,
  /// Número de entradas
  pub entry_count: u64,
  /// Última atualização
  pub last_update: DateTime<Utc>,
}

/// Filtros para busca de entradas do ledger
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LedgerFilters {
  /// Filtrar por tipo de entrada
  pub entry_types: Option<Vec<LedgerEntryType>>,
  /// Filtrar por ativo
  pub asset: Option<String>,
  /// Data inicial
  pub start_time: Option<DateTime<Utc>>,
  /// Data final
  pub end_time: Option<DateTime<Utc>>,
  /// Tipo de referência
  pub reference_type: Option<ReferenceType>,
  /// Limite de resultados
  pub limit: Option<u32>,
  /// Offset para paginação
  pub offset: Option<u32>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use rust_decimal_macros::dec;

  #[test]
  fn test_ledger_entry_creation() {
    let entry = LedgerEntry::new(
      "binance".to_string(),
      LedgerEntryType::Deposit,
      "USDT".to_string(),
      dec!(1000),
      dec!(1000),
      Utc::now(),
    );

    assert!(entry.is_credit());
    assert!(!entry.is_debit());
    assert_eq!(entry.entry_type, LedgerEntryType::Deposit);
  }

  #[test]
  fn test_ledger_entry_builder() {
    let entry = LedgerEntry::new(
      "binance".to_string(),
      LedgerEntryType::TradePnl,
      "USDT".to_string(),
      dec!(-50),
      dec!(950),
      Utc::now(),
    )
    .with_reference(ReferenceType::Trade, "trade123".to_string())
    .with_external_id("ext123".to_string())
    .with_description("Loss on BTCUSDT".to_string());

    assert!(entry.is_debit());
    assert_eq!(entry.reference_type, Some(ReferenceType::Trade));
    assert_eq!(entry.reference_id, Some("trade123".to_string()));
    assert_eq!(entry.external_id, Some("ext123".to_string()));
  }

  #[test]
  fn test_entry_type_parsing() {
    assert_eq!(
      "DEPOSIT".parse::<LedgerEntryType>().unwrap(),
      LedgerEntryType::Deposit
    );
    assert_eq!(
      "funding_payment".parse::<LedgerEntryType>().unwrap(),
      LedgerEntryType::FundingPayment
    );
  }
}
