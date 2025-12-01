//! Entidades de P&L (Profit and Loss)
//!
//! Tipos unificados para cálculos de lucro/prejuízo realizado e não realizado.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{CostBasisMethod, Exchange, PositionSide};

/// ID único de um registro de P&L realizado
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RealizedPnLId(pub String);

impl RealizedPnLId {
  pub fn new() -> Self {
    Self(uuid::Uuid::new_v4().to_string())
  }

  pub fn from_string(s: impl Into<String>) -> Self {
    Self(s.into())
  }
}

impl Default for RealizedPnLId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::fmt::Display for RealizedPnLId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl AsRef<str> for RealizedPnLId {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

/// P&L realizado de uma venda/fechamento de posição
///
/// Representa o lucro ou prejuízo efetivamente realizado quando uma posição é fechada
/// ou um ativo é vendido.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealizedPnL {
  /// ID único
  pub id: RealizedPnLId,
  /// Exchange onde ocorreu
  pub exchange: Exchange,
  /// Símbolo do par (ex: "BTCUSDT")
  pub symbol: String,
  /// Quantidade vendida/fechada
  pub quantity: Decimal,
  /// Valor recebido (proceeds)
  pub proceeds: Decimal,
  /// Cost basis usado
  pub cost_basis: Decimal,
  /// Lucro/prejuízo bruto (proceeds - cost_basis)
  pub gross_pnl: Decimal,
  /// Taxas pagas
  pub fees: Decimal,
  /// Lucro/prejuízo líquido (gross_pnl - fees)
  pub net_pnl: Decimal,
  /// Se é ganho de curto prazo (< 365 dias)
  pub is_short_term: bool,
  /// Se é ganho de longo prazo (>= 365 dias)
  pub is_long_term: bool,
  /// Data de aquisição (do lote mais antigo usado)
  pub acquired_at: DateTime<Utc>,
  /// Data da venda/fechamento
  pub disposed_at: DateTime<Utc>,
  /// Período de holding em dias
  pub holding_period_days: i64,
  /// ID do lote consumido (se FIFO/LIFO)
  pub lot_id: Option<String>,
  /// ID do trade que fechou a posição
  pub close_trade_id: Option<String>,
  /// Método de cost basis usado
  pub method: CostBasisMethod,
}

impl RealizedPnL {
  /// Cria um novo P&L realizado
  pub fn new(
    exchange: Exchange,
    symbol: String,
    quantity: Decimal,
    proceeds: Decimal,
    cost_basis: Decimal,
    fees: Decimal,
    acquired_at: DateTime<Utc>,
    disposed_at: DateTime<Utc>,
    method: CostBasisMethod,
  ) -> Self {
    let gross_pnl = proceeds - cost_basis;
    let net_pnl = gross_pnl - fees;
    let holding_period_days = (disposed_at - acquired_at).num_days();
    let is_short_term = holding_period_days < 365;
    let is_long_term = holding_period_days >= 365;

    Self {
      id: RealizedPnLId::new(),
      exchange,
      symbol,
      quantity,
      proceeds,
      cost_basis,
      gross_pnl,
      fees,
      net_pnl,
      is_short_term,
      is_long_term,
      acquired_at,
      disposed_at,
      holding_period_days,
      lot_id: None,
      close_trade_id: None,
      method,
    }
  }

  /// Verifica se houve lucro
  pub fn is_profit(&self) -> bool {
    self.net_pnl > Decimal::ZERO
  }

  /// Verifica se houve prejuízo
  pub fn is_loss(&self) -> bool {
    self.net_pnl < Decimal::ZERO
  }

  /// Builder: define o ID do lote
  pub fn with_lot_id(mut self, lot_id: impl Into<String>) -> Self {
    self.lot_id = Some(lot_id.into());
    self
  }

  /// Builder: define o ID do trade de fechamento
  pub fn with_close_trade_id(mut self, trade_id: impl Into<String>) -> Self {
    self.close_trade_id = Some(trade_id.into());
    self
  }
}

/// P&L não realizado de uma posição aberta
///
/// Representa o lucro ou prejuízo potencial de uma posição que ainda está aberta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnL {
  /// Exchange onde está a posição
  pub exchange: Exchange,
  /// Símbolo do par (ex: "BTCUSDT")
  pub symbol: String,
  /// Lado da posição
  pub side: PositionSide,
  /// Quantidade em posição
  pub quantity: Decimal,
  /// Cost basis total
  pub cost_basis: Decimal,
  /// Custo médio por unidade
  pub avg_cost_per_unit: Decimal,
  /// Preço de mercado atual
  pub current_price: Decimal,
  /// Valor de mercado atual
  pub current_value: Decimal,
  /// P&L não realizado
  pub unrealized_pnl: Decimal,
  /// P&L percentual
  pub unrealized_pnl_pct: Decimal,
  /// Data do lote mais antigo (para calcular holding period)
  pub oldest_lot_date: Option<DateTime<Utc>>,
  /// Quantidade em short-term (< 365 dias)
  pub short_term_quantity: Decimal,
  /// Quantidade em long-term (>= 365 dias)
  pub long_term_quantity: Decimal,
}

impl UnrealizedPnL {
  /// Calcula P&L não realizado
  pub fn calculate(
    exchange: Exchange,
    symbol: String,
    side: PositionSide,
    quantity: Decimal,
    cost_basis: Decimal,
    current_price: Decimal,
  ) -> Self {
    let avg_cost_per_unit = if quantity.is_zero() {
      Decimal::ZERO
    } else {
      cost_basis / quantity
    };

    let current_value = quantity * current_price;

    // Para posição long: lucro = valor_atual - custo
    // Para posição short: lucro = custo - valor_atual
    let unrealized_pnl = match side {
      PositionSide::Long => current_value - cost_basis,
      PositionSide::Short => cost_basis - current_value,
    };

    let unrealized_pnl_pct = if cost_basis.is_zero() {
      Decimal::ZERO
    } else {
      (unrealized_pnl / cost_basis) * Decimal::from(100)
    };

    Self {
      exchange,
      symbol,
      side,
      quantity,
      cost_basis,
      avg_cost_per_unit,
      current_price,
      current_value,
      unrealized_pnl,
      unrealized_pnl_pct,
      oldest_lot_date: None,
      short_term_quantity: Decimal::ZERO,
      long_term_quantity: Decimal::ZERO,
    }
  }

  /// Builder: define a data do lote mais antigo
  pub fn with_oldest_lot_date(mut self, date: DateTime<Utc>) -> Self {
    self.oldest_lot_date = Some(date);
    self
  }

  /// Builder: define as quantidades short/long term
  pub fn with_term_quantities(mut self, short_term: Decimal, long_term: Decimal) -> Self {
    self.short_term_quantity = short_term;
    self.long_term_quantity = long_term;
    self
  }

  /// Verifica se há lucro potencial
  pub fn is_profit(&self) -> bool {
    self.unrealized_pnl > Decimal::ZERO
  }

  /// Verifica se há prejuízo potencial
  pub fn is_loss(&self) -> bool {
    self.unrealized_pnl < Decimal::ZERO
  }
}

/// Sumário de P&L para um período
///
/// Agrega informações de P&L realizado para um período específico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnLSummary {
  /// Exchange (None = todas)
  pub exchange: Option<Exchange>,
  /// Símbolo (None = todos)
  pub symbol: Option<String>,
  /// Início do período
  pub period_start: DateTime<Utc>,
  /// Fim do período
  pub period_end: DateTime<Utc>,
  /// Total realizado (líquido)
  pub total_realized: Decimal,
  /// Ganhos de curto prazo (bruto positivo)
  pub short_term_gains: Decimal,
  /// Perdas de curto prazo (bruto negativo)
  pub short_term_losses: Decimal,
  /// Ganhos de longo prazo (bruto positivo)
  pub long_term_gains: Decimal,
  /// Perdas de longo prazo (bruto negativo)
  pub long_term_losses: Decimal,
  /// Net short-term (ganhos - perdas)
  pub net_short_term: Decimal,
  /// Net long-term (ganhos - perdas)
  pub net_long_term: Decimal,
  /// Total de proceeds (vendas)
  pub total_proceeds: Decimal,
  /// Total de cost basis usado
  pub total_cost_basis: Decimal,
  /// Total de taxas pagas
  pub total_fees: Decimal,
  /// Número de trades/eventos
  pub num_trades: u64,
}

impl PnLSummary {
  /// Cria um sumário vazio para o período
  pub fn new(period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
    Self {
      exchange: None,
      symbol: None,
      period_start,
      period_end,
      total_realized: Decimal::ZERO,
      short_term_gains: Decimal::ZERO,
      short_term_losses: Decimal::ZERO,
      long_term_gains: Decimal::ZERO,
      long_term_losses: Decimal::ZERO,
      net_short_term: Decimal::ZERO,
      net_long_term: Decimal::ZERO,
      total_proceeds: Decimal::ZERO,
      total_cost_basis: Decimal::ZERO,
      total_fees: Decimal::ZERO,
      num_trades: 0,
    }
  }

  /// Builder: define a exchange
  pub fn with_exchange(mut self, exchange: Exchange) -> Self {
    self.exchange = Some(exchange);
    self
  }

  /// Builder: define o símbolo
  pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
    self.symbol = Some(symbol.into());
    self
  }

  /// Adiciona um P&L realizado ao sumário
  pub fn add_realized_pnl(&mut self, pnl: &RealizedPnL) {
    self.total_realized += pnl.net_pnl;
    self.total_proceeds += pnl.proceeds;
    self.total_cost_basis += pnl.cost_basis;
    self.total_fees += pnl.fees;
    self.num_trades += 1;

    if pnl.is_short_term {
      if pnl.is_profit() {
        self.short_term_gains += pnl.gross_pnl;
      } else {
        self.short_term_losses += pnl.gross_pnl.abs();
      }
      self.net_short_term += pnl.gross_pnl;
    } else {
      if pnl.is_profit() {
        self.long_term_gains += pnl.gross_pnl;
      } else {
        self.long_term_losses += pnl.gross_pnl.abs();
      }
      self.net_long_term += pnl.gross_pnl;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::entities::{ExchangeId, ExchangeType};
  use rust_decimal_macros::dec;

  fn test_exchange() -> Exchange {
    Exchange::new(
      ExchangeId::BinanceFutures,
      "Binance Futures",
      ExchangeType::Cex,
    )
  }

  #[test]
  fn test_realized_pnl_profit() {
    let pnl = RealizedPnL::new(
      test_exchange(),
      "BTCUSDT".to_string(),
      dec!(1.0),
      dec!(50000), // proceeds
      dec!(45000), // cost_basis
      dec!(10),    // fees
      Utc::now() - chrono::Duration::days(100),
      Utc::now(),
      CostBasisMethod::Fifo,
    );

    assert!(pnl.is_profit());
    assert!(!pnl.is_loss());
    assert_eq!(pnl.gross_pnl, dec!(5000));
    assert_eq!(pnl.net_pnl, dec!(4990));
    assert!(pnl.is_short_term);
    assert!(!pnl.is_long_term);
  }

  #[test]
  fn test_realized_pnl_loss() {
    let pnl = RealizedPnL::new(
      test_exchange(),
      "BTCUSDT".to_string(),
      dec!(1.0),
      dec!(40000), // proceeds
      dec!(45000), // cost_basis
      dec!(10),    // fees
      Utc::now() - chrono::Duration::days(400),
      Utc::now(),
      CostBasisMethod::Fifo,
    );

    assert!(!pnl.is_profit());
    assert!(pnl.is_loss());
    assert_eq!(pnl.gross_pnl, dec!(-5000));
    assert_eq!(pnl.net_pnl, dec!(-5010));
    assert!(!pnl.is_short_term);
    assert!(pnl.is_long_term);
  }

  #[test]
  fn test_unrealized_pnl_long() {
    let pnl = UnrealizedPnL::calculate(
      test_exchange(),
      "BTCUSDT".to_string(),
      PositionSide::Long,
      dec!(1.0),
      dec!(45000),
      dec!(50000),
    );

    assert!(pnl.is_profit());
    assert_eq!(pnl.unrealized_pnl, dec!(5000));
    assert_eq!(pnl.current_value, dec!(50000));
  }

  #[test]
  fn test_unrealized_pnl_short() {
    let pnl = UnrealizedPnL::calculate(
      test_exchange(),
      "BTCUSDT".to_string(),
      PositionSide::Short,
      dec!(1.0),
      dec!(50000),
      dec!(45000),
    );

    assert!(pnl.is_profit());
    assert_eq!(pnl.unrealized_pnl, dec!(5000)); // Short profit when price goes down
  }

  #[test]
  fn test_pnl_summary() {
    let mut summary = PnLSummary::new(Utc::now() - chrono::Duration::days(30), Utc::now());

    let pnl1 = RealizedPnL::new(
      test_exchange(),
      "BTCUSDT".to_string(),
      dec!(1.0),
      dec!(50000),
      dec!(45000),
      dec!(10),
      Utc::now() - chrono::Duration::days(100),
      Utc::now(),
      CostBasisMethod::Fifo,
    );

    let pnl2 = RealizedPnL::new(
      test_exchange(),
      "ETHUSDT".to_string(),
      dec!(10.0),
      dec!(20000),
      dec!(22000),
      dec!(5),
      Utc::now() - chrono::Duration::days(50),
      Utc::now(),
      CostBasisMethod::Fifo,
    );

    summary.add_realized_pnl(&pnl1);
    summary.add_realized_pnl(&pnl2);

    assert_eq!(summary.num_trades, 2);
    assert_eq!(summary.short_term_gains, dec!(5000));
    assert_eq!(summary.short_term_losses, dec!(2000));
    assert_eq!(summary.net_short_term, dec!(3000));
  }
}
