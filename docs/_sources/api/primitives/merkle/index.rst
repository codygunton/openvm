primitives.merkle
=================

.. py:module:: primitives.merkle

.. autoapi-nested-parse::

   Binary Merkle tree using Poseidon2 hash.

   Leaf hash: PaddingFreeSponge (hash_to_digest).
   Node compress: TruncatedPermutation (compress).

   Reference:
       p3-merkle-tree (FieldMerkleTreeMmcs).



Functions
---------

.. autoapisummary::

   primitives.merkle.build_merkle_tree
   primitives.merkle.get_opening_proof
   primitives.merkle.verify_opening_prehashed
   primitives.merkle.verify_opening


Module Contents
---------------

.. py:function:: build_merkle_tree(leaves: list[list[int]]) -> tuple[primitives.field.Digest, list[list[primitives.field.Digest]]]

   Build binary Merkle tree, returning (root, levels).

   Uses batch Poseidon2 operations (rayon-parallelized) for leaf hashing
   and internal node compression.

   Args:
       leaves: Leaf data (each a list of field elements, hashed internally).

   Returns:
       (root_digest, tree_levels) where tree_levels[0] is leaf digests.

   Reference:
       p3-merkle-tree (FieldMerkleTreeMmcs)


.. py:function:: get_opening_proof(tree: list[list[primitives.field.Digest]], leaf_index: int) -> primitives.field.MerklePath

   Get Merkle opening proof (sibling digests, leaf to root).

   Reference:
       p3-merkle-tree (FieldMerkleTreeMmcs)


.. py:function:: verify_opening_prehashed(root: primitives.field.Digest, leaf_digest: primitives.field.Digest, leaf_index: int, proof: primitives.field.MerklePath) -> bool

   Verify Merkle opening proof for pre-hashed leaf.

   Reference:
       p3-merkle-tree (FieldMerkleTreeMmcs::verify_batch)


.. py:function:: verify_opening(root: primitives.field.Digest, leaf: list[int], leaf_index: int, proof: primitives.field.MerklePath) -> bool

   Verify Merkle opening proof for unhashed leaf data.

   Reference:
       p3-merkle-tree (FieldMerkleTreeMmcs::verify_batch)


