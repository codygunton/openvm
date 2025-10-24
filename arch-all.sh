#!/bin/bash
set -eu

BBIN=${BBIN:-0}

RUN=0 BBIN=$BBIN BTEST=0 ./arch-one.sh

cd zkevm-test-monitor

./run test --arch openvm
