-- Trading symbols for E2E testing

INSERT INTO symbols (
    exchange_id, symbol, base_asset, quote_asset, symbol_type, status,
    price_precision, quantity_precision, quote_precision,
    min_quantity, max_quantity, min_notional, tick_size, step_size,
    created_at, updated_at
)
VALUES
    -- BTC/USDT Perpetual
    ('mock', 'BTCUSDT', 'BTC', 'USDT', 'perpetual', 'active',
     2, 8, 8,
     '0.001', '1000.0', '10.0', '0.01', '0.001',
     '2025-11-01T00:00:00Z', '2025-11-01T00:00:00Z'),

    -- ETH/USDT Perpetual
    ('mock', 'ETHUSDT', 'ETH', 'USDT', 'perpetual', 'active',
     2, 8, 8,
     '0.01', '10000.0', '10.0', '0.01', '0.01',
     '2025-11-01T00:00:00Z', '2025-11-01T00:00:00Z'),

    -- SOL/USDT Perpetual
    ('mock', 'SOLUSDT', 'SOL', 'USDT', 'perpetual', 'active',
     2, 8, 8,
     '0.1', '100000.0', '10.0', '0.01', '0.1',
     '2025-11-01T00:00:00Z', '2025-11-01T00:00:00Z');
