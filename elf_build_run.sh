#!/bin/bash
set -eu

# Rebuild flags
BTEST=${BTEST:-0}
BBIN=${BBIN:-0}
RUN=${RUN:-1}

if [ $BTEST = "1" ]; then
    examples/float-minimal-test/build.sh
fi

if [ $BBIN = "1" ]; then
    cargo build -p cargo-openvm
fi

if [ $RUN = "1" ]; then
    TEST_FILE=examples/float-minimal-test/build/test.elf
    LOG_FILE=examples/float-minimal-test/elf.log
    RUST_BACKTRACE=1 RUST_LOG=trace ./target/debug/cargo-openvm openvm run \
        --exe $TEST_FILE \
        --config examples/float-minimal-test/openvm.toml \
        &> "$LOG_FILE"
fi
