#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SPEC_DIR="$SCRIPT_DIR/executable-spec"
WORKERS="${PYTEST_WORKERS:-48}"

usage() {
    echo "Usage: $0 [filter]"
    echo ""
    echo "Filters:"
    echo "  all           Run all tests (default)"
    echo "  primitives    Field, Poseidon2, NTT, Merkle tests"
    echo "  fri           FRI protocol tests"
    echo "  e2e           End-to-end prover/verifier tests"
    echo "  verifier      Verifier-only tests"
    echo "  -k PATTERN    Pass arbitrary pytest -k filter"
    echo ""
    echo "Environment:"
    echo "  PYTEST_WORKERS=N  Number of parallel workers (default: 48, 0=sequential)"
}

FILTER="${1:-all}"

PYTEST_ARGS=("-v" "--tb=short")

if [ "$WORKERS" -gt 0 ]; then
    PYTEST_ARGS+=("-n" "$WORKERS")
fi

case "$FILTER" in
    all)
        ;;
    primitives)
        PYTEST_ARGS+=("-k" "test_field or test_poseidon2 or test_ntt or test_merkle")
        ;;
    fri)
        PYTEST_ARGS+=("-k" "test_fri")
        ;;
    e2e)
        PYTEST_ARGS+=("-k" "test_stark_e2e or test_verifier_e2e")
        ;;
    verifier)
        PYTEST_ARGS+=("-k" "test_verifier")
        ;;
    -k)
        PYTEST_ARGS+=("-k" "${2:?Missing pattern after -k}")
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

cd "$SPEC_DIR"
exec python3 -m pytest "${PYTEST_ARGS[@]}"
