# Schema do Banco de Dados

Este documento descreve o schema completo do SQLite utilizado pelo RoboTrade.

## Visão Geral

O banco de dados SQLite armazena todos os dados persistentes da aplicação:
- Dados de mercado (candles, tickers)
- Indicadores calculados
- Sinais gerados
- Ordens e posições
- Histórico de trades
- Jobs do worker
- Configurações e logs

## Diagrama ER

```
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│     symbols     │       │     candles     │       │   indicators    │
├─────────────────┤       ├─────────────────┤       ├─────────────────┤
│ id              │───┐   │ id              │   ┌───│ id              │
│ symbol          │   │   │ symbol          │───┤   │ symbol          │
│ exchange        │   └──►│ timeframe       │   │   │ timeframe       │
│ base_asset      │       │ open_time       │   │   │ indicator_type  │
│ quote_asset     │       │ close_time      │   │   │ timestamp       │
│ status          │       │ open            │   │   │ value           │
│ ...             │       │ high            │   │   │ metadata        │
└─────────────────┘       │ low             │   │   │ ...             │
                          │ close           │   │   └─────────────────┘
                          │ volume          │   │
                          │ ...             │   │   ┌─────────────────┐
                          └─────────────────┘   │   │     signals     │
                                                │   ├─────────────────┤
┌─────────────────┐       ┌─────────────────┐   └──►│ id              │
│ fear_greed_index│       │ strategy_configs│       │ strategy_id     │
├─────────────────┤       ├─────────────────┤       │ symbol          │
│ id              │       │ id              │───────│ signal_type     │
│ date            │       │ strategy_id     │       │ side            │
│ value           │       │ name            │       │ entry_price     │
│ classification  │       │ version         │       │ stop_loss       │
│ timestamp       │       │ config          │       │ take_profit     │
│ ...             │       │ enabled         │       │ status          │
└─────────────────┘       │ ...             │       │ ...             │
                          └─────────────────┘       └─────────────────┘
                                                            │
┌─────────────────┐       ┌─────────────────┐               │
│      jobs       │       │     orders      │◄──────────────┘
├─────────────────┤       ├─────────────────┤
│ id              │───────│ id              │       ┌─────────────────┐
│ payload         │       │ exchange        │       │    positions    │
│ priority        │       │ symbol          │       ├─────────────────┤
│ status          │       │ side            │       │ id              │
│ retries         │       │ order_type      │───────│ exchange        │
│ scheduled_for   │       │ quantity        │       │ symbol          │
│ last_error      │       │ price           │       │ side            │
│ ...             │       │ status          │       │ quantity        │
└─────────────────┘       │ filled_quantity │       │ entry_price     │
                          │ ...             │       │ leverage        │
                          └─────────────────┘       │ pnl             │
                                    │               │ ...             │
                                    │               └─────────────────┘
                                    │                       │
                                    ▼                       │
                          ┌─────────────────┐               │
                          │     trades      │◄──────────────┘
                          ├─────────────────┤
                          │ id              │
                          │ position_id     │
                          │ order_id        │
                          │ symbol          │
                          │ side            │
                          │ quantity        │
                          │ entry_price     │
                          │ exit_price      │
                          │ pnl             │
                          │ fees            │
                          │ ...             │
                          └─────────────────┘

┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│    backtests    │       │ execution_logs  │       │      logs       │
├─────────────────┤       ├─────────────────┤       ├─────────────────┤
│ id              │       │ id              │       │ id              │
│ strategy_id     │       │ job_id          │       │ level           │
│ symbol          │       │ action          │       │ target          │
│ timeframe       │       │ status          │       │ message         │
│ start_date      │       │ details         │       │ context         │
│ end_date        │       │ error           │       │ timestamp       │
│ initial_capital │       │ executed_at     │       │ ...             │
│ final_capital   │       │ ...             │       └─────────────────┘
│ metrics         │       └─────────────────┘
│ trades          │
│ ...             │
└─────────────────┘
```

## Tabelas Detalhadas

### symbols

Armazena informações sobre os pares de trading suportados.

```sql
CREATE TABLE symbols (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    exchange TEXT NOT NULL,
    base_asset TEXT NOT NULL,
    quote_asset TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',  -- active, inactive, delisted
    price_precision INTEGER NOT NULL,
    quantity_precision INTEGER NOT NULL,
    min_quantity TEXT NOT NULL,
    max_quantity TEXT,
    tick_size TEXT NOT NULL,
    step_size TEXT NOT NULL,
    min_notional TEXT,
    contract_type TEXT,  -- perpetual, quarterly, etc.
    margin_asset TEXT,
    metadata TEXT,  -- JSON
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(symbol, exchange)
);

CREATE INDEX idx_symbols_exchange ON symbols(exchange);
CREATE INDEX idx_symbols_status ON symbols(status);
```

### candles

Armazena candlesticks de preço históricos.

```sql
CREATE TABLE candles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,  -- 1m, 5m, 15m, 1h, 4h, 1d, 1w
    open_time TEXT NOT NULL,
    close_time TEXT NOT NULL,
    open TEXT NOT NULL,       -- Decimal como string
    high TEXT NOT NULL,
    low TEXT NOT NULL,
    close TEXT NOT NULL,
    volume TEXT NOT NULL,
    quote_volume TEXT NOT NULL,
    trades_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(symbol, timeframe, open_time)
);

CREATE INDEX idx_candles_symbol_timeframe ON candles(symbol, timeframe);
CREATE INDEX idx_candles_open_time ON candles(open_time);
CREATE INDEX idx_candles_lookup ON candles(symbol, timeframe, open_time DESC);
```

### fear_greed_index

Armazena histórico do índice Fear & Greed.

```sql
CREATE TABLE fear_greed_index (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE,  -- YYYY-MM-DD
    value INTEGER NOT NULL,     -- 0-100
    classification TEXT NOT NULL,  -- extreme_fear, fear, neutral, greed, extreme_greed
    timestamp TEXT NOT NULL,
    provider TEXT NOT NULL DEFAULT 'alternative_me',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_fear_greed_date ON fear_greed_index(date DESC);
CREATE INDEX idx_fear_greed_value ON fear_greed_index(value);
```

### indicators

Armazena indicadores técnicos calculados.

```sql
CREATE TABLE indicators (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    indicator_type TEXT NOT NULL,  -- sma, ema, rsi, macd, bb, atr
    period INTEGER,                 -- Período do indicador
    timestamp TEXT NOT NULL,
    value TEXT NOT NULL,           -- Valor principal
    metadata TEXT,                 -- JSON para valores extras (ex: MACD histogram)
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(symbol, timeframe, indicator_type, period, timestamp)
);

CREATE INDEX idx_indicators_lookup ON indicators(symbol, timeframe, indicator_type, timestamp DESC);
```

### strategy_configs

Armazena configurações de estratégias.

```sql
CREATE TABLE strategy_configs (
    id TEXT PRIMARY KEY,           -- UUID
    strategy_id TEXT NOT NULL,     -- Identificador da estratégia
    name TEXT NOT NULL,
    description TEXT,
    version TEXT NOT NULL,
    config TEXT NOT NULL,          -- JSON com parâmetros
    enabled INTEGER NOT NULL DEFAULT 0,
    symbols TEXT NOT NULL,         -- JSON array de símbolos
    timeframes TEXT NOT NULL,      -- JSON array de timeframes
    risk_config TEXT,              -- JSON com configurações de risco
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(strategy_id, version)
);

CREATE INDEX idx_strategy_configs_enabled ON strategy_configs(enabled);
CREATE INDEX idx_strategy_configs_strategy_id ON strategy_configs(strategy_id);
```

### signals

Armazena sinais de trading gerados.

```sql
CREATE TABLE signals (
    id TEXT PRIMARY KEY,           -- UUID
    strategy_id TEXT NOT NULL,
    strategy_config_id TEXT REFERENCES strategy_configs(id),
    symbol TEXT NOT NULL,
    signal_type TEXT NOT NULL,     -- entry, exit, adjustment
    side TEXT NOT NULL,            -- buy, sell
    entry_price TEXT NOT NULL,
    stop_loss TEXT,
    take_profit TEXT,
    quantity TEXT,
    risk_reward_ratio TEXT,
    confidence TEXT NOT NULL,      -- 0.0 - 1.0
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, executed, expired, cancelled
    reason TEXT,                   -- Explicação do sinal
    metadata TEXT,                 -- JSON
    order_id TEXT REFERENCES orders(id),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TEXT,
    executed_at TEXT
);

CREATE INDEX idx_signals_status ON signals(status);
CREATE INDEX idx_signals_symbol ON signals(symbol);
CREATE INDEX idx_signals_strategy ON signals(strategy_id);
CREATE INDEX idx_signals_created ON signals(created_at DESC);
```

### orders

Armazena ordens de trading.

```sql
CREATE TABLE orders (
    id TEXT PRIMARY KEY,           -- UUID ou ID da exchange
    exchange TEXT NOT NULL,
    exchange_order_id TEXT,        -- ID na exchange
    client_order_id TEXT,          -- ID do cliente
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,            -- buy, sell
    order_type TEXT NOT NULL,      -- market, limit, stop_market, stop_limit, take_profit_market
    quantity TEXT NOT NULL,
    price TEXT,                    -- Para ordens limit
    stop_price TEXT,               -- Para ordens stop
    status TEXT NOT NULL,          -- pending, new, partially_filled, filled, cancelled, rejected, expired
    filled_quantity TEXT NOT NULL DEFAULT '0',
    average_price TEXT,
    commission TEXT,
    commission_asset TEXT,
    reduce_only INTEGER DEFAULT 0,
    signal_id TEXT REFERENCES signals(id),
    position_id TEXT REFERENCES positions(id),
    metadata TEXT,                 -- JSON
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_symbol ON orders(symbol);
CREATE INDEX idx_orders_exchange ON orders(exchange);
CREATE INDEX idx_orders_signal ON orders(signal_id);
CREATE INDEX idx_orders_position ON orders(position_id);
```

### positions

Armazena posições abertas e fechadas.

```sql
CREATE TABLE positions (
    id TEXT PRIMARY KEY,           -- UUID
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,            -- long, short
    status TEXT NOT NULL,          -- open, closed
    quantity TEXT NOT NULL,
    entry_price TEXT NOT NULL,
    current_price TEXT,
    liquidation_price TEXT,
    leverage INTEGER NOT NULL DEFAULT 1,
    margin TEXT,
    margin_type TEXT,              -- isolated, cross
    unrealized_pnl TEXT,
    realized_pnl TEXT DEFAULT '0',
    stop_loss TEXT,
    take_profit TEXT,
    trailing_stop TEXT,
    signal_id TEXT REFERENCES signals(id),
    entry_order_id TEXT REFERENCES orders(id),
    exit_order_id TEXT REFERENCES orders(id),
    opened_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    closed_at TEXT,
    metadata TEXT                  -- JSON
);

CREATE INDEX idx_positions_status ON positions(status);
CREATE INDEX idx_positions_symbol ON positions(symbol);
CREATE INDEX idx_positions_exchange ON positions(exchange);
CREATE INDEX idx_positions_open ON positions(status) WHERE status = 'open';
```

### trades

Armazena trades executados (posições fechadas com P&L).

```sql
CREATE TABLE trades (
    id TEXT PRIMARY KEY,           -- UUID
    position_id TEXT REFERENCES positions(id),
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,            -- long, short
    entry_order_id TEXT REFERENCES orders(id),
    exit_order_id TEXT REFERENCES orders(id),
    quantity TEXT NOT NULL,
    entry_price TEXT NOT NULL,
    exit_price TEXT NOT NULL,
    gross_pnl TEXT NOT NULL,
    fees TEXT NOT NULL DEFAULT '0',
    net_pnl TEXT NOT NULL,
    return_pct TEXT NOT NULL,
    duration_seconds INTEGER,
    strategy_id TEXT,
    signal_id TEXT REFERENCES signals(id),
    metadata TEXT,                 -- JSON
    opened_at TEXT NOT NULL,
    closed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_trades_symbol ON trades(symbol);
CREATE INDEX idx_trades_closed_at ON trades(closed_at DESC);
CREATE INDEX idx_trades_strategy ON trades(strategy_id);
CREATE INDEX idx_trades_pnl ON trades(net_pnl);
```

### jobs

Armazena jobs do worker.

```sql
CREATE TABLE jobs (
    id TEXT PRIMARY KEY,           -- UUID
    job_type TEXT NOT NULL,        -- place_order, cancel_order, sync_balances, etc.
    payload TEXT NOT NULL,         -- JSON
    priority INTEGER NOT NULL DEFAULT 2,  -- 0=critical, 1=high, 2=medium, 3=low
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, scheduled, running, completed, failed, cancelled
    retries INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    last_error TEXT,
    scheduled_for TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_priority ON jobs(priority);
CREATE INDEX idx_jobs_scheduled ON jobs(scheduled_for);
CREATE INDEX idx_jobs_pending ON jobs(status, priority, scheduled_for)
    WHERE status IN ('pending', 'scheduled');
```

### execution_logs

Armazena logs de execução de jobs.

```sql
CREATE TABLE execution_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT REFERENCES jobs(id),
    action TEXT NOT NULL,          -- started, completed, failed, retrying
    status TEXT NOT NULL,
    details TEXT,                  -- JSON
    error TEXT,
    duration_ms INTEGER,
    executed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_execution_logs_job ON execution_logs(job_id);
CREATE INDEX idx_execution_logs_executed ON execution_logs(executed_at DESC);
```

### backtests

Armazena resultados de backtests.

```sql
CREATE TABLE backtests (
    id TEXT PRIMARY KEY,           -- UUID
    strategy_id TEXT NOT NULL,
    strategy_config TEXT NOT NULL, -- JSON snapshot da config
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    initial_capital TEXT NOT NULL,
    final_capital TEXT NOT NULL,
    total_return TEXT NOT NULL,
    total_return_annualized TEXT,
    total_trades INTEGER NOT NULL,
    winning_trades INTEGER NOT NULL,
    losing_trades INTEGER NOT NULL,
    win_rate TEXT NOT NULL,
    profit_factor TEXT,
    sharpe_ratio TEXT,
    sortino_ratio TEXT,
    max_drawdown TEXT NOT NULL,
    max_drawdown_duration_seconds INTEGER,
    calmar_ratio TEXT,
    largest_win TEXT,
    largest_loss TEXT,
    average_win TEXT,
    average_loss TEXT,
    average_trade_duration_seconds INTEGER,
    exposure_time_pct TEXT,
    equity_curve TEXT NOT NULL,    -- JSON array
    drawdown_curve TEXT,           -- JSON array
    trades TEXT NOT NULL,          -- JSON array
    execution_time_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_backtests_strategy ON backtests(strategy_id);
CREATE INDEX idx_backtests_symbol ON backtests(symbol);
CREATE INDEX idx_backtests_created ON backtests(created_at DESC);
```

### logs

Armazena logs da aplicação (opcional, além do arquivo).

```sql
CREATE TABLE logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    level TEXT NOT NULL,           -- trace, debug, info, warn, error
    target TEXT NOT NULL,          -- Nome do módulo/componente
    message TEXT NOT NULL,
    context TEXT,                  -- JSON
    timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_logs_level ON logs(level);
CREATE INDEX idx_logs_target ON logs(target);
CREATE INDEX idx_logs_timestamp ON logs(timestamp DESC);

-- Auto-cleanup: manter apenas últimos 30 dias
-- (implementar via trigger ou job de manutenção)
```

### config_overrides

Armazena overrides de configuração em runtime.

```sql
CREATE TABLE config_overrides (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,           -- JSON
    description TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### balances

Armazena snapshot de saldos por exchange.

```sql
CREATE TABLE balances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exchange TEXT NOT NULL,
    asset TEXT NOT NULL,
    free TEXT NOT NULL,            -- Disponível
    locked TEXT NOT NULL,          -- Em ordens
    total TEXT NOT NULL,           -- free + locked
    usd_value TEXT,                -- Valor em USD
    timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(exchange, asset, timestamp)
);

CREATE INDEX idx_balances_exchange ON balances(exchange);
CREATE INDEX idx_balances_asset ON balances(asset);
CREATE INDEX idx_balances_timestamp ON balances(timestamp DESC);
```

### market_stats

Armazena estatísticas de mercado (24h stats).

```sql
CREATE TABLE market_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    exchange TEXT NOT NULL,
    price_change TEXT NOT NULL,
    price_change_pct TEXT NOT NULL,
    high_24h TEXT NOT NULL,
    low_24h TEXT NOT NULL,
    volume_24h TEXT NOT NULL,
    quote_volume_24h TEXT NOT NULL,
    open_price TEXT NOT NULL,
    last_price TEXT NOT NULL,
    trades_count_24h INTEGER,
    timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(symbol, exchange, timestamp)
);

CREATE INDEX idx_market_stats_symbol ON market_stats(symbol);
CREATE INDEX idx_market_stats_timestamp ON market_stats(timestamp DESC);
```

## Migrations

As migrations são gerenciadas pelo SQLx e ficam em `crates/infra/migrations/`.

### Estrutura de Migrations

```
crates/infra/migrations/
├── 20240101000000_create_symbols.sql
├── 20240101000001_create_candles.sql
├── 20240101000002_create_fear_greed.sql
├── 20240101000003_create_indicators.sql
├── 20240101000004_create_strategy_configs.sql
├── 20240101000005_create_signals.sql
├── 20240101000006_create_orders.sql
├── 20240101000007_create_positions.sql
├── 20240101000008_create_trades.sql
├── 20240101000009_create_jobs.sql
├── 20240101000010_create_execution_logs.sql
├── 20240101000011_create_backtests.sql
├── 20240101000012_create_logs.sql
├── 20240101000013_create_config_overrides.sql
├── 20240101000014_create_balances.sql
└── 20240101000015_create_market_stats.sql
```

### Exemplo de Migration

```sql
-- 20240101000001_create_candles.sql

-- Tabela de candles
CREATE TABLE IF NOT EXISTS candles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    open_time TEXT NOT NULL,
    close_time TEXT NOT NULL,
    open TEXT NOT NULL,
    high TEXT NOT NULL,
    low TEXT NOT NULL,
    close TEXT NOT NULL,
    volume TEXT NOT NULL,
    quote_volume TEXT NOT NULL,
    trades_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(symbol, timeframe, open_time)
);

-- Índices
CREATE INDEX IF NOT EXISTS idx_candles_symbol_timeframe ON candles(symbol, timeframe);
CREATE INDEX IF NOT EXISTS idx_candles_open_time ON candles(open_time);
CREATE INDEX IF NOT EXISTS idx_candles_lookup ON candles(symbol, timeframe, open_time DESC);
```

## Queries Comuns

### Buscar últimos N candles

```sql
SELECT * FROM candles
WHERE symbol = ? AND timeframe = ?
ORDER BY open_time DESC
LIMIT ?;
```

### Buscar candles em período

```sql
SELECT * FROM candles
WHERE symbol = ? AND timeframe = ?
AND open_time >= ? AND open_time <= ?
ORDER BY open_time ASC;
```

### Buscar próximo job pendente

```sql
SELECT * FROM jobs
WHERE status IN ('pending', 'scheduled')
AND scheduled_for <= datetime('now')
AND retries < max_retries
ORDER BY priority ASC, scheduled_for ASC
LIMIT 1;
```

### Calcular P&L por período

```sql
SELECT
    date(closed_at) as date,
    COUNT(*) as total_trades,
    SUM(CASE WHEN CAST(net_pnl AS REAL) > 0 THEN 1 ELSE 0 END) as winners,
    SUM(CASE WHEN CAST(net_pnl AS REAL) < 0 THEN 1 ELSE 0 END) as losers,
    SUM(CAST(net_pnl AS REAL)) as total_pnl,
    AVG(CAST(net_pnl AS REAL)) as avg_pnl
FROM trades
WHERE closed_at >= ? AND closed_at <= ?
GROUP BY date(closed_at)
ORDER BY date DESC;
```

### Posições abertas com P&L

```sql
SELECT
    p.*,
    (CAST(p.current_price AS REAL) - CAST(p.entry_price AS REAL)) *
    CAST(p.quantity AS REAL) *
    CASE WHEN p.side = 'long' THEN 1 ELSE -1 END as unrealized_pnl_calc
FROM positions p
WHERE p.status = 'open'
ORDER BY p.opened_at DESC;
```

## Considerações de Performance

### Índices

- Criar índices para colunas frequentemente usadas em WHERE, ORDER BY, JOIN
- Índices compostos para queries com múltiplas condições
- Índices parciais para queries com filtros fixos (ex: status = 'open')

### Vacuum

```sql
-- Executar periodicamente para otimizar
VACUUM;
ANALYZE;
```

### Cleanup de Dados Antigos

```sql
-- Remover candles antigos (manter últimos 2 anos)
DELETE FROM candles
WHERE open_time < datetime('now', '-2 years');

-- Remover logs antigos (manter últimos 30 dias)
DELETE FROM logs
WHERE timestamp < datetime('now', '-30 days');

-- Remover jobs completos antigos (manter últimos 7 dias)
DELETE FROM jobs
WHERE status IN ('completed', 'cancelled')
AND completed_at < datetime('now', '-7 days');
```

---

**Próximo**: [Tratamento de Erros](./error-handling.md)
