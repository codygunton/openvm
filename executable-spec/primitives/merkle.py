"""Binary Merkle tree using Poseidon2 hash.

Leaf hash: PaddingFreeSponge (hash_to_digest).
Node compress: TruncatedPermutation (compress).

Reference:
    p3-merkle-tree (FieldMerkleTreeMmcs).
"""

from primitives.poseidon2 import hash_to_digest, compress


def build_merkle_tree(
    leaves: list[list[int]],
) -> tuple[list[int], list[list[list[int]]]]:
    """Build a binary Merkle tree from leaves.

    Args:
        leaves: List of leaf data (each leaf is a list of field elements).

    Returns:
        Tuple of (root_digest, tree_levels) where tree_levels[0] is the leaf
        digests and tree_levels[-1] is [root].
    """
    # Hash each leaf to a digest
    digests = [hash_to_digest(leaf) for leaf in leaves]

    # Build tree bottom-up
    tree = [digests]
    current_level = digests
    while len(current_level) > 1:
        next_level = []
        for i in range(0, len(current_level), 2):
            next_level.append(compress(current_level[i], current_level[i + 1]))
        current_level = next_level
        tree.append(current_level)

    root = current_level[0]
    return root, tree


def get_opening_proof(
    tree: list[list[list[int]]], leaf_index: int
) -> list[list[int]]:
    """Get Merkle opening proof (sibling digests from leaf to root).

    Args:
        tree: Tree levels from build_merkle_tree.
        leaf_index: Index of the leaf to open.

    Returns:
        List of sibling digests, one per tree level (excluding root level).
    """
    proof = []
    idx = leaf_index
    for level in tree[:-1]:
        sibling_idx = idx ^ 1
        proof.append(level[sibling_idx])
        idx >>= 1
    return proof


def verify_opening(
    root: list[int],
    leaf: list[int],
    leaf_index: int,
    proof: list[list[int]],
) -> bool:
    """Verify a Merkle opening proof.

    Args:
        root: Expected root digest.
        leaf: Leaf data (unhashed).
        leaf_index: Index of the leaf in the tree.
        proof: Sibling digests from leaf level to root.

    Returns:
        True if the proof is valid.
    """
    current = hash_to_digest(leaf)
    idx = leaf_index
    for sibling in proof:
        if idx % 2 == 0:
            current = compress(current, sibling)
        else:
            current = compress(sibling, current)
        idx >>= 1
    return current == root


def verify_opening_prehashed(
    root: list[int],
    leaf_digest: list[int],
    leaf_index: int,
    proof: list[list[int]],
) -> bool:
    """Verify a Merkle opening proof when the leaf is already hashed.

    Used for FRI commit-phase proofs where the leaf (pair of extension field
    elements) is hashed externally.

    Args:
        root: Expected root digest.
        leaf_digest: Already-hashed leaf digest.
        leaf_index: Index of the leaf in the tree.
        proof: Sibling digests from leaf level to root.

    Returns:
        True if the proof is valid.
    """
    current = leaf_digest
    idx = leaf_index
    for sibling in proof:
        if idx % 2 == 0:
            current = compress(current, sibling)
        else:
            current = compress(sibling, current)
        idx >>= 1
    return current == root
