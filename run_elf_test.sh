#!/bin/bash
set -eu

cd "$(dirname "$(realpath "$0")")"

if [ $# -lt 1 ]; then
  echo "Usage: $0 <elf-file> [signature-output]"
  echo "Example: $0 test.elf test.signature"
  exit 1
fi

ELF_FILE=$1
# Use a simpler path for signature to avoid directory creation issues
SIG_FILE=${2:-"/tmp/$(basename ${ELF_FILE}).signature"}

echo "Testing ELF: $ELF_FILE"
echo "Signature output: $SIG_FILE"
echo "Signature region will be extracted from ELF symbols (begin_signature/end_signature)"

# Build cargo-openvm if needed
echo -e "\nBuilding cargo-openvm..."
cargo build --release -p cargo-openvm

# Run with the ELF file using cargo-openvm
echo -e "\nRunning test..."
/home/cody/openvm/target/release/cargo-openvm openvm run --elf "$ELF_FILE" --signatures "$SIG_FILE"

# Show results
if [ -f "$SIG_FILE" ]; then
  echo -e "\nSignature extracted successfully!"
  echo "First 10 lines:"
  head -10 "$SIG_FILE"
  echo "..."
  echo "Total lines: $(wc -l <"$SIG_FILE")"
else
  echo "ERROR: Signature file not created"
  exit 1
fi

