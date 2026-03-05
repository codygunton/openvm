"""Tests for Poseidon2 hash function.

Golden vectors generated from p3-poseidon2 (Plonky3's Poseidon2 over BabyBear).
OpenVM uses Poseidon2 with width=16 for Merkle tree hashing and Fiat-Shamir transcript.

Vectors are in tests/test-data/primitives/poseidon2.json.
"""
import pytest
from helpers import load_test_vectors


class TestPoseidon2:
    """Poseidon2 permutation and compression over BabyBear."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("primitives", "poseidon2")

    def test_permutation(self, vectors):
        """Full Poseidon2 permutation on width-16 state."""
        for case in vectors["permutation"]:
            input_state = case["input"]
            expected_output = case["expected"]
            assert False, f"Not implemented: permutation on {len(input_state)}-element state"

    def test_compress(self, vectors):
        """Poseidon2 compression: two 8-element inputs -> 8-element output."""
        for case in vectors["compress"]:
            left, right = case["left"], case["right"]
            expected = case["expected"]
            assert False, f"Not implemented: compress({len(left)} + {len(right)} elements)"
