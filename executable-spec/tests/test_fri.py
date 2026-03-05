"""Tests for FRI (Fast Reed-Solomon IOP of Proximity) protocol.

Golden vectors capture intermediate states at every FRI folding round,
generated from stark-backend's FRI implementation.

Vectors are in tests/test-data/fri/.
"""
import pytest
from helpers import load_test_vectors


class TestFRIFolding:
    """FRI folding produces golden-vector-identical intermediate states."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("fri", "fri_folding")

    def test_fold_per_round(self, vectors):
        """Each FRI folding round produces the expected polynomial."""
        for round_data in vectors["rounds"]:
            round_idx = round_data["round"]
            input_poly = round_data["input"]
            challenge = round_data["challenge"]
            expected_output = round_data["expected_output"]
            assert False, f"Not implemented: FRI fold round {round_idx}"

    def test_final_polynomial(self, vectors):
        """Final FRI polynomial matches golden value."""
        expected = vectors["final_polynomial"]
        assert False, f"Not implemented: FRI final polynomial ({len(expected)} coefficients)"


class TestFRIVerification:
    """FRI verification checks against golden query responses."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("fri", "fri_verification")

    def test_query_verification(self, vectors):
        """Each FRI query response verifies correctly."""
        for query in vectors["queries"]:
            query_index = query["index"]
            assert False, f"Not implemented: FRI query verification at index {query_index}"
