"""Shared test fixtures for executable-spec tests."""
import os
import sys
from pathlib import Path

# Set numpy/OpenBLAS thread count before numpy is imported.
# Default to 48 threads; override with OMP_NUM_THREADS env var.
if "OMP_NUM_THREADS" not in os.environ:
    os.environ["OMP_NUM_THREADS"] = "48"
    os.environ["OPENBLAS_NUM_THREADS"] = "48"

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
