//! # robotrade-core
//!
//! Crate fundamental do RoboTrade contendo:
//! - Entidades de domínio (Candle, Order, Position, Signal, etc.)
//! - DTOs para comunicação entre módulos
//! - Tipos de erro unificados
//! - Traits fundamentais para abstração
//!
//! Este crate não possui dependências de outros crates internos.

pub mod dto;
pub mod entities;
pub mod error;
pub mod traits;

// Re-exports para facilitar uso
pub use entities::*;
pub use error::*;
pub use traits::*;
