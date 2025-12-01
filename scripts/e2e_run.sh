#!/usr/bin/env bash
set -euo pipefail

echo "🧪 E2E Tests - Running..."

# Setup environment
./scripts/e2e_setup.sh

# Run tests
echo "🔬 Running E2E tests..."
export DATABASE_URL=sqlite://./test.sqlite3

cargo test --workspace --test e2e -- --nocapture --test-threads 1 2>&1 | tee ./artifacts/logs/test-output.log
TEST_EXIT_CODE=${PIPESTATUS[0]}

# Cleanup
./scripts/e2e_teardown.sh

# Final report
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 E2E Test Results"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo "✅ All tests passed!"
    exit 0
else
    echo "❌ Tests failed!"
    echo "📁 Artifacts saved in ./artifacts/"
    exit 1
fi
