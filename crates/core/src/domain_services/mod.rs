//! Domain Services - Pure Business Logic (NO IO)

pub mod pnl;
pub mod position;
pub mod reconciliation;
pub mod risk;

// Re-exports
pub use pnl::{
  CloseResult, CostBasisMethod, PnLCalculator, PnLError, RealizedPnL, TaxLot, UnrealizedPnL,
};
pub use risk::{
  OrderValidationRequest, RiskCheckResult, RiskLimits, RiskRejectionReason, RiskState,
  RiskValidator,
};
