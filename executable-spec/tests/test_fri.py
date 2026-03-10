"""Tests for FRI (Fast Reed-Solomon IOP of Proximity) protocol.

Golden vectors capture intermediate states at every FRI folding round,
generated from stark-backend's FRI implementation.

Vectors are in tests/test-data/fri/.
"""
import pytest
from helpers import load_test_vectors

from primitives.field import BABYBEAR_PRIME, GENERATOR
from protocol.fri import fri_fold, verify_fri


class TestFRIFolding:
    """FRI folding produces golden-vector-identical intermediate states."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("fri", "fri_folding")

    def test_fold_per_round(self, vectors):
        """Each FRI folding round produces the expected polynomial."""
        log_domain = vectors["log_poly_size"] + vectors["log_blowup"]
        shift = GENERATOR

        for round_data in vectors["rounds"]:
            result = fri_fold(
                round_data["input"],
                round_data["challenge"],
                log_domain,
                shift,
            )
            assert result == round_data["expected_output"], (
                f"Round {round_data['round']} mismatch"
            )
            shift = (shift * shift) % BABYBEAR_PRIME
            log_domain -= 1

    def test_final_polynomial(self, vectors):
        """Running all fold rounds produces the final polynomial evaluations."""
        log_domain = vectors["log_poly_size"] + vectors["log_blowup"]
        shift = GENERATOR

        current_evals = vectors["rounds"][0]["input"]
        for round_data in vectors["rounds"]:
            current_evals = fri_fold(
                current_evals,
                round_data["challenge"],
                log_domain,
                shift,
            )
            shift = (shift * shift) % BABYBEAR_PRIME
            log_domain -= 1

        # All remaining evaluations should equal the final polynomial constant
        expected = vectors["final_polynomial"]
        for val in current_evals:
            assert val == expected[0], (
                f"Final polynomial value mismatch: {val} != {expected[0]}"
            )


class TestFRIVerification:
    """FRI verification checks against golden query responses."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("fri", "fri_verification")

    def test_query_verification(self, vectors):
        """FRI proof structure verifies: transcript replay and Merkle proof consistency."""
        result = verify_fri(
            vectors["commit_phase_commits"],
            vectors["final_poly"],
            vectors["queries"],
            log_blowup=1,
            log_final_poly_len=0,
            num_queries=2,
        )
        assert result
