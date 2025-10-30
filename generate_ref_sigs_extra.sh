#!/bin/bash
set -eu

echo "🧪 Running extra tests to generate reference signatures..."
echo ""

cd zkevm-test-monitor

# Run extra tests
./run test --extra openvm

echo ""
echo "📋 Collecting reference signatures from test results..."
echo ""

# Go back to repo root
cd ..

# Create ref-sigs-extra directory if it doesn't exist
REF_SIGS_EXTRA_DIR="ref-sigs-extra"
mkdir -p "$REF_SIGS_EXTRA_DIR"

# Find all Reference-sail_c_simulator.signature files in test-results
# and copy them to ref-sigs-extra maintaining directory structure
COPIED_COUNT=0
while IFS= read -r sig_file; do
    # Extract the path relative to test-results/openvm/rv32i_m/
    # e.g., test-results/openvm/rv32i_m/F/src/fnmadd-baby.S/ref/Reference-sail_c_simulator.signature
    # becomes: F/src/fnmadd-baby.S/ref/Reference-sail_c_simulator.signature
    RELATIVE_PATH=$(echo "$sig_file" | sed 's|zkevm-test-monitor/test-results/openvm/rv32i_m/||')

    # Create destination directory
    DEST_DIR="$REF_SIGS_EXTRA_DIR/$(dirname "$RELATIVE_PATH")"
    mkdir -p "$DEST_DIR"

    # Copy the signature file
    cp "$sig_file" "$DEST_DIR/"

    # Get just the test name for display
    TEST_NAME=$(echo "$RELATIVE_PATH" | sed 's|/ref/Reference-sail_c_simulator.signature||')
    echo "  ✓ $TEST_NAME"

    COPIED_COUNT=$((COPIED_COUNT + 1))
done < <(find zkevm-test-monitor/test-results/openvm/rv32i_m -name "Reference-sail_c_simulator.signature" -path "*/ref/*" 2>/dev/null)

echo ""
echo "✅ Copied $COPIED_COUNT reference signatures to $REF_SIGS_EXTRA_DIR/"
echo ""
echo "📝 Directory structure:"
find "$REF_SIGS_EXTRA_DIR" -name "Reference-sail_c_simulator.signature" | sed "s|$REF_SIGS_EXTRA_DIR/||" | sort
echo ""
echo "💡 These signatures can now be committed to the repository"
echo "   and will be used by extra-debug.sh for signature comparison"
