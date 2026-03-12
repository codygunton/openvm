"""BabyBear field GF(p) and quartic extension GF(p^4).

Uses galois library for all field arithmetic. FF and FF4 are the field types.

Type Discipline
---------------
When to use FF (base field):
- Domain generators (omega, omega_extended)
- Shift values for coset evaluation
- Single field element constants
- Hash outputs (8 base field elements)

When to use FF4 (extension field):
- Polynomial evaluations (prover and verifier)
- Challenges (derived from Fiat-Shamir transcript)
- Constraint polynomial values
- FRI folding values

FF4 is loaded from a pre-computed cache file (ff4_cache.pkl) to avoid the ~3.5s
initialization cost of galois.GF() for extension fields. If the cache does not exist,
it is generated automatically on first import.
"""

import pickle
from pathlib import Path

import galois
import numpy as np

# --- Field Construction ---

BABYBEAR_PRIME = 2013265921  # 2^31 - 2^27 + 1
FIELD_EXTENSION_DEGREE = 4
TWO_ADICITY = 27  # p - 1 = 2^27 * 15
GENERATOR = 31  # Multiplicative generator (Val::GENERATOR in Plonky3)
TWO_INV = pow(2, BABYBEAR_PRIME - 2, BABYBEAR_PRIME)  # Multiplicative inverse of 2
MONTY_R = pow(2, 32, BABYBEAR_PRIME)  # Montgomery form multiplier: 2^32 mod p = 268435454
MONTY_RINV = pow(MONTY_R, BABYBEAR_PRIME - 2, BABYBEAR_PRIME)  # Montgomery inverse

FF = galois.GF(BABYBEAR_PRIME)
"""Base field GF(p) - BabyBear prime field."""

# Load FF4 from cache to avoid ~3.5s galois initialization.
# If cache doesn't exist, construct and save it.
_FF4_CACHE_PATH = Path(__file__).parent / "ff4_cache.pkl"


def _build_ff4():
    """Construct the quartic extension field GF(p^4) with irreducible polynomial x^4 - 11."""
    irr = galois.Poly([1, 0, 0, 0, BABYBEAR_PRIME - 11], field=FF)
    return galois.GF(BABYBEAR_PRIME, 4, irreducible_poly=irr)


def _regenerate_ff4_cache() -> None:
    """Regenerate the FF4 cache file. Only needed if galois version changes."""
    ff4_field = _build_ff4()
    with open(_FF4_CACHE_PATH, "wb") as f:
        pickle.dump(ff4_field, f)


if _FF4_CACHE_PATH.exists():
    with open(_FF4_CACHE_PATH, "rb") as _f:
        FF4 = pickle.load(_f)
else:
    FF4 = _build_ff4()
    _regenerate_ff4_cache()

"""Quartic extension field GF(p^4) with irreducible polynomial x^4 - 11."""

# --- Type Aliases ---

FF4Poly = FF4  # Polynomial over extension field
FFPoly = FF  # Polynomial over base field
HashOutput = list[int]  # 8-element Poseidon2 digest

# Semantic type aliases for protocol code
Fe = int                    # Base field element (BabyBear, in [0, p))
EF4Coeffs = list[int]       # Extension field element as [c0,c1,c2,c3]
Digest = list[int]          # 8-element Poseidon2 digest (alias of HashOutput)
MerklePath = list[Digest]   # Merkle opening proof (list of sibling digests)


# --- Coefficient Order Conversion ---
# Galois encodes GF(p^4) elements as integers: a0 + a1*p + a2*p^2 + a3*p^3
# where the element is a0 + a1*x + a2*x^2 + a3*x^3.
# Our convention uses ascending order [a0, a1, a2, a3].

_P = BABYBEAR_PRIME
_P2 = _P * _P
_P3 = _P2 * _P


def ff4_coeffs(elem) -> list[int]:
    """Extract ascending-order coefficients [a0, a1, a2, a3] from FF4 element.

    Args:
        elem: A single FF4 element.

    Returns:
        List of 4 integers in ascending order [a0, a1, a2, a3].
    """
    val = int(elem)
    return [val % _P, (val // _P) % _P, (val // _P2) % _P, (val // _P3) % _P]


def ff4(coeffs: list[int]) -> FF4:
    """Construct FF4 scalar from ascending-order coefficients [a0, a1, a2, a3].

    Args:
        coeffs: 4 integers [a0, a1, a2, a3].

    Returns:
        Single FF4 element representing a0 + a1*x + a2*x^2 + a3*x^3.
    """
    a0 = coeffs[0] % _P
    a1 = coeffs[1] % _P
    a2 = coeffs[2] % _P
    a3 = coeffs[3] % _P
    return FF4(a0 + a1 * _P + a2 * _P2 + a3 * _P3)


def ff4_from_base(val: int) -> FF4:
    """Embed base field element into FF4 as (val, 0, 0, 0).

    Args:
        val: Base field element.

    Returns:
        FF4 element with only the constant coefficient set.
    """
    return FF4(int(val) % _P)


def ff4_array(c0: list[int], c1: list[int], c2: list[int], c3: list[int]) -> FF4:
    """Construct FF4 array from parallel coefficient lists.

    Args:
        c0, c1, c2, c3: Lists of equal length, one per coefficient position.

    Returns:
        Array of FF4 elements.
    """
    n = len(c0)
    vals = [
        int(c0[k]) % _P
        + (int(c1[k]) % _P) * _P
        + (int(c2[k]) % _P) * _P2
        + (int(c3[k]) % _P) * _P3
        for k in range(n)
    ]
    return FF4(vals)


def ff4_from_json(json_arr: list[list[int]]) -> FF4:
    """Parse JSON [[c0,c1,c2,c3],...] to FF4 array.

    Args:
        json_arr: List of 4-element coefficient lists in ascending order.

    Returns:
        Array of FF4 elements.
    """
    n = len(json_arr)
    c0 = [json_arr[i][0] for i in range(n)]
    c1 = [json_arr[i][1] for i in range(n)]
    c2 = [json_arr[i][2] for i in range(n)]
    c3 = [json_arr[i][3] for i in range(n)]
    return ff4_array(c0, c1, c2, c3)


def ff4_to_json(arr) -> list[list[int]]:
    """Convert FF4 array to JSON [[c0,c1,c2,c3],...] format.

    Args:
        arr: Array of FF4 elements.

    Returns:
        List of 4-element coefficient lists in ascending order.
    """
    return [ff4_coeffs(elem) for elem in arr]


# --- Extension Field Helpers ---
# These operate on EF4Coeffs (list[int]) representation, delegating to
# the galois FF4 type for arithmetic.


def ef4_from_base(x: Fe) -> EF4Coeffs:
    """Embed base field element into extension field as (x, 0, 0, 0).

    Reference:
        p3-field ExtensionField::from_base
    """
    return [x % BABYBEAR_PRIME, 0, 0, 0]


def ef4_mul(a: EF4Coeffs, b: EF4Coeffs) -> EF4Coeffs:
    """Multiply two extension field elements.

    Reference:
        p3-field BinomialExtensionField::mul
    """
    return ff4_coeffs(ff4(a) * ff4(b))


def ef4_mul_base(a: EF4Coeffs, b: Fe) -> EF4Coeffs:
    """Multiply extension field element by a base field element.

    Reference:
        p3-field ExtensionField::mul_base
    """
    return ff4_coeffs(ff4(a) * ff4_from_base(b))


def ef4_add(a: EF4Coeffs, b: EF4Coeffs) -> EF4Coeffs:
    """Add two extension field elements.

    Reference:
        p3-field BinomialExtensionField::add
    """
    return ff4_coeffs(ff4(a) + ff4(b))


def ef4_sub(a: EF4Coeffs, b: EF4Coeffs) -> EF4Coeffs:
    """Subtract two extension field elements.

    Reference:
        p3-field BinomialExtensionField::sub
    """
    return ff4_coeffs(ff4(a) - ff4(b))


def ef4_neg(a: EF4Coeffs) -> EF4Coeffs:
    """Negate an extension field element: -a.

    Reference:
        p3-field BinomialExtensionField::neg
    """
    return [(BABYBEAR_PRIME - c) % BABYBEAR_PRIME for c in a]


def ef4_inv(x: EF4Coeffs) -> EF4Coeffs:
    """Multiplicative inverse in extension field.

    Reference:
        p3-field BinomialExtensionField::inverse
    """
    return ff4_coeffs(ff4(x) ** (-1))


def ef4_div(a: EF4Coeffs, b: EF4Coeffs) -> EF4Coeffs:
    """Division in extension field: a / b.

    Reference:
        p3-field BinomialExtensionField::div
    """
    return ff4_coeffs(ff4(a) * ff4(b) ** (-1))


def ef4_pow(x: EF4Coeffs, n: int) -> EF4Coeffs:
    """Exponentiation in extension field by non-negative integer.

    Uses square-and-multiply.

    Reference:
        p3-field FieldAlgebra::exp_u64
    """
    if n == 0:
        return [1, 0, 0, 0]
    result = ff4(x) ** n
    return ff4_coeffs(result)


def ef4_exp_power_of_2(x: EF4Coeffs, log_power: int) -> EF4Coeffs:
    """Compute x^(2^log_power) by repeated squaring.

    Reference:
        p3-field Field::exp_power_of_2
    """
    result = ff4(x)
    for _ in range(log_power):
        result = result * result
    return ff4_coeffs(result)


# --- NTT Support ---

# Precomputed roots of unity: W[n] is a primitive 2^n-th root of unity in BabyBear.
# Computed as: W[27] = GENERATOR^((p-1)/2^27) = 31^15 mod p
# Then W[k] = W[k+1]^2 for k < 27.
_root_27 = pow(GENERATOR, (BABYBEAR_PRIME - 1) >> TWO_ADICITY, BABYBEAR_PRIME)

W: list[int] = [0] * (TWO_ADICITY + 1)
W[TWO_ADICITY] = _root_27
for _k in range(TWO_ADICITY - 1, -1, -1):
    W[_k] = pow(W[_k + 1], 2, BABYBEAR_PRIME)

# Precomputed inverses: W_INV[n] = W[n]^(-1) mod p
W_INV: list[int] = [pow(w, BABYBEAR_PRIME - 2, BABYBEAR_PRIME) if w != 0 else 0 for w in W]


def get_omega(n_bits: int) -> int:
    """Return primitive 2^n_bits-th root of unity.

    Args:
        n_bits: Log2 of the subgroup size (0 <= n_bits <= 27).

    Returns:
        Integer root of unity in [0, p).
    """
    return W[n_bits]


def get_omega_inv(n_bits: int) -> int:
    """Return inverse of primitive 2^n_bits-th root of unity.

    Args:
        n_bits: Log2 of the subgroup size (0 <= n_bits <= 27).

    Returns:
        Integer inverse root in [0, p).
    """
    return W_INV[n_bits]


def inv_mod(x: int) -> int:
    """Multiplicative inverse of x modulo BABYBEAR_PRIME."""
    return pow(x, BABYBEAR_PRIME - 2, BABYBEAR_PRIME)


# --- Bit Reversal Utilities ---


def to_monty(x: int) -> int:
    """Convert a canonical integer to BabyBear Montgomery form.

    In Plonky3, BabyBear uses Montgomery representation internally.
    All serde-serialized BabyBear values are in Montgomery form.
    When the Rust verifier calls `BabyBear::from_usize(x)` or
    `BabyBear::from_canonical_u32(x)`, the result is stored as
    `x * 2^32 mod p` (Montgomery form).

    Reference:
        p3-monty-31/src/monty_31.rs (MontyField31 Serialize/Deserialize)
    """
    return (x * MONTY_R) % BABYBEAR_PRIME


def from_monty(x: int) -> int:
    """Convert a BabyBear Montgomery-form value to canonical form.

    Serde serialization preserves Montgomery form. To get the canonical
    integer value, multiply by R^{-1} mod p.

    Our Poseidon2 FFI treats inputs/outputs as canonical values (it
    internally converts to/from Montgomery for the permutation). So all
    values must be in canonical form when used in the Python transcript.

    Reference:
        p3-monty-31/src/monty_31.rs (MontyField31::as_canonical_u32)
    """
    return (x * MONTY_RINV) % BABYBEAR_PRIME


def reverse_bits_len(x: int, bit_len: int) -> int:
    """Reverse the lowest bit_len bits of x.

    Reference:
        p3-util-0.4.2/src/lib.rs
    """
    result = 0
    for _ in range(bit_len):
        result = (result << 1) | (x & 1)
        x >>= 1
    return result


def bit_reverse_list(lst: list) -> list:
    """Reorder list elements by bit-reversing their indices."""
    n = len(lst)
    if n <= 1:
        return list(lst)
    log_n = n.bit_length() - 1
    return [lst[reverse_bits_len(i, log_n)] for i in range(n)]


# --- Montgomery Batch Inversion ---


def batch_inverse(values):
    """Montgomery batch inversion for any galois array.

    Converts N field inversions into 3N-3 multiplications + 1 inversion.

    Args:
        values: Galois FieldArray to invert (must all be non-zero).

    Returns:
        Galois FieldArray where result[i] = values[i]^(-1).

    Reference:
        pil2-stark/src/goldilocks/src/goldilocks_base_field.hpp::batchInverse
    """
    n = len(values)
    if n == 0:
        return values
    if n == 1:
        return values ** -1

    field_type = type(values)

    # Forward pass: compute prefix products
    cumprods = field_type.Zeros(n)
    cumprods[0] = values[0]
    for i in range(1, n):
        cumprods[i] = cumprods[i - 1] * values[i]

    # Single inversion of the total product
    inv_total = cumprods[n - 1] ** -1

    # Backward pass: extract individual inverses
    results = field_type.Zeros(n)
    z = inv_total
    for i in range(n - 1, 0, -1):
        results[i] = z * cumprods[i - 1]
        z = z * values[i]
    results[0] = z
    return results


def ef4_batch_inverse(values: list[EF4Coeffs]) -> list[EF4Coeffs]:
    """Montgomery batch inversion for EF4 elements using coefficient representation.

    Converts N extension field inversions into 3N-3 multiplications + 1 inversion.

    Args:
        values: List of EF4Coeffs to invert (must all be non-zero).

    Returns:
        List of EF4Coeffs where result[i] = values[i]^{-1}.

    Reference:
        p3-field batch_multiplicative_inverse
    """
    n = len(values)
    if n == 0:
        return []
    if n == 1:
        return [ef4_inv(values[0])]

    # Forward pass: prefix products
    cumprods: list[EF4Coeffs] = [None] * n
    cumprods[0] = values[0]
    for i in range(1, n):
        cumprods[i] = ef4_mul(cumprods[i - 1], values[i])

    # Single inversion of the total product
    inv_total = ef4_inv(cumprods[n - 1])

    # Backward pass
    results: list[EF4Coeffs] = [None] * n
    z = inv_total
    for i in range(n - 1, 0, -1):
        results[i] = ef4_mul(z, cumprods[i - 1])
        z = ef4_mul(z, values[i])
    results[0] = z

    return results


# --- Vectorized EF4 Arithmetic (numpy int64) ---
# EF4 element = (c0, c1, c2, c3) where element = c0 + c1*x + c2*x^2 + c3*x^3
# and x^4 = _W_EXT = 11.
# Each coefficient is a numpy int64 array of shape (n,).

_W_EXT = 11  # Extension polynomial constant: x^4 - 11

EF4Vec = tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]


def _v_mod(a: np.ndarray) -> np.ndarray:
    return a % _P


def _v_mul(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    return (a * b) % _P


def _v_add(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    return (a + b) % _P


def _v_sub(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    return (a - b) % _P


def _v_neg(a: np.ndarray) -> np.ndarray:
    return (_P - a) % _P


def _v_modpow(base: np.ndarray, exp: int) -> np.ndarray:
    result = np.ones_like(base)
    base = base % _P
    while exp > 0:
        if exp & 1:
            result = (result * base) % _P
        base = (base * base) % _P
        exp >>= 1
    return result


def _v_inv(a: np.ndarray) -> np.ndarray:
    return _v_modpow(a, _P - 2)


def ef4v_add(a: EF4Vec, b: EF4Vec) -> EF4Vec:
    """Add two vectorized EF4 elements coefficient-wise."""
    return (_v_add(a[0], b[0]), _v_add(a[1], b[1]),
            _v_add(a[2], b[2]), _v_add(a[3], b[3]))


def ef4v_sub(a: EF4Vec, b: EF4Vec) -> EF4Vec:
    """Subtract two vectorized EF4 elements coefficient-wise."""
    return (_v_sub(a[0], b[0]), _v_sub(a[1], b[1]),
            _v_sub(a[2], b[2]), _v_sub(a[3], b[3]))


def ef4v_neg(a: EF4Vec) -> EF4Vec:
    """Negate a vectorized EF4 element coefficient-wise."""
    return (_v_neg(a[0]), _v_neg(a[1]), _v_neg(a[2]), _v_neg(a[3]))


def ef4v_mul(a: EF4Vec, b: EF4Vec) -> EF4Vec:
    """Multiply two vectorized EF4 elements using x^4 = W_EXT.

    Reference: p3-field BinomialExtensionField::mul (vectorized variant)
    """
    a0, a1, a2, a3 = a
    b0, b1, b2, b3 = b
    Wc = np.int64(_W_EXT)
    # c0 = a0*b0 + W*(a1*b3 + a2*b2 + a3*b1)
    cross_0 = _v_add(_v_add(_v_mul(a1, b3), _v_mul(a2, b2)), _v_mul(a3, b1))
    c0 = _v_add(_v_mul(a0, b0), _v_mul(Wc, cross_0))
    # c1 = a0*b1 + a1*b0 + W*(a2*b3 + a3*b2)
    cross_1 = _v_add(_v_mul(a2, b3), _v_mul(a3, b2))
    c1 = _v_add(_v_add(_v_mul(a0, b1), _v_mul(a1, b0)), _v_mul(Wc, cross_1))
    # c2 = a0*b2 + a1*b1 + a2*b0 + W*a3*b3
    c2 = _v_add(_v_add(_v_add(_v_mul(a0, b2), _v_mul(a1, b1)), _v_mul(a2, b0)),
                _v_mul(Wc, _v_mul(a3, b3)))
    # c3 = a0*b3 + a1*b2 + a2*b1 + a3*b0
    c3 = _v_add(_v_add(_v_add(_v_mul(a0, b3), _v_mul(a1, b2)), _v_mul(a2, b1)),
                _v_mul(a3, b0))
    return (c0, c1, c2, c3)


def ef4v_mul_base(a: EF4Vec, b: np.ndarray) -> EF4Vec:
    """Multiply vectorized EF4 by a base-field array."""
    if not isinstance(b, np.ndarray) or b.dtype != np.int64:
        b = np.asarray(b, dtype=np.int64)
    return (_v_mul(a[0], b), _v_mul(a[1], b), _v_mul(a[2], b), _v_mul(a[3], b))


def ef4v_from_base(b: np.ndarray) -> EF4Vec:
    """Lift a base-field array to EF4Vec: [val, 0, 0, 0]."""
    if not isinstance(b, np.ndarray) or b.dtype != np.int64:
        b = np.asarray(b, dtype=np.int64)
    z = np.zeros(len(b), dtype=np.int64)
    return (b % _P, z, z, z)


def ef4v_from_scalar(coeffs: EF4Coeffs, n: int) -> EF4Vec:
    """Broadcast a scalar EF4 element to an EF4Vec of length n."""
    return (
        np.full(n, coeffs[0] % _P, dtype=np.int64),
        np.full(n, coeffs[1] % _P, dtype=np.int64),
        np.full(n, coeffs[2] % _P, dtype=np.int64),
        np.full(n, coeffs[3] % _P, dtype=np.int64),
    )


def ef4v_inv(a: EF4Vec) -> EF4Vec:
    """Invert a vectorized EF4 element via tower decomposition.

    GF(p^4) = GF(p^2)[y] / (y^2 - W_EXT) where GF(p^2) = GF(p)[x] / (x^2 - W_EXT).
    Write element as (a_lo + a_hi*y) where a_lo = a0 + a2*x, a_hi = a1 + a3*x.
    Compute norm = a_lo^2 - W * a_hi^2 in GF(p^2), invert norm, multiply back.

    Reference: p3-field BinomialExtensionField::inverse (tower method)
    """
    a0, a1, a2, a3 = a
    Wc = np.int64(_W_EXT)
    a2_0 = _v_add(_v_mul(a0, a0), _v_mul(Wc, _v_mul(a2, a2)))
    a2_1 = _v_mul(np.int64(2), _v_mul(a0, a2))
    b2_0 = _v_add(_v_mul(a1, a1), _v_mul(Wc, _v_mul(a3, a3)))
    b2_1 = _v_mul(np.int64(2), _v_mul(a1, a3))
    b2u_0 = _v_mul(Wc, b2_1)
    b2u_1 = b2_0
    d0 = _v_sub(a2_0, b2u_0)
    d1 = _v_sub(a2_1, b2u_1)
    norm = _v_sub(_v_mul(d0, d0), _v_mul(Wc, _v_mul(d1, d1)))
    ni = _v_inv(norm)
    e0 = _v_mul(d0, ni)
    e1 = _v_neg(_v_mul(d1, ni))
    ae_0 = _v_add(_v_mul(a0, e0), _v_mul(Wc, _v_mul(a2, e1)))
    ae_1 = _v_add(_v_mul(a0, e1), _v_mul(a2, e0))
    be_0 = _v_add(_v_mul(a1, e0), _v_mul(Wc, _v_mul(a3, e1)))
    be_1 = _v_add(_v_mul(a1, e1), _v_mul(a3, e0))
    return (ae_0, _v_neg(be_0), ae_1, _v_neg(be_1))


def ef4v_mul_scalar(a: EF4Vec, s: EF4Coeffs) -> EF4Vec:
    """Multiply vectorized EF4 by a scalar EF4 element."""
    s0, s1, s2, s3 = np.int64(s[0] % _P), np.int64(s[1] % _P), np.int64(s[2] % _P), np.int64(s[3] % _P)
    a0, a1, a2, a3 = a
    Wc = np.int64(_W_EXT)
    # c0 = a0*s0 + W*(a1*s3 + a2*s2 + a3*s1)
    cross_0 = _v_add(_v_add(_v_mul(a1, s3), _v_mul(a2, s2)), _v_mul(a3, s1))
    c0 = _v_add(_v_mul(a0, s0), _v_mul(Wc, cross_0))
    # c1 = a0*s1 + a1*s0 + W*(a2*s3 + a3*s2)
    cross_1 = _v_add(_v_mul(a2, s3), _v_mul(a3, s2))
    c1 = _v_add(_v_add(_v_mul(a0, s1), _v_mul(a1, s0)), _v_mul(Wc, cross_1))
    # c2 = a0*s2 + a1*s1 + a2*s0 + W*a3*s3
    c2 = _v_add(_v_add(_v_add(_v_mul(a0, s2), _v_mul(a1, s1)), _v_mul(a2, s0)),
                _v_mul(Wc, _v_mul(a3, s3)))
    # c3 = a0*s3 + a1*s2 + a2*s1 + a3*s0
    c3 = _v_add(_v_add(_v_add(_v_mul(a0, s3), _v_mul(a1, s2)), _v_mul(a2, s1)),
                _v_mul(a3, s0))
    return (c0, c1, c2, c3)


# --- Base Field Column Helpers (galois FF) ---


def ff_column(data: list[Fe]) -> FF:
    """Create a base-field column vector from a list of field elements."""
    return FF(data)


def ff_zeros(n: int) -> FF:
    """Create a zero base-field column vector of length n."""
    return FF.Zeros(n)


def ff_constant(val: Fe, n: int) -> FF:
    """Create a constant base-field column vector: [val, val, ..., val]."""
    return FF.Ones(n) * FF(val % BABYBEAR_PRIME)


def ff_roll(arr, shift: int):
    """Circular shift a base-field column vector (wraps np.roll)."""
    return np.roll(arr, shift)


# --- EF4Vec Conversion Helpers ---


def ef4v_zeros(n: int) -> EF4Vec:
    """Create a zero EF4Vec of length n."""
    z = np.zeros(n, dtype=np.int64)
    return (z.copy(), z.copy(), z.copy(), z.copy())


def ef4v_from_rows(rows: list[EF4Coeffs]) -> EF4Vec:
    """Convert a list of EF4 coefficient lists to an EF4Vec.

    Input: [[c0, c1, c2, c3], ...] of length n
    Output: EF4Vec with 4 arrays of length n
    """
    arr = np.array(rows, dtype=np.int64)
    return (arr[:, 0].copy(), arr[:, 1].copy(), arr[:, 2].copy(), arr[:, 3].copy())


def ef4v_to_rows(v: EF4Vec) -> list[EF4Coeffs]:
    """Convert an EF4Vec to a list of EF4 coefficient lists.

    Input: EF4Vec with 4 arrays of length n
    Output: [[c0, c1, c2, c3], ...] of length n
    """
    return np.stack(v, axis=1).tolist()


def ef4v_roll(v: EF4Vec, shift: int) -> EF4Vec:
    """Circular shift an EF4Vec (applies np.roll to each component)."""
    return (np.roll(v[0], shift), np.roll(v[1], shift),
            np.roll(v[2], shift), np.roll(v[3], shift))


def ef4v_cumsum(v: EF4Vec) -> EF4Vec:
    """Compute prefix sum of an EF4Vec, reducing each component mod p."""
    return (
        np.cumsum(v[0].astype(np.int64)) % _P,
        np.cumsum(v[1].astype(np.int64)) % _P,
        np.cumsum(v[2].astype(np.int64)) % _P,
        np.cumsum(v[3].astype(np.int64)) % _P,
    )


def ef4_pairs_to_leaves(evals: list[EF4Coeffs]) -> list[list[int]]:
    """Pair consecutive EF4 elements into 8-element Merkle leaves."""
    return [evals[i] + evals[i + 1] for i in range(0, len(evals), 2)]


# --- Batch polynomial evaluation at a single EF4 point ---


def _ef4_mul_raw(
    a: EF4Coeffs,
    b: EF4Coeffs,
) -> EF4Coeffs:
    a0, a1, a2, a3 = a
    b0, b1, b2, b3 = b
    c0 = (a0 * b0 + _W_EXT * (a1 * b3 + a2 * b2 + a3 * b1)) % _P
    c1 = (a0 * b1 + a1 * b0 + _W_EXT * (a2 * b3 + a3 * b2)) % _P
    c2 = (a0 * b2 + a1 * b1 + a2 * b0 + _W_EXT * a3 * b3) % _P
    c3 = (a0 * b3 + a1 * b2 + a2 * b1 + a3 * b0) % _P
    return (c0, c1, c2, c3)


def _precompute_z_powers_bsgs(
    z: EF4Coeffs,
    degree: int,
) -> EF4Vec:
    if degree <= 0:
        return tuple(np.empty(0, dtype=np.int64) for _ in range(4))

    B = max(1, int(degree ** 0.5))
    num_blocks = (degree + B - 1) // B

    small = [(1, 0, 0, 0)]
    cur = (1, 0, 0, 0)
    for _ in range(B - 1):
        cur = _ef4_mul_raw(cur, z)
        small.append(cur)

    z_B = _ef4_mul_raw(cur, z)

    big = [(1, 0, 0, 0)]
    cur = (1, 0, 0, 0)
    for _ in range(num_blocks - 1):
        cur = _ef4_mul_raw(cur, z_B)
        big.append(cur)

    small_np = [np.array([s[c] for s in small], dtype=np.int64) for c in range(4)]
    big_np = [np.array([b[c] for b in big], dtype=np.int64) for c in range(4)]

    def _outer_flat(a: np.ndarray, b: np.ndarray) -> np.ndarray:
        return np.outer(a, b).ravel()[:degree]

    def _outer_mod(a: np.ndarray, b: np.ndarray) -> np.ndarray:
        return _outer_flat(a, b) % _P

    c0 = _outer_mod(big_np[0], small_np[0])
    w_sum = (_outer_flat(big_np[1], small_np[3]) + _outer_flat(big_np[2], small_np[2])) % _P
    w_sum = (w_sum + _outer_mod(big_np[3], small_np[1])) % _P
    c0 = (c0 + _W_EXT * w_sum) % _P

    t1 = (_outer_flat(big_np[0], small_np[1]) + _outer_flat(big_np[1], small_np[0])) % _P
    w_sum1 = (_outer_flat(big_np[2], small_np[3]) + _outer_flat(big_np[3], small_np[2])) % _P
    c1 = (t1 + _W_EXT * w_sum1) % _P

    t2 = (_outer_flat(big_np[0], small_np[2]) + _outer_flat(big_np[1], small_np[1])) % _P
    t2 = (t2 + _outer_mod(big_np[2], small_np[0])) % _P
    c2 = (t2 + _W_EXT * _outer_mod(big_np[3], small_np[3])) % _P

    t3 = (_outer_flat(big_np[0], small_np[3]) + _outer_flat(big_np[1], small_np[2])) % _P
    t3_2 = (_outer_flat(big_np[2], small_np[1]) + _outer_flat(big_np[3], small_np[0])) % _P
    c3 = (t3 + t3_2) % _P

    return (c0, c1, c2, c3)


def eval_poly_ef4_batch(
    coeffs_per_col: list[list[Fe]],
    eval_point: EF4Coeffs,
) -> list[EF4Coeffs]:
    """Evaluate multiple polynomials at a single EF4 point using baby-step giant-step.

    For each polynomial f, computes f(eval_point) where f(z) = sum_i coeffs[i] * z^i.
    BSGS precomputes z^0..z^{B-1} and z^B, z^{2B}, ... once, then evaluates each
    polynomial via a single accumulation. Faster than per-polynomial Horner when many
    polynomials share the same evaluation point.

    Args:
        coeffs_per_col: Polynomial coefficient vectors, all same degree.
        eval_point: EF4 point at which to evaluate.

    Returns:
        List of EF4Coeffs, one per polynomial.
    """
    num_cols = len(coeffs_per_col)
    if num_cols == 0:
        return []
    degree = len(coeffs_per_col[0])
    if degree == 0:
        return [[0, 0, 0, 0]] * num_cols

    z = tuple(int(c) for c in eval_point)
    z_powers = _precompute_z_powers_bsgs(z, degree)

    coeff_mat = np.array(coeffs_per_col, dtype=np.int64)

    results = []
    for zp in z_powers:
        products = (coeff_mat * zp) % _P
        result_j = products.sum(axis=1) % _P
        results.append(result_j)

    out = np.stack(results, axis=1)
    return out.tolist()
