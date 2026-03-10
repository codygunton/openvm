"""Tests for Number Theoretic Transform (NTT) over BabyBear.

Golden vectors generated from Plonky3's DFT implementation (p3-dft).
The NTT is used for polynomial evaluation/interpolation in the STARK prover.

Vectors are in tests/test-data/primitives/ntt.json.
"""
import pytest
from helpers import load_test_vectors

from primitives.ntt import ntt, intt


class TestNTT:
    """NTT and inverse NTT over BabyBear."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("primitives", "ntt")

    def test_forward_ntt(self, vectors):
        """Forward NTT: coefficient form -> evaluation form."""
        for case in vectors["forward"]:
            result = ntt(case["input"])
            assert result == case["expected"]

    def test_inverse_ntt(self, vectors):
        """Inverse NTT: evaluation form -> coefficient form."""
        for case in vectors["inverse"]:
            result = intt(case["input"])
            assert result == case["expected"]

    def test_ntt_round_trip(self, vectors):
        """NTT(INTT(x)) == x for all test cases."""
        for case in vectors["forward"]:
            coeffs = case["input"]
            assert intt(ntt(coeffs)) == coeffs
