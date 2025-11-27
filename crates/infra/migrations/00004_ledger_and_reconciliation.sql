-- Migration: 00004_ledger_and_reconciliation.sql
-- Description: Ledger entries, cost basis tracking, reconciliation, and sync state
-- Author: RoboTrade Team
-- Date: 2025-11-27

-- ============================================================================
-- LEDGER ENTRIES
-- Double-entry lite system for tracking all financial movements
-- ============================================================================

CREATE TABLE IF NOT EXISTS ledger_entries (
    id TEXT PRIMARY KEY,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id) ON DELETE CASCADE,
    account_id TEXT,

    -- Entry classification
    entry_type TEXT NOT NULL CHECK (entry_type IN (
        'DEPOSIT',
        'WITHDRAWAL',
        'TRADE_PNL',
        'FEE',
        'FUNDING_PAYMENT',
        'TRANSFER_IN',
        'TRANSFER_OUT',
        'ADJUSTMENT',
        'LIQUIDATION',
        'COMMISSION_REBATE'
    )),

    -- Financial data
    asset TEXT NOT NULL,
    amount TEXT NOT NULL,           -- Decimal as string (can be negative)
    balance_after TEXT NOT NULL,    -- Running balance after this entry

    -- Reference to source event
    reference_type TEXT CHECK (reference_type IN (
        'trade',
        'order',
        'funding',
        'deposit',
        'withdrawal',
        'transfer',
        'liquidation',
        'manual'
    )),
    reference_id TEXT,
    external_id TEXT UNIQUE,        -- Exchange's unique ID for deduplication

    -- Metadata
    description TEXT,
    metadata TEXT,                  -- JSON for extra data

    -- Timestamps
    timestamp INTEGER NOT NULL,     -- When the event occurred on exchange
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indices for ledger queries
CREATE INDEX IF NOT EXISTS idx_ledger_exchange_timestamp
    ON ledger_entries(exchange_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_ledger_asset
    ON ledger_entries(exchange_id, asset);
CREATE INDEX IF NOT EXISTS idx_ledger_external_id
    ON ledger_entries(external_id);
CREATE INDEX IF NOT EXISTS idx_ledger_entry_type
    ON ledger_entries(exchange_id, entry_type);
CREATE INDEX IF NOT EXISTS idx_ledger_reference
    ON ledger_entries(reference_type, reference_id);

-- ============================================================================
-- COST BASIS LOTS
-- Track individual acquisition lots for FIFO/LIFO/Average cost calculations
-- ============================================================================

CREATE TABLE IF NOT EXISTS cost_basis_lots (
    id TEXT PRIMARY KEY,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id) ON DELETE CASCADE,

    -- Asset identification
    asset TEXT NOT NULL,
    symbol TEXT,                    -- Trading pair if from trade (e.g., BTCUSDT)

    -- Quantity tracking
    quantity TEXT NOT NULL,         -- Original quantity acquired
    remaining_quantity TEXT NOT NULL, -- Quantity still held

    -- Cost information
    cost_per_unit TEXT NOT NULL,    -- Cost in quote currency
    total_cost TEXT NOT NULL,       -- Total cost basis
    fee_included TEXT DEFAULT '0',  -- Fee added to cost basis

    -- Method and classification
    cost_basis_method TEXT NOT NULL CHECK (cost_basis_method IN ('FIFO', 'LIFO', 'AVG')),
    acquisition_type TEXT NOT NULL CHECK (acquisition_type IN (
        'BUY',
        'TRANSFER_IN',
        'DEPOSIT',
        'AIRDROP',
        'FORK',
        'MINING',
        'STAKING_REWARD'
    )),

    -- Dates
    acquisition_date INTEGER NOT NULL,  -- Unix timestamp
    acquisition_timestamp TEXT,         -- ISO 8601 for display

    -- Reference
    reference_id TEXT,              -- Trade ID, deposit ID, etc.
    ledger_entry_id TEXT REFERENCES ledger_entries(id),

    -- Disposal tracking
    is_closed INTEGER NOT NULL DEFAULT 0,
    closed_at INTEGER,
    disposal_reference_id TEXT,

    -- Timestamps
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indices for cost basis queries
CREATE INDEX IF NOT EXISTS idx_cost_basis_asset_open
    ON cost_basis_lots(exchange_id, asset, is_closed) WHERE is_closed = 0;
CREATE INDEX IF NOT EXISTS idx_cost_basis_acquisition_date
    ON cost_basis_lots(exchange_id, asset, acquisition_date);
CREATE INDEX IF NOT EXISTS idx_cost_basis_method
    ON cost_basis_lots(exchange_id, cost_basis_method);

-- ============================================================================
-- RECONCILIATION SNAPSHOTS
-- Track balance comparisons between ledger and exchange
-- ============================================================================

CREATE TABLE IF NOT EXISTS reconciliation_snapshots (
    id TEXT PRIMARY KEY,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id) ON DELETE CASCADE,

    -- Snapshot type
    snapshot_type TEXT NOT NULL CHECK (snapshot_type IN (
        'DAILY',
        'MANUAL',
        'POST_SYNC',
        'PRE_TRADE',
        'SCHEDULED'
    )),

    -- Balance data (JSON)
    calculated_balances TEXT NOT NULL, -- JSON: {asset: amount}
    reported_balances TEXT NOT NULL,   -- JSON: {asset: amount}

    -- Discrepancy details (JSON array)
    discrepancies TEXT,                -- JSON: [{asset, calculated, reported, diff, pct}]

    -- Status
    status TEXT NOT NULL CHECK (status IN (
        'MATCHED',
        'DISCREPANCY',
        'PENDING_REVIEW',
        'RESOLVED',
        'IGNORED'
    )),

    -- Resolution
    reviewed_at INTEGER,
    reviewed_by TEXT,
    resolution_notes TEXT,

    -- Timestamps
    snapshot_timestamp INTEGER NOT NULL, -- When balances were captured
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indices for reconciliation queries
CREATE INDEX IF NOT EXISTS idx_reconciliation_exchange_date
    ON reconciliation_snapshots(exchange_id, snapshot_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_reconciliation_status
    ON reconciliation_snapshots(status) WHERE status = 'DISCREPANCY';

-- ============================================================================
-- SYNC STATE
-- Track synchronization progress for each data type per exchange
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync_state (
    id TEXT PRIMARY KEY,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id) ON DELETE CASCADE,

    -- Data type being synced
    data_type TEXT NOT NULL CHECK (data_type IN (
        'trades',
        'orders',
        'positions',
        'funding',
        'deposits',
        'withdrawals',
        'balances',
        'income'
    )),

    -- Sync cursors
    last_sync_id TEXT,              -- Last processed ID from exchange
    last_sync_timestamp INTEGER,    -- Last processed timestamp
    cursor TEXT,                    -- Pagination cursor if needed

    -- Sync range
    sync_start_time INTEGER,        -- Earliest data we've synced
    sync_end_time INTEGER,          -- Latest data we've synced

    -- Status
    status TEXT NOT NULL CHECK (status IN (
        'IDLE',
        'SYNCING',
        'PAUSED',
        'ERROR',
        'COMPLETED'
    )),
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,

    -- Progress tracking
    total_records INTEGER,
    processed_records INTEGER,

    -- Timestamps
    last_success_at INTEGER,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    UNIQUE(exchange_id, data_type)
);

-- Index for sync state queries
CREATE INDEX IF NOT EXISTS idx_sync_state_exchange
    ON sync_state(exchange_id);
CREATE INDEX IF NOT EXISTS idx_sync_state_status
    ON sync_state(status) WHERE status = 'ERROR';

-- ============================================================================
-- TAX REPORTS
-- Cache generated tax reports
-- ============================================================================

CREATE TABLE IF NOT EXISTS tax_reports (
    id TEXT PRIMARY KEY,

    -- Scope
    exchange_id TEXT REFERENCES exchanges(id) ON DELETE SET NULL, -- NULL = all exchanges
    report_year INTEGER NOT NULL,

    -- Method used
    cost_basis_method TEXT NOT NULL CHECK (cost_basis_method IN ('FIFO', 'LIFO', 'AVG')),

    -- Summary figures
    short_term_gains TEXT NOT NULL,     -- < 1 year holding
    long_term_gains TEXT NOT NULL,      -- >= 1 year holding
    total_gains TEXT NOT NULL,
    total_losses TEXT NOT NULL,
    net_gain_loss TEXT NOT NULL,
    total_fees TEXT NOT NULL,
    total_proceeds TEXT NOT NULL,
    total_cost_basis TEXT NOT NULL,

    -- Counts
    transactions_count INTEGER NOT NULL,
    short_term_count INTEGER NOT NULL,
    long_term_count INTEGER NOT NULL,

    -- Full report data
    report_data TEXT NOT NULL,          -- Complete JSON report with all transactions

    -- Currency
    report_currency TEXT NOT NULL DEFAULT 'USD',

    -- Timestamps
    period_start INTEGER NOT NULL,      -- Start of tax year
    period_end INTEGER NOT NULL,        -- End of tax year
    generated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indices for tax report queries
CREATE INDEX IF NOT EXISTS idx_tax_reports_year
    ON tax_reports(report_year, cost_basis_method);
CREATE INDEX IF NOT EXISTS idx_tax_reports_exchange
    ON tax_reports(exchange_id, report_year);

-- ============================================================================
-- FUNDING PAYMENTS
-- Track perpetual futures funding payments (8-hour intervals)
-- ============================================================================

CREATE TABLE IF NOT EXISTS funding_payments (
    id TEXT PRIMARY KEY,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id) ON DELETE CASCADE,

    -- Position info
    symbol TEXT NOT NULL,
    position_side TEXT CHECK (position_side IN ('LONG', 'SHORT', 'BOTH')),

    -- Funding data
    funding_rate TEXT NOT NULL,         -- Rate as decimal string
    payment_amount TEXT NOT NULL,       -- Positive = received, negative = paid
    position_size TEXT NOT NULL,        -- Position size at time of funding
    mark_price TEXT,                    -- Mark price at funding time

    -- Exchange reference
    external_id TEXT UNIQUE,

    -- Timestamps
    funding_time INTEGER NOT NULL,      -- When funding occurred
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indices for funding queries
CREATE INDEX IF NOT EXISTS idx_funding_exchange_time
    ON funding_payments(exchange_id, funding_time DESC);
CREATE INDEX IF NOT EXISTS idx_funding_symbol
    ON funding_payments(exchange_id, symbol, funding_time DESC);

-- ============================================================================
-- DEPOSITS AND WITHDRAWALS
-- Track crypto movements in/out of exchange
-- ============================================================================

CREATE TABLE IF NOT EXISTS deposits (
    id TEXT PRIMARY KEY,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id) ON DELETE CASCADE,

    -- Asset info
    asset TEXT NOT NULL,
    network TEXT,                       -- Blockchain network

    -- Amount
    amount TEXT NOT NULL,
    fee TEXT DEFAULT '0',

    -- Transaction details
    tx_id TEXT,                         -- Blockchain transaction hash
    address TEXT,                       -- Deposit address
    address_tag TEXT,                   -- Memo/tag if required

    -- Status
    status TEXT NOT NULL CHECK (status IN (
        'PENDING',
        'CONFIRMING',
        'COMPLETED',
        'FAILED',
        'CANCELLED'
    )),
    confirmations INTEGER,
    required_confirmations INTEGER,

    -- Exchange reference
    external_id TEXT UNIQUE,

    -- Timestamps
    initiated_at INTEGER,
    completed_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS withdrawals (
    id TEXT PRIMARY KEY,
    exchange_id TEXT NOT NULL REFERENCES exchanges(id) ON DELETE CASCADE,

    -- Asset info
    asset TEXT NOT NULL,
    network TEXT,

    -- Amount
    amount TEXT NOT NULL,
    fee TEXT NOT NULL,

    -- Transaction details
    tx_id TEXT,
    address TEXT NOT NULL,
    address_tag TEXT,

    -- Status
    status TEXT NOT NULL CHECK (status IN (
        'PENDING',
        'PROCESSING',
        'COMPLETED',
        'FAILED',
        'CANCELLED',
        'REJECTED'
    )),

    -- Exchange reference
    external_id TEXT UNIQUE,

    -- Timestamps
    initiated_at INTEGER NOT NULL,
    completed_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indices for deposit/withdrawal queries
CREATE INDEX IF NOT EXISTS idx_deposits_exchange_time
    ON deposits(exchange_id, completed_at DESC);
CREATE INDEX IF NOT EXISTS idx_deposits_asset
    ON deposits(exchange_id, asset);
CREATE INDEX IF NOT EXISTS idx_withdrawals_exchange_time
    ON withdrawals(exchange_id, initiated_at DESC);
CREATE INDEX IF NOT EXISTS idx_withdrawals_asset
    ON withdrawals(exchange_id, asset);

-- ============================================================================
-- VIEWS
-- ============================================================================

-- Current balances calculated from ledger
CREATE VIEW IF NOT EXISTS v_ledger_balances AS
SELECT
    exchange_id,
    asset,
    SUM(CAST(amount AS REAL)) as balance,
    COUNT(*) as entry_count,
    MAX(timestamp) as last_update
FROM ledger_entries
GROUP BY exchange_id, asset
HAVING balance != 0;

-- Open cost basis lots summary
CREATE VIEW IF NOT EXISTS v_open_lots_summary AS
SELECT
    exchange_id,
    asset,
    cost_basis_method,
    COUNT(*) as lot_count,
    SUM(CAST(remaining_quantity AS REAL)) as total_quantity,
    SUM(CAST(total_cost AS REAL) * (CAST(remaining_quantity AS REAL) / CAST(quantity AS REAL))) as total_cost_basis,
    MIN(acquisition_date) as oldest_acquisition,
    MAX(acquisition_date) as newest_acquisition
FROM cost_basis_lots
WHERE is_closed = 0
GROUP BY exchange_id, asset, cost_basis_method;

-- Recent reconciliation status
CREATE VIEW IF NOT EXISTS v_reconciliation_status AS
SELECT
    exchange_id,
    snapshot_type,
    status,
    snapshot_timestamp,
    json_array_length(discrepancies) as discrepancy_count,
    created_at
FROM reconciliation_snapshots
WHERE id IN (
    SELECT id FROM reconciliation_snapshots r2
    WHERE r2.exchange_id = reconciliation_snapshots.exchange_id
    ORDER BY snapshot_timestamp DESC
    LIMIT 1
);

-- Sync progress overview
CREATE VIEW IF NOT EXISTS v_sync_progress AS
SELECT
    exchange_id,
    data_type,
    status,
    CASE
        WHEN total_records > 0
        THEN ROUND(100.0 * processed_records / total_records, 2)
        ELSE NULL
    END as progress_pct,
    last_sync_timestamp,
    last_success_at,
    error_message
FROM sync_state;

-- P&L summary by symbol
CREATE VIEW IF NOT EXISTS v_pnl_by_symbol AS
SELECT
    t.exchange_id,
    t.symbol,
    COUNT(*) as trade_count,
    SUM(CASE WHEN CAST(t.realized_pnl AS REAL) > 0 THEN 1 ELSE 0 END) as winning_trades,
    SUM(CASE WHEN CAST(t.realized_pnl AS REAL) < 0 THEN 1 ELSE 0 END) as losing_trades,
    SUM(CAST(t.realized_pnl AS REAL)) as total_pnl,
    SUM(CAST(t.fee AS REAL)) as total_fees,
    SUM(CAST(t.realized_pnl AS REAL)) - SUM(CAST(t.fee AS REAL)) as net_pnl
FROM trades t
GROUP BY t.exchange_id, t.symbol;

-- Funding payments summary
CREATE VIEW IF NOT EXISTS v_funding_summary AS
SELECT
    exchange_id,
    symbol,
    COUNT(*) as payment_count,
    SUM(CAST(payment_amount AS REAL)) as total_funding,
    AVG(CAST(funding_rate AS REAL)) as avg_rate,
    MIN(funding_time) as first_funding,
    MAX(funding_time) as last_funding
FROM funding_payments
GROUP BY exchange_id, symbol;

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Update updated_at on cost_basis_lots changes
CREATE TRIGGER IF NOT EXISTS trg_cost_basis_lots_updated_at
AFTER UPDATE ON cost_basis_lots
BEGIN
    UPDATE cost_basis_lots
    SET updated_at = strftime('%s', 'now')
    WHERE id = NEW.id;
END;

-- Update sync_state updated_at on changes
CREATE TRIGGER IF NOT EXISTS trg_sync_state_updated_at
AFTER UPDATE ON sync_state
BEGIN
    UPDATE sync_state
    SET updated_at = strftime('%s', 'now')
    WHERE id = NEW.id;
END;

-- Auto-close lot when remaining_quantity reaches 0
CREATE TRIGGER IF NOT EXISTS trg_auto_close_lot
AFTER UPDATE OF remaining_quantity ON cost_basis_lots
WHEN CAST(NEW.remaining_quantity AS REAL) <= 0 AND NEW.is_closed = 0
BEGIN
    UPDATE cost_basis_lots
    SET is_closed = 1,
        closed_at = strftime('%s', 'now'),
        remaining_quantity = '0'
    WHERE id = NEW.id;
END;
