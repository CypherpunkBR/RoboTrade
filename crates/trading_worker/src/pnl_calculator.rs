//! P&L Calculator
//!
//! Calculates realized and unrealized P&L using different cost basis methods:
//! - FIFO (First In, First Out)
//! - Average Cost
//!
//! Supports:
//! - Per-trade P&L calculation
//! - Position-level P&L tracking
//! - Tax lot management
//! - Short/long-term gain classification

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use super::ws_manager::Exchange;

/// Cost basis calculation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CostBasisMethod {
  /// First In, First Out
  #[default]
  FIFO,
  /// Last In, First Out
  LIFO,
  /// Average Cost
  AverageCost,
}

/// Tax lot (a unit of acquisition for cost basis tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLot {
  pub id: String,
  pub exchange: Exchange,
  pub symbol: String,
  pub side: PositionSide,
  pub acquired_quantity: Decimal,
  pub remaining_quantity: Decimal,
  pub cost_per_unit: Decimal,
  pub total_cost: Decimal,
  pub acquired_at: DateTime<Utc>,
  pub acquisition_type: AcquisitionType,
  pub trade_id: String,
  pub is_closed: bool,
}

impl TaxLot {
  /// Create a new tax lot from a trade
  pub fn new(
    exchange: Exchange,
    symbol: String,
    side: PositionSide,
    quantity: Decimal,
    price: Decimal,
    acquired_at: DateTime<Utc>,
    trade_id: String,
  ) -> Self {
    Self {
      id: Uuid::new_v4().to_string(),
      exchange,
      symbol,
      side,
      acquired_quantity: quantity,
      remaining_quantity: quantity,
      cost_per_unit: price,
      total_cost: quantity * price,
      acquired_at,
      acquisition_type: AcquisitionType::Trade,
      trade_id,
      is_closed: false,
    }
  }

  /// Check if this lot is long-term (held > 1 year)
  pub fn is_long_term(&self, sale_date: DateTime<Utc>) -> bool {
    sale_date - self.acquired_at > Duration::days(365)
  }
}

/// Position side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionSide {
  Long,
  Short,
}

/// How the position was acquired
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquisitionType {
  Trade,
  Transfer,
  Airdrop,
  Other,
}

/// Realized P&L from closing a position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealizedPnL {
  pub id: String,
  pub exchange: Exchange,
  pub symbol: String,
  pub quantity: Decimal,
  pub proceeds: Decimal,
  pub cost_basis: Decimal,
  pub gain_loss: Decimal,
  pub is_short_term: bool,
  pub is_long_term: bool,
  pub acquired_at: DateTime<Utc>,
  pub disposed_at: DateTime<Utc>,
  pub lot_id: String,
  pub close_trade_id: String,
  pub method: CostBasisMethod,
}

/// Unrealized P&L for an open position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnL {
  pub exchange: Exchange,
  pub symbol: String,
  pub side: PositionSide,
  pub quantity: Decimal,
  pub cost_basis: Decimal,
  pub avg_cost_per_unit: Decimal,
  pub current_price: Decimal,
  pub current_value: Decimal,
  pub unrealized_gain_loss: Decimal,
  pub unrealized_pct: Decimal,
  pub oldest_lot_date: Option<DateTime<Utc>>,
  pub short_term_quantity: Decimal,
  pub long_term_quantity: Decimal,
}

/// P&L summary for a period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnLSummary {
  pub exchange: Exchange,
  pub symbol: Option<String>,
  pub period_start: DateTime<Utc>,
  pub period_end: DateTime<Utc>,
  pub total_realized: Decimal,
  pub short_term_gains: Decimal,
  pub short_term_losses: Decimal,
  pub long_term_gains: Decimal,
  pub long_term_losses: Decimal,
  pub net_short_term: Decimal,
  pub net_long_term: Decimal,
  pub total_proceeds: Decimal,
  pub total_cost_basis: Decimal,
  pub num_trades: u64,
}

/// Trait for tax lot persistence
#[async_trait::async_trait]
pub trait TaxLotStore: Send + Sync {
  /// Insert a new tax lot
  async fn insert_lot(&self, lot: &TaxLot) -> Result<(), String>;

  /// Update an existing tax lot
  async fn update_lot(&self, lot: &TaxLot) -> Result<(), String>;

  /// Get open lots for a symbol (ordered by acquisition date for FIFO)
  async fn get_open_lots(
    &self,
    exchange: Exchange,
    symbol: &str,
    side: PositionSide,
  ) -> Vec<TaxLot>;

  /// Get open lots ordered for LIFO (newest first)
  async fn get_open_lots_lifo(
    &self,
    exchange: Exchange,
    symbol: &str,
    side: PositionSide,
  ) -> Vec<TaxLot>;

  /// Save realized P&L record
  async fn save_realized_pnl(&self, pnl: &RealizedPnL) -> Result<(), String>;

  /// Get realized P&L for a period
  async fn get_realized_pnl(
    &self,
    exchange: Option<Exchange>,
    symbol: Option<&str>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
  ) -> Vec<RealizedPnL>;

  /// Get total open quantity for a position
  async fn get_total_open_quantity(
    &self,
    exchange: Exchange,
    symbol: &str,
    side: PositionSide,
  ) -> Decimal;

  /// Get total cost basis for open lots
  async fn get_total_cost_basis(
    &self,
    exchange: Exchange,
    symbol: &str,
    side: PositionSide,
  ) -> Decimal;

  /// Get average cost per unit for open lots
  async fn get_average_cost(&self, exchange: Exchange, symbol: &str, side: PositionSide)
    -> Decimal;
}

/// P&L Calculator
pub struct PnLCalculator<S: TaxLotStore> {
  store: S,
  method: CostBasisMethod,
}

impl<S: TaxLotStore> PnLCalculator<S> {
  /// Create a new P&L calculator
  pub fn new(store: S, method: CostBasisMethod) -> Self {
    Self { store, method }
  }

  /// Change the cost basis method
  pub fn set_method(&mut self, method: CostBasisMethod) {
    self.method = method;
  }

  /// Get current cost basis method
  pub fn method(&self) -> CostBasisMethod {
    self.method
  }

  /// Process an opening trade (buy for long, sell for short)
  pub async fn process_open(
    &self,
    exchange: Exchange,
    symbol: String,
    side: PositionSide,
    quantity: Decimal,
    price: Decimal,
    timestamp: DateTime<Utc>,
    trade_id: String,
  ) -> Result<TaxLot, String> {
    let lot = TaxLot::new(
      exchange,
      symbol.clone(),
      side,
      quantity,
      price,
      timestamp,
      trade_id,
    );

    self.store.insert_lot(&lot).await?;

    info!(
      "Created tax lot {} for {} {} {} @ {}",
      lot.id,
      quantity,
      symbol,
      if side == PositionSide::Long {
        "LONG"
      } else {
        "SHORT"
      },
      price
    );

    Ok(lot)
  }

  /// Process a closing trade (sell for long, buy for short)
  pub async fn process_close(
    &self,
    exchange: Exchange,
    symbol: String,
    side: PositionSide,
    quantity: Decimal,
    price: Decimal,
    timestamp: DateTime<Utc>,
    trade_id: String,
  ) -> Result<Vec<RealizedPnL>, String> {
    // Get open lots based on method
    let mut lots = match self.method {
      CostBasisMethod::FIFO | CostBasisMethod::AverageCost => {
        self.store.get_open_lots(exchange, &symbol, side).await
      }
      CostBasisMethod::LIFO => self.store.get_open_lots_lifo(exchange, &symbol, side).await,
    };

    if lots.is_empty() {
      // No open lots to close - this might be opening the opposite side
      return Err(format!(
        "No open {} lots to close for {}",
        if side == PositionSide::Long {
          "LONG"
        } else {
          "SHORT"
        },
        symbol
      ));
    }

    let mut remaining = quantity;
    let mut realized_pnls = Vec::new();

    match self.method {
      CostBasisMethod::FIFO | CostBasisMethod::LIFO => {
        // Use specific lot identification
        for lot in lots.iter_mut() {
          if remaining <= Decimal::ZERO {
            break;
          }

          let consume_qty = remaining.min(lot.remaining_quantity);
          remaining -= consume_qty;

          // Calculate P&L for this portion
          let cost_basis = consume_qty * lot.cost_per_unit;
          let proceeds = consume_qty * price;
          let gain_loss = if side == PositionSide::Long {
            proceeds - cost_basis
          } else {
            cost_basis - proceeds // Short: profit when price drops
          };

          let is_long_term = lot.is_long_term(timestamp);

          let pnl = RealizedPnL {
            id: Uuid::new_v4().to_string(),
            exchange,
            symbol: symbol.clone(),
            quantity: consume_qty,
            proceeds,
            cost_basis,
            gain_loss,
            is_short_term: !is_long_term,
            is_long_term,
            acquired_at: lot.acquired_at,
            disposed_at: timestamp,
            lot_id: lot.id.clone(),
            close_trade_id: trade_id.clone(),
            method: self.method,
          };

          // Update lot
          lot.remaining_quantity -= consume_qty;
          lot.is_closed = lot.remaining_quantity <= Decimal::ZERO;
          self.store.update_lot(lot).await?;

          // Save realized P&L
          self.store.save_realized_pnl(&pnl).await?;
          realized_pnls.push(pnl);

          debug!(
            "Closed {} from lot {} - P&L: {}",
            consume_qty, lot.id, gain_loss
          );
        }
      }
      CostBasisMethod::AverageCost => {
        // Use average cost for all lots
        let avg_cost = self.store.get_average_cost(exchange, &symbol, side).await;

        let cost_basis = quantity * avg_cost;
        let proceeds = quantity * price;
        let gain_loss = if side == PositionSide::Long {
          proceeds - cost_basis
        } else {
          cost_basis - proceeds
        };

        // For average cost, we still need to close lots proportionally
        // and track holding period for tax purposes
        let total_open = self
          .store
          .get_total_open_quantity(exchange, &symbol, side)
          .await;

        for lot in lots.iter_mut() {
          if remaining <= Decimal::ZERO {
            break;
          }

          // Proportional closing
          let lot_proportion = lot.remaining_quantity / total_open;
          let consume_qty = (quantity * lot_proportion)
            .min(lot.remaining_quantity)
            .min(remaining);
          remaining -= consume_qty;

          let lot_cost_basis = consume_qty * avg_cost;
          let lot_proceeds = consume_qty * price;
          let lot_gain_loss = if side == PositionSide::Long {
            lot_proceeds - lot_cost_basis
          } else {
            lot_cost_basis - lot_proceeds
          };

          let is_long_term = lot.is_long_term(timestamp);

          let pnl = RealizedPnL {
            id: Uuid::new_v4().to_string(),
            exchange,
            symbol: symbol.clone(),
            quantity: consume_qty,
            proceeds: lot_proceeds,
            cost_basis: lot_cost_basis,
            gain_loss: lot_gain_loss,
            is_short_term: !is_long_term,
            is_long_term,
            acquired_at: lot.acquired_at,
            disposed_at: timestamp,
            lot_id: lot.id.clone(),
            close_trade_id: trade_id.clone(),
            method: self.method,
          };

          lot.remaining_quantity -= consume_qty;
          lot.is_closed = lot.remaining_quantity <= Decimal::ZERO;
          self.store.update_lot(lot).await?;
          self.store.save_realized_pnl(&pnl).await?;
          realized_pnls.push(pnl);
        }
      }
    }

    if remaining > Decimal::ZERO {
      return Err(format!(
        "Could not fully close {} {} - {} remaining",
        quantity, symbol, remaining
      ));
    }

    Ok(realized_pnls)
  }

  /// Calculate unrealized P&L for a position
  pub async fn calculate_unrealized(
    &self,
    exchange: Exchange,
    symbol: &str,
    side: PositionSide,
    current_price: Decimal,
  ) -> Option<UnrealizedPnL> {
    let lots = self.store.get_open_lots(exchange, symbol, side).await;

    if lots.is_empty() {
      return None;
    }

    let mut total_quantity = Decimal::ZERO;
    let mut total_cost = Decimal::ZERO;
    let mut oldest_date: Option<DateTime<Utc>> = None;
    let mut short_term_qty = Decimal::ZERO;
    let mut long_term_qty = Decimal::ZERO;
    let now = Utc::now();

    for lot in &lots {
      total_quantity += lot.remaining_quantity;
      total_cost += lot.remaining_quantity * lot.cost_per_unit;

      if oldest_date.is_none() || lot.acquired_at < oldest_date.unwrap() {
        oldest_date = Some(lot.acquired_at);
      }

      if lot.is_long_term(now) {
        long_term_qty += lot.remaining_quantity;
      } else {
        short_term_qty += lot.remaining_quantity;
      }
    }

    let current_value = total_quantity * current_price;
    let unrealized_gain_loss = if side == PositionSide::Long {
      current_value - total_cost
    } else {
      total_cost - current_value
    };

    let avg_cost = if total_quantity > Decimal::ZERO {
      total_cost / total_quantity
    } else {
      Decimal::ZERO
    };

    let unrealized_pct = if total_cost > Decimal::ZERO {
      (unrealized_gain_loss / total_cost) * dec!(100)
    } else {
      Decimal::ZERO
    };

    Some(UnrealizedPnL {
      exchange,
      symbol: symbol.to_string(),
      side,
      quantity: total_quantity,
      cost_basis: total_cost,
      avg_cost_per_unit: avg_cost,
      current_price,
      current_value,
      unrealized_gain_loss,
      unrealized_pct,
      oldest_lot_date: oldest_date,
      short_term_quantity: short_term_qty,
      long_term_quantity: long_term_qty,
    })
  }

  /// Get P&L summary for a period
  pub async fn get_summary(
    &self,
    exchange: Option<Exchange>,
    symbol: Option<&str>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
  ) -> PnLSummary {
    let pnls = self
      .store
      .get_realized_pnl(exchange, symbol, start_time, end_time)
      .await;

    let mut summary = PnLSummary {
      exchange: exchange.unwrap_or(Exchange::Binance),
      symbol: symbol.map(String::from),
      period_start: start_time,
      period_end: end_time,
      total_realized: Decimal::ZERO,
      short_term_gains: Decimal::ZERO,
      short_term_losses: Decimal::ZERO,
      long_term_gains: Decimal::ZERO,
      long_term_losses: Decimal::ZERO,
      net_short_term: Decimal::ZERO,
      net_long_term: Decimal::ZERO,
      total_proceeds: Decimal::ZERO,
      total_cost_basis: Decimal::ZERO,
      num_trades: pnls.len() as u64,
    };

    for pnl in &pnls {
      summary.total_realized += pnl.gain_loss;
      summary.total_proceeds += pnl.proceeds;
      summary.total_cost_basis += pnl.cost_basis;

      if pnl.is_short_term {
        if pnl.gain_loss > Decimal::ZERO {
          summary.short_term_gains += pnl.gain_loss;
        } else {
          summary.short_term_losses += pnl.gain_loss.abs();
        }
      }

      if pnl.is_long_term {
        if pnl.gain_loss > Decimal::ZERO {
          summary.long_term_gains += pnl.gain_loss;
        } else {
          summary.long_term_losses += pnl.gain_loss.abs();
        }
      }
    }

    summary.net_short_term = summary.short_term_gains - summary.short_term_losses;
    summary.net_long_term = summary.long_term_gains - summary.long_term_losses;

    summary
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tax_lot_is_long_term() {
    let lot = TaxLot::new(
      Exchange::Binance,
      "BTCUSDT".to_string(),
      PositionSide::Long,
      dec!(1.0),
      dec!(50000),
      Utc::now() - Duration::days(400),
      "trade1".to_string(),
    );

    assert!(lot.is_long_term(Utc::now()));

    let recent_lot = TaxLot::new(
      Exchange::Binance,
      "BTCUSDT".to_string(),
      PositionSide::Long,
      dec!(1.0),
      dec!(50000),
      Utc::now() - Duration::days(100),
      "trade2".to_string(),
    );

    assert!(!recent_lot.is_long_term(Utc::now()));
  }

  #[test]
  fn test_cost_basis_method_default() {
    let method = CostBasisMethod::default();
    assert_eq!(method, CostBasisMethod::FIFO);
  }
}
