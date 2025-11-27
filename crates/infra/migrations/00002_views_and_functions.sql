-- ============================================================================
-- RoboTrade Database Views & Utility Functions
-- Migration: 00002_views_and_functions.sql
-- Description: Views for common queries and computed data
-- ============================================================================

-- ============================================================================
-- SECTION 1: MARKET DATA VIEWS
-- ============================================================================

-- Latest ticker for each symbol
CREATE VIEW IF NOT EXISTS v_latest_tickers AS
SELECT
    t.*,
    s.symbol,
    s.base_asset,
    s.quote_asset,
    e.name as exchange_name
FROM tickers t
INNER JOIN (
    SELECT symbol_id, MAX(timestamp) as max_ts
    FROM tickers
    GROUP BY symbol_id
) latest ON t.symbol_id = latest.symbol_id AND t.timestamp = latest.max_ts
JOIN symbols s ON t.symbol_id = s.id
JOIN exchanges e ON s.exchange_id = e.id;

-- Latest candle for each symbol/timeframe
CREATE VIEW IF NOT EXISTS v_latest_candles AS
SELECT
    c.*,
    s.symbol,
    e.id as exchange_id
FROM candles c
INNER JOIN (
    SELECT symbol_id, timeframe, MAX(open_time) as max_time
    FROM candles
    GROUP BY symbol_id, timeframe
) latest ON c.symbol_id = latest.symbol_id
    AND c.timeframe = latest.timeframe
    AND c.open_time = latest.max_time
JOIN symbols s ON c.symbol_id = s.id
JOIN exchanges e ON s.exchange_id = e.id;

-- Latest Fear & Greed Index
CREATE VIEW IF NOT EXISTS v_latest_fear_greed AS
SELECT *
FROM market_indicators
WHERE indicator_type = 'fear_greed'
ORDER BY timestamp DESC
LIMIT 1;

-- ============================================================================
-- SECTION 2: POSITION & ORDER VIEWS
-- ============================================================================

-- Open positions with current P&L
CREATE VIEW IF NOT EXISTS v_open_positions AS
SELECT
    p.id,
    p.exchange_id,
    p.symbol_id,
    s.symbol,
    p.side,
    p.quantity,
    p.entry_price,
    p.leverage,
    p.margin_type,
    p.stop_loss_price,
    p.take_profit_price,
    p.trading_mode,
    p.opened_at,
    p.strategy_instance_id,
    st.name as strategy_name,

    -- Get latest price
    t.last_price as current_price,

    -- Calculate unrealized P&L
    CASE
        WHEN p.side = 'long' THEN
            CAST((CAST(t.last_price AS REAL) - CAST(p.entry_price AS REAL)) * CAST(p.quantity AS REAL) AS TEXT)
        ELSE
            CAST((CAST(p.entry_price AS REAL) - CAST(t.last_price AS REAL)) * CAST(p.quantity AS REAL) AS TEXT)
    END as calculated_unrealized_pnl,

    -- Calculate P&L percentage
    CASE
        WHEN p.side = 'long' THEN
            CAST(((CAST(t.last_price AS REAL) - CAST(p.entry_price AS REAL)) / CAST(p.entry_price AS REAL) * 100) AS TEXT)
        ELSE
            CAST(((CAST(p.entry_price AS REAL) - CAST(t.last_price AS REAL)) / CAST(p.entry_price AS REAL) * 100) AS TEXT)
    END as pnl_pct,

    -- Position value
    CAST(CAST(t.last_price AS REAL) * CAST(p.quantity AS REAL) AS TEXT) as position_value,

    -- Distance to stop loss (%)
    CASE
        WHEN p.stop_loss_price IS NOT NULL THEN
            CAST(ABS(CAST(t.last_price AS REAL) - CAST(p.stop_loss_price AS REAL)) / CAST(t.last_price AS REAL) * 100 AS TEXT)
    END as distance_to_sl_pct,

    -- Distance to take profit (%)
    CASE
        WHEN p.take_profit_price IS NOT NULL THEN
            CAST(ABS(CAST(p.take_profit_price AS REAL) - CAST(t.last_price AS REAL)) / CAST(t.last_price AS REAL) * 100 AS TEXT)
    END as distance_to_tp_pct

FROM positions p
JOIN symbols s ON p.symbol_id = s.id
LEFT JOIN v_latest_tickers t ON p.symbol_id = t.symbol_id
LEFT JOIN strategy_instances si ON p.strategy_instance_id = si.id
LEFT JOIN strategy_versions sv ON si.strategy_version_id = sv.id
LEFT JOIN strategies st ON sv.strategy_id = st.id
WHERE p.status = 'open';

-- Open orders summary
CREATE VIEW IF NOT EXISTS v_open_orders AS
SELECT
    o.id,
    o.exchange_id,
    o.symbol_id,
    s.symbol,
    o.side,
    o.order_type,
    o.quantity,
    o.price,
    o.stop_price,
    o.executed_qty,
    o.status,
    o.trading_mode,
    o.created_at,

    -- Order value
    CASE
        WHEN o.price IS NOT NULL THEN
            CAST(CAST(o.price AS REAL) * CAST(o.quantity AS REAL) AS TEXT)
        WHEN o.stop_price IS NOT NULL THEN
            CAST(CAST(o.stop_price AS REAL) * CAST(o.quantity AS REAL) AS TEXT)
    END as order_value,

    -- Remaining quantity
    CAST(CAST(o.quantity AS REAL) - CAST(o.executed_qty AS REAL) AS TEXT) as remaining_qty,

    -- Fill percentage
    CAST(CAST(o.executed_qty AS REAL) / CAST(o.quantity AS REAL) * 100 AS TEXT) as fill_pct

FROM orders o
JOIN symbols s ON o.symbol_id = s.id
WHERE o.status IN ('pending_new', 'new', 'partially_filled');

-- ============================================================================
-- SECTION 3: PERFORMANCE ANALYTICS VIEWS
-- ============================================================================

-- Trade statistics by symbol
CREATE VIEW IF NOT EXISTS v_trade_stats_by_symbol AS
SELECT
    symbol_id,
    s.symbol,
    trading_mode,
    COUNT(*) as total_trades,
    SUM(CASE WHEN CAST(net_pnl AS REAL) > 0 THEN 1 ELSE 0 END) as winning_trades,
    SUM(CASE WHEN CAST(net_pnl AS REAL) < 0 THEN 1 ELSE 0 END) as losing_trades,
    SUM(CASE WHEN CAST(net_pnl AS REAL) = 0 THEN 1 ELSE 0 END) as breakeven_trades,

    -- P&L
    CAST(SUM(CAST(net_pnl AS REAL)) AS TEXT) as total_pnl,
    CAST(AVG(CAST(net_pnl AS REAL)) AS TEXT) as avg_pnl,
    CAST(MAX(CAST(net_pnl AS REAL)) AS TEXT) as largest_win,
    CAST(MIN(CAST(net_pnl AS REAL)) AS TEXT) as largest_loss,

    -- Fees
    CAST(SUM(CAST(total_fees AS REAL)) AS TEXT) as total_fees,

    -- Win rate
    CAST(
        CAST(SUM(CASE WHEN CAST(net_pnl AS REAL) > 0 THEN 1 ELSE 0 END) AS REAL) /
        CAST(COUNT(*) AS REAL) * 100
    AS TEXT) as win_rate,

    -- Profit factor
    CASE
        WHEN SUM(CASE WHEN CAST(net_pnl AS REAL) < 0 THEN ABS(CAST(net_pnl AS REAL)) ELSE 0 END) > 0 THEN
            CAST(
                SUM(CASE WHEN CAST(net_pnl AS REAL) > 0 THEN CAST(net_pnl AS REAL) ELSE 0 END) /
                SUM(CASE WHEN CAST(net_pnl AS REAL) < 0 THEN ABS(CAST(net_pnl AS REAL)) ELSE 0 END)
            AS TEXT)
    END as profit_factor,

    -- Duration
    CAST(AVG(duration_seconds) / 3600.0 AS TEXT) as avg_duration_hours,

    -- Date range
    MIN(entry_time) as first_trade,
    MAX(exit_time) as last_trade

FROM trades t
JOIN symbols s ON t.symbol_id = s.id
GROUP BY symbol_id, s.symbol, trading_mode;

-- Trade statistics by strategy
CREATE VIEW IF NOT EXISTS v_trade_stats_by_strategy AS
SELECT
    st.id as strategy_id,
    st.name as strategy_name,
    t.trading_mode,
    COUNT(*) as total_trades,
    SUM(CASE WHEN CAST(t.net_pnl AS REAL) > 0 THEN 1 ELSE 0 END) as winning_trades,
    SUM(CASE WHEN CAST(t.net_pnl AS REAL) < 0 THEN 1 ELSE 0 END) as losing_trades,

    CAST(SUM(CAST(t.net_pnl AS REAL)) AS TEXT) as total_pnl,
    CAST(AVG(CAST(t.net_pnl AS REAL)) AS TEXT) as avg_pnl,

    CAST(
        CAST(SUM(CASE WHEN CAST(t.net_pnl AS REAL) > 0 THEN 1 ELSE 0 END) AS REAL) /
        CAST(COUNT(*) AS REAL) * 100
    AS TEXT) as win_rate,

    CAST(AVG(CAST(t.return_pct AS REAL)) AS TEXT) as avg_return_pct,

    MIN(t.entry_time) as first_trade,
    MAX(t.exit_time) as last_trade

FROM trades t
JOIN strategy_instances si ON t.strategy_instance_id = si.id
JOIN strategy_versions sv ON si.strategy_version_id = sv.id
JOIN strategies st ON sv.strategy_id = st.id
GROUP BY st.id, st.name, t.trading_mode;

-- Daily performance summary
CREATE VIEW IF NOT EXISTS v_daily_pnl AS
SELECT
    DATE(exit_time) as trade_date,
    trading_mode,
    COUNT(*) as trades,
    SUM(CASE WHEN CAST(net_pnl AS REAL) > 0 THEN 1 ELSE 0 END) as winners,
    SUM(CASE WHEN CAST(net_pnl AS REAL) < 0 THEN 1 ELSE 0 END) as losers,
    CAST(SUM(CAST(gross_pnl AS REAL)) AS TEXT) as gross_pnl,
    CAST(SUM(CAST(total_fees AS REAL)) AS TEXT) as fees,
    CAST(SUM(CAST(net_pnl AS REAL)) AS TEXT) as net_pnl,
    CAST(AVG(CAST(net_pnl AS REAL)) AS TEXT) as avg_trade,
    CAST(MAX(CAST(net_pnl AS REAL)) AS TEXT) as best_trade,
    CAST(MIN(CAST(net_pnl AS REAL)) AS TEXT) as worst_trade
FROM trades
GROUP BY DATE(exit_time), trading_mode
ORDER BY trade_date DESC;

-- Weekly performance summary
CREATE VIEW IF NOT EXISTS v_weekly_pnl AS
SELECT
    strftime('%Y-W%W', exit_time) as trade_week,
    trading_mode,
    COUNT(*) as trades,
    SUM(CASE WHEN CAST(net_pnl AS REAL) > 0 THEN 1 ELSE 0 END) as winners,
    CAST(SUM(CAST(net_pnl AS REAL)) AS TEXT) as net_pnl,
    CAST(
        CAST(SUM(CASE WHEN CAST(net_pnl AS REAL) > 0 THEN 1 ELSE 0 END) AS REAL) /
        CAST(COUNT(*) AS REAL) * 100
    AS TEXT) as win_rate
FROM trades
GROUP BY strftime('%Y-W%W', exit_time), trading_mode
ORDER BY trade_week DESC;

-- Monthly performance summary
CREATE VIEW IF NOT EXISTS v_monthly_pnl AS
SELECT
    strftime('%Y-%m', exit_time) as trade_month,
    trading_mode,
    COUNT(*) as trades,
    SUM(CASE WHEN CAST(net_pnl AS REAL) > 0 THEN 1 ELSE 0 END) as winners,
    SUM(CASE WHEN CAST(net_pnl AS REAL) < 0 THEN 1 ELSE 0 END) as losers,
    CAST(SUM(CAST(net_pnl AS REAL)) AS TEXT) as net_pnl,
    CAST(
        CAST(SUM(CASE WHEN CAST(net_pnl AS REAL) > 0 THEN 1 ELSE 0 END) AS REAL) /
        CAST(COUNT(*) AS REAL) * 100
    AS TEXT) as win_rate
FROM trades
GROUP BY strftime('%Y-%m', exit_time), trading_mode
ORDER BY trade_month DESC;

-- ============================================================================
-- SECTION 4: STRATEGY VIEWS
-- ============================================================================

-- Active strategy instances
CREATE VIEW IF NOT EXISTS v_active_strategies AS
SELECT
    si.id as instance_id,
    si.status,
    si.trading_mode,
    si.timeframe,
    st.id as strategy_id,
    st.name as strategy_name,
    st.strategy_type,
    sv.id as version_id,
    sv.version,
    s.id as symbol_id,
    s.symbol,
    e.id as exchange_id,
    e.name as exchange_name,
    si.signals_generated,
    si.trades_executed,
    si.last_evaluation_at,
    si.last_signal_at,
    si.error_count,
    si.started_at
FROM strategy_instances si
JOIN strategy_versions sv ON si.strategy_version_id = sv.id
JOIN strategies st ON sv.strategy_id = st.id
JOIN symbols s ON si.symbol_id = s.id
JOIN exchanges e ON s.exchange_id = e.id
WHERE si.status IN ('running', 'paused');

-- Strategy version history with performance
CREATE VIEW IF NOT EXISTS v_strategy_versions AS
SELECT
    sv.id as version_id,
    sv.strategy_id,
    st.name as strategy_name,
    sv.version,
    sv.is_active,
    sv.change_description,
    sv.created_at,
    sv.created_by,

    -- Count instances
    (SELECT COUNT(*) FROM strategy_instances si WHERE si.strategy_version_id = sv.id) as total_instances,
    (SELECT COUNT(*) FROM strategy_instances si WHERE si.strategy_version_id = sv.id AND si.status = 'running') as running_instances,

    -- Trade stats
    (SELECT COUNT(*) FROM trades t WHERE t.strategy_version_id = sv.id) as total_trades,
    (SELECT CAST(SUM(CAST(net_pnl AS REAL)) AS TEXT) FROM trades t WHERE t.strategy_version_id = sv.id) as total_pnl

FROM strategy_versions sv
JOIN strategies st ON sv.strategy_id = st.id
ORDER BY sv.strategy_id, sv.version_major DESC, sv.version_minor DESC, sv.version_patch DESC;

-- Pending signals
CREATE VIEW IF NOT EXISTS v_pending_signals AS
SELECT
    sig.id,
    sig.signal_type,
    sig.direction,
    sig.strength,
    sig.entry_price,
    sig.stop_loss,
    sig.take_profit_1,
    sig.primary_reason,
    sig.valid_until,
    sig.created_at,
    s.symbol,
    st.name as strategy_name,
    sv.version as strategy_version
FROM signals sig
JOIN strategy_instances si ON sig.strategy_instance_id = si.id
JOIN strategy_versions sv ON si.strategy_version_id = sv.id
JOIN strategies st ON sv.strategy_id = st.id
JOIN symbols s ON sig.symbol_id = s.id
WHERE sig.status = 'pending'
  AND (sig.valid_until IS NULL OR sig.valid_until > datetime('now'))
ORDER BY sig.created_at DESC;

-- ============================================================================
-- SECTION 5: RISK & EXPOSURE VIEWS
-- ============================================================================

-- Current exposure by asset
CREATE VIEW IF NOT EXISTS v_current_exposure AS
SELECT
    s.base_asset,
    p.side,
    p.trading_mode,
    COUNT(*) as position_count,
    CAST(SUM(CAST(p.quantity AS REAL)) AS TEXT) as total_quantity,
    CAST(SUM(CAST(p.quantity AS REAL) * CAST(p.entry_price AS REAL)) AS TEXT) as total_value,
    CAST(SUM(CAST(p.unrealized_pnl AS REAL)) AS TEXT) as total_unrealized_pnl
FROM positions p
JOIN symbols s ON p.symbol_id = s.id
WHERE p.status = 'open'
GROUP BY s.base_asset, p.side, p.trading_mode;

-- Risk limit status
CREATE VIEW IF NOT EXISTS v_risk_status AS
SELECT
    rl.id,
    rl.name,
    rl.applies_to,
    rl.max_daily_loss_pct,
    rl.max_positions_per_symbol,
    rl.max_total_positions,
    rl.is_enabled,

    -- Current values
    (SELECT COUNT(*) FROM positions WHERE status = 'open') as current_open_positions,

    -- Daily loss
    (SELECT CAST(COALESCE(SUM(CAST(net_pnl AS REAL)), 0) AS TEXT)
     FROM trades
     WHERE DATE(exit_time) = DATE('now')) as today_realized_pnl,

    -- Open P&L
    (SELECT CAST(COALESCE(SUM(CAST(unrealized_pnl AS REAL)), 0) AS TEXT)
     FROM positions
     WHERE status = 'open') as current_unrealized_pnl

FROM risk_limits rl
WHERE rl.is_enabled = 1;

-- ============================================================================
-- SECTION 6: JOB & SYSTEM VIEWS
-- ============================================================================

-- Job queue status
CREATE VIEW IF NOT EXISTS v_job_queue_status AS
SELECT
    queue,
    status,
    COUNT(*) as job_count,
    MIN(scheduled_for) as next_scheduled,
    AVG(attempts) as avg_attempts
FROM jobs
GROUP BY queue, status;

-- Pending jobs ordered by priority
CREATE VIEW IF NOT EXISTS v_pending_jobs AS
SELECT
    j.id,
    j.job_type,
    j.queue,
    j.priority,
    j.scheduled_for,
    j.attempts,
    j.max_attempts,
    j.created_at,
    j.last_error,

    -- Time until scheduled
    CAST((julianday(j.scheduled_for) - julianday('now')) * 24 * 60 AS INTEGER) as minutes_until_scheduled

FROM jobs j
WHERE j.status IN ('pending', 'scheduled')
  AND j.scheduled_for <= datetime('now')
ORDER BY j.priority ASC, j.scheduled_for ASC;

-- Recent job failures
CREATE VIEW IF NOT EXISTS v_recent_failures AS
SELECT
    j.id,
    j.job_type,
    j.last_error,
    j.attempts,
    j.max_attempts,
    j.completed_at,
    j.payload
FROM jobs j
WHERE j.status = 'failed'
ORDER BY j.completed_at DESC
LIMIT 100;

-- System health overview
CREATE VIEW IF NOT EXISTS v_system_health AS
SELECT
    -- Jobs
    (SELECT COUNT(*) FROM jobs WHERE status = 'pending') as pending_jobs,
    (SELECT COUNT(*) FROM jobs WHERE status = 'running') as running_jobs,
    (SELECT COUNT(*) FROM jobs WHERE status = 'failed' AND completed_at > datetime('now', '-1 hour')) as failed_jobs_1h,

    -- Positions
    (SELECT COUNT(*) FROM positions WHERE status = 'open') as open_positions,
    (SELECT CAST(COALESCE(SUM(CAST(unrealized_pnl AS REAL)), 0) AS TEXT) FROM positions WHERE status = 'open') as total_unrealized_pnl,

    -- Orders
    (SELECT COUNT(*) FROM orders WHERE status IN ('pending_new', 'new', 'partially_filled')) as open_orders,

    -- Strategies
    (SELECT COUNT(*) FROM strategy_instances WHERE status = 'running') as running_strategies,
    (SELECT COUNT(*) FROM strategy_instances WHERE status = 'error') as error_strategies,

    -- Recent activity
    (SELECT COUNT(*) FROM trades WHERE exit_time > datetime('now', '-24 hours')) as trades_24h,
    (SELECT COUNT(*) FROM signals WHERE created_at > datetime('now', '-24 hours')) as signals_24h,

    -- Risk events
    (SELECT COUNT(*) FROM risk_events WHERE resolved = 0) as unresolved_risk_events,

    -- Data freshness
    (SELECT MAX(timestamp) FROM tickers) as latest_ticker_time,
    (SELECT MAX(created_at) FROM candles) as latest_candle_time;

-- ============================================================================
-- SECTION 7: ACCOUNT & BALANCE VIEWS
-- ============================================================================

-- Latest balance per account
CREATE VIEW IF NOT EXISTS v_latest_balances AS
SELECT
    bs.*,
    a.alias as account_alias,
    e.name as exchange_name
FROM balance_snapshots bs
INNER JOIN (
    SELECT account_id, MAX(timestamp) as max_ts
    FROM balance_snapshots
    GROUP BY account_id
) latest ON bs.account_id = latest.account_id AND bs.timestamp = latest.max_ts
JOIN accounts a ON bs.account_id = a.id
JOIN exchanges e ON a.exchange_id = e.id;

-- Account summary
CREATE VIEW IF NOT EXISTS v_account_summary AS
SELECT
    a.id as account_id,
    a.alias,
    e.id as exchange_id,
    e.name as exchange_name,
    a.status,
    a.last_sync_at,

    -- Latest balance
    lb.total_balance_usdt,
    lb.available_balance_usdt,
    lb.unrealized_pnl,

    -- Positions
    (SELECT COUNT(*) FROM positions p WHERE p.exchange_id = e.id AND p.status = 'open') as open_positions,

    -- Orders
    (SELECT COUNT(*) FROM orders o WHERE o.exchange_id = e.id AND o.status IN ('pending_new', 'new', 'partially_filled')) as open_orders,

    -- Today's P&L
    (SELECT CAST(COALESCE(SUM(CAST(net_pnl AS REAL)), 0) AS TEXT)
     FROM trades t
     WHERE t.exchange_id = e.id AND DATE(t.exit_time) = DATE('now')) as today_pnl

FROM accounts a
JOIN exchanges e ON a.exchange_id = e.id
LEFT JOIN v_latest_balances lb ON lb.account_id = a.id;

-- ============================================================================
-- Record migration
-- ============================================================================

INSERT INTO schema_migrations (version, name)
VALUES ('00002', 'views_and_functions');
