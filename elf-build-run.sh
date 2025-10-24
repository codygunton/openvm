#!/bin/bash
set -eu

# Rebuild flags
BTEST=${BTEST:-0}
BBIN=${BBIN:-0}
RUN=${RUN:-1}

if [ $BTEST = "1" ]; then
    examples/floats/build.sh
fi

if [ $BBIN = "1" ]; then
    echo "Building cargo-openvm..."
    cargo build -p cargo-openvm
fi

if [ $RUN = "1" ]; then
    echo "Running test..."
    TEST_FILE=examples/floats/build/test.elf
    LOG_FILE=examples/floats/elf.log

    # Run the test and capture exit code
    set +e # Temporarily disable exit on error
    RUST_BACKTRACE=1 RUST_LOG=debug target/debug/cargo-openvm openvm run \
        --exe $TEST_FILE \
        --config examples/floats/openvm.toml \
        &> "$LOG_FILE"
    EXIT_CODE=$?
    set -e # Re-enable exit on error

    # Check for test failure indicators
    if [ $EXIT_CODE -ne 0 ]; then
        echo "ERROR: cargo-openvm exited with code $EXIT_CODE"
        echo "tail of log:"
        tail -n 20 "$LOG_FILE"
        exit $EXIT_CODE
    fi

    echo "SUCCESS: All tests passed"
fi
