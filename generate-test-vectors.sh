#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    echo "Usage: $0 [target]"
    echo ""
    echo "Targets:"
    echo "  all          Generate all vectors (default)"
    echo "  primitives   Field, Poseidon2, NTT, Merkle, FRI vectors"
    echo "  e2e          E2E proof vectors (fibonacci_stark + rv32im_fibonacci)"
    echo "  heavy        Heavy vectors only (revm_transfer)"
    echo "  full         All vectors including heavy (primitives + e2e + heavy)"
    echo "  clean        Remove all generated vectors"
    echo ""
    echo "Environment:"
    echo "  CUDA=0       Disable GPU, use CPU-only proving for heavy vectors"
}

TARGET="${1:-all}"

# Resolve cargo test command: prefer nextest for parallelism.
if command -v cargo-nextest &> /dev/null; then
    cargo_test() { cargo nextest run --cargo-profile=fast "$@"; }
else
    cargo_test() { cargo test --profile fast "$@"; }
fi

generate_primitives() {
    echo "=== Generating primitive vectors ==="
    cargo_test -p openvm-test-vectors -- --ignored \
        generate_field \
        generate_ext_field \
        generate_poseidon2 \
        generate_ntt \
        generate_merkle \
        generate_fri
    echo "=== Primitive vectors complete ==="
}

generate_e2e() {
    echo "=== Generating E2E proof vectors ==="
    echo "--- fibonacci_stark (single-AIR) ---"
    cargo_test -p openvm-test-vectors -- --ignored generate_e2e --test-threads=1
    echo "--- rv32im_fibonacci (multi-AIR) ---"
    cargo_test -p openvm-test-vectors --test generate_rv32im_vectors -- --ignored --test-threads=1
    echo "=== E2E vectors complete ==="
}

generate_heavy() {
    echo "=== Generating heavy vectors ==="
    echo "--- revm_transfer (multi-AIR, multi-segment) ---"
    local features="revm-vectors,cuda"
    if [ "${CUDA:-1}" = "0" ]; then
        features="revm-vectors"
    fi
    RUST_MIN_STACK=67108864 cargo_test -p openvm-test-vectors --features "$features" \
        --test generate_revm_vectors -- --ignored --test-threads=1
    echo "=== Heavy vectors complete ==="
}

case "$TARGET" in
    all)
        generate_primitives
        generate_e2e
        ;;
    primitives)
        generate_primitives
        ;;
    e2e)
        generate_e2e
        ;;
    heavy)
        generate_heavy
        ;;
    full)
        generate_primitives
        generate_e2e
        generate_heavy
        ;;
    clean)
        echo "Removing generated vectors..."
        rm -f "$SCRIPT_DIR/executable-spec/tests/test-data/primitives/"*.json
        rm -f "$SCRIPT_DIR/executable-spec/tests/test-data/fri/"*.json
        rm -f "$SCRIPT_DIR/executable-spec/tests/test-data/e2e/"*.json
        rm -f "$SCRIPT_DIR/executable-spec/tests/test-data/e2e/"*.bin
        echo "Done."
        ;;
    -h|--help)
        usage
        ;;
    *)
        echo "Unknown target: $TARGET"
        usage
        exit 1
        ;;
esac
