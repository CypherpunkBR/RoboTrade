# Schema do Banco de Dados

Este documento descreve o schema completo do SQLite utilizado pelo RoboTrade.

## Visão Geral

O banco de dados SQLite armazena todos os dados persistentes da aplicação, organizado em 15 seções lógicas:

1. **Exchange & Market Configuration** - Exchanges e símbolos suportados
2. **Market Data** - Candles, tickers, orderbook
3. **Technical Analysis** - Indicadores técnicos calculados
4. **Strategy Management** - Estratégias, versões, instâncias
5. **Order Management** - Ordens e execuções
6. **Position Management** - Posições e ajustes
7. **Trade History** - Trades fechados e performance
8. **Account & Balances** - Contas e saldos
9. **Risk Management** - Limites e eventos de risco
10. **Backtesting** - Resultados de backtests
11. **Job Queue** - Jobs e scheduling
12. **Notifications** - Alertas e notificações
13. **Audit & Logging** - Logs e auditoria
14. **Configuration** - Configurações da aplicação
15. **System Metadata** - Métricas e status do sistema

## Diagrama ER Completo

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    MARKET DATA LAYER                                        │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                             │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌───────────────────────┐  │
│  │  exchanges   │────►│   symbols    │────►│   candles    │     │   market_indicators   │  │
│  │              │     │              │     │              │     │   (fear_greed, etc)   │  │
│  └──────────────┘     └──────┬───────┘     └──────────────┘     └───────────────────────┘  │
│                              │                                                              │
│                              ├─────────────────────────────────────────────┐               │
│                              │                                             │               │
│                              ▼                                             ▼               │
│                    ┌──────────────────┐                        ┌─────────────────────┐     │
│                    │     tickers      │                        │ technical_indicators│     │
│                    └──────────────────┘                        └─────────────────────┘     │
│                              │                                                              │
│                              ▼                                                              │
│                    ┌──────────────────────┐                                                │
│                    │ orderbook_snapshots  │                                                │
│                    └──────────────────────┘                                                │
│                                                                                             │
└─────────────────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    STRATEGY LAYER                                           │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                             │
│  ┌──────────────┐     ┌───────────────────┐     ┌─────────────────────┐                    │
│  │  strategies  │────►│ strategy_versions │────►│ strategy_instances  │                    │
│  └──────────────┘     └───────────────────┘     └──────────┬──────────┘                    │
│                                                             │                               │
│                                                             ▼                               │
│                                                    ┌──────────────┐                        │
│                                                    │   signals    │                        │
│                                                    └──────┬───────┘                        │
│                                                           │                                │
└───────────────────────────────────────────────────────────┼────────────────────────────────┘
                                                            │
┌───────────────────────────────────────────────────────────┼────────────────────────────────┐
│                                    TRADING LAYER          │                                │
├───────────────────────────────────────────────────────────┼────────────────────────────────┤
│                                                           │                                │
│                                          ┌────────────────┘                                │
│                                          ▼                                                 │
│  ┌──────────────┐     ┌──────────────┐  ┌──────────────┐     ┌─────────────────────┐      │
│  │   accounts   │     │    orders    │◄─┤  positions   │────►│ position_adjustments│      │
│  └──────┬───────┘     └──────┬───────┘  └──────┬───────┘     └─────────────────────┘      │
│         │                    │                 │                                           │
│         │                    ▼                 │                                           │
│         │            ┌──────────────┐          │                                           │
│         │            │ order_fills  │          │                                           │
│         │            └──────────────┘          │                                           │
│         │                                      │                                           │
│         ▼                                      ▼                                           │
│  ┌───────────────────┐              ┌──────────────┐     ┌──────────────────────┐         │
│  │ balance_snapshots │              │    trades    │────►│  daily_performance   │         │
│  └───────────────────┘              └──────────────┘     └──────────────────────┘         │
│                                                                                            │
└────────────────────────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    RISK & COMPLIANCE LAYER                                 │
├────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────────────┐                       │
│  │ risk_limits  │────►│ risk_events  │     │  alert_definitions   │                       │
│  └──────────────┘     └──────────────┘     └───────────┬──────────┘                       │
│                                                        │                                   │
│                                                        ▼                                   │
│                                              ┌──────────────────┐                          │
│                                              │  notifications   │                          │
│                                              └──────────────────┘                          │
│                                                                                            │
└────────────────────────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    SYSTEM & JOBS LAYER                                     │
├────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌─────────────────────┐   │
│  │     jobs     │────►│ job_history  │     │scheduled_tasks│     │   backtest_runs    │   │
│  └──────────────┘     └──────────────┘     └──────────────┘     └─────────────────────┘   │
│                                                                                            │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌─────────────────────┐   │
│  │  audit_log   │     │   app_logs   │     │ config_store │     │  user_preferences   │   │
│  └──────────────┘     └──────────────┘     └──────────────┘     └─────────────────────┘   │
│                                                                                            │
│  ┌───────────────────┐     ┌──────────────────┐                                           │
│  │ schema_migrations │     │  system_metrics  │                                           │
│  └───────────────────┘     └──────────────────┘                                           │
│                                                                                            │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Tabelas por Seção

### 1. Exchange & Market Configuration

#### exchanges

Exchanges suportadas pelo sistema.

```sql
CREATE TABLE exchanges (
    id TEXT PRIMARY KEY,                          -- 'binance_futures', 'binance_spot'
    name TEXT NOT NULL,                           -- 'Binance Futures'
    exchange_type TEXT NOT NULL,                  -- 'cex', 'dex', 'paper'
    api_base_url TEXT,
    ws_base_url TEXT,
    status TEXT NOT NULL DEFAULT 'active',        -- active, maintenance, disabled
    rate_limit_requests INTEGER DEFAULT 1200,
    rate_limit_orders INTEGER DEFAULT 10,
    supported_features TEXT,                      -- JSON: ['spot', 'futures', 'margin']
    metadata TEXT,                                -- JSON
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### symbols

Pares de trading disponíveis.

```sql
CREATE TABLE symbols (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id),
    symbol TEXT NOT NULL,                         -- 'BTCUSDT'
    base_asset TEXT NOT NULL,                     -- 'BTC'
    quote_asset TEXT NOT NULL,                    -- 'USDT'
    symbol_type TEXT NOT NULL DEFAULT 'spot',     -- spot, perpetual, delivery, option
    status TEXT NOT NULL DEFAULT 'active',

    -- Precision rules
    price_precision INTEGER NOT NULL,
    quantity_precision INTEGER NOT NULL,
    quote_precision INTEGER DEFAULT 8,

    -- Trading limits
    min_quantity TEXT NOT NULL,
    max_quantity TEXT,
    min_notional TEXT NOT NULL,
    max_notional TEXT,
    tick_size TEXT NOT NULL,
    step_size TEXT NOT NULL,

    -- Futures specific
    contract_type TEXT,                           -- perpetual, current_quarter
    contract_size TEXT,
    margin_asset TEXT,
    maintenance_margin_rate TEXT,
    max_leverage INTEGER,

    -- Fees
    maker_fee TEXT DEFAULT '0.001',
    taker_fee TEXT DEFAULT '0.001',

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(exchange_id, symbol)
);
```

### 2. Market Data

#### candles

Candlesticks históricos (OHLCV).

```sql
CREATE TABLE candles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),
    timeframe TEXT NOT NULL,                      -- 1m, 5m, 15m, 1h, 4h, 1d, 1w
    open_time INTEGER NOT NULL,                   -- Unix timestamp ms
    close_time INTEGER NOT NULL,

    -- OHLCV (TEXT para precisão Decimal)
    open TEXT NOT NULL,
    high TEXT NOT NULL,
    low TEXT NOT NULL,
    close TEXT NOT NULL,
    volume TEXT NOT NULL,
    quote_volume TEXT NOT NULL,

    -- Additional metrics
    trade_count INTEGER DEFAULT 0,
    taker_buy_volume TEXT,
    taker_buy_quote_volume TEXT,

    -- Computed (analytics)
    vwap TEXT,
    typical_price TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(symbol_id, timeframe, open_time)
);

CREATE INDEX idx_candles_lookup ON candles(symbol_id, timeframe, open_time DESC);
```

#### tickers

Snapshots de preço em tempo real.

```sql
CREATE TABLE tickers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    bid_price TEXT NOT NULL,
    bid_qty TEXT,
    ask_price TEXT NOT NULL,
    ask_qty TEXT,
    last_price TEXT NOT NULL,

    -- 24h stats
    price_change TEXT,
    price_change_pct TEXT,
    weighted_avg_price TEXT,
    high_price TEXT,
    low_price TEXT,
    volume TEXT,
    quote_volume TEXT,

    -- Futures
    funding_rate TEXT,
    next_funding_time INTEGER,
    open_interest TEXT,

    timestamp INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### market_indicators

Indicadores externos (Fear & Greed, etc).

```sql
CREATE TABLE market_indicators (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    indicator_type TEXT NOT NULL,                 -- fear_greed, btc_dominance, etc
    provider TEXT NOT NULL,                       -- alternative_me, coingecko

    value TEXT NOT NULL,
    value_classification TEXT,                    -- extreme_fear, fear, neutral, greed, extreme_greed
    secondary_value TEXT,

    timestamp INTEGER NOT NULL,
    valid_until INTEGER,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(indicator_type, provider, timestamp)
);
```

### 3. Technical Analysis

#### technical_indicators

Indicadores técnicos calculados.

```sql
CREATE TABLE technical_indicators (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),
    timeframe TEXT NOT NULL,
    timestamp INTEGER NOT NULL,

    indicator_name TEXT NOT NULL,                 -- sma, ema, rsi, macd, bb, atr
    indicator_params TEXT NOT NULL,               -- JSON: {"period": 14}

    value_1 TEXT NOT NULL,                        -- main value
    value_2 TEXT,                                 -- secondary (MACD signal)
    value_3 TEXT,                                 -- tertiary (MACD histogram)
    value_4 TEXT,                                 -- extra (BB middle)

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(symbol_id, timeframe, timestamp, indicator_name, indicator_params)
);
```

### 4. Strategy Management

#### strategies

Definições de estratégias.

```sql
CREATE TABLE strategies (
    id TEXT PRIMARY KEY,                          -- UUID
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    strategy_type TEXT NOT NULL,                  -- fear_greed, sma_crossover, custom

    default_symbols TEXT,                         -- JSON array
    default_timeframes TEXT,                      -- JSON array

    is_enabled INTEGER NOT NULL DEFAULT 0,
    is_live_allowed INTEGER NOT NULL DEFAULT 0,

    author TEXT,
    tags TEXT,
    documentation_url TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### strategy_versions

Versões imutáveis de configuração.

```sql
CREATE TABLE strategy_versions (
    id TEXT PRIMARY KEY,                          -- UUID
    strategy_id TEXT NOT NULL REFERENCES strategies(id),

    version TEXT NOT NULL,                        -- semver: 1.0.0
    version_major INTEGER NOT NULL,
    version_minor INTEGER NOT NULL,
    version_patch INTEGER NOT NULL,

    -- Immutable configuration
    config TEXT NOT NULL,                         -- JSON
    indicators_config TEXT,                       -- JSON
    entry_rules TEXT NOT NULL,                    -- JSON
    exit_rules TEXT NOT NULL,                     -- JSON
    filters TEXT,                                 -- JSON
    risk_config TEXT NOT NULL,                    -- JSON

    is_active INTEGER NOT NULL DEFAULT 0,

    change_description TEXT NOT NULL,
    parent_version_id TEXT REFERENCES strategy_versions(id),
    performance_metrics TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT DEFAULT 'user',

    UNIQUE(strategy_id, version)
);
```

#### strategy_instances

Instâncias de execução de estratégias.

```sql
CREATE TABLE strategy_instances (
    id TEXT PRIMARY KEY,
    strategy_version_id TEXT NOT NULL REFERENCES strategy_versions(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    trading_mode TEXT NOT NULL DEFAULT 'paper',   -- paper, live
    timeframe TEXT NOT NULL,
    config_overrides TEXT,                        -- JSON

    status TEXT NOT NULL DEFAULT 'stopped',       -- stopped, running, paused, error
    last_evaluation_at TEXT,
    last_signal_at TEXT,
    last_error TEXT,
    error_count INTEGER DEFAULT 0,

    signals_generated INTEGER DEFAULT 0,
    trades_executed INTEGER DEFAULT 0,

    started_at TEXT,
    stopped_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### signals

Sinais de trading gerados.

```sql
CREATE TABLE signals (
    id TEXT PRIMARY KEY,
    strategy_instance_id TEXT NOT NULL REFERENCES strategy_instances(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    signal_type TEXT NOT NULL,                    -- entry, exit, scale_in, scale_out
    direction TEXT NOT NULL,                      -- long, short, close
    strength TEXT NOT NULL,                       -- 0.0 to 1.0

    -- Targets
    entry_price TEXT,
    stop_loss TEXT,
    take_profit_1 TEXT,
    take_profit_2 TEXT,
    take_profit_3 TEXT,
    trailing_stop_pct TEXT,

    -- Sizing
    suggested_quantity TEXT,
    suggested_risk_pct TEXT,
    risk_reward_ratio TEXT,

    -- Reasoning
    primary_reason TEXT NOT NULL,
    secondary_reasons TEXT,                       -- JSON
    indicator_values TEXT,                        -- JSON
    market_context TEXT,                          -- JSON

    status TEXT NOT NULL DEFAULT 'pending',       -- pending, confirmed, executed, expired, cancelled
    confirmation_required INTEGER DEFAULT 0,
    confirmed_at TEXT,

    order_id TEXT REFERENCES orders(id),
    position_id TEXT REFERENCES positions(id),

    valid_from TEXT NOT NULL DEFAULT (datetime('now')),
    valid_until TEXT,
    executed_at TEXT,

    candle_open_time INTEGER,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 5. Order Management

#### orders

Ordens de trading.

```sql
CREATE TABLE orders (
    id TEXT PRIMARY KEY,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    exchange_order_id TEXT,
    client_order_id TEXT NOT NULL,

    side TEXT NOT NULL,                           -- buy, sell
    order_type TEXT NOT NULL,                     -- market, limit, stop_market, stop_limit, etc
    time_in_force TEXT DEFAULT 'GTC',             -- GTC, IOC, FOK

    quantity TEXT NOT NULL,
    executed_qty TEXT NOT NULL DEFAULT '0',
    remaining_qty TEXT,

    price TEXT,                                   -- limit price
    stop_price TEXT,                              -- trigger price
    avg_fill_price TEXT,

    reduce_only INTEGER DEFAULT 0,
    close_position INTEGER DEFAULT 0,
    position_side TEXT,                           -- LONG, SHORT, BOTH

    status TEXT NOT NULL DEFAULT 'pending_new',
    reject_reason TEXT,

    commission TEXT DEFAULT '0',
    commission_asset TEXT,

    signal_id TEXT REFERENCES signals(id),
    position_id TEXT REFERENCES positions(id),
    parent_order_id TEXT REFERENCES orders(id),
    order_group_id TEXT,
    order_group_type TEXT,                        -- oco, bracket, oto

    trading_mode TEXT NOT NULL DEFAULT 'paper',

    submitted_at TEXT,
    acknowledged_at TEXT,
    last_fill_at TEXT,
    completed_at TEXT,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_orders_open ON orders(status) WHERE status IN ('pending_new', 'new', 'partially_filled');
```

#### order_fills

Execuções de ordens.

```sql
CREATE TABLE order_fills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,

    fill_id TEXT,
    price TEXT NOT NULL,
    quantity TEXT NOT NULL,
    quote_quantity TEXT,

    commission TEXT NOT NULL,
    commission_asset TEXT NOT NULL,
    is_maker INTEGER,

    filled_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 6. Position Management

#### positions

Posições abertas e fechadas.

```sql
CREATE TABLE positions (
    id TEXT PRIMARY KEY,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    side TEXT NOT NULL,                           -- long, short
    status TEXT NOT NULL DEFAULT 'open',          -- open, closing, closed

    quantity TEXT NOT NULL,
    initial_quantity TEXT NOT NULL,

    entry_price TEXT NOT NULL,
    entry_value TEXT NOT NULL,
    entry_order_id TEXT REFERENCES orders(id),

    exit_price TEXT,
    exit_value TEXT,
    exit_order_id TEXT REFERENCES orders(id),

    leverage INTEGER NOT NULL DEFAULT 1,
    margin_type TEXT DEFAULT 'cross',
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

    -- P&L
    unrealized_pnl TEXT DEFAULT '0',
    realized_pnl TEXT DEFAULT '0',
    total_fees TEXT DEFAULT '0',
    funding_fees TEXT DEFAULT '0',

    -- Metrics
    max_profit TEXT,
    max_drawdown TEXT,
    risk_amount TEXT,

    signal_id TEXT REFERENCES signals(id),
    strategy_instance_id TEXT REFERENCES strategy_instances(id),

    trading_mode TEXT NOT NULL DEFAULT 'paper',

    opened_at TEXT NOT NULL DEFAULT (datetime('now')),
    closed_at TEXT,
    duration_seconds INTEGER,

    notes TEXT,
    tags TEXT,
    metadata TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_positions_open ON positions(status) WHERE status = 'open';
```

#### position_adjustments

Log de ajustes em posições.

```sql
CREATE TABLE position_adjustments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    position_id TEXT NOT NULL REFERENCES positions(id),

    adjustment_type TEXT NOT NULL,                -- scale_in, scale_out, adjust_stop, etc

    quantity_before TEXT,
    quantity_after TEXT,
    avg_price_before TEXT,
    avg_price_after TEXT,
    stop_loss_before TEXT,
    stop_loss_after TEXT,

    order_id TEXT REFERENCES orders(id),
    reason TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 7. Trade History & Performance

#### trades

Trades fechados com P&L completo.

```sql
CREATE TABLE trades (
    id TEXT PRIMARY KEY,
    position_id TEXT NOT NULL REFERENCES positions(id),
    exchange_id TEXT NOT NULL REFERENCES exchanges(id),
    symbol_id INTEGER NOT NULL REFERENCES symbols(id),

    side TEXT NOT NULL,
    quantity TEXT NOT NULL,

    entry_price TEXT NOT NULL,
    entry_value TEXT NOT NULL,
    entry_time TEXT NOT NULL,
    entry_order_id TEXT REFERENCES orders(id),

    exit_price TEXT NOT NULL,
    exit_value TEXT NOT NULL,
    exit_time TEXT NOT NULL,
    exit_order_id TEXT REFERENCES orders(id),
    exit_reason TEXT,                             -- stop_loss, take_profit, manual, signal

    gross_pnl TEXT NOT NULL,
    total_fees TEXT NOT NULL,
    funding_fees TEXT DEFAULT '0',
    net_pnl TEXT NOT NULL,
    return_pct TEXT NOT NULL,
    r_multiple TEXT,

    duration_seconds INTEGER NOT NULL,
    max_favorable_excursion TEXT,
    max_adverse_excursion TEXT,

    strategy_instance_id TEXT REFERENCES strategy_instances(id),
    strategy_version_id TEXT REFERENCES strategy_versions(id),
    signal_id TEXT REFERENCES signals(id),

    trading_mode TEXT NOT NULL DEFAULT 'paper',

    tags TEXT,
    notes TEXT,
    quality_score INTEGER,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### daily_performance

Agregação diária de performance.

```sql
CREATE TABLE daily_performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,                           -- YYYY-MM-DD
    trading_mode TEXT NOT NULL,

    starting_balance TEXT NOT NULL,
    ending_balance TEXT NOT NULL,

    gross_pnl TEXT NOT NULL,
    fees TEXT NOT NULL,
    funding TEXT DEFAULT '0',
    net_pnl TEXT NOT NULL,
    return_pct TEXT NOT NULL,

    total_trades INTEGER NOT NULL DEFAULT 0,
    winning_trades INTEGER NOT NULL DEFAULT 0,
    losing_trades INTEGER NOT NULL DEFAULT 0,

    total_win_amount TEXT DEFAULT '0',
    total_loss_amount TEXT DEFAULT '0',
    largest_win TEXT,
    largest_loss TEXT,

    win_rate TEXT,
    profit_factor TEXT,
    max_drawdown TEXT,

    by_symbol TEXT,                               -- JSON breakdown
    by_strategy TEXT,                             -- JSON breakdown

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(date, trading_mode)
);
```

### 8. Account & Balances

#### accounts

Configuração de contas por exchange.

```sql
CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id),

    account_type TEXT NOT NULL DEFAULT 'main',
    alias TEXT,

    api_key_ref TEXT,                             -- keyring reference
    has_trade_permission INTEGER DEFAULT 0,
    has_withdraw_permission INTEGER DEFAULT 0,
    ip_whitelist TEXT,

    status TEXT NOT NULL DEFAULT 'active',
    last_sync_at TEXT,
    sync_error TEXT,

    default_leverage INTEGER DEFAULT 1,
    margin_type TEXT DEFAULT 'cross',
    position_mode TEXT DEFAULT 'one_way',

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(exchange_id, account_type, alias)
);
```

#### balance_snapshots

Snapshots de saldo.

```sql
CREATE TABLE balance_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL REFERENCES accounts(id),

    snapshot_type TEXT NOT NULL DEFAULT 'periodic',

    total_balance_usdt TEXT NOT NULL,
    available_balance_usdt TEXT NOT NULL,
    locked_balance_usdt TEXT NOT NULL,

    total_margin TEXT,
    used_margin TEXT,
    available_margin TEXT,
    margin_level TEXT,

    unrealized_pnl TEXT DEFAULT '0',

    assets TEXT NOT NULL,                         -- JSON array

    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 9. Risk Management

#### risk_limits

Configuração de limites de risco.

```sql
CREATE TABLE risk_limits (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,

    applies_to TEXT NOT NULL DEFAULT 'global',    -- global, exchange, symbol, strategy
    exchange_id TEXT REFERENCES exchanges(id),
    symbol_id INTEGER REFERENCES symbols(id),
    strategy_id TEXT REFERENCES strategies(id),

    -- Position limits
    max_position_size TEXT,
    max_position_value TEXT,
    max_positions_per_symbol INTEGER,
    max_total_positions INTEGER,

    -- Exposure limits
    max_total_exposure TEXT,
    max_leverage INTEGER,

    -- Loss limits
    max_loss_per_trade TEXT,
    max_loss_per_trade_pct TEXT,
    max_daily_loss TEXT,
    max_daily_loss_pct TEXT,
    max_weekly_loss TEXT,
    max_drawdown_pct TEXT,

    -- Order limits
    max_order_size TEXT,
    max_orders_per_minute INTEGER,

    -- Circuit breaker
    consecutive_loss_limit INTEGER,
    pause_after_losses_minutes INTEGER,

    is_enabled INTEGER NOT NULL DEFAULT 1,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### risk_events

Log de eventos de risco.

```sql
CREATE TABLE risk_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    event_type TEXT NOT NULL,                     -- limit_breach, circuit_breaker, warning
    severity TEXT NOT NULL,                       -- info, warning, critical

    risk_limit_id TEXT REFERENCES risk_limits(id),
    account_id TEXT REFERENCES accounts(id),
    symbol_id INTEGER REFERENCES symbols(id),
    position_id TEXT REFERENCES positions(id),
    order_id TEXT REFERENCES orders(id),

    limit_name TEXT,
    limit_value TEXT,
    actual_value TEXT,
    breach_pct TEXT,

    action_taken TEXT,

    resolved INTEGER DEFAULT 0,
    resolved_at TEXT,
    resolution_notes TEXT,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 10. Backtesting

#### backtests

Execuções de backtest.

```sql
CREATE TABLE backtests (
    id TEXT PRIMARY KEY,
    strategy_version_id TEXT NOT NULL REFERENCES strategy_versions(id),

    symbol_id INTEGER NOT NULL REFERENCES symbols(id),
    timeframe TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,

    initial_capital TEXT NOT NULL,
    commission_rate TEXT NOT NULL,
    slippage_bps INTEGER DEFAULT 0,
    fill_model TEXT DEFAULT 'close',
    position_sizing TEXT DEFAULT 'fixed',

    status TEXT NOT NULL DEFAULT 'pending',       -- pending, running, completed, failed
    progress_pct INTEGER DEFAULT 0,
    error_message TEXT,

    started_at TEXT,
    completed_at TEXT,
    execution_time_ms INTEGER,

    -- Results
    final_capital TEXT,
    total_return TEXT,
    total_return_pct TEXT,
    annualized_return TEXT,

    sharpe_ratio TEXT,
    sortino_ratio TEXT,
    max_drawdown TEXT,
    max_drawdown_pct TEXT,

    total_trades INTEGER,
    winning_trades INTEGER,
    losing_trades INTEGER,
    win_rate TEXT,
    profit_factor TEXT,

    avg_trade_pnl TEXT,
    largest_win TEXT,
    largest_loss TEXT,

    equity_curve TEXT,                            -- JSON
    monthly_returns TEXT,                         -- JSON
    trades_json TEXT,                             -- JSON

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 11. Job Queue & Scheduling

#### jobs

Fila de jobs em background.

```sql
CREATE TABLE jobs (
    id TEXT PRIMARY KEY,

    job_type TEXT NOT NULL,
    queue TEXT NOT NULL DEFAULT 'default',
    priority INTEGER NOT NULL DEFAULT 2,          -- 0=critical, 3=low

    payload TEXT NOT NULL,                        -- JSON

    scheduled_for TEXT NOT NULL DEFAULT (datetime('now')),
    not_before TEXT,
    deadline TEXT,

    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,

    worker_id TEXT,
    locked_at TEXT,
    lock_expires_at TEXT,

    result TEXT,
    last_error TEXT,
    error_count INTEGER DEFAULT 0,

    started_at TEXT,
    completed_at TEXT,
    execution_time_ms INTEGER,

    depends_on TEXT,                              -- JSON array
    idempotency_key TEXT UNIQUE,
    metadata TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_jobs_pending ON jobs(queue, priority, scheduled_for)
    WHERE status IN ('pending', 'scheduled');
```

#### scheduled_tasks

Tarefas recorrentes (cron-like).

```sql
CREATE TABLE scheduled_tasks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,

    job_type TEXT NOT NULL,
    payload_template TEXT NOT NULL,
    queue TEXT DEFAULT 'default',
    priority INTEGER DEFAULT 2,

    schedule_type TEXT NOT NULL,                  -- cron, interval, fixed_time
    cron_expression TEXT,
    interval_seconds INTEGER,
    timezone TEXT DEFAULT 'UTC',

    is_enabled INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    next_run_at TEXT,
    last_job_id TEXT REFERENCES jobs(id),

    max_concurrent INTEGER DEFAULT 1,
    timeout_seconds INTEGER DEFAULT 300,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 12. Notifications & Alerts

#### alert_definitions

Definições de alertas.

```sql
CREATE TABLE alert_definitions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,

    alert_type TEXT NOT NULL,                     -- price, indicator, position, risk, system
    conditions TEXT NOT NULL,                     -- JSON

    symbol_id INTEGER REFERENCES symbols(id),
    strategy_id TEXT REFERENCES strategies(id),

    notification_channels TEXT NOT NULL,          -- JSON: ["desktop", "sound"]
    message_template TEXT,

    is_enabled INTEGER NOT NULL DEFAULT 1,
    is_recurring INTEGER DEFAULT 0,
    cooldown_seconds INTEGER DEFAULT 300,

    last_triggered_at TEXT,
    trigger_count INTEGER DEFAULT 0,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### notifications

Notificações geradas.

```sql
CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    notification_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'info',

    title TEXT NOT NULL,
    message TEXT NOT NULL,

    alert_definition_id TEXT REFERENCES alert_definitions(id),
    signal_id TEXT REFERENCES signals(id),
    order_id TEXT REFERENCES orders(id),
    position_id TEXT REFERENCES positions(id),

    channels_sent TEXT,

    is_read INTEGER DEFAULT 0,
    read_at TEXT,
    is_dismissed INTEGER DEFAULT 0,
    dismissed_at TEXT,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 13. Audit & Logging

#### audit_log

Log de auditoria.

```sql
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    event_category TEXT NOT NULL,                 -- auth, config, trading, system
    event_type TEXT NOT NULL,

    actor_type TEXT DEFAULT 'system',
    actor_id TEXT,

    target_type TEXT,
    target_id TEXT,

    action TEXT NOT NULL,                         -- create, update, delete, execute
    changes TEXT,                                 -- JSON

    ip_address TEXT,
    user_agent TEXT,

    success INTEGER NOT NULL DEFAULT 1,
    error_message TEXT,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### app_logs

Logs estruturados da aplicação.

```sql
CREATE TABLE app_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    level TEXT NOT NULL,                          -- trace, debug, info, warn, error
    target TEXT NOT NULL,
    message TEXT NOT NULL,

    fields TEXT,                                  -- JSON
    span_id TEXT,
    span_name TEXT,
    parent_span_id TEXT,

    error_type TEXT,
    error_message TEXT,
    backtrace TEXT,

    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 14. Configuration

#### config_store

Configurações key-value.

```sql
CREATE TABLE config_store (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,                          -- JSON
    value_type TEXT NOT NULL DEFAULT 'string',

    category TEXT NOT NULL DEFAULT 'general',
    description TEXT,

    is_secret INTEGER DEFAULT 0,
    is_readonly INTEGER DEFAULT 0,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### user_preferences

Preferências do usuário.

```sql
CREATE TABLE user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,                          -- JSON
    category TEXT NOT NULL DEFAULT 'ui',

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 15. System Metadata

#### schema_migrations

Tracking de migrations.

```sql
CREATE TABLE schema_migrations (
    version TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### system_metrics

Métricas do sistema.

```sql
CREATE TABLE system_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_name TEXT NOT NULL,
    metric_value TEXT NOT NULL,
    tags TEXT,                                    -- JSON
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### sync_status

Status de sincronização de dados.

```sql
CREATE TABLE sync_status (
    id TEXT PRIMARY KEY,
    sync_type TEXT NOT NULL,

    last_sync_at TEXT,
    last_sync_status TEXT,
    last_error TEXT,

    last_id TEXT,
    last_timestamp INTEGER,

    records_synced INTEGER DEFAULT 0,
    sync_duration_ms INTEGER,

    next_sync_at TEXT,

    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

## Views Pré-definidas

O schema inclui views para consultas comuns:

| View | Descrição |
|------|-----------|
| `v_latest_tickers` | Último ticker por símbolo |
| `v_latest_candles` | Último candle por símbolo/timeframe |
| `v_latest_fear_greed` | Último índice Fear & Greed |
| `v_open_positions` | Posições abertas com P&L calculado |
| `v_open_orders` | Ordens abertas |
| `v_trade_stats_by_symbol` | Estatísticas por símbolo |
| `v_trade_stats_by_strategy` | Estatísticas por estratégia |
| `v_daily_pnl` | P&L diário |
| `v_weekly_pnl` | P&L semanal |
| `v_monthly_pnl` | P&L mensal |
| `v_active_strategies` | Estratégias em execução |
| `v_pending_signals` | Sinais pendentes |
| `v_current_exposure` | Exposição atual por ativo |
| `v_risk_status` | Status de limites de risco |
| `v_job_queue_status` | Status da fila de jobs |
| `v_system_health` | Visão geral de saúde do sistema |
| `v_latest_balances` | Último saldo por conta |
| `v_account_summary` | Resumo de contas |

## Triggers

O schema inclui triggers para:

1. **Auto-update timestamps** - Atualiza `updated_at` automaticamente
2. **Audit logging** - Registra mudanças em ordens, posições, configs
3. **Status tracking** - Atualiza campos derivados (completed_at, duration)
4. **Notifications** - Cria notificações para eventos críticos
5. **Cleanup** - Remove dados antigos automaticamente (tickers, orderbook)

## Índices

Índices otimizados para:

- Lookups por chave primária e foreign keys
- Queries de range em timestamps
- Filtros por status (orders, positions, jobs)
- Busca de dados mais recentes (DESC indexes)
- Índices parciais para queries frequentes

## Migrations

As migrations estão em `crates/infra/migrations/`:

| Migration | Descrição |
|-----------|-----------|
| `00001_initial_schema.sql` | Schema completo inicial |
| `00002_views_and_functions.sql` | Views e funções utilitárias |
| `00003_triggers_and_maintenance.sql` | Triggers e manutenção |

Execute com:

```bash
# Via aplicação
cargo run -p robotrade-infra --bin migrate

# Ou via sqlx-cli
sqlx migrate run --source crates/infra/migrations
```

## Considerações de Performance

### Configurações Recomendadas

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 30000;
```

### Manutenção Periódica

```sql
-- Otimizar banco (executar periodicamente)
VACUUM;
ANALYZE;

-- Cleanup de dados antigos
DELETE FROM candles WHERE open_time < strftime('%s', 'now', '-2 years') * 1000;
DELETE FROM app_logs WHERE timestamp < datetime('now', '-30 days');
DELETE FROM jobs WHERE status IN ('completed', 'cancelled') AND completed_at < datetime('now', '-7 days');
DELETE FROM tickers WHERE timestamp < (strftime('%s', 'now') - 86400) * 1000;
```

---

**Total de Tabelas**: 35+
**Views**: 18
**Triggers**: 20+
**Índices**: 50+
