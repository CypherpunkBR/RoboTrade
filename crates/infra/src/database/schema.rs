//! Schema SQL do banco de dados

/// SQL para criar todas as tabelas
pub const SCHEMA_SQL: &str = r#"
-- Tabela de versionamento do schema
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insere versão inicial se não existir
INSERT OR IGNORE INTO schema_version (version) VALUES (1);

-- ===========================================================================
-- Tabela de configurações chave-valor
-- ===========================================================================
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ===========================================================================
-- Tabela de dados do Fear & Greed Index
-- ===========================================================================
CREATE TABLE IF NOT EXISTS fear_greed_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    value INTEGER NOT NULL CHECK (value >= 0 AND value <= 100),
    classification TEXT NOT NULL,
    date TEXT NOT NULL UNIQUE,
    collected_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_fear_greed_date ON fear_greed_data(date);

-- ===========================================================================
-- Tabela de candles (OHLCV)
-- ===========================================================================
CREATE TABLE IF NOT EXISTS candles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    open_time TEXT NOT NULL,
    close_time TEXT NOT NULL,
    open REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    close REAL NOT NULL,
    volume REAL NOT NULL,
    quote_volume REAL,
    trade_count INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(symbol, timeframe, open_time)
);

CREATE INDEX IF NOT EXISTS idx_candles_symbol_tf ON candles(symbol, timeframe);
CREATE INDEX IF NOT EXISTS idx_candles_open_time ON candles(open_time);

-- ===========================================================================
-- Tabela de ordens
-- ===========================================================================
CREATE TABLE IF NOT EXISTS orders (
    id TEXT PRIMARY KEY,
    client_order_id TEXT NOT NULL UNIQUE,
    exchange_order_id TEXT,
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    order_type TEXT NOT NULL,
    quantity TEXT NOT NULL,
    price TEXT,
    stop_price TEXT,
    stop_loss TEXT,
    take_profit TEXT,
    time_in_force TEXT NOT NULL,
    status TEXT NOT NULL,
    filled_quantity TEXT NOT NULL DEFAULT '0',
    average_fill_price TEXT,
    source_type TEXT NOT NULL,
    source_data TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL,
    submitted_at TEXT,
    filled_at TEXT,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_orders_symbol ON orders(symbol);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_orders_exchange ON orders(exchange);
CREATE INDEX IF NOT EXISTS idx_orders_created ON orders(created_at);

-- ===========================================================================
-- Tabela de posições
-- ===========================================================================
CREATE TABLE IF NOT EXISTS positions (
    id TEXT PRIMARY KEY,
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    quantity TEXT NOT NULL,
    entry_price TEXT NOT NULL,
    current_price TEXT NOT NULL,
    leverage INTEGER NOT NULL DEFAULT 1,
    margin TEXT NOT NULL,
    unrealized_pnl TEXT NOT NULL DEFAULT '0',
    unrealized_pnl_pct TEXT NOT NULL DEFAULT '0',
    realized_pnl TEXT NOT NULL DEFAULT '0',
    stop_loss_order_id TEXT,
    stop_loss_price TEXT,
    take_profit_order_id TEXT,
    take_profit_price TEXT,
    entry_order_id TEXT NOT NULL,
    status TEXT NOT NULL,
    liquidation_price TEXT,
    opened_at TEXT NOT NULL,
    closed_at TEXT,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_positions_symbol ON positions(symbol);
CREATE INDEX IF NOT EXISTS idx_positions_status ON positions(status);
CREATE INDEX IF NOT EXISTS idx_positions_exchange ON positions(exchange);

-- ===========================================================================
-- Tabela de sinais
-- ===========================================================================
CREATE TABLE IF NOT EXISTS signals (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    strategy_name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    signal_type TEXT NOT NULL,
    direction TEXT NOT NULL,
    strength TEXT NOT NULL,
    trigger_price TEXT NOT NULL,
    suggested_entry TEXT,
    suggested_stop_loss TEXT,
    suggested_take_profit TEXT,
    risk_reward_ratio TEXT,
    reason TEXT NOT NULL,
    confidence INTEGER NOT NULL,
    status TEXT NOT NULL,
    requires_confirmation INTEGER NOT NULL DEFAULT 0,
    generated_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    confirmed_at TEXT,
    executed_at TEXT,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_signals_symbol ON signals(symbol);
CREATE INDEX IF NOT EXISTS idx_signals_status ON signals(status);
CREATE INDEX IF NOT EXISTS idx_signals_strategy ON signals(strategy_id);
CREATE INDEX IF NOT EXISTS idx_signals_generated ON signals(generated_at);

-- ===========================================================================
-- Tabela de trades (histórico)
-- ===========================================================================
CREATE TABLE IF NOT EXISTS trades (
    id TEXT PRIMARY KEY,
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    position_id TEXT NOT NULL,
    signal_id TEXT,
    entry_order_id TEXT NOT NULL,
    exit_order_id TEXT NOT NULL,
    quantity TEXT NOT NULL,
    entry_price TEXT NOT NULL,
    exit_price TEXT NOT NULL,
    gross_pnl TEXT NOT NULL,
    total_fees TEXT NOT NULL,
    net_pnl TEXT NOT NULL,
    pnl_pct TEXT NOT NULL,
    roi_pct TEXT NOT NULL,
    leverage INTEGER NOT NULL DEFAULT 1,
    close_reason TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    entered_at TEXT NOT NULL,
    exited_at TEXT NOT NULL,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_trades_symbol ON trades(symbol);
CREATE INDEX IF NOT EXISTS idx_trades_exchange ON trades(exchange);
CREATE INDEX IF NOT EXISTS idx_trades_entered ON trades(entered_at);
CREATE INDEX IF NOT EXISTS idx_trades_exited ON trades(exited_at);

-- ===========================================================================
-- Tabela de resultados de backtest
-- ===========================================================================
CREATE TABLE IF NOT EXISTS backtest_results (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    initial_capital TEXT NOT NULL,
    final_capital TEXT NOT NULL,
    total_trades INTEGER NOT NULL,
    winning_trades INTEGER NOT NULL,
    win_rate TEXT NOT NULL,
    profit_factor TEXT NOT NULL,
    sharpe_ratio TEXT NOT NULL,
    max_drawdown TEXT NOT NULL,
    trades_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_backtest_strategy ON backtest_results(strategy_id);
CREATE INDEX IF NOT EXISTS idx_backtest_created ON backtest_results(created_at);

-- ===========================================================================
-- Tabela de logs de auditoria
-- ===========================================================================
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    user_action TEXT,
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_event ON audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_log(created_at);
"#;
