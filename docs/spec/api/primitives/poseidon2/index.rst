primitives.poseidon2
====================

.. py:module:: primitives.poseidon2

.. autoapi-nested-parse::

   Poseidon2 hash over BabyBear via Rust FFI.

   Wraps Plonky3's production Poseidon2BabyBear<16> permutation.

   Reference:
       p3-symmetric (PaddingFreeSponge, TruncatedPermutation).



Functions
---------

.. autoapisummary::

   primitives.poseidon2.permute
   primitives.poseidon2.compress
   primitives.poseidon2.compress_batch
   primitives.poseidon2.hash_batch
   primitives.poseidon2.hash_to_digest


Module Contents
---------------

.. py:function:: permute(state: list[int]) -> list[int]

   Apply Poseidon2 permutation to width-16 state.

   Args:
       state: 16 BabyBear field elements.

   Returns:
       Permuted 16-element state.


.. py:function:: compress(left: list[int], right: list[int]) -> list[int]

   Compress two 8-element digests into one.

   TruncatedPermutation: concatenate inputs, permute, truncate to DIGEST_SIZE.

   Args:
       left: First 8-element digest.
       right: Second 8-element digest.

   Returns:
       8-element compressed digest.

   Reference:
       p3-symmetric TruncatedPermutation<Perm, 2, 8, 16>


.. py:function:: compress_batch(lefts: list[list[int]], rights: list[list[int]]) -> list[list[int]]

   Batch compress N pairs of 8-element digests in parallel via rayon.

   Args:
       lefts: N left digests (each 8 elements).
       rights: N right digests (each 8 elements).

   Returns:
       N compressed 8-element digests.


.. py:function:: hash_batch(inputs: list[list[int]]) -> list[list[int]]

   Batch hash N variable-length inputs to 8-element digests in parallel via rayon.

   Each input is hashed via PaddingFreeSponge (same as hash_to_digest).

   Args:
       inputs: N variable-length lists of BabyBear field elements.

   Returns:
       N 8-element digests.


.. py:function:: hash_to_digest(inputs: list[int]) -> list[int]

   Hash variable-length input to 8-element digest.

   PaddingFreeSponge: overwrite rate portion, permute, repeat.

   Args:
       inputs: Variable-length list of BabyBear field elements.

   Returns:
       8-element digest.

   Reference:
       p3-symmetric PaddingFreeSponge<Perm, 16, 8, 8>


