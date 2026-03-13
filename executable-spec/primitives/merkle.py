"""Binary Merkle tree using Poseidon2 hash.

Leaf hash: PaddingFreeSponge (hash_to_digest).
Node compress: TruncatedPermutation (compress).

Reference:
    p3-merkle-tree (FieldMerkleTreeMmcs).
"""

from primitives.field import Digest, MerklePath
from primitives.poseidon2 import hash_to_digest, compress, compress_batch, hash_batch


# <doc-anchor id="build-tree">
def build_merkle_tree(
    leaves: list[list[int]],
) -> tuple[Digest, list[list[Digest]]]:
    """Build binary Merkle tree, returning (root, levels).

    Uses batch Poseidon2 operations (rayon-parallelized) for leaf hashing
    and internal node compression.

    Args:
        leaves: Leaf data (each a list of field elements, hashed internally).

    Returns:
        (root_digest, tree_levels) where tree_levels[0] is leaf digests.

    Reference:
        p3-merkle-tree (FieldMerkleTreeMmcs)
    """
    # Batch hash all leaves in parallel
    digests = hash_batch(leaves)

    # Build tree bottom-up with batch compression
    tree = [digests]
    current_level = digests
    while len(current_level) > 1:
        lefts = current_level[0::2]
        rights = current_level[1::2]
        current_level = compress_batch(lefts, rights)
        tree.append(current_level)

    root = list(current_level[0])  # defensive copy to avoid aliasing with tree
    return root, tree


# <doc-anchor id="open-proof">
def get_opening_proof(
    tree: list[list[Digest]], leaf_index: int
) -> MerklePath:
    """Get Merkle opening proof (sibling digests, leaf to root).

    Reference:
        p3-merkle-tree (FieldMerkleTreeMmcs)
    """
    proof = []
    idx = leaf_index
    for level in tree[:-1]:
        sibling_idx = idx ^ 1
        proof.append(level[sibling_idx])
        idx >>= 1
    return proof


# <doc-anchor id="verify-opening">
def verify_opening_prehashed(
    root: Digest,
    leaf_digest: Digest,
    leaf_index: int,
    proof: MerklePath,
) -> bool:
    """Verify Merkle opening proof for pre-hashed leaf.

    Reference:
        p3-merkle-tree (FieldMerkleTreeMmcs::verify_batch)
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


def verify_opening(
    root: Digest,
    leaf: list[int],
    leaf_index: int,
    proof: MerklePath,
) -> bool:
    """Verify Merkle opening proof for unhashed leaf data.

    Reference:
        p3-merkle-tree (FieldMerkleTreeMmcs::verify_batch)
    """
    return verify_opening_prehashed(root, hash_to_digest(leaf), leaf_index, proof)
