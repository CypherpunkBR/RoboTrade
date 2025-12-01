//! Risk Validator - Pure Business Logic

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
  pub max_daily_loss: Decimal,
  pub max_total_exposure: Decimal,
  pub max_positions: usize,
  pub max_leverage: u32,
}

impl Default for RiskLimits {
  fn default() -> Self {
    Self {
      max_daily_loss: dec!(1000.0),
      max_total_exposure: dec!(50000.0),
      max_positions: 5,
      max_leverage: 10,
    }
  }
}

#[derive(Debug, Clone)]
pub struct RiskState {
  pub daily_loss: Decimal,
  pub total_exposure: Decimal,
  pub open_positions_count: usize,
}

#[derive(Debug, Clone)]
pub struct OrderValidationRequest {
  pub symbol: String,
  pub quantity: Decimal,
  pub price: Decimal,
  pub leverage: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskCheckResult {
  Approved,
  Rejected(RiskRejectionReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskRejectionReason {
  DailyLossLimitReached,
  MaxExposureExceeded,
  MaxPositionsReached,
  LeverageTooHigh,
  InvalidInput(String),
}

pub struct RiskValidator;

impl RiskValidator {
  pub fn validate_order(
    order: &OrderValidationRequest,
    state: &RiskState,
    limits: &RiskLimits,
  ) -> RiskCheckResult {
    if order.quantity <= dec!(0) || order.price <= dec!(0) {
      return RiskCheckResult::Rejected(RiskRejectionReason::InvalidInput(
        "Invalid quantity or price".to_string(),
      ));
    }

    if state.daily_loss.abs() >= limits.max_daily_loss {
      return RiskCheckResult::Rejected(RiskRejectionReason::DailyLossLimitReached);
    }

    if state.open_positions_count >= limits.max_positions {
      return RiskCheckResult::Rejected(RiskRejectionReason::MaxPositionsReached);
    }

    if order.leverage > limits.max_leverage {
      return RiskCheckResult::Rejected(RiskRejectionReason::LeverageTooHigh);
    }

    let order_notional = order.quantity * order.price * Decimal::from(order.leverage);
    if state.total_exposure + order_notional > limits.max_total_exposure {
      return RiskCheckResult::Rejected(RiskRejectionReason::MaxExposureExceeded);
    }

    RiskCheckResult::Approved
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_approve_valid_order() {
    let order = OrderValidationRequest {
      symbol: "BTCUSDT".to_string(),
      quantity: dec!(0.01),
      price: dec!(50000.0),
      leverage: 5,
    };

    let state = RiskState {
      daily_loss: dec!(0),
      total_exposure: dec!(5000.0),
      open_positions_count: 1,
    };

    let result = RiskValidator::validate_order(&order, &state, &RiskLimits::default());
    assert_eq!(result, RiskCheckResult::Approved);
  }

  #[test]
  fn test_reject_daily_loss() {
    let order = OrderValidationRequest {
      symbol: "BTCUSDT".to_string(),
      quantity: dec!(0.01),
      price: dec!(50000.0),
      leverage: 5,
    };

    let state = RiskState {
      daily_loss: dec!(-1500.0),
      total_exposure: dec!(5000.0),
      open_positions_count: 1,
    };

    let result = RiskValidator::validate_order(&order, &state, &RiskLimits::default());
    assert_eq!(
      result,
      RiskCheckResult::Rejected(RiskRejectionReason::DailyLossLimitReached)
    );
  }

  #[test]
  fn test_reject_max_positions() {
    let order = OrderValidationRequest {
      symbol: "BTCUSDT".to_string(),
      quantity: dec!(0.01),
      price: dec!(50000.0),
      leverage: 5,
    };

    let state = RiskState {
      daily_loss: dec!(0),
      total_exposure: dec!(5000.0),
      open_positions_count: 5,
    };

    let result = RiskValidator::validate_order(&order, &state, &RiskLimits::default());
    assert_eq!(
      result,
      RiskCheckResult::Rejected(RiskRejectionReason::MaxPositionsReached)
    );
  }

  #[test]
  fn test_reject_leverage() {
    let order = OrderValidationRequest {
      symbol: "BTCUSDT".to_string(),
      quantity: dec!(0.01),
      price: dec!(50000.0),
      leverage: 20,
    };

    let state = RiskState {
      daily_loss: dec!(0),
      total_exposure: dec!(5000.0),
      open_positions_count: 1,
    };

    let result = RiskValidator::validate_order(&order, &state, &RiskLimits::default());
    assert_eq!(
      result,
      RiskCheckResult::Rejected(RiskRejectionReason::LeverageTooHigh)
    );
  }

  #[test]
  fn test_reject_exposure() {
    let order = OrderValidationRequest {
      symbol: "BTCUSDT".to_string(),
      quantity: dec!(10.0),
      price: dec!(50000.0),
      leverage: 10,
    };

    let state = RiskState {
      daily_loss: dec!(0),
      total_exposure: dec!(45000.0),
      open_positions_count: 1,
    };

    let limits = RiskLimits {
      max_total_exposure: dec!(50000.0),
      ..Default::default()
    };

    let result = RiskValidator::validate_order(&order, &state, &limits);
    assert_eq!(
      result,
      RiskCheckResult::Rejected(RiskRejectionReason::MaxExposureExceeded)
    );
  }

  #[test]
  fn test_reject_invalid_input() {
    let order = OrderValidationRequest {
      symbol: "BTCUSDT".to_string(),
      quantity: dec!(0),
      price: dec!(50000.0),
      leverage: 5,
    };

    let state = RiskState {
      daily_loss: dec!(0),
      total_exposure: dec!(5000.0),
      open_positions_count: 1,
    };

    let result = RiskValidator::validate_order(&order, &state, &RiskLimits::default());
    assert!(matches!(
      result,
      RiskCheckResult::Rejected(RiskRejectionReason::InvalidInput(_))
    ));
  }
}
