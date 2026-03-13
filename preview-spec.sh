#!/usr/bin/env bash
# Preview the OpenVM STARK specification on http://localhost:8037
# Builds HTML then serves with live-reload on file changes.
set -euo pipefail

cd "$(dirname "$0")/docs/spec"

# Install dependencies if needed
if ! python -c "import sphinx" 2>/dev/null; then
    echo "Installing Sphinx dependencies..."
    pip install -r requirements.txt
fi

if ! python -c "import sphinx_autobuild" 2>/dev/null; then
    echo "Installing sphinx-autobuild..."
    pip install sphinx-autobuild
fi

echo "Serving spec at http://localhost:8037 (Ctrl-C to stop)"
exec sphinx-autobuild . _build/html --watch ../../executable-spec --port 8037
