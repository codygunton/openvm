#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SPEC_DIR="$SCRIPT_DIR/executable-spec"

echo "=== Setting up executable-spec ==="

# Check Python version
python3 -c "import sys; assert sys.version_info >= (3, 10), f'Python 3.10+ required, got {sys.version}'" || {
    echo "ERROR: Python 3.10+ is required"
    exit 1
}

# Install Python dependencies
echo "Installing Python dependencies..."
if command -v uv &> /dev/null; then
    cd "$SPEC_DIR" && uv sync
else
    pip install -e "$SPEC_DIR"
fi

# Build Poseidon2 BabyBear FFI (Rust -> Python via PyO3/Maturin)
echo "Building Poseidon2 FFI..."
FFI_DIR="$SPEC_DIR/primitives/poseidon2_ffi"
if command -v maturin &> /dev/null; then
    WHEEL=$(cd "$FFI_DIR" && maturin build --release 2>&1 | grep -oP '(?<=Built wheel .* to )\S+')
    pip install "$WHEEL" --force-reinstall
else
    echo "ERROR: maturin is required for Poseidon2 FFI. Install with: pip install maturin"
    exit 1
fi

echo "=== Setup complete ==="
