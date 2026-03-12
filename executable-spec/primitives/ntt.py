"""Number Theoretic Transform (NTT) over BabyBear.

Uses Plonky3's SIMD-optimized Radix2DitParallel DFT via Rust FFI.
Fallback to galois library if FFI is unavailable.

Reference:
    p3-dft (Plonky3's DFT implementation for BabyBear).
"""

try:
    from poseidon2_ffi import ntt as _rust_ntt, intt as _rust_intt

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
