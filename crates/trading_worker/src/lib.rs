//! # robotrade-trading-worker
//!
//! Worker de execução de trades e coleta de dados:
//! - Scheduler de tarefas periódicas
//! - Coleta de dados de mercado
//! - Fila de jobs (sinais → ordens)
//! - Gestão de posições
//! - Risk management
//! - Circuit breaker

pub mod collector;
pub mod scheduler;
pub mod job_queue;
pub mod position_manager;
pub mod risk;
pub mod circuit_breaker;

// Re-exports
pub use collector::DataCollector;
pub use scheduler::{Scheduler, SchedulerConfig, SchedulerEvent};
pub use job_queue::{Job, JobPayload, JobPriority, JobQueue, JobStatus, JobType};
pub use position_manager::{PositionManager, PositionManagerConfig, PositionEvent};
pub use risk::{RiskManager, RiskConfig, RiskCheckResult, RiskRejectionReason};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState, CircuitBreakerEvent, FailureType};
