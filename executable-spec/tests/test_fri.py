"""Tests for FRI (Fast Reed-Solomon IOP of Proximity) protocol.

Golden vectors capture intermediate states at every FRI folding round,
generated from stark-backend's FRI implementation.

Vectors are in tests/test-data/fri/.
"""
import pytest
from helpers import load_test_vectors

from primitives.field import BABYBEAR_PRIME, GENERATOR
from primitives.transcript import Challenger
from protocol.fri import (
    fold_matrix,
    fri_fold,
    fri_verify_query,
    prove_fri,
    verify_fri,
)


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

    def test_full_fri_verification(self, vectors):
        """Full byte-for-byte FRI verification: fold chain + Merkle proofs + final poly."""
        # Seed challenger from exported state at FRI entry point
        cs = vectors["challenger_state"]
        challenger = Challenger.from_state(
            cs["sponge_state"],
            cs["input_buffer"],
            cs["output_buffer"],
        )

        log_max_height = vectors["log_max_height"]

        # Sample FRI alpha (batch combination challenge — consumed but not checked here)
        _fri_alpha = challenger.sample_ext()

        # Derive betas from transcript and verify against golden values
        betas = []
        for commit in vectors["commit_phase_commits"]:
            challenger.observe_many(commit)
            beta = challenger.sample_ext()
            betas.append(beta)
        assert betas == vectors["betas"], "Beta derivation mismatch"

        # Observe final polynomial coefficients
        for coeff in vectors["final_poly"]:
            challenger.observe_many(coeff)

        # Verify each query with full fold chain
        for qi, qv in enumerate(vectors["query_verifications"]):
            query_index = challenger.sample_bits(log_max_height)
            assert query_index == qv["query_index"], (
                f"Query {qi}: index mismatch {query_index} != {qv['query_index']}"
            )

            # Full fold chain verification with Merkle proofs
            result = fri_verify_query(
                vectors["commit_phase_commits"],
                vectors["betas"],
                query_index,
                vectors["queries"][qi],
                qv["reduced_opening"],
                vectors["final_poly"],
                log_max_height,
                vectors["log_final_poly_len"],
            )

            # Byte-for-byte check: final folded value matches golden
            assert result == qv["final_folded_eval"], (
                f"Query {qi}: final folded eval mismatch"
            )
            assert result == qv["expected_final_eval"], (
                f"Query {qi}: expected final eval mismatch"
            )

    def test_fold_chain_intermediates(self, vectors):
        """Each fold round produces golden-vector-identical intermediate values."""
        from primitives.field import W, ff4, ff4_coeffs, ff4_from_base
        from protocol.fri import fold_row, hash_fri_leaf, reverse_bits_len
        from primitives.merkle import verify_opening_prehashed

        log_max_height = vectors["log_max_height"]

        for qi, qv in enumerate(vectors["query_verifications"]):
            start_index = qv["query_index"]
            folded_eval = ff4(qv["reduced_opening"])

            for round_idx, expected_folded in enumerate(qv["per_round_folded_eval"]):
                log_folded_height = log_max_height - 1 - round_idx
                opening = vectors["queries"][qi]["commit_phase_openings"][round_idx]
                sibling = ff4(opening["sibling_value"])
                beta = ff4(vectors["betas"][round_idx])

                # Arrange evals
                index_sibling = start_index ^ 1
                if index_sibling % 2 == 0:
                    e0, e1 = sibling, folded_eval
                else:
                    e0, e1 = folded_eval, sibling

                parent_index = start_index >> 1

                # Verify Merkle proof
                leaf_digest = hash_fri_leaf(e0, e1)
                assert verify_opening_prehashed(
                    vectors["commit_phase_commits"][round_idx],
                    leaf_digest,
                    parent_index,
                    opening["opening_proof"],
                ), f"Query {qi}, round {round_idx}: Merkle proof failed"

                # Fold
                folded_eval = fold_row(parent_index, log_folded_height, beta, e0, e1)

                # Byte-for-byte check against golden intermediate
                assert ff4_coeffs(folded_eval) == expected_folded, (
                    f"Query {qi}, round {round_idx}: fold mismatch "
                    f"{ff4_coeffs(folded_eval)} != {expected_folded}"
                )

                start_index = parent_index


class TestFRIProver:
    """FRI prover produces byte-for-byte identical outputs to Rust."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("fri", "fri_prover")

    def test_fold_matrix_per_round(self, vectors):
        """Each fold_matrix round produces golden-identical folded evaluations."""
        folded = vectors["input_evals_bit_reversed"]
        for round_idx, expected in enumerate(vectors["per_round_folded_evals"]):
            height = len(folded) // 2
            log_height = height.bit_length() - 1
            folded = fold_matrix(folded, vectors["betas"][round_idx], log_height)
            assert folded == expected, (
                f"Round {round_idx}: fold_matrix mismatch"
            )

    def test_commit_phase_merkle_roots(self, vectors):
        """Merkle roots from commit_phase match golden commits."""
        challenger = Challenger()
        result = prove_fri(
            vectors["input_evals_bit_reversed"],
            log_blowup=vectors["log_blowup"],
            log_final_poly_len=vectors["log_final_poly_len"],
            num_queries=vectors["num_queries"],
            challenger=challenger,
        )
        assert result["commit_phase_commits"] == vectors["commit_phase_commits"], (
            "Merkle root mismatch"
        )

    def test_final_polynomial(self, vectors):
        """Final polynomial (IDFT of truncated folded evals) matches golden."""
        challenger = Challenger()
        result = prove_fri(
            vectors["input_evals_bit_reversed"],
            log_blowup=vectors["log_blowup"],
            log_final_poly_len=vectors["log_final_poly_len"],
            num_queries=vectors["num_queries"],
            challenger=challenger,
        )
        assert result["final_poly"] == vectors["final_poly"], (
            "Final polynomial mismatch"
        )

    def test_betas(self, vectors):
        """Folding challenges from transcript match golden betas."""
        challenger = Challenger()
        result = prove_fri(
            vectors["input_evals_bit_reversed"],
            log_blowup=vectors["log_blowup"],
            log_final_poly_len=vectors["log_final_poly_len"],
            num_queries=vectors["num_queries"],
            challenger=challenger,
        )
        assert result["betas"] == vectors["betas"], "Beta mismatch"

    def test_query_indices(self, vectors):
        """Query indices from transcript match golden indices."""
        challenger = Challenger()
        result = prove_fri(
            vectors["input_evals_bit_reversed"],
            log_blowup=vectors["log_blowup"],
            log_final_poly_len=vectors["log_final_poly_len"],
            num_queries=vectors["num_queries"],
            challenger=challenger,
        )
        for qi in range(vectors["num_queries"]):
            assert result["query_proofs"][qi]["index"] == \
                vectors["query_proofs"][qi]["index"], (
                    f"Query {qi}: index mismatch"
                )

    def test_query_sibling_values(self, vectors):
        """Sibling values per query per round match golden values."""
        challenger = Challenger()
        result = prove_fri(
            vectors["input_evals_bit_reversed"],
            log_blowup=vectors["log_blowup"],
            log_final_poly_len=vectors["log_final_poly_len"],
            num_queries=vectors["num_queries"],
            challenger=challenger,
        )
        for qi in range(vectors["num_queries"]):
            for ri, step in enumerate(
                result["query_proofs"][qi]["commit_phase_openings"]
            ):
                expected = vectors["query_proofs"][qi][
                    "commit_phase_openings"
                ][ri]["sibling_value"]
                assert step["sibling_value"] == expected, (
                    f"Query {qi}, round {ri}: sibling_value mismatch"
                )

    def test_query_opening_proofs(self, vectors):
        """Merkle opening proofs per query per round match golden proofs."""
        challenger = Challenger()
        result = prove_fri(
            vectors["input_evals_bit_reversed"],
            log_blowup=vectors["log_blowup"],
            log_final_poly_len=vectors["log_final_poly_len"],
            num_queries=vectors["num_queries"],
            challenger=challenger,
        )
        for qi in range(vectors["num_queries"]):
            for ri, step in enumerate(
                result["query_proofs"][qi]["commit_phase_openings"]
            ):
                expected = vectors["query_proofs"][qi][
                    "commit_phase_openings"
                ][ri]["opening_proof"]
                assert step["opening_proof"] == expected, (
                    f"Query {qi}, round {ri}: opening_proof mismatch"
                )

    def test_full_prove_fri(self, vectors):
        """Full prove_fri output matches golden vectors byte-for-byte."""
        challenger = Challenger()
        proof = prove_fri(
            vectors["input_evals_bit_reversed"],
            log_blowup=vectors["log_blowup"],
            log_final_poly_len=vectors["log_final_poly_len"],
            num_queries=vectors["num_queries"],
            challenger=challenger,
        )
        # Check commits
        assert proof["commit_phase_commits"] == \
            vectors["commit_phase_commits"], "Commits mismatch"
        # Check final poly
        assert proof["final_poly"] == vectors["final_poly"], \
            "Final poly mismatch"
        # Check betas
        assert proof["betas"] == vectors["betas"], "Betas mismatch"
        # Check folded evals per round
        assert proof["folded_per_round"] == \
            vectors["per_round_folded_evals"], "Folded evals mismatch"
        # Check query proofs
        for qi in range(vectors["num_queries"]):
            assert proof["query_proofs"][qi] == \
                vectors["query_proofs"][qi], (
                    f"Query {qi}: full proof mismatch"
                )
