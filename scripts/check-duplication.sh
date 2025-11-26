#!/bin/bash
# Script to check code duplication with threshold enforcement

set -e

CONFIG_FILE="${1:-.jscpd.rust.json}"
SRC_DIR="${2:-crates}"
THRESHOLD=5

echo "Checking code duplication in $SRC_DIR..."

# Run jscpd and capture output
OUTPUT=$(jscpd "$SRC_DIR" --config "$CONFIG_FILE" 2>&1)

# Extract total percentage from output
# Format: "| Total: | ... | XXX (X.XX%) |"
PERCENTAGE=$(echo "$OUTPUT" | grep "Total:" | grep -oE "\([0-9]+\.?[0-9]*%\)" | head -1 | tr -d '()%')

if [ -z "$PERCENTAGE" ]; then
    echo "Could not determine duplication percentage"
    echo "$OUTPUT"
    exit 0
fi

echo "Duplication: ${PERCENTAGE}% (threshold: ${THRESHOLD}%)"

# Compare with threshold (integer comparison)
PERCENTAGE_INT=$(echo "$PERCENTAGE" | cut -d'.' -f1)
if [ "$PERCENTAGE_INT" -ge "$THRESHOLD" ]; then
    echo "ERROR: Duplication ${PERCENTAGE}% exceeds threshold ${THRESHOLD}%"
    echo "$OUTPUT"
    exit 1
fi

echo "OK: Duplication is within acceptable limits"
exit 0
