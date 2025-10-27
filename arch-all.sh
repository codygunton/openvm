#!/bin/bash
set -eu

BBIN=${BBIN:-0}

# hack -- just using this to build the binary
RUN=0 BBIN=$BBIN BTEST=0 ./arch-one.sh

cd zkevm-test-monitor

# Rebuild float library before running all tests
echo "📦 Rebuilding float library..."
riscof/plugins/openvm/env/build_float_lib.sh

./run test --arch openvm
