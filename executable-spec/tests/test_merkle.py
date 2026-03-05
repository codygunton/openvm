"""Tests for Merkle tree construction using Poseidon2.

Golden vectors generated from OpenVM's Merkle tree implementation
(which uses p3-merkle-tree with Poseidon2 compression).

Vectors are in tests/test-data/primitives/merkle.json.
"""
import pytest
from helpers import load_test_vectors


class TestMerkleTree:
    """Poseidon2-based Merkle tree construction and opening verification."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("primitives", "merkle")

    def test_tree_construction(self, vectors):
        """Build Merkle tree from leaves and verify root hash."""
        for case in vectors["construction"]:
            leaves = case["leaves"]
            expected_root = case["expected_root"]
            assert False, f"Not implemented: Merkle tree from {len(leaves)} leaves"

    def test_opening_verification(self, vectors):
        """Verify Merkle opening proof for a specific leaf."""
        for case in vectors["openings"]:
            root = case["root"]
            leaf_index = case["leaf_index"]
            leaf = case["leaf"]
            proof = case["proof"]
            assert False, f"Not implemented: verify opening at index {leaf_index}"
