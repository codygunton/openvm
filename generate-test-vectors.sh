#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    echo "Usage: $0 [target]"
    echo ""
    echo "Targets:"
    echo "  all          Generate all vectors (default)"
    echo "  primitives   Field, Poseidon2, NTT, Merkle vectors only"
    echo "  e2e          E2E proof vectors only (slow)"
    echo "  clean        Remove all generated vectors"
}

TARGET="${1:-all}"

generate_primitives() {
    echo "=== Generating primitive vectors ==="
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_field
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_ext_field
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_poseidon2
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_ntt
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_merkle
    echo "=== Primitive vectors complete ==="
}

generate_e2e() {
    echo "=== Generating E2E proof vectors (this may take a while) ==="
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_e2e --test-threads=1
    echo "=== E2E vectors complete ==="
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
    clean)
        echo "Removing generated vectors..."
        rm -rf "$SCRIPT_DIR/executable-spec/tests/test-data/primitives/"*.json
        rm -rf "$SCRIPT_DIR/executable-spec/tests/test-data/fri/"*.json
        rm -rf "$SCRIPT_DIR/executable-spec/tests/test-data/e2e/"*.json
        rm -rf "$SCRIPT_DIR/executable-spec/tests/test-data/e2e/"*.bin
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
