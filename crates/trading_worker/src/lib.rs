//! # robotrade-trading-worker
//!
//! Worker de execução de trades e coleta de dados:
//! - Scheduler de tarefas periódicas
//! - Coleta de dados de mercado
//! - Fila de jobs (sinais → ordens)
//! - Gestão de posições
//! - Risk management
//! - Circuit breaker
//! - WebSocket management (multi-exchange)
//! - Historical sync engine

pub mod circuit_breaker;
pub mod collector;
pub mod job_queue;
pub mod ledger_service;
pub mod pnl_calculator;
pub mod position_manager;
pub mod reconciliation_service;
pub mod risk;
pub mod scheduler;
pub mod sync_engine;
pub mod ws_manager;

// Re-exports
pub use circuit_breaker::{
  CircuitBreaker, CircuitBreakerConfig, CircuitBreakerEvent, CircuitState, FailureType,
};
pub use collector::DataCollector;
pub use job_queue::{Job, JobPayload, JobPriority, JobQueue, JobStatus, JobType};
pub use ledger_service::{
  AssetBalance, LedgerEntry, LedgerEntryType, LedgerFilters, LedgerService, LedgerStore,
  LedgerSummary,
};
pub use pnl_calculator::{
  AcquisitionType, CostBasisMethod, PnLCalculator, PnLSummary, PositionSide as PnLPositionSide,
  RealizedPnL, TaxLot, TaxLotStore, UnrealizedPnL,
};
pub use position_manager::{PositionEvent, PositionManager, PositionManagerConfig};
pub use reconciliation_service::{
  BalanceFetcher, BalanceSnapshot, Discrepancy, DiscrepancyType, HealthStatus,
  LedgerBalanceFetcher, ReconciliationConfig, ReconciliationHealth, ReconciliationService,
  ReconciliationSnapshot, ReconciliationStatus, ReconciliationStore,
};
pub use risk::{RiskCheckResult, RiskConfig, RiskManager, RiskRejectionReason};
pub use scheduler::{Scheduler, SchedulerConfig, SchedulerEvent};
pub use sync_engine::{
  ExchangeSyncAdapter, SyncConfig, SyncDataStore, SyncDataType, SyncEngine, SyncEngineHandle,
  SyncProgressEvent, SyncState, SyncStatus, SyncedBalance, SyncedDeposit, SyncedFunding,
  SyncedOrder, SyncedPosition, SyncedTrade, SyncedWithdrawal,
};
pub use ws_manager::{
  AccountLogEvent, AccountLogType, BalanceUpdateEvent, ConnectionStateEvent, ErrorEvent, Exchange,
  NormalizedEvent, OrderSide, OrderStatus, OrderUpdateEvent, PositionSide, PositionUpdateEvent,
  TradeEvent, WsConnectionState, WsManager, WsManagerConfig, WsManagerHandle, WsManagerState,
};
