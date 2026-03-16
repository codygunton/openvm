#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: bench-proof-size.sh <benchmark>

Benchmarks:
  cpu-fibonacci   Prove fibonacci (n=100k) on CPU
  gpu-revm        Prove revm_transfer (100 EVM transfers) on GPU (CUDA)

Examples:
  ./bench-proof-size.sh cpu-fibonacci
  ./bench-proof-size.sh gpu-revm
EOF
    exit 1
}

if [[ $# -ne 1 ]]; then
    usage
fi

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"

# Ensure the nightly toolchain and rust-src are available (needed to build guest programs via -Z build-std)
ensure_toolchain() {
    local TOOLCHAIN="nightly-2025-08-02"
    if ! rustup toolchain list | grep -q "$TOOLCHAIN"; then
        echo "Installing Rust toolchain $TOOLCHAIN..."
        rustup toolchain install "$TOOLCHAIN" --profile minimal --component rust-src
    fi
    if ! rustup component list --toolchain "$TOOLCHAIN" --installed | grep -q "rust-src"; then
        echo "Adding rust-src component to $TOOLCHAIN..."
        rustup component add rust-src --toolchain "$TOOLCHAIN"
    fi
}

report_sizes() {
    local dir="$1"
    echo ""
    echo "=== Proof sizes ==="
    du -sh "$dir"/*.bin | sort -rh
    echo "---"
    du -sh "$dir"
}

case "$1" in
    cpu-fibonacci)
        ensure_toolchain
        OUTPUT_DIR="$REPO_ROOT/proof-output-fibonacci"
        rm -rf "$OUTPUT_DIR"

        echo "Building proof_size (CPU)..."
        cargo build --release --bin proof_size -p openvm-benchmarks-prove \
            --manifest-path "$REPO_ROOT/Cargo.toml" 2>&1

        cargo run --release --bin proof_size -p openvm-benchmarks-prove \
            --manifest-path "$REPO_ROOT/Cargo.toml" \
            -- fibonacci -o "$OUTPUT_DIR"

        report_sizes "$OUTPUT_DIR"
        ;;
    gpu-revm)
        ensure_toolchain
        if ! command -v nvidia-smi &>/dev/null; then
            echo "Error: nvidia-smi not found. GPU benchmarks require CUDA." >&2
            exit 1
        fi
        echo "Detected GPU:"
        nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader
        echo ""

        OUTPUT_DIR="$REPO_ROOT/proof-output-revm"
        rm -rf "$OUTPUT_DIR"

        echo "Building proof_size (CUDA)..."
        cargo build --release --features cuda --bin proof_size -p openvm-benchmarks-prove \
            --manifest-path "$REPO_ROOT/Cargo.toml" 2>&1

        cargo run --release --features cuda --bin proof_size -p openvm-benchmarks-prove \
            --manifest-path "$REPO_ROOT/Cargo.toml" \
            -- revm-transfer -o "$OUTPUT_DIR"

        report_sizes "$OUTPUT_DIR"
        ;;
    *)
        echo "Error: unknown benchmark '$1'" >&2
        echo ""
        usage
        ;;
esac
