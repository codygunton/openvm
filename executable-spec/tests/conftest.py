"""Shared test fixtures for executable-spec tests."""
import sys
from pathlib import Path

import pytest

from helpers import TEST_DATA_DIR, load_test_vectors

# Add executable-spec root to Python path for absolute imports.
SPEC_ROOT = Path(__file__).resolve().parent.parent
if str(SPEC_ROOT) not in sys.path:
    sys.path.insert(0, str(SPEC_ROOT))


@pytest.fixture
def test_data_dir():
    """Return the path to the test-data directory."""
    return TEST_DATA_DIR
