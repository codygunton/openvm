"""Poseidon2 hash over BabyBear via Rust FFI.

Wraps Plonky3's production Poseidon2BabyBear<16> permutation.

Reference:
    p3-symmetric (PaddingFreeSponge, TruncatedPermutation).
"""

import os

from poseidon2_ffi import (
    poseidon2_permute, poseidon2_compress_batch, poseidon2_hash_batch,
    set_num_threads, WIDTH, RATE, DIGEST_SIZE,
)

# Initialize rayon thread pool (default: 48 threads, override with RAYON_NUM_THREADS)
_NUM_THREADS = int(os.environ.get("RAYON_NUM_THREADS", "48"))
try:
    set_num_threads(_NUM_THREADS)
except RuntimeError:
    pass  # already initialized


def permute(state: list[int]) -> list[int]:
    """Apply Poseidon2 permutation to width-16 state.

    Args:
        state: 16 BabyBear field elements.

    Returns:
        Permuted 16-element state.
    """
    assert len(state) == WIDTH
    return poseidon2_permute(state)


def compress(left: list[int], right: list[int]) -> list[int]:
    """Compress two 8-element digests into one.

    TruncatedPermutation: concatenate inputs, permute, truncate to DIGEST_SIZE.

    Args:
        left: First 8-element digest.
        right: Second 8-element digest.

    Returns:
        8-element compressed digest.

    Reference:
        p3-symmetric TruncatedPermutation<Perm, 2, 8, 16>
    """
    assert len(left) == DIGEST_SIZE and len(right) == DIGEST_SIZE
    state = list(left) + list(right)
    state = permute(state)
    return state[:DIGEST_SIZE]


def compress_batch(lefts: list[list[int]], rights: list[list[int]]) -> list[list[int]]:
    """Batch compress N pairs of 8-element digests in parallel via rayon.

    Args:
        lefts: N left digests (each 8 elements).
        rights: N right digests (each 8 elements).

    Returns:
        N compressed 8-element digests.
    """
    return poseidon2_compress_batch(lefts, rights)


def hash_batch(inputs: list[list[int]]) -> list[list[int]]:
    """Batch hash N variable-length inputs to 8-element digests in parallel via rayon.

    Each input is hashed via PaddingFreeSponge (same as hash_to_digest).

    Args:
        inputs: N variable-length lists of BabyBear field elements.

    Returns:
        N 8-element digests.
    """
    return poseidon2_hash_batch(inputs)


def hash_to_digest(inputs: list[int]) -> list[int]:
    """Hash variable-length input to 8-element digest.

    PaddingFreeSponge: overwrite rate portion, permute, repeat.

    Args:
        inputs: Variable-length list of BabyBear field elements.

    Returns:
        8-element digest.

    Reference:
        p3-symmetric PaddingFreeSponge<Perm, 16, 8, 8>
    """
    state = [0] * WIDTH
    i = 0
    while i < len(inputs):
        for j in range(RATE):
            if i < len(inputs):
                state[j] = inputs[i]
                i += 1
        state = permute(state)
    return state[:DIGEST_SIZE]
