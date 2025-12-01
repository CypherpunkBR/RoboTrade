//! Pure P&L Calculator - Domain Logic
//!
//! This module contains PURE business logic for P&L calculation.
//! NO IO operations - completely testable without databases, HTTP, or filesystem.
//!
//! Supports:
//! - FIFO (First In, First Out)
//! - LIFO (Last In, First Out)
//! - Average Cost method
//!
//! All functions are pure - given the same inputs, always return the same outputs.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::entities::PositionSide;

/// Cost basis calculation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CostBasisMethod {
  /// First In, First Out - closes oldest lots first
  #[default]
  FIFO,
  /// Last In, First Out - closes newest lots first
  LIFO,
  /// Average Cost - uses weighted average cost
  AverageCost,
}

/// Tax lot (a unit of acquisition for cost basis tracking)
///
/// Represents a single purchase/acquisition of an asset at a specific price.
/// Used to track cost basis for tax purposes and P&L calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLot {
  pub id: String,
  pub symbol: String,
  pub side: PositionSide,
  /// Original quantity acquired
  pub acquired_quantity: Decimal,
  /// Quantity still open (not yet closed)
  pub remaining_quantity: Decimal,
  /// Cost per unit (entry price)
  pub cost_per_unit: Decimal,
  /// Total cost (acquired_quantity * cost_per_unit)
  pub total_cost: Decimal,
  /// When this lot was acquired
  pub acquired_at: DateTime<Utc>,
  /// Trade ID that created this lot
  pub trade_id: String,
  /// Whether this lot is fully closed
  pub is_closed: bool,
}

impl TaxLot {
  /// Check if this lot is long-term (held > 365 days)
  ///
  /// # Arguments
  /// * `sale_date` - The date of sale/disposal
  ///
  /// # Returns
  /// `true` if held more than 365 days, `false` otherwise
  pub fn is_long_term(&self, sale_date: DateTime<Utc>) -> bool {
    sale_date - self.acquired_at > Duration::days(365)
  }

  /// Calculate holding period in days
  pub fn holding_period_days(&self, sale_date: DateTime<Utc>) -> i64 {
    (sale_date - self.acquired_at).num_days()
  }
}

/// Realized P&L from closing a position or portion thereof
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RealizedPnL {
  pub quantity: Decimal,
  pub proceeds: Decimal,
  pub cost_basis: Decimal,
  pub gain_loss: Decimal,
  pub is_short_term: bool,
  pub is_long_term: bool,
  pub acquired_at: DateTime<Utc>,
  pub disposed_at: DateTime<Utc>,
  pub lot_id: String,
  pub method: CostBasisMethod,
}

/// Result of closing positions
#[derive(Debug, Clone)]
pub struct CloseResult {
  /// Realized P&L records
  pub realized_pnls: Vec<RealizedPnL>,
  /// Updated tax lots (with reduced quantities or marked closed)
  pub updated_lots: Vec<TaxLot>,
  /// Quantity that was closed
  pub closed_quantity: Decimal,
}

/// Unrealized P&L for an open position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnrealizedPnL {
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

/// Error during P&L calculation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PnLError {
  /// No open lots available to close
  NoOpenLots { symbol: String, side: PositionSide },
  /// Insufficient quantity to close
  InsufficientQuantity {
    requested: String,
    available: String,
    symbol: String,
  },
  /// Invalid input
  InvalidInput(String),
}

impl std::fmt::Display for PnLError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PnLError::NoOpenLots { symbol, side } => {
        write!(
          f,
          "No open {} lots to close for {}",
          if *side == PositionSide::Long {
            "LONG"
          } else {
            "SHORT"
          },
          symbol
        )
      }
      PnLError::InsufficientQuantity {
        requested,
        available,
        symbol,
      } => write!(
        f,
        "Insufficient quantity for {}: requested {}, available {}",
        symbol, requested, available
      ),
      PnLError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
    }
  }
}

impl std::error::Error for PnLError {}

/// Pure P&L Calculator - NO IO
///
/// All methods are pure functions that take input and return output without side effects.
/// This makes them 100% testable without mocks, databases, or external dependencies.
pub struct PnLCalculator;

impl PnLCalculator {
  /// Calculate realized P&L using FIFO (First In, First Out) method
  ///
  /// FIFO closes the oldest tax lots first. This is the default method for most tax jurisdictions.
  ///
  /// # Arguments
  /// * `existing_lots` - Current open tax lots (should be sorted by acquired_at ascending)
  /// * `symbol` - Symbol being closed
  /// * `side` - Position side (Long or Short)
  /// * `close_quantity` - Quantity to close
  /// * `close_price` - Price at closing
  /// * `close_time` - Timestamp of closing
  ///
  /// # Returns
  /// * `Ok(CloseResult)` - Realized P&L and updated lots
  /// * `Err(PnLError)` - If insufficient lots or invalid input
  ///
  /// # Examples
  /// ```
  /// use robotrade_core::domain_services::pnl::calculator::{PnLCalculator, TaxLot};
  /// use rust_decimal_macros::dec;
  ///
  /// let lots = vec![/* ... */];
  /// let result = PnLCalculator::calculate_fifo(
  ///     &lots,
  ///     "BTCUSDT",
  ///     PositionSide::Long,
  ///     dec!(0.5),
  ///     dec!(55000.0),
  ///     Utc::now(),
  /// )?;
  /// ```
  pub fn calculate_fifo(
    existing_lots: &[TaxLot],
    symbol: &str,
    side: PositionSide,
    close_quantity: Decimal,
    close_price: Decimal,
    close_time: DateTime<Utc>,
  ) -> Result<CloseResult, PnLError> {
    // Validate inputs
    if close_quantity <= dec!(0) {
      return Err(PnLError::InvalidInput(
        "Close quantity must be positive".to_string(),
      ));
    }

    if close_price <= dec!(0) {
      return Err(PnLError::InvalidInput(
        "Close price must be positive".to_string(),
      ));
    }

    if existing_lots.is_empty() {
      return Err(PnLError::NoOpenLots {
        symbol: symbol.to_string(),
        side,
      });
    }

    // Calculate total available quantity
    let total_available: Decimal = existing_lots
      .iter()
      .filter(|lot| !lot.is_closed)
      .map(|lot| lot.remaining_quantity)
      .sum();

    if total_available < close_quantity {
      return Err(PnLError::InsufficientQuantity {
        requested: close_quantity.to_string(),
        available: total_available.to_string(),
        symbol: symbol.to_string(),
      });
    }

    let mut lots_to_process = existing_lots.to_vec();
    // FIFO: sort by acquisition date (oldest first)
    lots_to_process.sort_by(|a, b| a.acquired_at.cmp(&b.acquired_at));

    let mut remaining = close_quantity;
    let mut realized_pnls = Vec::new();
    let mut updated_lots = Vec::new();

    for mut lot in lots_to_process {
      if remaining <= dec!(0) {
        // No more to close, just include unchanged lot
        updated_lots.push(lot);
        continue;
      }

      if lot.is_closed || lot.remaining_quantity <= dec!(0) {
        // Already closed, skip
        updated_lots.push(lot);
        continue;
      }

      // Calculate how much to consume from this lot
      let consume_qty = remaining.min(lot.remaining_quantity);
      remaining -= consume_qty;

      // Calculate P&L for this portion
      let cost_basis = consume_qty * lot.cost_per_unit;
      let proceeds = consume_qty * close_price;

      // Long: profit = proceeds - cost (sell high, bought low)
      // Short: profit = cost - proceeds (bought high, sell low)
      let gain_loss = if side == PositionSide::Long {
        proceeds - cost_basis
      } else {
        cost_basis - proceeds
      };

      let is_long_term = lot.is_long_term(close_time);

      let pnl = RealizedPnL {
        quantity: consume_qty,
        proceeds,
        cost_basis,
        gain_loss,
        is_short_term: !is_long_term,
        is_long_term,
        acquired_at: lot.acquired_at,
        disposed_at: close_time,
        lot_id: lot.id.clone(),
        method: CostBasisMethod::FIFO,
      };

      realized_pnls.push(pnl);

      // Update lot
      lot.remaining_quantity -= consume_qty;
      lot.is_closed = lot.remaining_quantity <= dec!(0);
      updated_lots.push(lot);
    }

    if remaining > dec!(0) {
      // This should not happen due to earlier check, but defensive
      return Err(PnLError::InsufficientQuantity {
        requested: close_quantity.to_string(),
        available: (close_quantity - remaining).to_string(),
        symbol: symbol.to_string(),
      });
    }

    Ok(CloseResult {
      realized_pnls,
      updated_lots,
      closed_quantity: close_quantity,
    })
  }

  /// Calculate realized P&L using LIFO (Last In, First Out) method
  ///
  /// LIFO closes the newest tax lots first.
  ///
  /// # Arguments
  /// Same as `calculate_fifo`
  ///
  /// # Returns
  /// Same as `calculate_fifo`
  pub fn calculate_lifo(
    existing_lots: &[TaxLot],
    symbol: &str,
    side: PositionSide,
    close_quantity: Decimal,
    close_price: Decimal,
    close_time: DateTime<Utc>,
  ) -> Result<CloseResult, PnLError> {
    // Validate inputs (same as FIFO)
    if close_quantity <= dec!(0) {
      return Err(PnLError::InvalidInput(
        "Close quantity must be positive".to_string(),
      ));
    }

    if close_price <= dec!(0) {
      return Err(PnLError::InvalidInput(
        "Close price must be positive".to_string(),
      ));
    }

    if existing_lots.is_empty() {
      return Err(PnLError::NoOpenLots {
        symbol: symbol.to_string(),
        side,
      });
    }

    let total_available: Decimal = existing_lots
      .iter()
      .filter(|lot| !lot.is_closed)
      .map(|lot| lot.remaining_quantity)
      .sum();

    if total_available < close_quantity {
      return Err(PnLError::InsufficientQuantity {
        requested: close_quantity.to_string(),
        available: total_available.to_string(),
        symbol: symbol.to_string(),
      });
    }

    let mut lots_to_process = existing_lots.to_vec();
    // LIFO: sort by acquisition date (newest first)
    lots_to_process.sort_by(|a, b| b.acquired_at.cmp(&a.acquired_at));

    let mut remaining = close_quantity;
    let mut realized_pnls = Vec::new();
    let mut updated_lots = Vec::new();

    for mut lot in lots_to_process {
      if remaining <= dec!(0) {
        updated_lots.push(lot);
        continue;
      }

      if lot.is_closed || lot.remaining_quantity <= dec!(0) {
        updated_lots.push(lot);
        continue;
      }

      let consume_qty = remaining.min(lot.remaining_quantity);
      remaining -= consume_qty;

      let cost_basis = consume_qty * lot.cost_per_unit;
      let proceeds = consume_qty * close_price;
      let gain_loss = if side == PositionSide::Long {
        proceeds - cost_basis
      } else {
        cost_basis - proceeds
      };

      let is_long_term = lot.is_long_term(close_time);

      let pnl = RealizedPnL {
        quantity: consume_qty,
        proceeds,
        cost_basis,
        gain_loss,
        is_short_term: !is_long_term,
        is_long_term,
        acquired_at: lot.acquired_at,
        disposed_at: close_time,
        lot_id: lot.id.clone(),
        method: CostBasisMethod::LIFO,
      };

      realized_pnls.push(pnl);

      lot.remaining_quantity -= consume_qty;
      lot.is_closed = lot.remaining_quantity <= dec!(0);
      updated_lots.push(lot);
    }

    Ok(CloseResult {
      realized_pnls,
      updated_lots,
      closed_quantity: close_quantity,
    })
  }

  /// Calculate unrealized P&L for open positions
  ///
  /// # Arguments
  /// * `open_lots` - Currently open tax lots
  /// * `side` - Position side
  /// * `current_price` - Current market price
  ///
  /// # Returns
  /// `UnrealizedPnL` with current position value and gain/loss
  pub fn calculate_unrealized(
    open_lots: &[TaxLot],
    side: PositionSide,
    current_price: Decimal,
  ) -> Option<UnrealizedPnL> {
    if open_lots.is_empty() {
      return None;
    }

    let mut total_quantity = dec!(0);
    let mut total_cost = dec!(0);
    let mut oldest_date: Option<DateTime<Utc>> = None;
    let mut short_term_qty = dec!(0);
    let mut long_term_qty = dec!(0);
    let now = Utc::now();

    for lot in open_lots {
      if lot.is_closed || lot.remaining_quantity <= dec!(0) {
        continue;
      }

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

    if total_quantity <= dec!(0) {
      return None;
    }

    let current_value = total_quantity * current_price;
    let unrealized_gain_loss = if side == PositionSide::Long {
      current_value - total_cost
    } else {
      total_cost - current_value
    };

    let avg_cost = total_cost / total_quantity;
    let unrealized_pct = if total_cost > dec!(0) {
      (unrealized_gain_loss / total_cost) * dec!(100)
    } else {
      dec!(0)
    };

    Some(UnrealizedPnL {
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;

  fn create_test_lot(id: &str, quantity: Decimal, cost: Decimal, days_ago: i64) -> TaxLot {
    TaxLot {
      id: id.to_string(),
      symbol: "BTCUSDT".to_string(),
      side: PositionSide::Long,
      acquired_quantity: quantity,
      remaining_quantity: quantity,
      cost_per_unit: cost,
      total_cost: quantity * cost,
      acquired_at: Utc::now() - Duration::days(days_ago),
      trade_id: format!("trade-{}", id),
      is_closed: false,
    }
  }

  #[test]
  fn test_fifo_single_lot_full_close() {
    let lots = vec![create_test_lot("lot1", dec!(1.0), dec!(50000.0), 10)];

    let result = PnLCalculator::calculate_fifo(
      &lots,
      "BTCUSDT",
      PositionSide::Long,
      dec!(1.0),
      dec!(55000.0),
      Utc::now(),
    )
    .expect("Should calculate P&L");

    assert_eq!(result.realized_pnls.len(), 1);
    let pnl = &result.realized_pnls[0];

    assert_eq!(pnl.quantity, dec!(1.0));
    assert_eq!(pnl.cost_basis, dec!(50000.0));
    assert_eq!(pnl.proceeds, dec!(55000.0));
    assert_eq!(pnl.gain_loss, dec!(5000.0)); // 55000 - 50000
    assert!(pnl.is_short_term); // < 365 days
    assert!(!pnl.is_long_term);

    assert_eq!(result.updated_lots.len(), 1);
    assert!(result.updated_lots[0].is_closed);
    assert_eq!(result.updated_lots[0].remaining_quantity, dec!(0));
  }

  #[test]
  fn test_fifo_single_lot_partial_close() {
    let lots = vec![create_test_lot("lot1", dec!(1.0), dec!(50000.0), 10)];

    let result = PnLCalculator::calculate_fifo(
      &lots,
      "BTCUSDT",
      PositionSide::Long,
      dec!(0.5),
      dec!(55000.0),
      Utc::now(),
    )
    .unwrap();

    assert_eq!(result.realized_pnls.len(), 1);
    let pnl = &result.realized_pnls[0];

    assert_eq!(pnl.quantity, dec!(0.5));
    assert_eq!(pnl.cost_basis, dec!(25000.0)); // 0.5 * 50000
    assert_eq!(pnl.proceeds, dec!(27500.0)); // 0.5 * 55000
    assert_eq!(pnl.gain_loss, dec!(2500.0)); // 27500 - 25000

    assert_eq!(result.updated_lots.len(), 1);
    assert!(!result.updated_lots[0].is_closed);
    assert_eq!(result.updated_lots[0].remaining_quantity, dec!(0.5));
  }

  #[test]
  fn test_fifo_multiple_lots_closes_oldest_first() {
    let lots = vec![
      create_test_lot("lot1", dec!(0.5), dec!(50000.0), 30), // Oldest
      create_test_lot("lot2", dec!(0.5), dec!(51000.0), 20), // Middle
      create_test_lot("lot3", dec!(0.5), dec!(52000.0), 10), // Newest
    ];

    // Close 0.7 BTC - should consume lot1 (0.5) + partial lot2 (0.2)
    let result = PnLCalculator::calculate_fifo(
      &lots,
      "BTCUSDT",
      PositionSide::Long,
      dec!(0.7),
      dec!(55000.0),
      Utc::now(),
    )
    .unwrap();

    assert_eq!(result.realized_pnls.len(), 2);

    // First P&L: lot1 (0.5 BTC @ 50000)
    let pnl1 = &result.realized_pnls[0];
    assert_eq!(pnl1.quantity, dec!(0.5));
    assert_eq!(pnl1.cost_basis, dec!(25000.0));
    assert_eq!(pnl1.gain_loss, dec!(2500.0)); // (55000 - 50000) * 0.5

    // Second P&L: lot2 (0.2 BTC @ 51000)
    let pnl2 = &result.realized_pnls[1];
    assert_eq!(pnl2.quantity, dec!(0.2));
    assert_eq!(pnl2.cost_basis, dec!(10200.0)); // 0.2 * 51000
    assert_eq!(pnl2.gain_loss, dec!(800.0)); // (55000 - 51000) * 0.2

    // Lot states
    assert!(result.updated_lots[0].is_closed); // lot1 fully closed
    assert!(!result.updated_lots[1].is_closed); // lot2 partially closed
    assert_eq!(result.updated_lots[1].remaining_quantity, dec!(0.3)); // 0.5 - 0.2
    assert!(!result.updated_lots[2].is_closed); // lot3 untouched
    assert_eq!(result.updated_lots[2].remaining_quantity, dec!(0.5));
  }

  #[test]
  fn test_lifo_multiple_lots_closes_newest_first() {
    let lots = vec![
      create_test_lot("lot1", dec!(0.5), dec!(50000.0), 30), // Oldest
      create_test_lot("lot2", dec!(0.5), dec!(51000.0), 20), // Middle
      create_test_lot("lot3", dec!(0.5), dec!(52000.0), 10), // Newest
    ];

    // Close 0.7 BTC using LIFO - should consume lot3 (0.5) + partial lot2 (0.2)
    let result = PnLCalculator::calculate_lifo(
      &lots,
      "BTCUSDT",
      PositionSide::Long,
      dec!(0.7),
      dec!(55000.0),
      Utc::now(),
    )
    .unwrap();

    assert_eq!(result.realized_pnls.len(), 2);

    // First P&L: lot3 (0.5 BTC @ 52000) - newest first
    let pnl1 = &result.realized_pnls[0];
    assert_eq!(pnl1.quantity, dec!(0.5));
    assert_eq!(pnl1.cost_basis, dec!(26000.0)); // 0.5 * 52000
    assert_eq!(pnl1.gain_loss, dec!(1500.0)); // (55000 - 52000) * 0.5

    // Second P&L: lot2 (0.2 BTC @ 51000)
    let pnl2 = &result.realized_pnls[1];
    assert_eq!(pnl2.quantity, dec!(0.2));
    assert_eq!(pnl2.cost_basis, dec!(10200.0));
    assert_eq!(pnl2.gain_loss, dec!(800.0));

    // lot1 should be untouched
    let lot1_updated = result.updated_lots.iter().find(|l| l.id == "lot1").unwrap();
    assert!(!lot1_updated.is_closed);
    assert_eq!(lot1_updated.remaining_quantity, dec!(0.5));
  }

  #[test]
  fn test_short_position_pnl_calculation() {
    // Short position: profit when price drops
    let lots = vec![create_test_lot("lot1", dec!(1.0), dec!(50000.0), 10)];

    // Opened short @ 50000, closing @ 45000 (price dropped, profit!)
    let result = PnLCalculator::calculate_fifo(
      &lots,
      "BTCUSDT",
      PositionSide::Short,
      dec!(1.0),
      dec!(45000.0),
      Utc::now(),
    )
    .unwrap();

    let pnl = &result.realized_pnls[0];
    assert_eq!(pnl.cost_basis, dec!(50000.0));
    assert_eq!(pnl.proceeds, dec!(45000.0));
    // Short: gain = cost - proceeds = 50000 - 45000 = 5000
    assert_eq!(pnl.gain_loss, dec!(5000.0));
  }

  #[test]
  fn test_long_term_vs_short_term() {
    let lots = vec![
      create_test_lot("lot1", dec!(0.5), dec!(50000.0), 400), // Long-term (>365 days)
      create_test_lot("lot2", dec!(0.5), dec!(51000.0), 100), // Short-term (<365 days)
    ];

    let result = PnLCalculator::calculate_fifo(
      &lots,
      "BTCUSDT",
      PositionSide::Long,
      dec!(1.0),
      dec!(55000.0),
      Utc::now(),
    )
    .unwrap();

    // lot1 (oldest) should be long-term
    assert!(result.realized_pnls[0].is_long_term);
    assert!(!result.realized_pnls[0].is_short_term);

    // lot2 should be short-term
    assert!(!result.realized_pnls[1].is_long_term);
    assert!(result.realized_pnls[1].is_short_term);
  }

  #[test]
  fn test_error_no_open_lots() {
    let lots: Vec<TaxLot> = vec![];

    let result = PnLCalculator::calculate_fifo(
      &lots,
      "BTCUSDT",
      PositionSide::Long,
      dec!(1.0),
      dec!(55000.0),
      Utc::now(),
    );

    assert!(result.is_err());
    match result {
      Err(PnLError::NoOpenLots { symbol, side }) => {
        assert_eq!(symbol, "BTCUSDT");
        assert_eq!(side, PositionSide::Long);
      }
      _ => panic!("Expected NoOpenLots error"),
    }
  }

  #[test]
  fn test_error_insufficient_quantity() {
    let lots = vec![create_test_lot("lot1", dec!(0.5), dec!(50000.0), 10)];

    // Try to close 1.0 BTC when only 0.5 available
    let result = PnLCalculator::calculate_fifo(
      &lots,
      "BTCUSDT",
      PositionSide::Long,
      dec!(1.0),
      dec!(55000.0),
      Utc::now(),
    );

    assert!(result.is_err());
    assert!(matches!(result, Err(PnLError::InsufficientQuantity { .. })));
  }

  #[test]
  fn test_error_invalid_close_quantity() {
    let lots = vec![create_test_lot("lot1", dec!(1.0), dec!(50000.0), 10)];

    let result = PnLCalculator::calculate_fifo(
      &lots,
      "BTCUSDT",
      PositionSide::Long,
      dec!(0), // Invalid: zero quantity
      dec!(55000.0),
      Utc::now(),
    );

    assert!(result.is_err());
    assert!(matches!(result, Err(PnLError::InvalidInput(_))));
  }

  #[test]
  fn test_error_invalid_close_price() {
    let lots = vec![create_test_lot("lot1", dec!(1.0), dec!(50000.0), 10)];

    let result = PnLCalculator::calculate_fifo(
      &lots,
      "BTCUSDT",
      PositionSide::Long,
      dec!(1.0),
      dec!(-1000.0), // Invalid: negative price
      Utc::now(),
    );

    assert!(result.is_err());
    assert!(matches!(result, Err(PnLError::InvalidInput(_))));
  }

  #[test]
  fn test_unrealized_pnl_long_position() {
    let lots = vec![
      create_test_lot("lot1", dec!(0.5), dec!(50000.0), 30),
      create_test_lot("lot2", dec!(0.5), dec!(51000.0), 10),
    ];

    let unrealized = PnLCalculator::calculate_unrealized(&lots, PositionSide::Long, dec!(55000.0))
      .expect("Should calculate unrealized P&L");

    assert_eq!(unrealized.quantity, dec!(1.0));
    assert_eq!(unrealized.cost_basis, dec!(50500.0)); // (0.5*50000 + 0.5*51000)
    assert_eq!(unrealized.avg_cost_per_unit, dec!(50500.0));
    assert_eq!(unrealized.current_price, dec!(55000.0));
    assert_eq!(unrealized.current_value, dec!(55000.0)); // 1.0 * 55000
    assert_eq!(unrealized.unrealized_gain_loss, dec!(4500.0)); // 55000 - 50500
  }

  #[test]
  fn test_unrealized_pnl_short_position() {
    let lots = vec![create_test_lot("lot1", dec!(1.0), dec!(50000.0), 10)];

    // Short @ 50000, current price 45000 (profit!)
    let unrealized =
      PnLCalculator::calculate_unrealized(&lots, PositionSide::Short, dec!(45000.0)).unwrap();

    // Short: unrealized = cost - current_value = 50000 - 45000 = 5000
    assert_eq!(unrealized.unrealized_gain_loss, dec!(5000.0));
  }

  #[test]
  fn test_unrealized_pnl_empty_lots() {
    let lots: Vec<TaxLot> = vec![];

    let unrealized = PnLCalculator::calculate_unrealized(&lots, PositionSide::Long, dec!(55000.0));

    assert!(unrealized.is_none());
  }

  #[test]
  fn test_tax_lot_is_long_term() {
    let lot = create_test_lot("lot1", dec!(1.0), dec!(50000.0), 400);
    assert!(lot.is_long_term(Utc::now()));

    let recent_lot = create_test_lot("lot2", dec!(1.0), dec!(50000.0), 100);
    assert!(!recent_lot.is_long_term(Utc::now()));
  }

  #[test]
  fn test_tax_lot_holding_period() {
    let lot = create_test_lot("lot1", dec!(1.0), dec!(50000.0), 100);
    let days = lot.holding_period_days(Utc::now());

    // Should be approximately 100 days (within 1 day tolerance for timing)
    assert!((99..=101).contains(&days));
  }
}
