"""End-to-end STARK prover tests.

These tests exercise the complete proving pipeline: trace generation,
constraint evaluation, polynomial commitment, FRI, and proof serialization.

Vectors are in tests/test-data/e2e/.
"""
import pytest
from helpers import load_test_vectors


class TestStarkProverE2E:
    """Full STARK prover produces proofs matching the Rust implementation."""

    @pytest.fixture(params=["fibonacci_stark"])
    def program_name(self, request):
        return request.param

    @pytest.fixture
    def vectors(self, program_name):
        return load_test_vectors("e2e", program_name)

    def test_proof_binary_equivalence(self, program_name, vectors):
        """Python prover output matches Rust prover output byte-for-byte."""
        expected_proof_hex = vectors["proof_bytes_hex"]
        assert False, (
            f"Not implemented: full STARK proof for '{program_name}' "
            f"({len(expected_proof_hex) // 2} bytes expected)"
        )

    def test_trace_commitment(self, program_name, vectors):
        """Trace commitment matches golden value."""
        expected_commitment = vectors["main_trace_commitments"]
        assert False, f"Not implemented: trace commitment for '{program_name}'"

    def test_quotient_commitment(self, program_name, vectors):
        """Quotient polynomial commitment matches golden value."""
        expected = vectors["quotient_commitment"]
        assert False, f"Not implemented: quotient commitment for '{program_name}'"
