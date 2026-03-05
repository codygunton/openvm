"""Shared test fixtures for executable-spec tests."""
import json
import sys
from pathlib import Path

# Add executable-spec root to Python path for absolute imports.
SPEC_ROOT = Path(__file__).resolve().parent.parent
if str(SPEC_ROOT) not in sys.path:
    sys.path.insert(0, str(SPEC_ROOT))

TEST_DATA_DIR = Path(__file__).resolve().parent / "test-data"


def load_test_vectors(category: str, name: str) -> dict:
    """Load JSON test vectors from test-data/<category>/<name>.json.

    Args:
        category: Subdirectory under test-data (e.g., "primitives", "fri", "e2e").
        name: Vector file name without extension.

    Returns:
        Parsed JSON dict.

    Raises:
        FileNotFoundError: If the vector file does not exist.
    """
    path = TEST_DATA_DIR / category / f"{name}.json"
    if not path.exists():
        raise FileNotFoundError(
            f"Test vector not found: {path}\n"
            f"Run generate-test-vectors.sh to create test vectors."
        )
    with open(path) as f:
        return json.load(f)
