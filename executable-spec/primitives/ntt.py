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
