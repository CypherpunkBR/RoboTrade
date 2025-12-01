//! P&L Domain Services
//!
//! Pure business logic for Profit & Loss calculation.
//! All modules here contain NO IO - completely testable.

pub mod calculator;

// Re-exports
pub use calculator::{
  CloseResult, CostBasisMethod, PnLCalculator, PnLError, RealizedPnL, TaxLot, UnrealizedPnL,
};
