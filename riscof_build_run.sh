#!/bin/bash
set -e

# Rebuild flags
BTESTS=${BTESTS:-0}
BBIN=${BBIN:-0}
RUN=${RUN-1}

PATTERN=${1:-fadd_b1}

BINARY_TO_RUN=zkevm-test-monitor/binaries/openvm-binary
rm -f $BINARY_TO_RUN

# Build cargo-openvm from the openvm repository
if [ $BBIN = "1" ]; then
    echo "🔨 Building cargo-openvm..."
    cargo build -p cargo-openvm
fi

# Copy the binary to zkevm-test-monitor
cp target/debug/cargo-openvm $BINARY_TO_RUN

# Run the test
cd zkevm-test-monitor

# Optionally rebuild tests (set REBUILD_TESTS=1 to enable)
if [ $BTESTS = "1" ]; then
    echo "📦 Rebuilding tests..."
    ./run test --arch openvm --build-only
else
    echo "⏭️  Skipping test rebuild (set REBUILD_TESTS=1 to rebuild)"
fi

# Run the specific test in debug mode
if [ $RUN = "1" ]; then
    echo "🐛 Running debug test: $PATTERN"
    ./run debug --arch openvm "$PATTERN"
fi

EXIT_CODE=$?
echo ""
if [ $EXIT_CODE -ne 0 ]; then
    echo "⚠️  Test exited with code: $EXIT_CODE"
fi

exit $EXIT_CODE
