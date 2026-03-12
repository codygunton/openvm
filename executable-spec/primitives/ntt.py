"""Number Theoretic Transform (NTT) over BabyBear.

Uses Plonky3's SIMD-optimized Radix2DitParallel DFT via Rust FFI.
Fallback to galois library if FFI is unavailable.

Reference:
    p3-dft (Plonky3's DFT implementation for BabyBear).
"""

try:
    from poseidon2_ffi import (
        ntt as _rust_ntt,
        intt as _rust_intt,
        coset_lde_batch as _rust_coset_lde_batch,
        ntt_batch as _rust_ntt_batch,
        intt_batch as _rust_intt_batch,
    )

    def ntt(coeffs: list[int]) -> list[int]:
        """Forward NTT: coefficient form -> evaluation form.

        Uses Plonky3's SIMD-optimized Radix2DitParallel DFT (~50x faster than galois).

        Args:
            coeffs: Polynomial coefficients [a0, a1, ..., a_{n-1}].
                    Length must be a power of 2.

        Returns:
            Evaluations at the n-th roots of unity.
        """
        return list(_rust_ntt(coeffs))

    def intt(evals: list[int]) -> list[int]:
        """Inverse NTT: evaluation form -> coefficient form.

        Uses Plonky3's SIMD-optimized Radix2DitParallel IDFT.

        Args:
            evals: Evaluations at [1, omega, omega^2, ..., omega^(n-1)].
                   Length must be a power of 2.

        Returns:
            Polynomial coefficients [a0, a1, ..., a_{n-1}].
        """
        return list(_rust_intt(evals))

    def coset_lde_batch(columns, shift, log_blowup):
        """Batch coset LDE via Rust FFI."""
        return _rust_coset_lde_batch(columns, shift, log_blowup)

    def ntt_batch(columns):
        """Batch forward NTT via Rust FFI."""
        return _rust_ntt_batch(columns)

    def intt_batch(columns):
        """Batch inverse NTT via Rust FFI."""
        return _rust_intt_batch(columns)

except ImportError:
    # Fallback to galois library
    import galois
    from primitives.field import FF, BABYBEAR_PRIME

    def ntt(coeffs: list[int]) -> list[int]:
        ff_coeffs = FF(coeffs)
        result = galois.ntt(ff_coeffs, modulus=BABYBEAR_PRIME)
        return [int(x) for x in result]

    def intt(evals: list[int]) -> list[int]:
        ff_evals = FF(evals)
        result = galois.intt(ff_evals, modulus=BABYBEAR_PRIME)
        return [int(x) for x in result]

    def coset_lde_batch(columns, shift, log_blowup):
        raise ImportError("coset_lde_batch requires poseidon2_ffi")

    def ntt_batch(columns):
        raise ImportError("ntt_batch requires poseidon2_ffi")

    def intt_batch(columns):
        raise ImportError("intt_batch requires poseidon2_ffi")


# ---------------------------------------------------------------------------
# Higher-level NTT-based primitives
# ---------------------------------------------------------------------------

from primitives.field import BABYBEAR_PRIME, EF4Coeffs, Fe

_P = BABYBEAR_PRIME


def coset_lde(evals: list[Fe], shift: Fe, log_blowup: int) -> list[Fe]:
    """Low-degree extension onto a coset: INTT, zero-pad, coset shift, NTT.

    Given evaluations of a polynomial on a subgroup H, compute evaluations
    on the coset shift*K where K is a larger subgroup (|K| = |H| << log_blowup).

    Algorithm (matches p3-dft coset_lde_batch):
    1. INTT to recover polynomial coefficients from subgroup evaluations.
    2. Zero-pad coefficients to the target domain size.
    3. Multiply coefficient[j] by shift^j (transforms p(x) -> p(shift*x)).
    4. NTT to evaluate on the larger subgroup.

    Reference:
        p3-dft-0.4.1/src/traits.rs coset_lde_batch (lines 226-249)
        p3-dft-0.4.1/src/util.rs   coset_shift_cols (lines 28-36)

    Args:
        evals: Evaluations on the original subgroup (length must be a power of 2).
        shift: Coset shift element (the LDE shift).
        log_blowup: log2 of the blowup factor (target size = len(evals) << log_blowup).

    Returns:
        Evaluations on the shifted coset of size len(evals) << log_blowup.
    """
    n = len(evals)
    target_size = n << log_blowup

    # Step 1: INTT to get coefficients
    coeffs = intt(evals)

    # Step 2: Zero-pad to target size
    coeffs.extend([0] * (target_size - n))

    # Step 3: Coset shift — multiply coeffs[j] by shift^j
    shift_pow = 1
    for j in range(target_size):
        coeffs[j] = (coeffs[j] * shift_pow) % _P
        shift_pow = (shift_pow * shift) % _P

    # Step 4: NTT on the larger subgroup
    return ntt(coeffs)


def ef4_idft(evals: list[EF4Coeffs]) -> list[EF4Coeffs]:
    """Inverse DFT for extension field evaluations (channel-wise INTT).

    Decomposes each EF4 element into 4 base-field channels, applies INTT
    to each channel independently, then recombines into EF4 coefficients.

    Reference:
        p3-dft traits.rs (idft_algebra)

    Args:
        evals: Extension field evaluations, each a 4-element list [c0, c1, c2, c3].
               Length must be a power of 2.

    Returns:
        Polynomial coefficients in extension field form.
    """
    n = len(evals)
    if n == 1:
        return [list(evals[0])]

    # Transpose: extract each coefficient channel
    channels = [[evals[j][k] for j in range(n)] for k in range(4)]

    # INTT each channel independently
    channels_coeffs = [intt(ch) for ch in channels]

    # Transpose back to EF4 elements
    return [[channels_coeffs[k][j] for k in range(4)] for j in range(n)]
