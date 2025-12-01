#!/usr/bin/env bash
set -euo pipefail

echo "🚀 E2E Test Setup - Starting..."

# Configuration
export DATABASE_URL="${DATABASE_URL:-sqlite://./test.sqlite3}"
export RUST_ENV=test
export RUST_LOG="${RUST_LOG:-info,robotrade=debug}"
export RUST_TEST_SEED=42
export MOCK_EXCHANGE_PORT="${MOCK_EXCHANGE_PORT:-9999}"
export MOCK_EXCHANGE_WS_PORT="${MOCK_EXCHANGE_WS_PORT:-9998}"

# Cleanup old artifacts
echo "🧹 Cleaning up old test artifacts..."
rm -f ./test.sqlite3
rm -rf ./artifacts/logs/*
rm -rf ./artifacts/db_snapshots/*
mkdir -p ./artifacts/logs
mkdir -p ./artifacts/db_snapshots
mkdir -p ./artifacts/coverage

# Apply migrations manually
echo "📚 Applying database migrations..."
for migration in crates/infra/migrations/*.sql; do
    if [ -f "$migration" ]; then
        echo "   📝 Applying: $(basename "$migration")"
        sqlite3 test.sqlite3 < "$migration" 2>&1 | grep -v "already exists" || true
    fi
done

# Verify database
echo "✅ Verifying database schema..."
TABLES=$(sqlite3 test.sqlite3 "SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
if [ "$TABLES" -lt 40 ]; then
    echo "❌ Database migration failed: only $TABLES tables found"
    exit 1
fi
echo "   ✅ Found $TABLES tables"

# Apply fixtures
echo "📦 Loading test fixtures..."
for fixture in tests/fixtures/db/*.sql; do
    if [ -f "$fixture" ]; then
        echo "   📝 Loading: $(basename "$fixture")"
        sqlite3 test.sqlite3 < "$fixture" 2>&1 || true
    fi
done

echo "✅ E2E Test Setup Complete!"
echo ""
echo "Environment:"
echo "  DATABASE_URL: $DATABASE_URL"
echo "  Tables created: $TABLES"
echo "  Fixtures loaded: $(ls -1 tests/fixtures/db/*.sql 2>/dev/null | wc -l)"
echo ""
