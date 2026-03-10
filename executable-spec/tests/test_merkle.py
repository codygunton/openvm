"""Tests for Merkle tree construction using Poseidon2.

Golden vectors generated from OpenVM's Merkle tree implementation
(which uses p3-merkle-tree with Poseidon2 compression).

Vectors are in tests/test-data/primitives/merkle.json.
"""
import pytest
from helpers import load_test_vectors

from primitives.merkle import build_merkle_tree, verify_opening


class TestMerkleTree:
    """Poseidon2-based Merkle tree construction and opening verification."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("primitives", "merkle")

    def test_tree_construction(self, vectors):
        """Build Merkle tree from leaves and verify root hash."""
        for case in vectors["construction"]:
            root, _ = build_merkle_tree(case["leaves"])
            assert root == case["expected_root"]

    def test_opening_verification(self, vectors):
        """Verify Merkle opening proof for a specific leaf."""
        for case in vectors["openings"]:
            assert verify_opening(
                case["root"], case["leaf"], case["leaf_index"], case["proof"]
            )
