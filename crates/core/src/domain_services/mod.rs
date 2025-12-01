//! Domain Services
//!
//! Pure business logic layer - NO IO operations.
//! All services here are completely testable without databases, HTTP, or filesystem.
//!
//! These services implement the core business rules and calculations,
//! following Clean Architecture principles where domain logic is independent
//! of infrastructure concerns.

pub mod pnl;
pub mod position;
pub mod reconciliation;
pub mod risk;

// Re-exports for convenience
pub use pnl::{
  CloseResult, CostBasisMethod, PnLCalculator, PnLError, RealizedPnL, TaxLot, UnrealizedPnL,
};
