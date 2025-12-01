-- Test accounts for E2E testing
-- These accounts are used across all E2E test scenarios

INSERT INTO accounts (id, exchange_id, account_type, status, created_at, updated_at)
VALUES
    ('test-account-1', 'mock', 'futures', 'active', '2025-11-01T00:00:00Z', '2025-11-01T00:00:00Z'),
    ('test-account-2', 'mock', 'spot', 'active', '2025-11-01T00:00:00Z', '2025-11-01T00:00:00Z'),
    ('test-account-paper', 'mock', 'futures', 'active', '2025-11-01T00:00:00Z', '2025-11-01T00:00:00Z');

-- Insert exchange configuration
INSERT INTO exchanges (id, name, exchange_type, status, created_at, updated_at)
VALUES ('mock', 'Mock Exchange', 'cex', 'active', '2025-11-01T00:00:00Z', '2025-11-01T00:00:00Z')
ON CONFLICT(id) DO NOTHING;
