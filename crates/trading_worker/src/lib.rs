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

pub mod collector;
pub mod scheduler;
pub mod job_queue;
pub mod position_manager;
pub mod risk;
pub mod circuit_breaker;
pub mod ws_manager;
pub mod sync_engine;
pub mod ledger_service;
pub mod pnl_calculator;
pub mod reconciliation_service;

// Re-exports
pub use collector::DataCollector;
pub use scheduler::{Scheduler, SchedulerConfig, SchedulerEvent};
pub use job_queue::{Job, JobPayload, JobPriority, JobQueue, JobStatus, JobType};
pub use position_manager::{PositionManager, PositionManagerConfig, PositionEvent};
pub use risk::{RiskManager, RiskConfig, RiskCheckResult, RiskRejectionReason};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState, CircuitBreakerEvent, FailureType};
pub use ws_manager::{
    WsManager, WsManagerConfig, WsManagerHandle, WsManagerState,
    NormalizedEvent, Exchange, WsConnectionState,
    BalanceUpdateEvent, PositionUpdateEvent, OrderUpdateEvent, TradeEvent,
    AccountLogEvent, ConnectionStateEvent, ErrorEvent,
    PositionSide, OrderSide, OrderStatus, AccountLogType,
};
pub use sync_engine::{
    SyncEngine, SyncEngineHandle, SyncConfig, SyncDataType, SyncStatus,
    SyncState, SyncProgressEvent, SyncDataStore, ExchangeSyncAdapter,
    SyncedTrade, SyncedOrder, SyncedPosition, SyncedBalance,
    SyncedDeposit, SyncedWithdrawal, SyncedFunding,
};
pub use ledger_service::{
    LedgerService, LedgerStore, LedgerEntry, LedgerEntryType,
    LedgerFilters, LedgerSummary, AssetBalance,
};
pub use pnl_calculator::{
    PnLCalculator, TaxLotStore, CostBasisMethod,
    TaxLot, RealizedPnL, UnrealizedPnL, PnLSummary,
    PositionSide as PnLPositionSide, AcquisitionType,
};
pub use reconciliation_service::{
    ReconciliationService, ReconciliationConfig, ReconciliationStore,
    ReconciliationSnapshot, ReconciliationStatus, ReconciliationHealth,
    Discrepancy, DiscrepancyType, BalanceSnapshot, HealthStatus,
    BalanceFetcher, LedgerBalanceFetcher,
};
