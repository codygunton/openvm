#!/bin/bash
set -eu

cd "$(dirname "$(realpath "$0")")"

if [ $# -lt 1 ]; then
  echo "Usage: $0 <elf-file> [signature-output]"
  echo "Example: $0 test.elf test.signature"
  exit 1
fi

ELF_FILE=$1
SIG_FILE=${2:-"${ELF_FILE}.signature"}

echo "Testing ELF: $ELF_FILE"
echo "Signature output: $SIG_FILE"
echo "Signature region will be extracted from ELF symbols (begin_signature/end_signature)"

# Use a fixed directory
cd /home/cody/openvm/elf-test

# Build
echo -e "\nBuilding test program..."
cargo build --release

# Run with the ELF file
echo -e "\nRunning test..."
cd /home/cody/openvm
/home/cody/openvm/elf-test/target/release/test-elf "$ELF_FILE" "$SIG_FILE"

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