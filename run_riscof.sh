#!/bin/bash

docker run --name riscof-ovm --rm \
  -v "$PWD/riscof/plugins/openvm:/dut/plugin" \
  -v "$PWD/target/release/cargo-openvm:/dut/bin/dut-exe" \
  -v "$PWD/riscof/results:/riscof/riscof_work" \
  riscof:latest
