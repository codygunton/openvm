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
