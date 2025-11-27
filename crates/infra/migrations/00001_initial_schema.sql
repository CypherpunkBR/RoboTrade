-- ============================================================================
-- RoboTrade Database Schema v1.0
-- Migration: 00001_initial_schema.sql
-- Description: Initial database schema with all core tables
-- ============================================================================

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- SECTION 1: EXCHANGE & MARKET CONFIGURATION
-- ============================================================================

-- Exchanges supported by the system
CREATE TABLE IF NOT EXISTS exchanges (
    id TEXT PRIMARY KEY,                          -- 'binance', 'kraken', etc
    name TEXT NOT NULL,
    exchange_type TEXT NOT NULL,                  -- 'cex', 'dex'
    api_base_url TEXT,
    ws_base_url TEXT,
    status TEXT NOT NULL DEFAULT 'active',        -- active, maintenance, disabled
    rate_limit_requests INTEGER DEFAULT 1200,     -- requests per minute
    rate_limit_orders INTEGER DEFAULT 10,         -- orders per second
    supported_features TEXT,                      -- JSON: ['spot', 'futures', 'margin']
    metadata TEXT,                                -- JSON for extra config
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Trading pairs/symbols configuration
CREATE TABLE IF NOT EXISTS symbols (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id),
    symbol TEXT NOT NULL,                         -- 'BTCUSDT'
    base_asset TEXT NOT NULL,                     -- 'BTC'
    quote_asset TEXT NOT NULL,                    -- 'USDT'
    symbol_type TEXT NOT NULL DEFAULT 'spot',     -- spot, perpetual, delivery, option
    status TEXT NOT NULL DEFAULT 'active',        -- active, inactive, delisted, pre_trading

    -- Precision rules
    price_precision INTEGER NOT NULL,             -- decimal places for price
    quantity_precision INTEGER NOT NULL,          -- decimal places for quantity
    quote_precision INTEGER DEFAULT 8,            -- decimal places for quote

    -- Trading limits
    min_quantity TEXT NOT NULL,                   -- minimum order quantity
    max_quantity TEXT,                            -- maximum order quantity
    min_notional TEXT NOT NULL,                   -- minimum order value
    max_notional TEXT,                            -- maximum order value
    tick_size TEXT NOT NULL,                      -- price increment
    step_size TEXT NOT NULL,                      -- quantity increment

    -- Futures specific
    contract_type TEXT,                           -- perpetual, current_quarter, next_quarter
    contract_size TEXT,                           -- contract multiplier
    margin_asset TEXT,                            -- margin currency
    maintenance_margin_rate TEXT,                 -- maintenance margin %
    max_leverage INTEGER,                         -- maximum allowed leverage

    -- Fee structure
    maker_fee TEXT DEFAULT '0.001',               -- maker fee rate
    taker_fee TEXT DEFAULT '0.001',               -- taker fee rate

    metadata TEXT,                                -- JSON for extra data
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(exchange_id, symbol)
);

CREATE INDEX idx_symbols_exchange ON symbols(exchange_id);
CREATE INDEX idx_symbols_status ON symbols(status);
CREATE INDEX idx_symbols_type ON symbols(symbol_type);
CREATE INDEX idx_symbols_base ON symbols(base_asset);
CREATE INDEX idx_symbols_quote ON symbols(quote_asset);

-- ============================================================================
-- SECTION 2: MARKET DATA
-- ============================================================================

-- Historical candlestick/OHLCV data
CREATE TABLE IF NOT EXISTS candles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),
    timeframe TEXT NOT NULL,                      -- 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M
    open_time INTEGER NOT NULL,                   -- Unix timestamp ms
    close_time INTEGER NOT NULL,                  -- Unix timestamp ms

    -- OHLCV data (stored as TEXT for Decimal precision)
    open TEXT NOT NULL,
    high TEXT NOT NULL,
    low TEXT NOT NULL,
    close TEXT NOT NULL,
    volume TEXT NOT NULL,                         -- base asset volume
    quote_volume TEXT NOT NULL,                   -- quote asset volume

    -- Additional metrics
    trade_count INTEGER DEFAULT 0,                -- number of trades
    taker_buy_volume TEXT,                        -- taker buy base volume
    taker_buy_quote_volume TEXT,                  -- taker buy quote volume

    -- Computed fields (can be null, filled by analytics)
    vwap TEXT,                                    -- volume weighted average price
    typical_price TEXT,                           -- (H+L+C)/3

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(symbol_id, timeframe, open_time)
);

CREATE INDEX idx_candles_symbol_tf ON candles(symbol_id, timeframe);
CREATE INDEX idx_candles_time ON candles(open_time DESC);
CREATE INDEX idx_candles_lookup ON candles(symbol_id, timeframe, open_time DESC);

-- Real-time ticker data snapshots
CREATE TABLE IF NOT EXISTS tickers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    -- Price data
    bid_price TEXT NOT NULL,
    bid_qty TEXT,
    ask_price TEXT NOT NULL,
    ask_qty TEXT,
    last_price TEXT NOT NULL,

    -- 24h statistics
    price_change TEXT,
    price_change_pct TEXT,
    weighted_avg_price TEXT,
    open_price TEXT,
    high_price TEXT,
    low_price TEXT,
    volume TEXT,
    quote_volume TEXT,

    -- Funding rate (futures)
    funding_rate TEXT,
    next_funding_time INTEGER,

    -- Open interest (futures)
    open_interest TEXT,
    open_interest_value TEXT,

    timestamp INTEGER NOT NULL,                   -- Unix timestamp ms
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_tickers_symbol ON tickers(symbol_id);
CREATE INDEX idx_tickers_time ON tickers(timestamp DESC);

-- Order book snapshots
CREATE TABLE IF NOT EXISTS orderbook_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    -- Aggregated data
    best_bid TEXT NOT NULL,
    best_ask TEXT NOT NULL,
    spread TEXT NOT NULL,
    spread_pct TEXT NOT NULL,
    mid_price TEXT NOT NULL,

    -- Depth data (JSON arrays of [price, qty] pairs)
    bids TEXT NOT NULL,                           -- JSON: [[price, qty], ...]
    asks TEXT NOT NULL,                           -- JSON: [[price, qty], ...]

    -- Imbalance metrics
    bid_volume_total TEXT,
    ask_volume_total TEXT,
    imbalance_ratio TEXT,                         -- bid_vol / (bid_vol + ask_vol)

    depth_levels INTEGER DEFAULT 20,              -- number of levels captured
    timestamp INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_orderbook_symbol ON orderbook_snapshots(symbol_id);
CREATE INDEX idx_orderbook_time ON orderbook_snapshots(timestamp DESC);

-- External market indicators (Fear & Greed, etc)
CREATE TABLE IF NOT EXISTS market_indicators (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    indicator_type TEXT NOT NULL,                 -- fear_greed, btc_dominance, total_market_cap, etc
    provider TEXT NOT NULL,                       -- alternative_me, coingecko, etc

    value TEXT NOT NULL,                          -- main value
    value_classification TEXT,                    -- extreme_fear, fear, neutral, greed, extreme_greed
    secondary_value TEXT,                         -- optional secondary metric

    -- Time bounds
    timestamp INTEGER NOT NULL,                   -- Unix timestamp ms
    valid_until INTEGER,                          -- expiry timestamp

    metadata TEXT,                                -- JSON for extra data
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(indicator_type, provider, timestamp)
);

CREATE INDEX idx_market_ind_type ON market_indicators(indicator_type);
CREATE INDEX idx_market_ind_time ON market_indicators(timestamp DESC);

-- ============================================================================
-- SECTION 3: TECHNICAL ANALYSIS & INDICATORS
-- ============================================================================

-- Computed technical indicators
CREATE TABLE IF NOT EXISTS technical_indicators (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),
    timeframe TEXT NOT NULL,
    timestamp INTEGER NOT NULL,                   -- aligned to candle open_time

    indicator_name TEXT NOT NULL,                 -- sma, ema, rsi, macd, bb, atr, etc
    indicator_params TEXT NOT NULL,               -- JSON: {"period": 14} or {"fast": 12, "slow": 26}

    -- Values (depends on indicator type)
    value_1 TEXT NOT NULL,                        -- main value (SMA, EMA, RSI, etc)
    value_2 TEXT,                                 -- secondary (MACD signal, BB upper, etc)
    value_3 TEXT,                                 -- tertiary (MACD histogram, BB lower, etc)
    value_4 TEXT,                                 -- extra (BB middle, etc)

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(symbol_id, timeframe, timestamp, indicator_name, indicator_params)
);

CREATE INDEX idx_tech_ind_lookup ON technical_indicators(symbol_id, timeframe, indicator_name, timestamp DESC);
CREATE INDEX idx_tech_ind_time ON technical_indicators(timestamp DESC);

-- ============================================================================
-- SECTION 4: STRATEGY & SIGNAL MANAGEMENT
-- ============================================================================

-- Strategy definitions
CREATE TABLE IF NOT EXISTS strategies (
    id TEXT PRIMARY KEY,                          -- UUID
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    strategy_type TEXT NOT NULL,                  -- fear_greed, sma_crossover, rsi_divergence, custom

    -- Target configuration
    default_symbols TEXT,                         -- JSON array of symbol names
    default_timeframes TEXT,                      -- JSON array of timeframes

    -- Status
    is_enabled INTEGER NOT NULL DEFAULT 0,
    is_live_allowed INTEGER NOT NULL DEFAULT 0,   -- can run in live mode

    -- Metadata
    author TEXT,
    tags TEXT,                                    -- JSON array
    documentation_url TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Strategy versions (immutable configurations)
CREATE TABLE IF NOT EXISTS strategy_versions (
    id TEXT PRIMARY KEY,                          -- UUID
    strategy_id TEXT NOT NULL REFERENCES strategies(id) ON DELETE CASCADE,

    version TEXT NOT NULL,                        -- semver: 1.0.0
    version_major INTEGER NOT NULL,
    version_minor INTEGER NOT NULL,
    version_patch INTEGER NOT NULL,

    -- Configuration snapshot (immutable)
    config TEXT NOT NULL,                         -- JSON: full strategy configuration
    indicators_config TEXT,                       -- JSON: indicator parameters
    entry_rules TEXT NOT NULL,                    -- JSON: entry conditions
    exit_rules TEXT NOT NULL,                     -- JSON: exit conditions
    filters TEXT,                                 -- JSON: market filters

    -- Risk configuration
    risk_config TEXT NOT NULL,                    -- JSON: position sizing, stops, etc

    -- Status
    is_active INTEGER NOT NULL DEFAULT 0,         -- currently active version

    -- Change tracking
    change_description TEXT NOT NULL,
    parent_version_id TEXT REFERENCES strategy_versions(id),

    -- Performance snapshot (updated periodically)
    performance_metrics TEXT,                     -- JSON: cached metrics

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT DEFAULT 'user',

    UNIQUE(strategy_id, version)
);

CREATE INDEX idx_strat_ver_strategy ON strategy_versions(strategy_id);
CREATE INDEX idx_strat_ver_active ON strategy_versions(strategy_id, is_active) WHERE is_active = 1;

-- Strategy execution instances
CREATE TABLE IF NOT EXISTS strategy_instances (
    id TEXT PRIMARY KEY,                          -- UUID
    strategy_version_id TEXT NOT NULL REFERENCES strategy_versions(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    -- Runtime configuration
    trading_mode TEXT NOT NULL DEFAULT 'paper',   -- paper, live
    timeframe TEXT NOT NULL,

    -- Instance-specific overrides
    config_overrides TEXT,                        -- JSON: instance-specific config

    -- Status
    status TEXT NOT NULL DEFAULT 'stopped',       -- stopped, running, paused, error
    last_evaluation_at TEXT,
    last_signal_at TEXT,
    last_error TEXT,
    error_count INTEGER DEFAULT 0,

    -- Runtime metrics
    signals_generated INTEGER DEFAULT 0,
    trades_executed INTEGER DEFAULT 0,

    started_at TEXT,
    stopped_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_strat_inst_version ON strategy_instances(strategy_version_id);
CREATE INDEX idx_strat_inst_symbol ON strategy_instances(symbol_id);
CREATE INDEX idx_strat_inst_status ON strategy_instances(status);

-- Trading signals generated by strategies
CREATE TABLE IF NOT EXISTS signals (
    id TEXT PRIMARY KEY,                          -- UUID
    strategy_instance_id TEXT NOT NULL REFERENCES strategy_instances(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    -- Signal details
    signal_type TEXT NOT NULL,                    -- entry, exit, scale_in, scale_out, adjust_stop
    direction TEXT NOT NULL,                      -- long, short, close
    strength TEXT NOT NULL,                       -- 0.0 to 1.0

    -- Price targets
    entry_price TEXT,
    stop_loss TEXT,
    take_profit_1 TEXT,
    take_profit_2 TEXT,
    take_profit_3 TEXT,
    trailing_stop_pct TEXT,

    -- Position sizing
    suggested_quantity TEXT,
    suggested_risk_pct TEXT,                      -- % of capital to risk
    risk_reward_ratio TEXT,

    -- Signal reasoning
    primary_reason TEXT NOT NULL,                 -- main trigger
    secondary_reasons TEXT,                       -- JSON array of supporting factors
    indicator_values TEXT,                        -- JSON: indicator values at signal time
    market_context TEXT,                          -- JSON: market conditions

    -- Status tracking
    status TEXT NOT NULL DEFAULT 'pending',       -- pending, confirmed, executed, expired, cancelled, rejected
    confirmation_required INTEGER DEFAULT 0,
    confirmed_at TEXT,

    -- Execution link
    order_id TEXT REFERENCES orders(id),
    position_id TEXT REFERENCES positions(id),

    -- Timing
    valid_from TEXT NOT NULL DEFAULT (datetime('now')),
    valid_until TEXT,
    executed_at TEXT,

    -- Metadata
    candle_open_time INTEGER,                     -- reference candle
    metadata TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_signals_instance ON signals(strategy_instance_id);
CREATE INDEX idx_signals_symbol ON signals(symbol_id);
CREATE INDEX idx_signals_status ON signals(status);
CREATE INDEX idx_signals_time ON signals(created_at DESC);
CREATE INDEX idx_signals_pending ON signals(status, valid_until) WHERE status = 'pending';

-- ============================================================================
-- SECTION 5: ORDER MANAGEMENT
-- ============================================================================

-- Order records
CREATE TABLE IF NOT EXISTS orders (
    id TEXT PRIMARY KEY,                          -- UUID (internal)
    exchange_id TEXT NOT NULL REFERENCES exchanges(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    -- Exchange identifiers
    exchange_order_id TEXT,                       -- ID from exchange
    client_order_id TEXT NOT NULL,                -- our tracking ID

    -- Order specification
    side TEXT NOT NULL,                           -- buy, sell
    order_type TEXT NOT NULL,                     -- market, limit, stop_market, stop_limit, take_profit, take_profit_limit, trailing_stop
    time_in_force TEXT DEFAULT 'GTC',             -- GTC, IOC, FOK, GTX

    -- Quantities
    quantity TEXT NOT NULL,                       -- original quantity
    executed_qty TEXT NOT NULL DEFAULT '0',       -- filled quantity
    remaining_qty TEXT,                           -- unfilled quantity

    -- Prices
    price TEXT,                                   -- limit price
    stop_price TEXT,                              -- trigger price
    avg_fill_price TEXT,                          -- average execution price

    -- Futures specific
    reduce_only INTEGER DEFAULT 0,
    close_position INTEGER DEFAULT 0,
    position_side TEXT,                           -- LONG, SHORT, BOTH

    -- Status
    status TEXT NOT NULL DEFAULT 'pending_new',   -- pending_new, new, partially_filled, filled, cancelled, rejected, expired, pending_cancel
    reject_reason TEXT,

    -- Fees
    commission TEXT DEFAULT '0',
    commission_asset TEXT,

    -- Links
    signal_id TEXT REFERENCES signals(id),
    position_id TEXT REFERENCES positions(id),
    parent_order_id TEXT REFERENCES orders(id),   -- for OCO, bracket orders

    -- OCO/Bracket info
    order_group_id TEXT,                          -- groups related orders
    order_group_type TEXT,                        -- oco, bracket, oto

    -- Trading mode
    trading_mode TEXT NOT NULL DEFAULT 'paper',   -- paper, live

    -- Timing
    submitted_at TEXT,
    acknowledged_at TEXT,
    last_fill_at TEXT,
    completed_at TEXT,

    -- Metadata
    metadata TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_orders_exchange ON orders(exchange_id);
CREATE INDEX idx_orders_symbol ON orders(symbol_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_signal ON orders(signal_id);
CREATE INDEX idx_orders_position ON orders(position_id);
CREATE INDEX idx_orders_client_id ON orders(client_order_id);
CREATE INDEX idx_orders_exchange_id ON orders(exchange_order_id);
CREATE INDEX idx_orders_time ON orders(created_at DESC);
CREATE INDEX idx_orders_open ON orders(status) WHERE status IN ('pending_new', 'new', 'partially_filled');
CREATE INDEX idx_orders_group ON orders(order_group_id);

-- Order fills/executions
CREATE TABLE IF NOT EXISTS order_fills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,

    -- Fill details
    fill_id TEXT,                                 -- exchange trade ID
    price TEXT NOT NULL,
    quantity TEXT NOT NULL,
    quote_quantity TEXT,

    -- Fees
    commission TEXT NOT NULL,
    commission_asset TEXT NOT NULL,

    -- Counterparty info
    is_maker INTEGER,                             -- was this a maker order

    -- Timing
    filled_at TEXT NOT NULL,

    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_fills_order ON order_fills(order_id);
CREATE INDEX idx_fills_time ON order_fills(filled_at DESC);

-- ============================================================================
-- SECTION 6: POSITION MANAGEMENT
-- ============================================================================

-- Active and historical positions
CREATE TABLE IF NOT EXISTS positions (
    id TEXT PRIMARY KEY,                          -- UUID
    exchange_id TEXT NOT NULL REFERENCES exchanges(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    -- Position details
    side TEXT NOT NULL,                           -- long, short
    status TEXT NOT NULL DEFAULT 'open',          -- open, closing, closed

    -- Quantities
    quantity TEXT NOT NULL,                       -- current position size
    initial_quantity TEXT NOT NULL,               -- original position size

    -- Entry info
    entry_price TEXT NOT NULL,                    -- average entry price
    entry_value TEXT NOT NULL,                    -- total entry value (price * qty)
    entry_order_id TEXT REFERENCES orders(id),

    -- Exit info (populated on close)
    exit_price TEXT,
    exit_value TEXT,
    exit_order_id TEXT REFERENCES orders(id),

    -- Leverage & Margin
    leverage INTEGER NOT NULL DEFAULT 1,
    margin_type TEXT DEFAULT 'cross',             -- cross, isolated
    initial_margin TEXT,
    maintenance_margin TEXT,
    liquidation_price TEXT,

    -- Risk management
    stop_loss_price TEXT,
    stop_loss_order_id TEXT REFERENCES orders(id),
    take_profit_price TEXT,
    take_profit_order_id TEXT REFERENCES orders(id),
    trailing_stop_pct TEXT,
    trailing_stop_activation TEXT,
    break_even_price TEXT,

    -- P&L tracking
    unrealized_pnl TEXT DEFAULT '0',
    realized_pnl TEXT DEFAULT '0',
    total_fees TEXT DEFAULT '0',
    funding_fees TEXT DEFAULT '0',                -- futures only

    -- Performance metrics
    max_profit TEXT,                              -- highest unrealized P&L
    max_drawdown TEXT,                            -- lowest unrealized P&L
    risk_amount TEXT,                             -- amount at risk (entry - stop)

    -- Links
    signal_id TEXT REFERENCES signals(id),
    strategy_instance_id TEXT REFERENCES strategy_instances(id),

    -- Trading mode
    trading_mode TEXT NOT NULL DEFAULT 'paper',   -- paper, live

    -- Timing
    opened_at TEXT NOT NULL DEFAULT (datetime('now')),
    closed_at TEXT,
    duration_seconds INTEGER,                     -- computed on close

    -- Notes
    notes TEXT,
    tags TEXT,                                    -- JSON array
    metadata TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_positions_exchange ON positions(exchange_id);
CREATE INDEX idx_positions_symbol ON positions(symbol_id);
CREATE INDEX idx_positions_status ON positions(status);
CREATE INDEX idx_positions_strategy ON positions(strategy_instance_id);
CREATE INDEX idx_positions_signal ON positions(signal_id);
CREATE INDEX idx_positions_time ON positions(opened_at DESC);
CREATE INDEX idx_positions_open ON positions(status) WHERE status = 'open';
CREATE INDEX idx_positions_mode ON positions(trading_mode);

-- Position adjustments log
CREATE TABLE IF NOT EXISTS position_adjustments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    position_id TEXT NOT NULL REFERENCES positions(id) ON DELETE CASCADE,

    adjustment_type TEXT NOT NULL,                -- scale_in, scale_out, adjust_stop, adjust_tp, add_tp, move_to_breakeven

    -- Before/After values
    quantity_before TEXT,
    quantity_after TEXT,
    avg_price_before TEXT,
    avg_price_after TEXT,
    stop_loss_before TEXT,
    stop_loss_after TEXT,
    take_profit_before TEXT,
    take_profit_after TEXT,

    -- Related order
    order_id TEXT REFERENCES orders(id),

    -- Reason
    reason TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_pos_adj_position ON position_adjustments(position_id);

-- ============================================================================
-- SECTION 7: TRADE HISTORY & PERFORMANCE
-- ============================================================================

-- Completed trades (closed positions with full P&L)
CREATE TABLE IF NOT EXISTS trades (
    id TEXT PRIMARY KEY,                          -- UUID
    position_id TEXT NOT NULL REFERENCES positions(id),
    exchange_id TEXT NOT NULL REFERENCES exchanges(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    -- Trade details
    side TEXT NOT NULL,                           -- long, short
    quantity TEXT NOT NULL,

    -- Entry
    entry_price TEXT NOT NULL,
    entry_value TEXT NOT NULL,
    entry_time TEXT NOT NULL,
    entry_order_id TEXT REFERENCES orders(id),

    -- Exit
    exit_price TEXT NOT NULL,
    exit_value TEXT NOT NULL,
    exit_time TEXT NOT NULL,
    exit_order_id TEXT REFERENCES orders(id),
    exit_reason TEXT,                             -- stop_loss, take_profit, trailing_stop, manual, signal, liquidation

    -- P&L
    gross_pnl TEXT NOT NULL,                      -- before fees
    total_fees TEXT NOT NULL,
    funding_fees TEXT DEFAULT '0',
    net_pnl TEXT NOT NULL,                        -- after all fees
    return_pct TEXT NOT NULL,                     -- percentage return
    r_multiple TEXT,                              -- return in R (risk units)

    -- Trade metrics
    duration_seconds INTEGER NOT NULL,
    max_favorable_excursion TEXT,                 -- MFE: max profit during trade
    max_adverse_excursion TEXT,                   -- MAE: max loss during trade

    -- Strategy link
    strategy_instance_id TEXT REFERENCES strategy_instances(id),
    strategy_version_id TEXT REFERENCES strategy_versions(id),
    signal_id TEXT REFERENCES signals(id),

    -- Trading mode
    trading_mode TEXT NOT NULL DEFAULT 'paper',

    -- Analysis tags
    tags TEXT,                                    -- JSON array
    notes TEXT,
    quality_score INTEGER,                        -- 1-10 trade quality rating

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_trades_position ON trades(position_id);
CREATE INDEX idx_trades_exchange ON trades(exchange_id);
CREATE INDEX idx_trades_symbol ON trades(symbol_id);
CREATE INDEX idx_trades_strategy ON trades(strategy_instance_id);
CREATE INDEX idx_trades_exit_time ON trades(exit_time DESC);
CREATE INDEX idx_trades_pnl ON trades(net_pnl);
CREATE INDEX idx_trades_mode ON trades(trading_mode);
CREATE INDEX idx_trades_exit_reason ON trades(exit_reason);

-- Daily performance aggregation
CREATE TABLE IF NOT EXISTS daily_performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,                           -- YYYY-MM-DD
    trading_mode TEXT NOT NULL,                   -- paper, live

    -- Balance tracking
    starting_balance TEXT NOT NULL,
    ending_balance TEXT NOT NULL,

    -- P&L summary
    gross_pnl TEXT NOT NULL,
    fees TEXT NOT NULL,
    funding TEXT DEFAULT '0',
    net_pnl TEXT NOT NULL,
    return_pct TEXT NOT NULL,

    -- Trade statistics
    total_trades INTEGER NOT NULL DEFAULT 0,
    winning_trades INTEGER NOT NULL DEFAULT 0,
    losing_trades INTEGER NOT NULL DEFAULT 0,
    break_even_trades INTEGER DEFAULT 0,

    -- Amounts
    total_win_amount TEXT DEFAULT '0',
    total_loss_amount TEXT DEFAULT '0',
    largest_win TEXT,
    largest_loss TEXT,
    average_win TEXT,
    average_loss TEXT,

    -- Ratios
    win_rate TEXT,
    profit_factor TEXT,
    average_rr TEXT,                              -- average risk/reward

    -- Drawdown
    max_drawdown TEXT,
    max_drawdown_pct TEXT,

    -- Volume
    total_volume TEXT,
    long_volume TEXT,
    short_volume TEXT,

    -- By symbol breakdown (JSON)
    by_symbol TEXT,

    -- By strategy breakdown (JSON)
    by_strategy TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(date, trading_mode)
);

CREATE INDEX idx_daily_perf_date ON daily_performance(date DESC);
CREATE INDEX idx_daily_perf_mode ON daily_performance(trading_mode);

-- ============================================================================
-- SECTION 8: ACCOUNT & BALANCES
-- ============================================================================

-- Account configuration per exchange
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,                          -- UUID
    exchange_id TEXT NOT NULL REFERENCES exchanges(id),

    account_type TEXT NOT NULL DEFAULT 'main',    -- main, sub, isolated_margin
    alias TEXT,                                   -- user-friendly name

    -- API configuration (encrypted references, not actual keys)
    api_key_ref TEXT,                             -- reference to keyring
    has_trade_permission INTEGER DEFAULT 0,
    has_withdraw_permission INTEGER DEFAULT 0,
    ip_whitelist TEXT,                            -- JSON array

    -- Status
    status TEXT NOT NULL DEFAULT 'active',        -- active, disabled, error
    last_sync_at TEXT,
    sync_error TEXT,

    -- Settings
    default_leverage INTEGER DEFAULT 1,
    margin_type TEXT DEFAULT 'cross',
    position_mode TEXT DEFAULT 'one_way',         -- one_way, hedge

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(exchange_id, account_type, alias)
);

CREATE INDEX idx_accounts_exchange ON accounts(exchange_id);
CREATE INDEX idx_accounts_status ON accounts(status);

-- Balance snapshots
CREATE TABLE IF NOT EXISTS balance_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL REFERENCES accounts(id),

    -- Snapshot type
    snapshot_type TEXT NOT NULL DEFAULT 'periodic', -- periodic, pre_trade, post_trade, manual

    -- Total balance
    total_balance_usdt TEXT NOT NULL,             -- total in USDT equivalent
    available_balance_usdt TEXT NOT NULL,
    locked_balance_usdt TEXT NOT NULL,

    -- Margin info (futures)
    total_margin TEXT,
    used_margin TEXT,
    available_margin TEXT,
    margin_level TEXT,

    -- Unrealized P&L
    unrealized_pnl TEXT DEFAULT '0',

    -- Asset breakdown
    assets TEXT NOT NULL,                         -- JSON: [{"asset": "BTC", "free": "0.5", "locked": "0.1", "usdt_value": "25000"}]

    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_balance_snap_account ON balance_snapshots(account_id);
CREATE INDEX idx_balance_snap_time ON balance_snapshots(timestamp DESC);
CREATE INDEX idx_balance_snap_type ON balance_snapshots(snapshot_type);

-- ============================================================================
-- SECTION 9: RISK MANAGEMENT
-- ============================================================================

-- Risk limits configuration
CREATE TABLE IF NOT EXISTS risk_limits (
    id TEXT PRIMARY KEY,                          -- UUID
    name TEXT NOT NULL UNIQUE,
    description TEXT,

    -- Scope
    applies_to TEXT NOT NULL DEFAULT 'global',    -- global, exchange, symbol, strategy
    exchange_id TEXT REFERENCES exchanges(id),
    symbol_id INTEGER REFERENCES symbols(id),
    strategy_id TEXT REFERENCES strategies(id),

    -- Position limits
    max_position_size TEXT,                       -- max size per position
    max_position_value TEXT,                      -- max value in quote
    max_positions_per_symbol INTEGER,
    max_total_positions INTEGER,
    max_correlated_positions INTEGER,

    -- Exposure limits
    max_total_exposure TEXT,                      -- total exposure in quote
    max_long_exposure TEXT,
    max_short_exposure TEXT,
    max_leverage INTEGER,

    -- Loss limits
    max_loss_per_trade TEXT,                      -- max loss per single trade
    max_loss_per_trade_pct TEXT,                  -- as % of capital
    max_daily_loss TEXT,
    max_daily_loss_pct TEXT,
    max_weekly_loss TEXT,
    max_weekly_loss_pct TEXT,
    max_monthly_loss TEXT,
    max_monthly_loss_pct TEXT,
    max_drawdown_pct TEXT,

    -- Order limits
    max_order_size TEXT,
    max_order_value TEXT,
    max_orders_per_minute INTEGER,
    max_orders_per_day INTEGER,

    -- Time-based
    allowed_trading_hours TEXT,                   -- JSON: {"start": "00:00", "end": "23:59"}
    blocked_dates TEXT,                           -- JSON array of dates

    -- Circuit breaker
    consecutive_loss_limit INTEGER,
    pause_after_losses_minutes INTEGER,

    is_enabled INTEGER NOT NULL DEFAULT 1,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_risk_limits_scope ON risk_limits(applies_to);
CREATE INDEX idx_risk_limits_enabled ON risk_limits(is_enabled);

-- Risk events log
CREATE TABLE IF NOT EXISTS risk_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    event_type TEXT NOT NULL,                     -- limit_breach, circuit_breaker, warning, position_liquidated
    severity TEXT NOT NULL,                       -- info, warning, critical

    -- Context
    risk_limit_id TEXT REFERENCES risk_limits(id),
    account_id TEXT REFERENCES accounts(id),
    symbol_id INTEGER REFERENCES symbols(id),
    position_id TEXT REFERENCES positions(id),
    order_id TEXT REFERENCES orders(id),

    -- Event details
    limit_name TEXT,
    limit_value TEXT,
    actual_value TEXT,
    breach_pct TEXT,                              -- how much over limit

    -- Action taken
    action_taken TEXT,                            -- blocked, warned, position_closed, trading_paused

    -- Resolution
    resolved INTEGER DEFAULT 0,
    resolved_at TEXT,
    resolved_by TEXT,
    resolution_notes TEXT,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_risk_events_type ON risk_events(event_type);
CREATE INDEX idx_risk_events_severity ON risk_events(severity);
CREATE INDEX idx_risk_events_resolved ON risk_events(resolved);
CREATE INDEX idx_risk_events_time ON risk_events(created_at DESC);

-- ============================================================================
-- SECTION 10: BACKTESTING
-- ============================================================================

-- Backtest runs
CREATE TABLE IF NOT EXISTS backtests (
    id TEXT PRIMARY KEY,                          -- UUID
    strategy_version_id TEXT NOT NULL REFERENCES strategy_versions(id),

    -- Test parameters
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),
    timeframe TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,

    -- Initial conditions
    initial_capital TEXT NOT NULL,
    commission_rate TEXT NOT NULL,
    slippage_bps INTEGER DEFAULT 0,               -- basis points

    -- Execution parameters
    fill_model TEXT DEFAULT 'close',              -- close, ohlc, random
    position_sizing TEXT DEFAULT 'fixed',         -- fixed, kelly, volatility

    -- Status
    status TEXT NOT NULL DEFAULT 'pending',       -- pending, running, completed, failed, cancelled
    progress_pct INTEGER DEFAULT 0,
    current_date TEXT,
    error_message TEXT,

    -- Runtime
    started_at TEXT,
    completed_at TEXT,
    execution_time_ms INTEGER,
    candles_processed INTEGER,

    -- ============ RESULTS (populated on completion) ============

    -- Capital
    final_capital TEXT,
    peak_capital TEXT,

    -- Returns
    total_return TEXT,
    total_return_pct TEXT,
    annualized_return TEXT,

    -- Risk metrics
    sharpe_ratio TEXT,
    sortino_ratio TEXT,
    calmar_ratio TEXT,
    max_drawdown TEXT,
    max_drawdown_pct TEXT,
    max_drawdown_duration_days INTEGER,
    avg_drawdown TEXT,
    ulcer_index TEXT,

    -- Trade statistics
    total_trades INTEGER,
    winning_trades INTEGER,
    losing_trades INTEGER,
    win_rate TEXT,
    profit_factor TEXT,

    -- Trade amounts
    gross_profit TEXT,
    gross_loss TEXT,
    net_profit TEXT,
    total_fees TEXT,

    -- Average trade
    avg_trade_pnl TEXT,
    avg_winning_trade TEXT,
    avg_losing_trade TEXT,
    largest_win TEXT,
    largest_loss TEXT,

    -- Trade duration
    avg_trade_duration_hours TEXT,
    avg_winner_duration_hours TEXT,
    avg_loser_duration_hours TEXT,
    max_trade_duration_hours TEXT,

    -- Streaks
    max_consecutive_wins INTEGER,
    max_consecutive_losses INTEGER,

    -- Exposure
    avg_exposure_pct TEXT,
    max_exposure_pct TEXT,
    time_in_market_pct TEXT,

    -- Risk/Reward
    avg_risk_reward TEXT,
    expectancy TEXT,
    expectancy_ratio TEXT,

    -- Curves (JSON arrays)
    equity_curve TEXT,                            -- [{date, equity, drawdown}]
    monthly_returns TEXT,                         -- [{year, month, return}]

    -- Trade list (can be large, consider separate table for production)
    trades_json TEXT,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_backtests_strategy ON backtests(strategy_version_id);
CREATE INDEX idx_backtests_symbol ON backtests(symbol_id);
CREATE INDEX idx_backtests_status ON backtests(status);
CREATE INDEX idx_backtests_time ON backtests(created_at DESC);

-- Backtest comparison groups
CREATE TABLE IF NOT EXISTS backtest_comparisons (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,

    backtest_ids TEXT NOT NULL,                   -- JSON array of backtest IDs

    -- Comparison results (JSON)
    comparison_results TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- SECTION 11: JOB QUEUE & SCHEDULING
-- ============================================================================

-- Background jobs
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,                          -- UUID

    job_type TEXT NOT NULL,                       -- fetch_candles, execute_strategy, place_order, sync_balance, etc
    queue TEXT NOT NULL DEFAULT 'default',        -- default, priority, slow

    -- Priority: 0 = critical, 1 = high, 2 = normal, 3 = low
    priority INTEGER NOT NULL DEFAULT 2,

    -- Job payload
    payload TEXT NOT NULL,                        -- JSON

    -- Scheduling
    scheduled_for TEXT NOT NULL DEFAULT (datetime('now')),
    not_before TEXT,                              -- don't run before this time
    deadline TEXT,                                -- must complete by this time

    -- Execution
    status TEXT NOT NULL DEFAULT 'pending',       -- pending, scheduled, running, completed, failed, cancelled, dead_letter
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,

    -- Worker info
    worker_id TEXT,
    locked_at TEXT,
    lock_expires_at TEXT,

    -- Results
    result TEXT,                                  -- JSON
    last_error TEXT,
    error_count INTEGER DEFAULT 0,

    -- Timing
    started_at TEXT,
    completed_at TEXT,
    execution_time_ms INTEGER,

    -- Dependencies
    depends_on TEXT,                              -- JSON array of job IDs

    -- Retry config
    retry_delay_ms INTEGER DEFAULT 1000,
    retry_backoff_multiplier REAL DEFAULT 2.0,
    max_retry_delay_ms INTEGER DEFAULT 300000,

    -- Metadata
    idempotency_key TEXT UNIQUE,
    metadata TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_queue ON jobs(queue);
CREATE INDEX idx_jobs_priority ON jobs(priority);
CREATE INDEX idx_jobs_scheduled ON jobs(scheduled_for);
CREATE INDEX idx_jobs_type ON jobs(job_type);
CREATE INDEX idx_jobs_pending ON jobs(queue, priority, scheduled_for)
    WHERE status IN ('pending', 'scheduled');
CREATE INDEX idx_jobs_running ON jobs(worker_id, lock_expires_at)
    WHERE status = 'running';

-- Scheduled/recurring jobs
CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id TEXT PRIMARY KEY,                          -- UUID
    name TEXT NOT NULL UNIQUE,
    description TEXT,

    job_type TEXT NOT NULL,
    payload_template TEXT NOT NULL,               -- JSON template
    queue TEXT DEFAULT 'default',
    priority INTEGER DEFAULT 2,

    -- Schedule (cron-like)
    schedule_type TEXT NOT NULL,                  -- cron, interval, fixed_time
    cron_expression TEXT,                         -- "0 * * * *" for hourly
    interval_seconds INTEGER,
    fixed_times TEXT,                             -- JSON array of times
    timezone TEXT DEFAULT 'UTC',

    -- Status
    is_enabled INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    next_run_at TEXT,
    last_job_id TEXT REFERENCES jobs(id),

    -- Limits
    max_concurrent INTEGER DEFAULT 1,
    timeout_seconds INTEGER DEFAULT 300,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_sched_tasks_enabled ON scheduled_tasks(is_enabled);
CREATE INDEX idx_sched_tasks_next ON scheduled_tasks(next_run_at);

-- Job execution history
CREATE TABLE IF NOT EXISTS job_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL,                         -- may reference deleted job

    event_type TEXT NOT NULL,                     -- created, started, completed, failed, retrying, cancelled

    status_before TEXT,
    status_after TEXT,

    worker_id TEXT,
    error TEXT,

    details TEXT,                                 -- JSON

    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_job_hist_job ON job_history(job_id);
CREATE INDEX idx_job_hist_time ON job_history(created_at DESC);
CREATE INDEX idx_job_hist_type ON job_history(event_type);

-- ============================================================================
-- SECTION 12: NOTIFICATIONS & ALERTS
-- ============================================================================

-- Alert definitions
CREATE TABLE IF NOT EXISTS alert_definitions (
    id TEXT PRIMARY KEY,                          -- UUID
    name TEXT NOT NULL,
    description TEXT,

    alert_type TEXT NOT NULL,                     -- price, indicator, position, risk, system

    -- Conditions (JSON structure)
    conditions TEXT NOT NULL,                     -- JSON: {"field": "price", "operator": "crosses_above", "value": "50000"}

    -- Target
    symbol_id INTEGER REFERENCES symbols(id),
    strategy_id TEXT REFERENCES strategies(id),

    -- Actions
    notification_channels TEXT NOT NULL,          -- JSON: ["desktop", "sound", "telegram"]
    message_template TEXT,

    -- Behavior
    is_enabled INTEGER NOT NULL DEFAULT 1,
    is_recurring INTEGER DEFAULT 0,               -- alert multiple times
    cooldown_seconds INTEGER DEFAULT 300,

    -- Status
    last_triggered_at TEXT,
    trigger_count INTEGER DEFAULT 0,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_alerts_type ON alert_definitions(alert_type);
CREATE INDEX idx_alerts_enabled ON alert_definitions(is_enabled);
CREATE INDEX idx_alerts_symbol ON alert_definitions(symbol_id);

-- Triggered notifications
CREATE TABLE IF NOT EXISTS notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    notification_type TEXT NOT NULL,              -- alert, signal, order_update, position_update, system
    severity TEXT NOT NULL DEFAULT 'info',        -- info, warning, error, critical

    -- Content
    title TEXT NOT NULL,
    message TEXT NOT NULL,

    -- Links
    alert_definition_id TEXT REFERENCES alert_definitions(id),
    signal_id TEXT REFERENCES signals(id),
    order_id TEXT REFERENCES orders(id),
    position_id TEXT REFERENCES positions(id),

    -- Delivery
    channels_sent TEXT,                           -- JSON: channels where notification was sent

    -- Status
    is_read INTEGER DEFAULT 0,
    read_at TEXT,
    is_dismissed INTEGER DEFAULT 0,
    dismissed_at TEXT,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_notifications_type ON notifications(notification_type);
CREATE INDEX idx_notifications_read ON notifications(is_read);
CREATE INDEX idx_notifications_time ON notifications(created_at DESC);

-- ============================================================================
-- SECTION 13: AUDIT & LOGGING
-- ============================================================================

-- System audit log
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    event_category TEXT NOT NULL,                 -- auth, config, trading, system
    event_type TEXT NOT NULL,                     -- login, config_change, order_placed, etc

    -- Actor
    actor_type TEXT DEFAULT 'system',             -- system, user, api, scheduled
    actor_id TEXT,

    -- Target
    target_type TEXT,                             -- order, position, strategy, config
    target_id TEXT,

    -- Change details
    action TEXT NOT NULL,                         -- create, update, delete, execute
    changes TEXT,                                 -- JSON: {"field": {"old": x, "new": y}}

    -- Context
    ip_address TEXT,
    user_agent TEXT,

    -- Result
    success INTEGER NOT NULL DEFAULT 1,
    error_message TEXT,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_audit_category ON audit_log(event_category);
CREATE INDEX idx_audit_type ON audit_log(event_type);
CREATE INDEX idx_audit_target ON audit_log(target_type, target_id);
CREATE INDEX idx_audit_time ON audit_log(created_at DESC);
CREATE INDEX idx_audit_actor ON audit_log(actor_type, actor_id);

-- Application logs (structured)
CREATE TABLE IF NOT EXISTS app_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    level TEXT NOT NULL,                          -- trace, debug, info, warn, error
    target TEXT NOT NULL,                         -- module/component name

    message TEXT NOT NULL,

    -- Structured fields
    fields TEXT,                                  -- JSON

    -- Span info
    span_id TEXT,
    span_name TEXT,
    parent_span_id TEXT,

    -- Error info
    error_type TEXT,
    error_message TEXT,
    backtrace TEXT,

    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_app_logs_level ON app_logs(level);
CREATE INDEX idx_app_logs_target ON app_logs(target);
CREATE INDEX idx_app_logs_time ON app_logs(timestamp DESC);
CREATE INDEX idx_app_logs_span ON app_logs(span_id);
CREATE INDEX idx_app_logs_error ON app_logs(level) WHERE level IN ('warn', 'error');

-- ============================================================================
-- SECTION 14: CONFIGURATION & SETTINGS
-- ============================================================================

-- Key-value configuration store
CREATE TABLE IF NOT EXISTS config_store (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,                          -- JSON
    value_type TEXT NOT NULL DEFAULT 'string',    -- string, number, boolean, json

    category TEXT NOT NULL DEFAULT 'general',
    description TEXT,

    is_secret INTEGER DEFAULT 0,                  -- should be encrypted
    is_readonly INTEGER DEFAULT 0,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_config_category ON config_store(category);

-- User preferences
CREATE TABLE IF NOT EXISTS user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,                          -- JSON

    category TEXT NOT NULL DEFAULT 'ui',          -- ui, trading, notifications, display

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- SECTION 15: METADATA & SYSTEM
-- ============================================================================

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- System health metrics
CREATE TABLE IF NOT EXISTS system_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    metric_name TEXT NOT NULL,
    metric_value TEXT NOT NULL,

    tags TEXT,                                    -- JSON

    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_sys_metrics_name ON system_metrics(metric_name);
CREATE INDEX idx_sys_metrics_time ON system_metrics(timestamp DESC);

-- Data sync status
CREATE TABLE IF NOT EXISTS sync_status (
    id TEXT PRIMARY KEY,                          -- e.g., "candles:BTCUSDT:1h"

    sync_type TEXT NOT NULL,                      -- candles, orders, positions, balances

    last_sync_at TEXT,
    last_sync_status TEXT,                        -- success, partial, failed
    last_error TEXT,

    -- For incremental syncs
    last_id TEXT,
    last_timestamp INTEGER,

    -- Stats
    records_synced INTEGER DEFAULT 0,
    sync_duration_ms INTEGER,

    next_sync_at TEXT,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_sync_status_type ON sync_status(sync_type);
CREATE INDEX idx_sync_status_next ON sync_status(next_sync_at);

-- ============================================================================
-- INITIAL DATA
-- ============================================================================

-- Insert default exchange
INSERT OR IGNORE INTO exchanges (id, name, exchange_type, api_base_url, ws_base_url, status)
VALUES
    ('binance_futures', 'Binance Futures', 'cex', 'https://fapi.binance.com', 'wss://fstream.binance.com', 'active'),
    ('binance_spot', 'Binance Spot', 'cex', 'https://api.binance.com', 'wss://stream.binance.com:9443', 'active'),
    ('paper', 'Paper Trading', 'paper', NULL, NULL, 'active');

-- Insert schema version
INSERT INTO schema_migrations (version, name) VALUES ('00001', 'initial_schema');

-- Default configuration
INSERT OR IGNORE INTO config_store (key, value, value_type, category, description) VALUES
    ('app.version', '"0.1.0"', 'string', 'system', 'Application version'),
    ('app.trading_mode', '"paper"', 'string', 'trading', 'Current trading mode'),
    ('app.default_leverage', '1', 'number', 'trading', 'Default leverage for new positions'),
    ('app.max_daily_loss_pct', '0.05', 'number', 'risk', 'Maximum daily loss percentage'),
    ('app.data_retention_days', '365', 'number', 'system', 'Days to retain historical data');
