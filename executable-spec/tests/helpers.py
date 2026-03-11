"""Shared test utilities for executable-spec tests."""
import json
from pathlib import Path

import pytest

TEST_DATA_DIR = Path(__file__).resolve().parent / "test-data"

# All E2E programs. Every program listed here MUST have complete test vectors
# (including VK data). Run generate-test-vectors.sh to create/refresh vectors.
E2E_PROGRAMS = ["fibonacci_stark", "rv32im_fibonacci", "revm_transfer"]


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


def e2e_program_params():
    """Build pytest params for all E2E programs."""
    return [pytest.param(name) for name in E2E_PROGRAMS]
