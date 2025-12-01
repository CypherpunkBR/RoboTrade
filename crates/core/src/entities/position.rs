//! Entidade Position (Posição aberta)

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ExchangeId, OrderId};

/// ID único de uma posição
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionId(pub Uuid);

impl PositionId {
  /// Cria um novo ID de posição
  pub fn new() -> Self {
    Self(Uuid::new_v4())
  }
}

impl Default for PositionId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::fmt::Display for PositionId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// Lado da posição (long ou short)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PositionSide {
  /// Posição comprada (aposta na alta)
  Long,
  /// Posição vendida (aposta na baixa)
  Short,
}

impl std::fmt::Display for PositionSide {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PositionSide::Long => write!(f, "long"),
      PositionSide::Short => write!(f, "short"),
    }
  }
}

/// Status da posição
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PositionStatus {
  /// Posição aberta
  Open,
  /// Posição fechada
  Closed,
  /// Liquidada (margin call)
  Liquidated,
}

/// Posição aberta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
  /// ID único da posição
  pub id: PositionId,
  /// Exchange da posição
  pub exchange: ExchangeId,
  /// Símbolo do par
  pub symbol: String,
  /// Lado (long/short)
  pub side: PositionSide,
  /// Quantidade
  pub quantity: Decimal,
  /// Preço médio de entrada
  pub entry_price: Decimal,
  /// Preço atual de mercado
  pub current_price: Decimal,
  /// Alavancagem utilizada
  pub leverage: u32,
  /// Margem utilizada
  pub margin: Decimal,
  /// P&L não realizado (em USDT ou quote asset)
  pub unrealized_pnl: Decimal,
  /// P&L não realizado em percentual
  pub unrealized_pnl_pct: Decimal,
  /// P&L realizado (após fechamentos parciais)
  pub realized_pnl: Decimal,
  /// ID da ordem de stop loss (se configurado)
  pub stop_loss_order_id: Option<OrderId>,
  /// Preço do stop loss
  pub stop_loss_price: Option<Decimal>,
  /// ID da ordem de take profit (se configurado)
  pub take_profit_order_id: Option<OrderId>,
  /// Preço do take profit
  pub take_profit_price: Option<Decimal>,
  /// ID da ordem que abriu a posição
  pub entry_order_id: OrderId,
  /// Status da posição
  pub status: PositionStatus,
  /// Preço de liquidação (para futuros)
  pub liquidation_price: Option<Decimal>,
  /// Timestamp de abertura
  pub opened_at: DateTime<Utc>,
  /// Timestamp de fechamento
  pub closed_at: Option<DateTime<Utc>>,
  /// Última atualização
  pub updated_at: DateTime<Utc>,
}

impl Position {
  /// Cria uma nova posição a partir de uma ordem preenchida
  pub fn from_entry_order(
    exchange: ExchangeId,
    symbol: String,
    side: PositionSide,
    quantity: Decimal,
    entry_price: Decimal,
    leverage: u32,
    entry_order_id: OrderId,
  ) -> Self {
    let now = Utc::now();
    let margin = (quantity * entry_price) / Decimal::from(leverage);

    Self {
      id: PositionId::new(),
      exchange,
      symbol,
      side,
      quantity,
      entry_price,
      current_price: entry_price,
      leverage,
      margin,
      unrealized_pnl: Decimal::ZERO,
      unrealized_pnl_pct: Decimal::ZERO,
      realized_pnl: Decimal::ZERO,
      stop_loss_order_id: None,
      stop_loss_price: None,
      take_profit_order_id: None,
      take_profit_price: None,
      entry_order_id,
      status: PositionStatus::Open,
      liquidation_price: None,
      opened_at: now,
      closed_at: None,
      updated_at: now,
    }
  }

  /// Atualiza o preço atual e recalcula o P&L
  pub fn update_price(&mut self, new_price: Decimal) {
    self.current_price = new_price;
    self.updated_at = Utc::now();

    // Calcula P&L baseado no lado da posição
    let price_diff = match self.side {
      PositionSide::Long => new_price - self.entry_price,
      PositionSide::Short => self.entry_price - new_price,
    };

    self.unrealized_pnl = price_diff * self.quantity;

    // Calcula P&L em percentual
    if self.entry_price != Decimal::ZERO {
      self.unrealized_pnl_pct = (price_diff / self.entry_price) * Decimal::from(100);
      // Ajusta para alavancagem
      self.unrealized_pnl_pct *= Decimal::from(self.leverage);
    }
  }

  /// Verifica se a posição deve ser liquidada
  pub fn should_liquidate(&self) -> bool {
    if let Some(liq_price) = self.liquidation_price {
      match self.side {
        PositionSide::Long => self.current_price <= liq_price,
        PositionSide::Short => self.current_price >= liq_price,
      }
    } else {
      false
    }
  }

  /// Verifica se o stop loss foi atingido
  pub fn should_stop_loss(&self) -> bool {
    if let Some(stop_price) = self.stop_loss_price {
      match self.side {
        PositionSide::Long => self.current_price <= stop_price,
        PositionSide::Short => self.current_price >= stop_price,
      }
    } else {
      false
    }
  }

  /// Verifica se o take profit foi atingido
  pub fn should_take_profit(&self) -> bool {
    if let Some(tp_price) = self.take_profit_price {
      match self.side {
        PositionSide::Long => self.current_price >= tp_price,
        PositionSide::Short => self.current_price <= tp_price,
      }
    } else {
      false
    }
  }

  /// Valor total da posição (nocional)
  pub fn notional_value(&self) -> Decimal {
    self.quantity * self.current_price
  }

  /// ROI (Return on Investment) baseado na margem
  pub fn roi(&self) -> Decimal {
    if self.margin != Decimal::ZERO {
      (self.unrealized_pnl / self.margin) * Decimal::from(100)
    } else {
      Decimal::ZERO
    }
  }
}

/// Saldo de um ativo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
  /// Ativo
  pub asset: String,
  /// Saldo livre (disponível para trading)
  pub free: Decimal,
  /// Saldo bloqueado (em ordens abertas)
  pub locked: Decimal,
}

impl Balance {
  /// Saldo total (livre + bloqueado)
  pub fn total(&self) -> Decimal {
    self.free + self.locked
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rust_decimal_macros::dec;

  #[test]
  fn test_position_pnl_long() {
    let mut position = Position::from_entry_order(
      ExchangeId::Paper,
      "BTCUSDT".to_string(),
      PositionSide::Long,
      dec!(0.1),
      dec!(40000),
      5,
      OrderId::new(),
    );

    // Preço subiu para 44000 (+10%)
    position.update_price(dec!(44000));

    assert_eq!(position.unrealized_pnl, dec!(400)); // 0.1 * 4000
    assert_eq!(position.unrealized_pnl_pct, dec!(50)); // 10% * 5x leverage
  }

  #[test]
  fn test_position_pnl_short() {
    let mut position = Position::from_entry_order(
      ExchangeId::Paper,
      "BTCUSDT".to_string(),
      PositionSide::Short,
      dec!(0.1),
      dec!(40000),
      5,
      OrderId::new(),
    );

    // Preço caiu para 36000 (-10%)
    position.update_price(dec!(36000));

    assert_eq!(position.unrealized_pnl, dec!(400)); // 0.1 * 4000
    assert_eq!(position.unrealized_pnl_pct, dec!(50)); // 10% * 5x leverage
  }

  #[test]
  fn test_position_stop_loss() {
    let mut position = Position::from_entry_order(
      ExchangeId::Paper,
      "BTCUSDT".to_string(),
      PositionSide::Long,
      dec!(0.1),
      dec!(40000),
      5,
      OrderId::new(),
    );

    position.stop_loss_price = Some(dec!(38000));
    position.update_price(dec!(39000));
    assert!(!position.should_stop_loss());

    position.update_price(dec!(37500));
    assert!(position.should_stop_loss());
  }
}
