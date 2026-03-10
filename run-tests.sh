#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SPEC_DIR="$SCRIPT_DIR/executable-spec"
WORKERS="${PYTEST_WORKERS:-48}"

usage() {
    echo "Usage: $0 [filter]"
    echo ""
    echo "Filters:"
    echo "  default       Python executable-spec tests only (default)"
    echo "  all           Everything including heavy (revm_transfer re-proving)"
    echo "  rust          Rust-side vector verification tests only"
    echo "  python        Python executable-spec tests only"
    echo "  primitives    Primitive tests (field, Poseidon2, NTT, Merkle)"
    echo "  fri           FRI protocol tests"
    echo "  e2e           End-to-end prover/verifier tests"
    echo "  heavy         Heavy vectors only (revm_transfer re-proving)"
    echo "  verifier      Verifier-only tests"
    echo "  -k PATTERN    Pass arbitrary pytest -k filter (Python only)"
    echo ""
    echo "Environment:"
    echo "  CUDA=0           Disable GPU, use CPU-only proving for heavy tests"
    echo "  PYTEST_WORKERS=N Number of parallel workers (default: 48, 0=sequential)"
}

FILTER="${1:-default}"

# Resolve cargo test command: prefer nextest for parallelism.
if command -v cargo-nextest &> /dev/null; then
    cargo_test() { cargo nextest run --cargo-profile=fast "$@"; }
else
    cargo_test() { cargo test --profile fast "$@"; }
fi

run_rust() {
    echo "=== Running Rust verification tests ==="
    cargo_test -p openvm-test-vectors
    echo "=== Rust verification tests complete ==="
}

run_rust_heavy() {
    echo "=== Running heavy Rust verification tests ==="
    local features="revm-vectors,cuda"
    if [ "${CUDA:-1}" = "0" ]; then
        features="revm-vectors"
    fi
    RUST_MIN_STACK=67108864 cargo_test -p openvm-test-vectors --features "$features" \
        --test generate_revm_vectors -- --ignored test_revm_transfer_proof_unchanged --test-threads=1
    echo "=== Heavy Rust verification tests complete ==="
}

run_python() {
    local pytest_args=("-v" "--tb=short")
    if [ "$WORKERS" -gt 0 ] && python3 -c "import xdist" 2>/dev/null; then
        pytest_args+=("-n" "$WORKERS")
    fi
    pytest_args+=("$@")

    echo "=== Running Python spec tests ==="
    cd "$SPEC_DIR"
    python3 -m pytest "${pytest_args[@]}"
    echo "=== Python spec tests complete ==="
}

case "$FILTER" in
    default)
        run_python
        ;;
    all)
        run_rust
        run_rust_heavy
        run_python
        ;;
    rust)
        run_rust
        ;;
    python)
        run_python
        ;;
    primitives)
        run_python -k "test_field or test_poseidon2 or test_ntt or test_merkle"
        ;;
    fri)
        run_python -k "test_fri"
        ;;
    e2e)
        run_rust
        run_python -k "test_stark_e2e or test_verifier_e2e"
        ;;
    heavy)
        run_rust_heavy
        ;;
    verifier)
        run_python -k "test_verifier"
        ;;
    -k)
        run_python -k "${2:?Missing pattern after -k}"
        ;;
    -h|--help)
        usage
        exit 0
        ;;
    *)
        echo "Unknown filter: $FILTER"
        usage
        exit 1
        ;;
esac
