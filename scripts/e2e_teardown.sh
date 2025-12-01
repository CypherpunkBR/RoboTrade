#!/usr/bin/env bash
set -euo pipefail

echo "🧹 E2E Teardown - Cleaning up..."

# Save database snapshot
if [ -f ./test.sqlite3 ]; then
    echo "💾 Saving database snapshot..."
    TIMESTAMP=$(date +%Y%m%d-%H%M%S)
    cp ./test.sqlite3 "./artifacts/db_snapshots/test-${TIMESTAMP}.sqlite3"

    # Generate DB report
    echo "📊 Database Statistics:" > "./artifacts/db_snapshots/report-${TIMESTAMP}.txt"
    sqlite3 ./test.sqlite3 <<EOF >> "./artifacts/db_snapshots/report-${TIMESTAMP}.txt"
.mode column
.headers on
SELECT 'ledger_entries' as table_name, COUNT(*) as count FROM ledger_entries
UNION ALL SELECT 'orders', COUNT(*) FROM orders
UNION ALL SELECT 'positions', COUNT(*) FROM positions
UNION ALL SELECT 'trades', COUNT(*) FROM trades
UNION ALL SELECT 'reconciliation_snapshots', COUNT(*) FROM reconciliation_snapshots;
EOF
fi

echo "✅ Teardown complete!"
echo ""
echo "📁 Artifacts:"
ls -lh ./artifacts/db_snapshots/ 2>/dev/null || true
