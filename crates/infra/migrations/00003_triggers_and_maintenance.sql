-- ============================================================================
-- RoboTrade Database Triggers & Maintenance
-- Migration: 00003_triggers_and_maintenance.sql
-- Description: Triggers for auto-updates, data integrity, and maintenance
-- ============================================================================

-- ============================================================================
-- SECTION 1: TIMESTAMP TRIGGERS (auto-update updated_at)
-- ============================================================================

-- Exchanges
CREATE TRIGGER IF NOT EXISTS trg_exchanges_updated_at
AFTER UPDATE ON exchanges
FOR EACH ROW
BEGIN
    UPDATE exchanges SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Symbols
CREATE TRIGGER IF NOT EXISTS trg_symbols_updated_at
AFTER UPDATE ON symbols
FOR EACH ROW
BEGIN
    UPDATE symbols SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Strategies
CREATE TRIGGER IF NOT EXISTS trg_strategies_updated_at
AFTER UPDATE ON strategies
FOR EACH ROW
BEGIN
    UPDATE strategies SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Strategy instances
CREATE TRIGGER IF NOT EXISTS trg_strategy_instances_updated_at
AFTER UPDATE ON strategy_instances
FOR EACH ROW
BEGIN
    UPDATE strategy_instances SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Orders
CREATE TRIGGER IF NOT EXISTS trg_orders_updated_at
AFTER UPDATE ON orders
FOR EACH ROW
BEGIN
    UPDATE orders SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Positions
CREATE TRIGGER IF NOT EXISTS trg_positions_updated_at
AFTER UPDATE ON positions
FOR EACH ROW
BEGIN
    UPDATE positions SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Accounts
CREATE TRIGGER IF NOT EXISTS trg_accounts_updated_at
AFTER UPDATE ON accounts
FOR EACH ROW
BEGIN
    UPDATE accounts SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Risk limits
CREATE TRIGGER IF NOT EXISTS trg_risk_limits_updated_at
AFTER UPDATE ON risk_limits
FOR EACH ROW
BEGIN
    UPDATE risk_limits SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Alert definitions
CREATE TRIGGER IF NOT EXISTS trg_alert_definitions_updated_at
AFTER UPDATE ON alert_definitions
FOR EACH ROW
BEGIN
    UPDATE alert_definitions SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Jobs
CREATE TRIGGER IF NOT EXISTS trg_jobs_updated_at
AFTER UPDATE ON jobs
FOR EACH ROW
BEGIN
    UPDATE jobs SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Scheduled tasks
CREATE TRIGGER IF NOT EXISTS trg_scheduled_tasks_updated_at
AFTER UPDATE ON scheduled_tasks
FOR EACH ROW
BEGIN
    UPDATE scheduled_tasks SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Daily performance
CREATE TRIGGER IF NOT EXISTS trg_daily_performance_updated_at
AFTER UPDATE ON daily_performance
FOR EACH ROW
BEGIN
    UPDATE daily_performance SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Config store
CREATE TRIGGER IF NOT EXISTS trg_config_store_updated_at
AFTER UPDATE ON config_store
FOR EACH ROW
BEGIN
    UPDATE config_store SET updated_at = datetime('now') WHERE key = NEW.key;
END;

-- User preferences
CREATE TRIGGER IF NOT EXISTS trg_user_preferences_updated_at
AFTER UPDATE ON user_preferences
FOR EACH ROW
BEGIN
    UPDATE user_preferences SET updated_at = datetime('now') WHERE key = NEW.key;
END;

-- Sync status
CREATE TRIGGER IF NOT EXISTS trg_sync_status_updated_at
AFTER UPDATE ON sync_status
FOR EACH ROW
BEGIN
    UPDATE sync_status SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- ============================================================================
-- SECTION 2: ORDER LIFECYCLE TRIGGERS
-- ============================================================================

-- Log order status changes
CREATE TRIGGER IF NOT EXISTS trg_order_status_change
AFTER UPDATE OF status ON orders
WHEN OLD.status != NEW.status
BEGIN
    INSERT INTO audit_log (
        event_category,
        event_type,
        actor_type,
        target_type,
        target_id,
        action,
        changes,
        created_at
    ) VALUES (
        'trading',
        'order_status_change',
        'system',
        'order',
        NEW.id,
        'update',
        json_object(
            'status', json_object('old', OLD.status, 'new', NEW.status),
            'executed_qty', json_object('old', OLD.executed_qty, 'new', NEW.executed_qty)
        ),
        datetime('now')
    );
END;

-- Update order completed_at when filled or cancelled
CREATE TRIGGER IF NOT EXISTS trg_order_completion
AFTER UPDATE OF status ON orders
WHEN NEW.status IN ('filled', 'cancelled', 'rejected', 'expired') AND OLD.status NOT IN ('filled', 'cancelled', 'rejected', 'expired')
BEGIN
    UPDATE orders SET completed_at = datetime('now') WHERE id = NEW.id;
END;

-- ============================================================================
-- SECTION 3: POSITION LIFECYCLE TRIGGERS
-- ============================================================================

-- Log position status changes
CREATE TRIGGER IF NOT EXISTS trg_position_status_change
AFTER UPDATE OF status ON positions
WHEN OLD.status != NEW.status
BEGIN
    INSERT INTO audit_log (
        event_category,
        event_type,
        actor_type,
        target_type,
        target_id,
        action,
        changes,
        created_at
    ) VALUES (
        'trading',
        'position_status_change',
        'system',
        'position',
        NEW.id,
        'update',
        json_object(
            'status', json_object('old', OLD.status, 'new', NEW.status),
            'quantity', json_object('old', OLD.quantity, 'new', NEW.quantity),
            'unrealized_pnl', json_object('old', OLD.unrealized_pnl, 'new', NEW.unrealized_pnl)
        ),
        datetime('now')
    );
END;

-- Set closed_at and duration when position closes
CREATE TRIGGER IF NOT EXISTS trg_position_closed
AFTER UPDATE OF status ON positions
WHEN NEW.status = 'closed' AND OLD.status != 'closed'
BEGIN
    UPDATE positions SET
        closed_at = datetime('now'),
        duration_seconds = CAST((julianday('now') - julianday(opened_at)) * 86400 AS INTEGER)
    WHERE id = NEW.id;
END;

-- ============================================================================
-- SECTION 4: STRATEGY INSTANCE TRIGGERS
-- ============================================================================

-- Update strategy instance on new signal
CREATE TRIGGER IF NOT EXISTS trg_signal_created
AFTER INSERT ON signals
BEGIN
    UPDATE strategy_instances
    SET
        signals_generated = signals_generated + 1,
        last_signal_at = datetime('now')
    WHERE id = NEW.strategy_instance_id;
END;

-- Track strategy instance errors
CREATE TRIGGER IF NOT EXISTS trg_strategy_instance_error
AFTER UPDATE OF status ON strategy_instances
WHEN NEW.status = 'error' AND OLD.status != 'error'
BEGIN
    UPDATE strategy_instances
    SET error_count = error_count + 1
    WHERE id = NEW.id;
END;

-- ============================================================================
-- SECTION 5: JOB LIFECYCLE TRIGGERS
-- ============================================================================

-- Log job events
CREATE TRIGGER IF NOT EXISTS trg_job_created
AFTER INSERT ON jobs
BEGIN
    INSERT INTO job_history (job_id, event_type, status_after, created_at)
    VALUES (NEW.id, 'created', NEW.status, datetime('now'));
END;

CREATE TRIGGER IF NOT EXISTS trg_job_status_change
AFTER UPDATE OF status ON jobs
WHEN OLD.status != NEW.status
BEGIN
    INSERT INTO job_history (
        job_id,
        event_type,
        status_before,
        status_after,
        worker_id,
        error,
        created_at
    ) VALUES (
        NEW.id,
        CASE
            WHEN NEW.status = 'running' THEN 'started'
            WHEN NEW.status = 'completed' THEN 'completed'
            WHEN NEW.status = 'failed' THEN 'failed'
            WHEN NEW.status = 'cancelled' THEN 'cancelled'
            ELSE 'status_change'
        END,
        OLD.status,
        NEW.status,
        NEW.worker_id,
        NEW.last_error,
        datetime('now')
    );
END;

-- ============================================================================
-- SECTION 6: RISK EVENT TRIGGERS
-- ============================================================================

-- Create notification for critical risk events
CREATE TRIGGER IF NOT EXISTS trg_risk_event_notification
AFTER INSERT ON risk_events
WHEN NEW.severity = 'critical'
BEGIN
    INSERT INTO notifications (
        notification_type,
        severity,
        title,
        message,
        metadata,
        created_at
    ) VALUES (
        'risk',
        'critical',
        'Critical Risk Alert: ' || NEW.event_type,
        'Risk limit breached: ' || COALESCE(NEW.limit_name, NEW.event_type) ||
        '. Actual: ' || COALESCE(NEW.actual_value, 'N/A') ||
        ', Limit: ' || COALESCE(NEW.limit_value, 'N/A'),
        json_object('risk_event_id', NEW.id),
        datetime('now')
    );
END;

-- ============================================================================
-- SECTION 7: ALERT TRIGGERS
-- ============================================================================

-- Update alert trigger count
CREATE TRIGGER IF NOT EXISTS trg_alert_triggered
AFTER INSERT ON notifications
WHEN NEW.alert_definition_id IS NOT NULL
BEGIN
    UPDATE alert_definitions
    SET
        last_triggered_at = datetime('now'),
        trigger_count = trigger_count + 1
    WHERE id = NEW.alert_definition_id;
END;

-- ============================================================================
-- SECTION 8: DATA CLEANUP TRIGGERS
-- ============================================================================

-- Auto-cleanup old tickers (keep last 24 hours)
CREATE TRIGGER IF NOT EXISTS trg_cleanup_old_tickers
AFTER INSERT ON tickers
WHEN (SELECT COUNT(*) FROM tickers) > 100000
BEGIN
    DELETE FROM tickers
    WHERE timestamp < (strftime('%s', 'now') - 86400) * 1000
    AND id NOT IN (
        SELECT id FROM tickers ORDER BY timestamp DESC LIMIT 10000
    );
END;

-- Auto-cleanup old orderbook snapshots (keep last 6 hours)
CREATE TRIGGER IF NOT EXISTS trg_cleanup_old_orderbooks
AFTER INSERT ON orderbook_snapshots
WHEN (SELECT COUNT(*) FROM orderbook_snapshots) > 50000
BEGIN
    DELETE FROM orderbook_snapshots
    WHERE timestamp < (strftime('%s', 'now') - 21600) * 1000
    AND id NOT IN (
        SELECT id FROM orderbook_snapshots ORDER BY timestamp DESC LIMIT 5000
    );
END;

-- ============================================================================
-- SECTION 9: CONFIGURATION AUDIT TRIGGERS
-- ============================================================================

-- Log configuration changes
CREATE TRIGGER IF NOT EXISTS trg_config_change
AFTER UPDATE ON config_store
BEGIN
    INSERT INTO audit_log (
        event_category,
        event_type,
        actor_type,
        target_type,
        target_id,
        action,
        changes,
        created_at
    ) VALUES (
        'config',
        'config_change',
        'user',
        'config',
        NEW.key,
        'update',
        json_object(
            'key', NEW.key,
            'old_value', OLD.value,
            'new_value', NEW.value
        ),
        datetime('now')
    );
END;

-- Log strategy config changes
CREATE TRIGGER IF NOT EXISTS trg_strategy_version_created
AFTER INSERT ON strategy_versions
BEGIN
    INSERT INTO audit_log (
        event_category,
        event_type,
        actor_type,
        target_type,
        target_id,
        action,
        changes,
        created_at
    ) VALUES (
        'config',
        'strategy_version_created',
        NEW.created_by,
        'strategy_version',
        NEW.id,
        'create',
        json_object(
            'strategy_id', NEW.strategy_id,
            'version', NEW.version,
            'change_description', NEW.change_description
        ),
        datetime('now')
    );
END;

-- ============================================================================
-- SECTION 10: TRADE CREATION TRIGGER
-- ============================================================================

-- Auto-calculate trade metrics
CREATE TRIGGER IF NOT EXISTS trg_trade_metrics
AFTER INSERT ON trades
BEGIN
    -- Update strategy instance trade count
    UPDATE strategy_instances
    SET trades_executed = trades_executed + 1
    WHERE id = NEW.strategy_instance_id;

    -- Create notification for significant trades
    INSERT INTO notifications (
        notification_type,
        severity,
        title,
        message,
        position_id,
        metadata,
        created_at
    )
    SELECT
        'trade',
        CASE
            WHEN CAST(NEW.net_pnl AS REAL) > 0 THEN 'info'
            ELSE 'warning'
        END,
        CASE
            WHEN CAST(NEW.net_pnl AS REAL) > 0 THEN 'Trade Closed: +' || NEW.net_pnl
            ELSE 'Trade Closed: ' || NEW.net_pnl
        END,
        'Closed ' || NEW.side || ' position on ' ||
        (SELECT symbol FROM symbols WHERE id = NEW.symbol_id) ||
        '. Return: ' || NEW.return_pct || '%',
        NEW.position_id,
        json_object(
            'trade_id', NEW.id,
            'symbol_id', NEW.symbol_id,
            'side', NEW.side,
            'net_pnl', NEW.net_pnl,
            'return_pct', NEW.return_pct
        ),
        datetime('now')
    WHERE ABS(CAST(NEW.net_pnl AS REAL)) > 100; -- Only notify for trades > $100
END;

-- ============================================================================
-- Record migration
-- ============================================================================

INSERT INTO schema_migrations (version, name)
VALUES ('00003', 'triggers_and_maintenance');
