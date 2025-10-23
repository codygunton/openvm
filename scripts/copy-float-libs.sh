#!/bin/bash
set -e

# This script copies float library artifacts from cargo build output to binaries/
# This allows zkevm-test-monitor to use the same float library that cargo-openvm built

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Find the openvm-floats-guest build directory
TARGET_DIR="$REPO_ROOT/target"
if [ ! -d "$TARGET_DIR" ]; then
    echo "Error: Target directory not found. Build cargo-openvm first."
    exit 1
fi

# Find the build output directory for openvm-floats-guest
# Look for the libziskfloat.a file in the build output
FLOAT_LIB=$(find "$TARGET_DIR" -name "libziskfloat.a" -path "*/openvm-floats-guest-*/out/*" | head -1)

if [ -z "$FLOAT_LIB" ]; then
    echo "Error: Could not find libziskfloat.a"
    echo "Make sure to build cargo-openvm with: cargo build -p cargo-openvm"
    echo "The build process should compile openvm-floats-guest which builds libziskfloat.a"
    exit 1
fi

FLOAT_BUILD_DIR=$(dirname "$FLOAT_LIB")

echo "Found float library build directory: $FLOAT_BUILD_DIR"

# Output directory
OUT_DIR="$REPO_ROOT/zkevm-test-monitor/binaries/float-libs"
mkdir -p "$OUT_DIR"

# Copy the library files
if [ -f "$FLOAT_BUILD_DIR/libziskfloat.a" ]; then
    cp "$FLOAT_BUILD_DIR/libziskfloat.a" "$OUT_DIR/"
    echo "✅ Copied libziskfloat.a"
else
    echo "Error: libziskfloat.a not found in $FLOAT_BUILD_DIR"
    exit 1
fi

# Copy individual object files if they exist
for obj_file in float.o compiler_builtins.o; do
    if [ -f "$FLOAT_BUILD_DIR/$obj_file" ]; then
        cp "$FLOAT_BUILD_DIR/$obj_file" "$OUT_DIR/"
        echo "✅ Copied $obj_file"
    fi
done

echo ""
echo "Float library artifacts copied to: $OUT_DIR"
ls -lh "$OUT_DIR"
