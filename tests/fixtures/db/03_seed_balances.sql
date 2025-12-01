-- Initial balances for test accounts
-- Using balance_snapshots structure with JSON assets column

INSERT INTO balance_snapshots (
    account_id, snapshot_type,
    total_balance_usdt, available_balance_usdt, locked_balance_usdt,
    unrealized_pnl, assets, timestamp
)
VALUES
    -- Test Account 1: Large balance for comprehensive testing
    ('test-account-1', 'manual',
     '100000.0', '100000.0', '0',
     '0', '[{"asset":"USDT","free":"100000.0","locked":"0"},{"asset":"BTC","free":"0","locked":"0"}]',
     '2025-11-01T00:00:00Z'),

    -- Test Account 2: Medium balance
    ('test-account-2', 'manual',
     '50000.0', '50000.0', '0',
     '0', '[{"asset":"USDT","free":"50000.0","locked":"0"},{"asset":"BTC","free":"0","locked":"0"}]',
     '2025-11-01T00:00:00Z'),

    -- Paper Trading Account: Unlimited virtual funds
    ('test-account-paper', 'manual',
     '1000000.0', '1000000.0', '0',
     '0', '[{"asset":"USDT","free":"1000000.0","locked":"0"},{"asset":"BTC","free":"10.0","locked":"0"}]',
     '2025-11-01T00:00:00Z');
