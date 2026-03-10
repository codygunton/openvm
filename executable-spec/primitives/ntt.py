"""Number Theoretic Transform (NTT) over BabyBear.

Uses galois library's NTT/INTT which operates on GF(p) arrays with
BabyBear-compatible roots of unity.

Reference:
    p3-dft (Plonky3's DFT implementation for BabyBear).
"""

import galois

from primitives.field import FF, BABYBEAR_PRIME


def ntt(coeffs: list[int]) -> list[int]:
    """Forward NTT: coefficient form -> evaluation form.

    Evaluates polynomial at [1, omega, omega^2, ..., omega^(n-1)]
    where omega is a primitive n-th root of unity in BabyBear.

    Args:
        coeffs: Polynomial coefficients [a0, a1, ..., a_{n-1}].
                Length must be a power of 2.

    Returns:
        Evaluations at the n-th roots of unity.
    """
    ff_coeffs = FF(coeffs)
    result = galois.ntt(ff_coeffs, modulus=BABYBEAR_PRIME)
    return [int(x) for x in result]


def intt(evals: list[int]) -> list[int]:
    """Inverse NTT: evaluation form -> coefficient form.

    Interpolates polynomial from evaluations at roots of unity.

    Args:
        evals: Evaluations at [1, omega, omega^2, ..., omega^(n-1)].
               Length must be a power of 2.

    Returns:
        Polynomial coefficients [a0, a1, ..., a_{n-1}].
    """
    ff_evals = FF(evals)
    result = galois.intt(ff_evals, modulus=BABYBEAR_PRIME)
    return [int(x) for x in result]
