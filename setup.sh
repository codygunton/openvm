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

echo "=== Setup complete ==="
