//! Entidades de domínio do RoboTrade
//!
//! Contém todos os tipos fundamentais usados no sistema de trading.

mod account;
mod asset;
mod backtest;
mod candle;
mod cost_basis;
mod exchange;
mod fear_greed;
mod job;
mod ledger;
mod notification;
mod order;
mod pnl;
mod position;
mod price_alert;
mod reconciliation;
mod risk;
mod signal;
mod strategy;
mod sync_state;
mod trade;

// Re-exports de Account
pub use account::{
    Account, AccountId, AccountStatus, AccountType, AssetBalance, BalanceChange, BalanceSnapshot,
    MarginType, PositionMode, SnapshotType,
};

// Re-exports de Asset
pub use asset::{Asset, ExchangeId, TradingPair};

// Re-exports de Backtest
pub use backtest::{
    Backtest, BacktestComparison, BacktestConfig, BacktestId, BacktestResults, BacktestStatus,
    EquityPoint, FillModel, MonthlyReturn, PositionSizing,
};

// Re-exports de Candle
pub use candle::{Candle, Ticker, TimeFrame};

// Re-exports de Exchange
pub use exchange::{
    Exchange, ExchangeStatus, ExchangeType, Symbol, SymbolStatus, SymbolType,
};

// Re-exports de FearGreed
pub use fear_greed::{FearGreedClassification, FearGreedData, FearGreedHistory, FearGreedTrend};

// Re-exports de Job
pub use job::{
    Job, JobId, JobPriority, JobStatus, JobType, ScheduleType, ScheduledTask, ScheduledTaskId,
};

// Re-exports de Notification
pub use notification::{
    AlertCondition, AlertDefinition, AlertDefinitionId, AlertType, ConditionOperator,
    Notification, NotificationChannel, NotificationSeverity, NotificationSummary, NotificationType,
};

// Re-exports de Order
pub use order::{
    Order, OrderFill, OrderId, OrderRequest, OrderSide, OrderSource, OrderStatus, OrderType,
    TimeInForce,
};

// Re-exports de Position
pub use position::{Balance, Position, PositionId, PositionSide, PositionStatus};

// Re-exports de Risk
pub use risk::{
    RiskAction, RiskEvent, RiskEventType, RiskLimit, RiskLimitId, RiskLimitScope, RiskSeverity,
    RiskStatus,
};

// Re-exports de Signal
pub use signal::{Signal, SignalId, SignalStatus, SignalStrength, SignalType, TradeDirection};

// Re-exports de Strategy
pub use strategy::{
    StrategyDefinition, StrategyId, StrategyInstance, StrategyInstanceId, StrategyInstanceStatus,
    StrategyRiskConfig, StrategyType, StrategyVersion, StrategyVersionId, TradingMode,
};

// Re-exports de Trade
pub use trade::{Trade, TradeCloseReason, TradeId, TradeStats};

// Re-exports de PriceAlert
pub use price_alert::{
    AlertCondition as PriceAlertCondition, AlertStatus as PriceAlertStatus,
    NotificationType as PriceAlertNotificationType, PriceAlert, PriceAlertId,
};

// Re-exports de Ledger
pub use ledger::{
    AssetBalanceSummary, LedgerEntry, LedgerEntryId, LedgerEntryType, LedgerFilters, ReferenceType,
};

// Re-exports de CostBasis
pub use cost_basis::{
    AcquisitionType, ConsumedLot, CostBasisLot, CostBasisLotId, CostBasisMethod,
};

// Re-exports de PnL (tipos consolidados)
pub use pnl::{PnLSummary, RealizedPnL, RealizedPnLId, UnrealizedPnL};

// Re-exports de Reconciliation
pub use reconciliation::{
    Discrepancy, ReconciliationResult, ReconciliationSnapshot, ReconciliationSnapshotId,
    ReconciliationStatus, SnapshotType as ReconciliationSnapshotType,
};

// Re-exports de SyncState
pub use sync_state::{
    SyncDataType, SyncProgress, SyncReport, SyncState, SyncStateId, SyncStatus,
};
