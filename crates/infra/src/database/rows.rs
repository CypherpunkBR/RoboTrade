//! Tipos de linha do SQLite para mapeamento com SQLx
//!
//! Estes tipos representam as linhas raw do banco de dados e são convertidos
//! para as entidades de domínio do crate core.

use sqlx::FromRow;

// ============================================================================
// Market Data Rows
// ============================================================================

/// Linha da tabela exchanges
#[derive(Debug, Clone, FromRow)]
pub struct ExchangeRow {
    pub id: String,
    pub name: String,
    pub exchange_type: String,
    pub api_base_url: Option<String>,
    pub ws_base_url: Option<String>,
    pub status: String,
    pub rate_limit_requests: i64,
    pub rate_limit_orders: i64,
    pub supported_features: Option<String>, // JSON
    pub metadata: Option<String>,           // JSON
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela symbols
#[derive(Debug, Clone, FromRow)]
pub struct SymbolRow {
    pub id: i64,
    pub exchange_id: String,
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub symbol_type: String,
    pub status: String,
    pub price_precision: i64,
    pub quantity_precision: i64,
    pub quote_precision: i64,
    pub min_quantity: String,
    pub max_quantity: Option<String>,
    pub min_notional: String,
    pub max_notional: Option<String>,
    pub tick_size: String,
    pub step_size: String,
    pub contract_type: Option<String>,
    pub contract_size: Option<String>,
    pub margin_asset: Option<String>,
    pub maintenance_margin_rate: Option<String>,
    pub max_leverage: Option<i64>,
    pub maker_fee: String,
    pub taker_fee: String,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela candles
#[derive(Debug, Clone, FromRow)]
pub struct CandleRow {
    pub id: i64,
    pub symbol_id: i64,
    pub timeframe: String,
    pub open_time: i64,
    pub close_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub quote_volume: String,
    pub trade_count: Option<i64>,
    pub taker_buy_volume: Option<String>,
    pub taker_buy_quote_volume: Option<String>,
    pub vwap: Option<String>,
    pub typical_price: Option<String>,
    pub created_at: String,
}

/// Linha da tabela tickers
#[derive(Debug, Clone, FromRow)]
pub struct TickerRow {
    pub id: i64,
    pub symbol_id: i64,
    pub bid_price: String,
    pub bid_qty: Option<String>,
    pub ask_price: String,
    pub ask_qty: Option<String>,
    pub last_price: String,
    pub price_change: Option<String>,
    pub price_change_pct: Option<String>,
    pub weighted_avg_price: Option<String>,
    pub high_price: Option<String>,
    pub low_price: Option<String>,
    pub volume: Option<String>,
    pub quote_volume: Option<String>,
    pub funding_rate: Option<String>,
    pub next_funding_time: Option<i64>,
    pub open_interest: Option<String>,
    pub timestamp: i64,
    pub created_at: String,
}

/// Linha da tabela market_indicators
#[derive(Debug, Clone, FromRow)]
pub struct MarketIndicatorRow {
    pub id: i64,
    pub indicator_type: String,
    pub provider: String,
    pub value: String,
    pub value_classification: Option<String>,
    pub secondary_value: Option<String>,
    pub timestamp: i64,
    pub valid_until: Option<i64>,
    pub metadata: Option<String>,
    pub created_at: String,
}

/// Linha da tabela technical_indicators
#[derive(Debug, Clone, FromRow)]
pub struct TechnicalIndicatorRow {
    pub id: i64,
    pub symbol_id: i64,
    pub timeframe: String,
    pub timestamp: i64,
    pub indicator_name: String,
    pub indicator_params: String, // JSON
    pub value_1: String,
    pub value_2: Option<String>,
    pub value_3: Option<String>,
    pub value_4: Option<String>,
    pub created_at: String,
}

// ============================================================================
// Strategy Rows
// ============================================================================

/// Linha da tabela strategies
#[derive(Debug, Clone, FromRow)]
pub struct StrategyRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub strategy_type: String,
    pub default_symbols: Option<String>,    // JSON
    pub default_timeframes: Option<String>, // JSON
    pub is_enabled: i64,
    pub is_live_allowed: i64,
    pub author: Option<String>,
    pub tags: Option<String>,
    pub documentation_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela strategy_versions
#[derive(Debug, Clone, FromRow)]
pub struct StrategyVersionRow {
    pub id: String,
    pub strategy_id: String,
    pub version: String,
    pub version_major: i64,
    pub version_minor: i64,
    pub version_patch: i64,
    pub config: String,              // JSON
    pub indicators_config: Option<String>, // JSON
    pub entry_rules: String,         // JSON
    pub exit_rules: String,          // JSON
    pub filters: Option<String>,     // JSON
    pub risk_config: String,         // JSON
    pub is_active: i64,
    pub change_description: String,
    pub parent_version_id: Option<String>,
    pub performance_metrics: Option<String>, // JSON
    pub created_at: String,
    pub created_by: String,
}

/// Linha da tabela strategy_instances
#[derive(Debug, Clone, FromRow)]
pub struct StrategyInstanceRow {
    pub id: String,
    pub strategy_version_id: String,
    pub symbol_id: i64,
    pub trading_mode: String,
    pub timeframe: String,
    pub config_overrides: Option<String>, // JSON
    pub status: String,
    pub last_evaluation_at: Option<String>,
    pub last_signal_at: Option<String>,
    pub last_error: Option<String>,
    pub error_count: i64,
    pub signals_generated: i64,
    pub trades_executed: i64,
    pub started_at: Option<String>,
    pub stopped_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela signals
#[derive(Debug, Clone, FromRow)]
pub struct SignalRow {
    pub id: String,
    pub strategy_instance_id: String,
    pub symbol_id: i64,
    pub signal_type: String,
    pub direction: String,
    pub strength: String,
    pub entry_price: Option<String>,
    pub stop_loss: Option<String>,
    pub take_profit_1: Option<String>,
    pub take_profit_2: Option<String>,
    pub take_profit_3: Option<String>,
    pub trailing_stop_pct: Option<String>,
    pub suggested_quantity: Option<String>,
    pub suggested_risk_pct: Option<String>,
    pub risk_reward_ratio: Option<String>,
    pub primary_reason: String,
    pub secondary_reasons: Option<String>, // JSON
    pub indicator_values: Option<String>,  // JSON
    pub market_context: Option<String>,    // JSON
    pub status: String,
    pub confirmation_required: i64,
    pub confirmed_at: Option<String>,
    pub order_id: Option<String>,
    pub position_id: Option<String>,
    pub valid_from: String,
    pub valid_until: Option<String>,
    pub executed_at: Option<String>,
    pub candle_open_time: Option<i64>,
    pub metadata: Option<String>,
    pub created_at: String,
}

// ============================================================================
// Order Rows
// ============================================================================

/// Linha da tabela orders
#[derive(Debug, Clone, FromRow)]
pub struct OrderRow {
    pub id: String,
    pub exchange_id: String,
    pub symbol_id: i64,
    pub exchange_order_id: Option<String>,
    pub client_order_id: String,
    pub side: String,
    pub order_type: String,
    pub time_in_force: String,
    pub quantity: String,
    pub executed_qty: String,
    pub remaining_qty: Option<String>,
    pub price: Option<String>,
    pub stop_price: Option<String>,
    pub avg_fill_price: Option<String>,
    pub reduce_only: i64,
    pub close_position: i64,
    pub position_side: Option<String>,
    pub status: String,
    pub reject_reason: Option<String>,
    pub commission: String,
    pub commission_asset: Option<String>,
    pub signal_id: Option<String>,
    pub position_id: Option<String>,
    pub parent_order_id: Option<String>,
    pub order_group_id: Option<String>,
    pub order_group_type: Option<String>,
    pub trading_mode: String,
    pub submitted_at: Option<String>,
    pub acknowledged_at: Option<String>,
    pub last_fill_at: Option<String>,
    pub completed_at: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela order_fills
#[derive(Debug, Clone, FromRow)]
pub struct OrderFillRow {
    pub id: i64,
    pub order_id: String,
    pub fill_id: Option<String>,
    pub price: String,
    pub quantity: String,
    pub quote_quantity: Option<String>,
    pub commission: String,
    pub commission_asset: String,
    pub is_maker: Option<i64>,
    pub filled_at: String,
    pub created_at: String,
}

// ============================================================================
// Position Rows
// ============================================================================

/// Linha da tabela positions
#[derive(Debug, Clone, FromRow)]
pub struct PositionRow {
    pub id: String,
    pub exchange_id: String,
    pub symbol_id: i64,
    pub side: String,
    pub status: String,
    pub quantity: String,
    pub initial_quantity: String,
    pub entry_price: String,
    pub entry_value: String,
    pub entry_order_id: Option<String>,
    pub exit_price: Option<String>,
    pub exit_value: Option<String>,
    pub exit_order_id: Option<String>,
    pub leverage: i64,
    pub margin_type: String,
    pub initial_margin: Option<String>,
    pub maintenance_margin: Option<String>,
    pub liquidation_price: Option<String>,
    pub stop_loss_price: Option<String>,
    pub stop_loss_order_id: Option<String>,
    pub take_profit_price: Option<String>,
    pub take_profit_order_id: Option<String>,
    pub trailing_stop_pct: Option<String>,
    pub trailing_stop_activation: Option<String>,
    pub break_even_price: Option<String>,
    pub unrealized_pnl: String,
    pub realized_pnl: String,
    pub total_fees: String,
    pub funding_fees: String,
    pub max_profit: Option<String>,
    pub max_drawdown: Option<String>,
    pub risk_amount: Option<String>,
    pub signal_id: Option<String>,
    pub strategy_instance_id: Option<String>,
    pub trading_mode: String,
    pub opened_at: String,
    pub closed_at: Option<String>,
    pub duration_seconds: Option<i64>,
    pub notes: Option<String>,
    pub tags: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela trades
#[derive(Debug, Clone, FromRow)]
pub struct TradeRow {
    pub id: String,
    pub position_id: String,
    pub exchange_id: String,
    pub symbol_id: i64,
    pub side: String,
    pub quantity: String,
    pub entry_price: String,
    pub entry_value: String,
    pub entry_time: String,
    pub entry_order_id: Option<String>,
    pub exit_price: String,
    pub exit_value: String,
    pub exit_time: String,
    pub exit_order_id: Option<String>,
    pub exit_reason: Option<String>,
    pub gross_pnl: String,
    pub total_fees: String,
    pub funding_fees: String,
    pub net_pnl: String,
    pub return_pct: String,
    pub r_multiple: Option<String>,
    pub duration_seconds: i64,
    pub max_favorable_excursion: Option<String>,
    pub max_adverse_excursion: Option<String>,
    pub strategy_instance_id: Option<String>,
    pub strategy_version_id: Option<String>,
    pub signal_id: Option<String>,
    pub trading_mode: String,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub quality_score: Option<i64>,
    pub metadata: Option<String>,
    pub created_at: String,
}

// ============================================================================
// Account Rows
// ============================================================================

/// Linha da tabela accounts
#[derive(Debug, Clone, FromRow)]
pub struct AccountRow {
    pub id: String,
    pub exchange_id: String,
    pub account_type: String,
    pub alias: Option<String>,
    pub api_key_ref: Option<String>,
    pub has_trade_permission: i64,
    pub has_withdraw_permission: i64,
    pub ip_whitelist: Option<String>,
    pub status: String,
    pub last_sync_at: Option<String>,
    pub sync_error: Option<String>,
    pub default_leverage: i64,
    pub margin_type: String,
    pub position_mode: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela balance_snapshots
#[derive(Debug, Clone, FromRow)]
pub struct BalanceSnapshotRow {
    pub id: i64,
    pub account_id: String,
    pub snapshot_type: String,
    pub total_balance_usdt: String,
    pub available_balance_usdt: String,
    pub locked_balance_usdt: String,
    pub total_margin: Option<String>,
    pub used_margin: Option<String>,
    pub available_margin: Option<String>,
    pub margin_level: Option<String>,
    pub unrealized_pnl: String,
    pub assets: String, // JSON
    pub timestamp: String,
}

/// Linha da tabela daily_performance
#[derive(Debug, Clone, FromRow)]
pub struct DailyPerformanceRow {
    pub id: i64,
    pub date: String,
    pub trading_mode: String,
    pub starting_balance: String,
    pub ending_balance: String,
    pub gross_pnl: String,
    pub fees: String,
    pub funding: String,
    pub net_pnl: String,
    pub return_pct: String,
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub total_win_amount: String,
    pub total_loss_amount: String,
    pub largest_win: Option<String>,
    pub largest_loss: Option<String>,
    pub win_rate: Option<String>,
    pub profit_factor: Option<String>,
    pub max_drawdown: Option<String>,
    pub by_symbol: Option<String>,   // JSON
    pub by_strategy: Option<String>, // JSON
    pub created_at: String,
    pub updated_at: String,
}

// ============================================================================
// Risk Rows
// ============================================================================

/// Linha da tabela risk_limits
#[derive(Debug, Clone, FromRow)]
pub struct RiskLimitRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub applies_to: String,
    pub exchange_id: Option<String>,
    pub symbol_id: Option<i64>,
    pub strategy_id: Option<String>,
    pub max_position_size: Option<String>,
    pub max_position_value: Option<String>,
    pub max_positions_per_symbol: Option<i64>,
    pub max_total_positions: Option<i64>,
    pub max_total_exposure: Option<String>,
    pub max_leverage: Option<i64>,
    pub max_loss_per_trade: Option<String>,
    pub max_loss_per_trade_pct: Option<String>,
    pub max_daily_loss: Option<String>,
    pub max_daily_loss_pct: Option<String>,
    pub max_weekly_loss: Option<String>,
    pub max_drawdown_pct: Option<String>,
    pub max_order_size: Option<String>,
    pub max_orders_per_minute: Option<i64>,
    pub consecutive_loss_limit: Option<i64>,
    pub pause_after_losses_minutes: Option<i64>,
    pub is_enabled: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela risk_events
#[derive(Debug, Clone, FromRow)]
pub struct RiskEventRow {
    pub id: i64,
    pub event_type: String,
    pub severity: String,
    pub risk_limit_id: Option<String>,
    pub account_id: Option<String>,
    pub symbol_id: Option<i64>,
    pub position_id: Option<String>,
    pub order_id: Option<String>,
    pub limit_name: Option<String>,
    pub limit_value: Option<String>,
    pub actual_value: Option<String>,
    pub breach_pct: Option<String>,
    pub action_taken: Option<String>,
    pub resolved: i64,
    pub resolved_at: Option<String>,
    pub resolution_notes: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

// ============================================================================
// Backtest Rows
// ============================================================================

/// Linha da tabela backtests
#[derive(Debug, Clone, FromRow)]
pub struct BacktestRow {
    pub id: String,
    pub strategy_version_id: String,
    pub symbol_id: i64,
    pub timeframe: String,
    pub start_date: String,
    pub end_date: String,
    pub initial_capital: String,
    pub commission_rate: String,
    pub slippage_bps: i64,
    pub fill_model: String,
    pub position_sizing: String,
    pub status: String,
    pub progress_pct: i64,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub execution_time_ms: Option<i64>,
    pub final_capital: Option<String>,
    pub total_return: Option<String>,
    pub total_return_pct: Option<String>,
    pub annualized_return: Option<String>,
    pub sharpe_ratio: Option<String>,
    pub sortino_ratio: Option<String>,
    pub max_drawdown: Option<String>,
    pub max_drawdown_pct: Option<String>,
    pub total_trades: Option<i64>,
    pub winning_trades: Option<i64>,
    pub losing_trades: Option<i64>,
    pub win_rate: Option<String>,
    pub profit_factor: Option<String>,
    pub avg_trade_pnl: Option<String>,
    pub largest_win: Option<String>,
    pub largest_loss: Option<String>,
    pub equity_curve: Option<String>,   // JSON
    pub monthly_returns: Option<String>, // JSON
    pub trades_json: Option<String>,     // JSON
    pub metadata: Option<String>,
    pub created_at: String,
}

// ============================================================================
// Job Rows
// ============================================================================

/// Linha da tabela jobs
#[derive(Debug, Clone, FromRow)]
pub struct JobRow {
    pub id: String,
    pub job_type: String,
    pub queue: String,
    pub priority: i64,
    pub payload: String, // JSON
    pub scheduled_for: String,
    pub not_before: Option<String>,
    pub deadline: Option<String>,
    pub status: String,
    pub attempts: i64,
    pub max_attempts: i64,
    pub worker_id: Option<String>,
    pub locked_at: Option<String>,
    pub lock_expires_at: Option<String>,
    pub result: Option<String>,    // JSON
    pub last_error: Option<String>,
    pub error_count: i64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub execution_time_ms: Option<i64>,
    pub depends_on: Option<String>, // JSON
    pub idempotency_key: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela scheduled_tasks
#[derive(Debug, Clone, FromRow)]
pub struct ScheduledTaskRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub job_type: String,
    pub payload_template: String, // JSON
    pub queue: String,
    pub priority: i64,
    pub schedule_type: String,
    pub cron_expression: Option<String>,
    pub interval_seconds: Option<i64>,
    pub timezone: String,
    pub is_enabled: i64,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub last_job_id: Option<String>,
    pub max_concurrent: i64,
    pub timeout_seconds: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela job_history
#[derive(Debug, Clone, FromRow)]
pub struct JobHistoryRow {
    pub id: i64,
    pub job_id: String,
    pub event_type: String,
    pub status_before: Option<String>,
    pub status_after: String,
    pub worker_id: Option<String>,
    pub error: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

// ============================================================================
// Notification Rows
// ============================================================================

/// Linha da tabela alert_definitions
#[derive(Debug, Clone, FromRow)]
pub struct AlertDefinitionRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub alert_type: String,
    pub conditions: String, // JSON
    pub symbol_id: Option<i64>,
    pub strategy_id: Option<String>,
    pub notification_channels: String, // JSON
    pub message_template: Option<String>,
    pub is_enabled: i64,
    pub is_recurring: i64,
    pub cooldown_seconds: i64,
    pub last_triggered_at: Option<String>,
    pub trigger_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela notifications
#[derive(Debug, Clone, FromRow)]
pub struct NotificationRow {
    pub id: i64,
    pub notification_type: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub alert_definition_id: Option<String>,
    pub signal_id: Option<String>,
    pub order_id: Option<String>,
    pub position_id: Option<String>,
    pub channels_sent: Option<String>,
    pub is_read: i64,
    pub read_at: Option<String>,
    pub is_dismissed: i64,
    pub dismissed_at: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

// ============================================================================
// Audit & Config Rows
// ============================================================================

/// Linha da tabela audit_log
#[derive(Debug, Clone, FromRow)]
pub struct AuditLogRow {
    pub id: i64,
    pub event_category: String,
    pub event_type: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub action: String,
    pub changes: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: i64,
    pub error_message: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

/// Linha da tabela app_logs
#[derive(Debug, Clone, FromRow)]
pub struct AppLogRow {
    pub id: i64,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: Option<String>,
    pub span_id: Option<String>,
    pub span_name: Option<String>,
    pub parent_span_id: Option<String>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub backtrace: Option<String>,
    pub timestamp: String,
}

/// Linha da tabela config_store
#[derive(Debug, Clone, FromRow)]
pub struct ConfigStoreRow {
    pub key: String,
    pub value: String,
    pub value_type: String,
    pub category: String,
    pub description: Option<String>,
    pub is_secret: i64,
    pub is_readonly: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela user_preferences
#[derive(Debug, Clone, FromRow)]
pub struct UserPreferencesRow {
    pub key: String,
    pub value: String,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Linha da tabela schema_migrations
#[derive(Debug, Clone, FromRow)]
pub struct SchemaMigrationRow {
    pub version: String,
    pub name: String,
    pub applied_at: String,
}

/// Linha da tabela system_metrics
#[derive(Debug, Clone, FromRow)]
pub struct SystemMetricRow {
    pub id: i64,
    pub metric_name: String,
    pub metric_value: String,
    pub tags: Option<String>,
    pub timestamp: String,
}

/// Linha da tabela sync_status
#[derive(Debug, Clone, FromRow)]
pub struct SyncStatusRow {
    pub id: String,
    pub sync_type: String,
    pub last_sync_at: Option<String>,
    pub last_sync_status: Option<String>,
    pub last_error: Option<String>,
    pub last_id: Option<String>,
    pub last_timestamp: Option<i64>,
    pub records_synced: i64,
    pub sync_duration_ms: Option<i64>,
    pub next_sync_at: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ============================================================================
// View Rows (para consultas agregadas)
// ============================================================================

/// Resultado da view v_open_positions
#[derive(Debug, Clone, FromRow)]
pub struct OpenPositionViewRow {
    pub id: String,
    pub exchange_id: String,
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub side: String,
    pub quantity: String,
    pub entry_price: String,
    pub stop_loss_price: Option<String>,
    pub take_profit_price: Option<String>,
    pub leverage: i64,
    pub unrealized_pnl: String,
    pub strategy_name: Option<String>,
    pub trading_mode: String,
    pub opened_at: String,
}

/// Resultado da view v_trade_stats_by_symbol
#[derive(Debug, Clone, FromRow)]
pub struct TradeStatsBySymbolRow {
    pub symbol: String,
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub win_rate: Option<String>,
    pub total_pnl: String,
    pub avg_pnl: Option<String>,
    pub largest_win: Option<String>,
    pub largest_loss: Option<String>,
}

/// Resultado da view v_daily_pnl
#[derive(Debug, Clone, FromRow)]
pub struct DailyPnlRow {
    pub date: String,
    pub trading_mode: String,
    pub trades: i64,
    pub gross_pnl: String,
    pub fees: String,
    pub net_pnl: String,
}

/// Resultado da view v_risk_status
#[derive(Debug, Clone, FromRow)]
pub struct RiskStatusViewRow {
    pub limit_name: String,
    pub applies_to: String,
    pub limit_type: String,
    pub limit_value: Option<String>,
    pub breach_count: i64,
    pub last_breach: Option<String>,
}
