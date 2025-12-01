//! # robotrade-core
//!
//! Crate fundamental do RoboTrade contendo:
//! - Entidades de domínio (Candle, Order, Position, Signal, etc.)
//! - DTOs para comunicação entre módulos
//! - Tipos de erro unificados
//! - Traits fundamentais para abstração
//! - Mensagens para comunicação entre actors
//! - Domain Services (lógica de negócio pura - sem IO)
//!
//! Este crate não possui dependências de outros crates internos.

pub mod domain_services;
pub mod dto;
pub mod entities;
pub mod error;
pub mod messages;
pub mod traits;

// Re-exports para facilitar uso
pub use entities::*;
pub use error::*;
pub use messages::*;
pub use traits::*;

// Domain services (selective re-exports to avoid conflicts)
pub use domain_services::pnl::{CloseResult, PnLCalculator, PnLError, TaxLot};
