"""BabyBear field GF(p) and quartic extension GF(p^4).

Uses galois library for base field GF(p) arithmetic.
Uses custom EF4 class for fast quartic extension field arithmetic.

Type Discipline
---------------
FF (galois GF(p)):
    Base field columns and scalars. Fast jit-compiled numpy operations.

EF4:
    Extension field GF(p^4) scalars and columns. Stores coefficients as int64 arrays.
    Supports operator overloads: +, -, *, /, **, negation.
    Scalars: shape (4,). Columns: shape (N, 4).
    Coefficients in ascending degree: c0 + c1*x + c2*x^2 + c3*x^3 as [c0, c1, c2, c3].
"""

import pickle
from pathlib import Path

import galois
import numpy as np

# --- Field Construction ---

BABYBEAR_PRIME = 2013265921  # 2^31 - 2^27 + 1
FIELD_EXTENSION_DEGREE = 4
DIGEST_WIDTH = 8  # Poseidon2 digest width in BabyBear elements
TWO_ADICITY = 27  # p - 1 = 2^27 * 15
GENERATOR = 31  # Multiplicative generator (Val::GENERATOR in Plonky3)
TWO_INV = pow(2, BABYBEAR_PRIME - 2, BABYBEAR_PRIME)  # Multiplicative inverse of 2
MONTY_R = pow(2, 32, BABYBEAR_PRIME)  # Montgomery form multiplier: 2^32 mod p = 268435454
MONTY_RINV = pow(MONTY_R, BABYBEAR_PRIME - 2, BABYBEAR_PRIME)  # Montgomery inverse

FF = galois.GF(BABYBEAR_PRIME)
"""Base field GF(p) - BabyBear prime field."""

# Load galois FF4 from cache (used only for test backward compat and JSON roundtrip).
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
        _GaloisFF4 = pickle.load(_f)
else:
    _GaloisFF4 = _build_ff4()
    _regenerate_ff4_cache()


# --- Internal constants ---

_P = BABYBEAR_PRIME
_P2 = _P * _P
_P3 = _P2 * _P
_W_EXT = 11  # Extension polynomial constant: x^4 = 11


# --- EF4: Fast Extension Field GF(p^4) ---


def _modpow_arr(base: np.ndarray, exp: int) -> np.ndarray:
    """Vectorized modular exponentiation for int64 arrays."""
    result = np.ones_like(base)
    base = base % _P
    while exp > 0:
        if exp & 1:
            result = (result * base) % _P
        base = (base * base) % _P
        exp >>= 1
    return result


class EF4:
    """Extension field GF(p^4) = GF(p)[x]/(x^4 - 11) element or column.

    Internal storage: int64 ndarray of shape (4,) for scalar or (N, 4) for column.
    Coefficients in ascending degree: c0 + c1*x + c2*x^2 + c3*x^3 stored as [c0, c1, c2, c3].

    Supports operator overloads for clean mathematical notation:
        c = a + b          # addition
        c = a - b          # subtraction
        c = a * b          # polynomial multiplication
        c = a / b          # division (multiply by inverse)
        c = a ** n         # exponentiation (n can be -1)
        c = -a             # negation
    """
    __slots__ = ('_d',)

    def __init__(self, data=None):
        if isinstance(data, EF4):
            self._d = data._d.copy()
        elif isinstance(data, int):
            # Embed base field element as (val, 0, 0, 0)
            self._d = np.array([data % _P, 0, 0, 0], dtype=np.int64)
        elif isinstance(data, np.integer):
            self._d = np.array([int(data) % _P, 0, 0, 0], dtype=np.int64)
        elif isinstance(data, (list, tuple)):
            arr = np.asarray(data, dtype=np.int64)
            if arr.ndim == 1 and arr.shape[0] == 4:
                self._d = arr % _P
            elif arr.ndim == 2 and arr.shape[1] == 4:
                self._d = arr % _P
            elif arr.ndim == 1 and arr.shape[0] != 4:
                raise ValueError(f"EF4 1D array must have 4 elements, got {arr.shape[0]}")
            else:
                raise ValueError(f"EF4 array must be shape (4,) or (N,4), got {arr.shape}")
        elif hasattr(data, 'vector'):
            # galois FieldArray — extract polynomial coefficients.
            # Must check before np.ndarray since FieldArray is a subclass.
            # galois uses highest-degree-first; we use lowest-degree-first.
            v = np.array(data.vector().view(np.ndarray), dtype=np.int64)
            if v.ndim == 1:
                self._d = v[::-1].copy() % _P
            elif v.ndim == 2:
                self._d = v[:, ::-1].copy() % _P
            else:
                raise ValueError(f"Unexpected galois vector shape: {v.shape}")
        elif isinstance(data, np.ndarray):
            if data.ndim == 1 and data.shape[0] == 4:
                self._d = data.astype(np.int64) % _P
            elif data.ndim == 2 and data.shape[1] == 4:
                self._d = data.astype(np.int64) % _P
            else:
                raise ValueError(f"EF4 array must be shape (4,) or (N,4), got {data.shape}")
        elif data is None:
            self._d = np.array([0, 0, 0, 0], dtype=np.int64)
        else:
            raise TypeError(f"Cannot create EF4 from {type(data)}")

    @classmethod
    def _wrap(cls, data: np.ndarray) -> 'EF4':
        """Wrap raw int64 array without copying or mod. Internal use only."""
        obj = object.__new__(cls)
        obj._d = data
        return obj

    # --- Constructors ---

    @classmethod
    def zeros(cls, n: int) -> 'EF4':
        """Zero column of length n."""
        return cls._wrap(np.zeros((n, 4), dtype=np.int64))

    @classmethod
    def one(cls) -> 'EF4':
        """Multiplicative identity scalar."""
        return cls._wrap(np.array([1, 0, 0, 0], dtype=np.int64))

    @classmethod
    def zero(cls) -> 'EF4':
        """Additive identity scalar."""
        return cls._wrap(np.array([0, 0, 0, 0], dtype=np.int64))

    @classmethod
    def from_base(cls, base) -> 'EF4':
        """Lift base field values to EF4 as (val, 0, 0, 0).

        Args:
            base: int, FF array, list[int], or ndarray of base field elements.
        """
        if isinstance(base, (int, np.integer)):
            return cls._wrap(np.array([int(base) % _P, 0, 0, 0], dtype=np.int64))
        if isinstance(base, np.ndarray):
            vals = base.view(np.ndarray).ravel().astype(np.int64) % _P
            d = np.zeros((len(vals), 4), dtype=np.int64)
            d[:, 0] = vals
            return cls._wrap(d)
        if isinstance(base, list):
            vals = np.array(base, dtype=np.int64) % _P
            d = np.zeros((len(vals), 4), dtype=np.int64)
            d[:, 0] = vals
            return cls._wrap(d)
        raise TypeError(f"Cannot create EF4.from_base from {type(base)}")

    @classmethod
    def from_rows(cls, rows: list) -> 'EF4':
        """Create from list of [c0, c1, c2, c3] coefficient lists."""
        return cls(np.array(rows, dtype=np.int64))

    @classmethod
    def broadcast(cls, scalar: 'EF4', n: int) -> 'EF4':
        """Broadcast a scalar EF4 to a column of length n."""
        if isinstance(scalar, cls):
            coeffs = scalar._d if scalar._d.ndim == 1 else scalar._d[0]
        elif isinstance(scalar, (list, tuple)):
            coeffs = np.array(scalar, dtype=np.int64) % _P
        else:
            raise TypeError(f"Cannot broadcast {type(scalar)}")
        return cls._wrap(np.tile(coeffs, (n, 1)))

    # --- Properties ---

    @property
    def is_scalar(self) -> bool:
        return self._d.ndim == 1

    @property
    def coeffs(self) -> tuple:
        """Return coefficients as tuple (c0, c1, c2, c3) for a scalar."""
        if self._d.ndim == 1:
            return tuple(int(x) for x in self._d)
        raise ValueError("coeffs only available for scalars, use to_rows() for columns")

    @property
    def c0(self) -> int:
        """Constant coefficient (base field component)."""
        if self._d.ndim == 1:
            return int(self._d[0])
        raise ValueError("c0 only available for scalars")

    # --- Access ---

    def __len__(self) -> int:
        if self._d.ndim == 1:
            return 1
        return self._d.shape[0]

    def __getitem__(self, key):
        if self._d.ndim == 1:
            if isinstance(key, (int, np.integer)):
                # Return individual coefficient
                return int(self._d[key])
            raise IndexError(f"Invalid index for scalar EF4: {key}")
        result = self._d[key]
        if result.ndim == 1:
            return EF4._wrap(result.copy())
        return EF4._wrap(result)

    def __setitem__(self, key, value):
        if self._d.ndim == 1:
            raise IndexError("Cannot set items on scalar EF4")
        if isinstance(value, EF4):
            self._d[key] = value._d
        else:
            raise TypeError(f"Can only assign EF4, got {type(value)}")

    def to_rows(self) -> list:
        """Convert to list of [c0, c1, c2, c3] coefficient lists."""
        if self._d.ndim == 1:
            return [self._d.tolist()]
        return self._d.tolist()

    def to_list(self) -> list:
        """Convert scalar to [c0, c1, c2, c3] list (backward compat with EF4Coeffs)."""
        if self._d.ndim == 1:
            return self._d.tolist()
        raise ValueError("to_list() only for scalars")

    # --- Arithmetic ---

    def __add__(self, other):
        if isinstance(other, EF4):
            return EF4._wrap((self._d + other._d) % _P)
        return NotImplemented

    def __radd__(self, other):
        if other == 0:  # Support sum()
            return self
        if isinstance(other, EF4):
            return other.__add__(self)
        return NotImplemented

    def __sub__(self, other):
        if isinstance(other, EF4):
            return EF4._wrap((self._d - other._d) % _P)
        return NotImplemented

    def __rsub__(self, other):
        if isinstance(other, EF4):
            return other.__sub__(self)
        return NotImplemented

    def __neg__(self):
        d = self._d
        result = np.where(d == 0, d, _P - d)
        return EF4._wrap(result)

    def __mul__(self, other):
        if isinstance(other, EF4):
            return _ef4_mul(self._d, other._d)
        if isinstance(other, (int, np.integer)):
            return EF4._wrap((self._d * (int(other) % _P)) % _P)
        return NotImplemented

    def __rmul__(self, other):
        if isinstance(other, (int, np.integer)):
            return EF4._wrap((self._d * (int(other) % _P)) % _P)
        return NotImplemented

    def __truediv__(self, other):
        if isinstance(other, EF4):
            return self * other.inv()
        return NotImplemented

    def __pow__(self, exp):
        if exp == -1:
            return self.inv()
        if exp == 0:
            if self._d.ndim == 1:
                return EF4.one()
            return EF4._wrap(np.tile(np.array([1, 0, 0, 0], dtype=np.int64),
                                     (self._d.shape[0], 1)))
        if exp == 1:
            return EF4._wrap(self._d.copy())
        if exp < 0:
            return self.inv() ** (-exp)
        # Square-and-multiply
        result = self ** 0  # identity
        base = EF4._wrap(self._d.copy())
        n = exp
        while n > 0:
            if n & 1:
                result = result * base
            base = base * base
            n >>= 1
        return result

    def inv(self) -> 'EF4':
        """Multiplicative inverse via tower decomposition."""
        return _ef4_inv(self._d)

    def mul_base(self, base_vals) -> 'EF4':
        """Multiply each element by base field values (coefficient-wise scaling).

        More efficient than a * EF4.from_base(b) because it avoids
        the full polynomial multiply (only 4 multiplications vs 16).
        """
        if isinstance(base_vals, np.ndarray):
            b = base_vals.view(np.ndarray).ravel().astype(np.int64) % _P
            if self._d.ndim == 2:
                return EF4._wrap((self._d * b[:, np.newaxis]) % _P)
            return EF4._wrap((self._d * int(b[0])) % _P)
        if isinstance(base_vals, (int, np.integer)):
            return EF4._wrap((self._d * (int(base_vals) % _P)) % _P)
        if isinstance(base_vals, list):
            b = np.array(base_vals, dtype=np.int64) % _P
            if self._d.ndim == 2:
                return EF4._wrap((self._d * b[:, np.newaxis]) % _P)
            return EF4._wrap((self._d * int(b[0])) % _P)
        raise TypeError(f"Cannot mul_base with {type(base_vals)}")

    # --- Column operations ---

    def roll(self, shift: int) -> 'EF4':
        """Circular shift (for columns)."""
        return EF4._wrap(np.roll(self._d, shift, axis=0))

    def cumsum(self) -> 'EF4':
        """Prefix sum (for columns). Safe for height <= 2^27."""
        d = np.cumsum(self._d, axis=0) % _P
        return EF4._wrap(d)

    # --- Comparison ---

    def __eq__(self, other):
        if isinstance(other, EF4):
            return np.array_equal(self._d, other._d)
        if isinstance(other, (list, tuple)):
            return np.array_equal(self._d, np.array(other, dtype=np.int64) % _P)
        return NotImplemented

    def __ne__(self, other):
        eq = self.__eq__(other)
        if eq is NotImplemented:
            return eq
        return not eq

    def __hash__(self):
        if self._d.ndim == 1:
            return hash(tuple(self._d.tolist()))
        raise TypeError("Unhashable: column EF4")

    def __repr__(self):
        if self._d.ndim == 1:
            return f"EF4({self._d.tolist()})"
        return f"EF4(shape=({self._d.shape[0]}, 4))"

    def __bool__(self):
        raise ValueError("Truth value of EF4 is ambiguous. Use == EF4.zero() for comparison.")


def _ef4_mul(a: np.ndarray, b: np.ndarray) -> EF4:
    """Polynomial multiply for EF4 internal arrays.

    Handles all broadcasting: scalar*scalar, scalar*column, column*column.
    """
    W = _W_EXT

    if a.ndim == 1 and b.ndim == 1:
        # Scalar * scalar (Python ints for precision)
        a0, a1, a2, a3 = int(a[0]), int(a[1]), int(a[2]), int(a[3])
        b0, b1, b2, b3 = int(b[0]), int(b[1]), int(b[2]), int(b[3])
        c0 = (a0 * b0 + W * (a1 * b3 + a2 * b2 + a3 * b1)) % _P
        c1 = (a0 * b1 + a1 * b0 + W * (a2 * b3 + a3 * b2)) % _P
        c2 = (a0 * b2 + a1 * b1 + a2 * b0 + W * a3 * b3) % _P
        c3 = (a0 * b3 + a1 * b2 + a2 * b1 + a3 * b0) % _P
        return EF4._wrap(np.array([c0, c1, c2, c3], dtype=np.int64))

    # At least one operand is a column — use vectorized numpy.
    # Use intermediate mod to prevent int64 overflow: (p-1)^2 ~ 2^62, sum of 3 can overflow.
    if a.ndim == 1:
        a0 = np.int64(a[0]); a1 = np.int64(a[1])
        a2 = np.int64(a[2]); a3 = np.int64(a[3])
        b0 = b[:, 0]; b1 = b[:, 1]; b2 = b[:, 2]; b3 = b[:, 3]
    elif b.ndim == 1:
        a0 = a[:, 0]; a1 = a[:, 1]; a2 = a[:, 2]; a3 = a[:, 3]
        b0 = np.int64(b[0]); b1 = np.int64(b[1])
        b2 = np.int64(b[2]); b3 = np.int64(b[3])
    else:
        a0 = a[:, 0]; a1 = a[:, 1]; a2 = a[:, 2]; a3 = a[:, 3]
        b0 = b[:, 0]; b1 = b[:, 1]; b2 = b[:, 2]; b3 = b[:, 3]

    # Reduce each product mod p before summing to avoid int64 overflow
    _m = lambda x, y: (x * y) % _P  # noqa: E731
    c0 = (_m(a0, b0) + W * ((_m(a1, b3) + _m(a2, b2)) % _P + _m(a3, b1)) % _P) % _P
    c1 = ((_m(a0, b1) + _m(a1, b0)) % _P + W * ((_m(a2, b3) + _m(a3, b2)) % _P)) % _P
    c2 = ((_m(a0, b2) + _m(a1, b1)) % _P + (_m(a2, b0) + W * _m(a3, b3)) % _P) % _P
    c3 = ((_m(a0, b3) + _m(a1, b2)) % _P + (_m(a2, b1) + _m(a3, b0)) % _P) % _P

    return EF4._wrap(np.column_stack([c0, c1, c2, c3]))


def _ef4_inv(d: np.ndarray) -> EF4:
    """Inverse via tower decomposition.

    GF(p^4) = GF(p^2)[y]/(y^2 - W) where GF(p^2) = GF(p)[u]/(u^2 - W).
    Element a = a0 + a1*y + a2*u + a3*u*y = (a0 + a2*u) + (a1 + a3*u)*y.
    Norm = a_lo^2 - u*a_hi^2 in GF(p^2), then GF(p) norm, invert, multiply back.
    """
    W = np.int64(_W_EXT)

    if d.ndim == 1:
        # Scalar inverse using Python ints
        a0, a1, a2, a3 = int(d[0]), int(d[1]), int(d[2]), int(d[3])
        # a_lo = a0 + a2*u, a_hi = a1 + a3*u
        # a_lo^2 in GF(p^2)
        a2_0 = (a0 * a0 + _W_EXT * a2 * a2) % _P
        a2_1 = (2 * a0 * a2) % _P
        # a_hi^2 in GF(p^2)
        b2_0 = (a1 * a1 + _W_EXT * a3 * a3) % _P
        b2_1 = (2 * a1 * a3) % _P
        # u * a_hi^2: multiply by u where u^2=W
        b2u_0 = (_W_EXT * b2_1) % _P
        b2u_1 = b2_0
        # norm in GF(p^2)
        d0 = (a2_0 - b2u_0) % _P
        d1 = (a2_1 - b2u_1) % _P
        # GF(p) norm
        norm = (d0 * d0 - _W_EXT * d1 * d1) % _P
        ni = pow(int(norm), _P - 2, _P)
        # d^(-1) in GF(p^2)
        e0 = (d0 * ni) % _P
        e1 = (_P - (d1 * ni) % _P) % _P
        # Multiply back
        r0 = (a0 * e0 + _W_EXT * a2 * e1) % _P
        r1 = (_P - (a1 * e0 + _W_EXT * a3 * e1) % _P) % _P
        r2 = (a0 * e1 + a2 * e0) % _P
        r3 = (_P - (a1 * e1 + a3 * e0) % _P) % _P
        return EF4._wrap(np.array([r0, r1, r2, r3], dtype=np.int64))

    # Column inverse — vectorized with intermediate mod to prevent int64 overflow.
    # W=11, so W*x*y can reach 11*(P-1)^2 ~ 5e19 > 2^63.
    a0 = d[:, 0]; a1 = d[:, 1]; a2 = d[:, 2]; a3 = d[:, 3]
    _m = lambda x, y: (x * y) % _P  # noqa: E731
    # a_lo^2 in GF(p^2): (a0 + a2*u)^2 = (a0^2 + W*a2^2) + 2*a0*a2*u
    a2_0 = (_m(a0, a0) + W * _m(a2, a2)) % _P
    a2_1 = (2 * _m(a0, a2)) % _P
    # a_hi^2 in GF(p^2): (a1 + a3*u)^2 = (a1^2 + W*a3^2) + 2*a1*a3*u
    b2_0 = (_m(a1, a1) + W * _m(a3, a3)) % _P
    b2_1 = (2 * _m(a1, a3)) % _P
    # u * a_hi^2: multiply (b2_0 + b2_1*u) by u where u^2=W
    b2u_0 = (W * b2_1) % _P
    b2u_1 = b2_0
    # norm in GF(p^2)
    d0 = (a2_0 - b2u_0) % _P
    d1 = (a2_1 - b2u_1) % _P
    # GF(p) norm: d0^2 - W*d1^2
    norm = (_m(d0, d0) - W * _m(d1, d1)) % _P
    ni = _modpow_arr(norm, _P - 2)
    # d^(-1) in GF(p^2): (d0*ni, -d1*ni)
    e0 = _m(d0, ni)
    e1 = (_P - _m(d1, ni)) % _P
    # Multiply back: result = conj(a) * d^(-1)
    r0 = (_m(a0, e0) + W * _m(a2, e1)) % _P
    r1 = (_P - (_m(a1, e0) + W * _m(a3, e1)) % _P) % _P
    r2 = (_m(a0, e1) + _m(a2, e0)) % _P
    r3 = (_P - (_m(a1, e1) + _m(a3, e0)) % _P) % _P
    return EF4._wrap(np.column_stack([r0, r1, r2, r3]))


# --- Type Aliases ---

# Keep galois FF4 accessible for tests and backward compat
FF4 = _GaloisFF4
FF4Poly = _GaloisFF4
FFPoly = FF
HashOutput = list[int]

# Semantic type aliases for protocol code
Fe = int                    # Base field element (BabyBear, in [0, p))
EF4Coeffs = list[int]       # Extension field element as [c0,c1,c2,c3] — DEPRECATED, use EF4
Digest = list[int]          # 8-element Poseidon2 digest
MerklePath = list[Digest]   # Merkle opening proof


# --- Galois FF4 / EF4 Conversion (boundary helpers) ---


def ff4_coeffs(elem) -> list[int]:
    """Extract ascending-order coefficients [a0, a1, a2, a3] from FF4 element."""
    val = int(elem)
    return [val % _P, (val // _P) % _P, (val // _P2) % _P, (val // _P3) % _P]


def ff4(coeffs) -> 'FF4':
    """Construct galois FF4 scalar from ascending-order coefficients [a0, a1, a2, a3].

    Also accepts EF4 objects for backward compat.
    """
    if isinstance(coeffs, EF4):
        c = coeffs._d if coeffs._d.ndim == 1 else coeffs._d[0]
        a0, a1, a2, a3 = int(c[0]) % _P, int(c[1]) % _P, int(c[2]) % _P, int(c[3]) % _P
    else:
        a0 = int(coeffs[0]) % _P
        a1 = int(coeffs[1]) % _P
        a2 = int(coeffs[2]) % _P
        a3 = int(coeffs[3]) % _P
    return _GaloisFF4(a0 + a1 * _P + a2 * _P2 + a3 * _P3)


def ff4_from_base(val: int) -> 'FF4':
    """Embed base field element into galois FF4 as (val, 0, 0, 0)."""
    return _GaloisFF4(int(val) % _P)


def ff4_array(c0: list[int], c1: list[int], c2: list[int], c3: list[int]) -> 'FF4':
    """Construct galois FF4 array from parallel coefficient lists."""
    n = len(c0)
    vals = [
        int(c0[k]) % _P
        + (int(c1[k]) % _P) * _P
        + (int(c2[k]) % _P) * _P2
        + (int(c3[k]) % _P) * _P3
        for k in range(n)
    ]
    return _GaloisFF4(vals)


def ff4_from_json(json_arr: list[list[int]]) -> 'FF4':
    """Parse JSON [[c0,c1,c2,c3],...] to galois FF4 array."""
    n = len(json_arr)
    c0 = [json_arr[i][0] for i in range(n)]
    c1 = [json_arr[i][1] for i in range(n)]
    c2 = [json_arr[i][2] for i in range(n)]
    c3 = [json_arr[i][3] for i in range(n)]
    return ff4_array(c0, c1, c2, c3)


def ff4_to_json(arr) -> list[list[int]]:
    """Convert galois FF4 array to JSON [[c0,c1,c2,c3],...] format."""
    return [ff4_coeffs(elem) for elem in arr]


# --- EF4 JSON converters ---


def ef4_from_json(json_arr: list[list[int]]) -> EF4:
    """Parse JSON [[c0,c1,c2,c3],...] to EF4 column."""
    return EF4(np.array(json_arr, dtype=np.int64))


def ef4_to_json(ef4_col: EF4) -> list[list[int]]:
    """Convert EF4 column/scalar to JSON [[c0,c1,c2,c3],...] format."""
    return ef4_col.to_rows()


# --- Deprecated Extension Field Helpers (kept for backward compat) ---


def ef4_from_base(x: int) -> EF4:
    """Embed base field element into extension field as (x, 0, 0, 0).

    DEPRECATED: Use EF4(x) or EF4.from_base(x) instead.
    """
    return EF4(int(x))


def ef4_mul(a, b) -> EF4:
    """Multiply two extension field elements.

    DEPRECATED: Use a * b instead.
    """
    if not isinstance(a, EF4):
        a = EF4(a)
    if not isinstance(b, EF4):
        b = EF4(b)
    return a * b


def ef4_mul_base(a, b: int) -> EF4:
    """Multiply extension field element by a base field element.

    DEPRECATED: Use a.mul_base(b) instead.
    """
    if not isinstance(a, EF4):
        a = EF4(a)
    return a.mul_base(b)


def ef4_add(a, b) -> EF4:
    """Add two extension field elements.

    DEPRECATED: Use a + b instead.
    """
    if not isinstance(a, EF4):
        a = EF4(a)
    if not isinstance(b, EF4):
        b = EF4(b)
    return a + b


def ef4_sub(a, b) -> EF4:
    """Subtract two extension field elements.

    DEPRECATED: Use a - b instead.
    """
    if not isinstance(a, EF4):
        a = EF4(a)
    if not isinstance(b, EF4):
        b = EF4(b)
    return a - b


def ef4_neg(a) -> EF4:
    """Negate an extension field element.

    DEPRECATED: Use -a instead.
    """
    if not isinstance(a, EF4):
        a = EF4(a)
    return -a


def ef4_inv(x) -> EF4:
    """Multiplicative inverse in extension field.

    DEPRECATED: Use x ** -1 instead.
    """
    if not isinstance(x, EF4):
        x = EF4(x)
    return x ** -1


def ef4_div(a, b) -> EF4:
    """Division in extension field: a / b.

    DEPRECATED: Use a / b instead.
    """
    if not isinstance(a, EF4):
        a = EF4(a)
    if not isinstance(b, EF4):
        b = EF4(b)
    return a / b


def ef4_pow(x, n: int) -> EF4:
    """Exponentiation in extension field.

    DEPRECATED: Use x ** n instead.
    """
    if not isinstance(x, EF4):
        x = EF4(x)
    return x ** n


def ef4_exp_power_of_2(x, log_power: int) -> EF4:
    """Compute x^(2^log_power) by repeated squaring.

    DEPRECATED: Use x ** (2 ** log_power) instead.
    """
    if not isinstance(x, EF4):
        x = EF4(x)
    return x ** (2 ** log_power)


# --- Deprecated EF4Vec Wrappers ---

EF4Vec = tuple  # Backward compat type alias


def ef4v_add(a: EF4, b: EF4) -> EF4:
    """DEPRECATED: Use a + b."""
    return a + b


def ef4v_sub(a: EF4, b: EF4) -> EF4:
    """DEPRECATED: Use a - b."""
    return a - b


def ef4v_neg(a: EF4) -> EF4:
    """DEPRECATED: Use -a."""
    return -a


def ef4v_mul(a: EF4, b: EF4) -> EF4:
    """DEPRECATED: Use a * b."""
    return a * b


def ef4v_mul_base(a: EF4, b) -> EF4:
    """DEPRECATED: Use a.mul_base(b)."""
    return a.mul_base(b)


def ef4v_from_base(b) -> EF4:
    """DEPRECATED: Use EF4.from_base(b)."""
    return EF4.from_base(b)


def ef4v_from_scalar(coeffs, n: int) -> EF4:
    """DEPRECATED: Use EF4.broadcast(scalar, n)."""
    if isinstance(coeffs, EF4):
        return EF4.broadcast(coeffs, n)
    return EF4.broadcast(EF4(coeffs), n)


def ef4v_inv(a: EF4) -> EF4:
    """DEPRECATED: Use a.inv() or a ** -1."""
    return a.inv()


def ef4v_mul_scalar(a: EF4, s) -> EF4:
    """DEPRECATED: Use a * s."""
    if not isinstance(s, EF4):
        s = EF4(s)
    return a * s


def ef4v_zeros(n: int) -> EF4:
    """DEPRECATED: Use EF4.zeros(n)."""
    return EF4.zeros(n)


def ef4v_from_rows(rows: list) -> EF4:
    """DEPRECATED: Use EF4.from_rows(rows)."""
    return EF4.from_rows(rows)


def ef4v_to_rows(v: EF4) -> list:
    """DEPRECATED: Use v.to_rows()."""
    return v.to_rows()


def ef4v_roll(v: EF4, shift: int) -> EF4:
    """DEPRECATED: Use v.roll(shift)."""
    return v.roll(shift)


def ef4v_cumsum(v: EF4) -> EF4:
    """DEPRECATED: Use v.cumsum()."""
    return v.cumsum()


# --- Deprecated Base Field Column Helpers ---


def ff_column(data) -> FF:
    """DEPRECATED: Use FF(data) directly."""
    return FF(data)


def ff_zeros(n: int) -> FF:
    """DEPRECATED: Use FF.Zeros(n) directly."""
    return FF.Zeros(n)


def ff_constant(val: int, n: int) -> FF:
    """DEPRECATED: Use FF(np.full(n, val % BABYBEAR_PRIME)) directly."""
    return FF(np.full(n, val % BABYBEAR_PRIME, dtype=np.int64))


def ff_roll(arr, shift: int):
    """DEPRECATED: Use np.roll(arr, shift) directly."""
    return np.roll(arr, shift)


# --- NTT Support ---

# Precomputed roots of unity: W[n] is a primitive 2^n-th root of unity in BabyBear.
_root_27 = pow(GENERATOR, (BABYBEAR_PRIME - 1) >> TWO_ADICITY, BABYBEAR_PRIME)

W: list[int] = [0] * (TWO_ADICITY + 1)
W[TWO_ADICITY] = _root_27
for _k in range(TWO_ADICITY - 1, -1, -1):
    W[_k] = pow(W[_k + 1], 2, BABYBEAR_PRIME)

# Precomputed inverses: W_INV[n] = W[n]^(-1) mod p
W_INV: list[int] = [pow(w, BABYBEAR_PRIME - 2, BABYBEAR_PRIME) if w != 0 else 0 for w in W]


def get_omega(n_bits: int) -> int:
    """Return primitive 2^n_bits-th root of unity."""
    return W[n_bits]


def get_omega_inv(n_bits: int) -> int:
    """Return inverse of primitive 2^n_bits-th root of unity."""
    return W_INV[n_bits]


def inv_mod(x: int) -> int:
    """Multiplicative inverse of x modulo BABYBEAR_PRIME."""
    return pow(x, BABYBEAR_PRIME - 2, BABYBEAR_PRIME)


# --- Bit Reversal Utilities ---


def to_monty(x: int) -> int:
    """Convert a canonical integer to BabyBear Montgomery form."""
    return (x * MONTY_R) % BABYBEAR_PRIME


def from_monty(x: int) -> int:
    """Convert a BabyBear Montgomery-form value to canonical form."""
    return (x * MONTY_RINV) % BABYBEAR_PRIME


def reverse_bits_len(x: int, bit_len: int) -> int:
    """Reverse the lowest bit_len bits of x."""
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
    """
    n = len(values)
    if n == 0:
        return values
    if n == 1:
        return values ** -1

    field_type = type(values)

    cumprods = field_type.Zeros(n)
    cumprods[0] = values[0]
    for i in range(1, n):
        cumprods[i] = cumprods[i - 1] * values[i]

    inv_total = cumprods[n - 1] ** -1

    results = field_type.Zeros(n)
    z = inv_total
    for i in range(n - 1, 0, -1):
        results[i] = z * cumprods[i - 1]
        z = z * values[i]
    results[0] = z
    return results


def ef4_batch_inverse(values: list) -> list:
    """Montgomery batch inversion for EF4 elements.

    Converts N extension field inversions into 3N-3 multiplications + 1 inversion.
    Accepts list of EF4 scalars or list of EF4Coeffs (list[int]).
    """
    n = len(values)
    if n == 0:
        return []

    # Convert to EF4 if needed
    vals = [v if isinstance(v, EF4) else EF4(v) for v in values]

    if n == 1:
        return [vals[0] ** -1]

    cumprods = [None] * n
    cumprods[0] = vals[0]
    for i in range(1, n):
        cumprods[i] = cumprods[i - 1] * vals[i]

    inv_total = cumprods[n - 1] ** -1

    results = [None] * n
    z = inv_total
    for i in range(n - 1, 0, -1):
        results[i] = z * cumprods[i - 1]
        z = z * vals[i]
    results[0] = z
    return results


def batch_inverse_base(values: list) -> list:
    """Montgomery batch inversion in the base field."""
    p = BABYBEAR_PRIME
    n = len(values)
    if n == 0:
        return []
    if n == 1:
        return [inv_mod(values[0])]

    cumprods = [0] * n
    cumprods[0] = values[0] % p
    for i in range(1, n):
        cumprods[i] = (cumprods[i - 1] * values[i]) % p

    inv_total = inv_mod(cumprods[n - 1])

    results = [0] * n
    z = inv_total
    for i in range(n - 1, 0, -1):
        results[i] = (z * cumprods[i - 1]) % p
        z = (z * values[i]) % p
    results[0] = z
    return results


# --- Batch polynomial evaluation at a single EF4 point ---


def _ef4_mul_raw(a, b):
    """Raw EF4 multiply on tuples/lists of ints. Internal use for BSGS."""
    a0, a1, a2, a3 = a
    b0, b1, b2, b3 = b
    c0 = (a0 * b0 + _W_EXT * (a1 * b3 + a2 * b2 + a3 * b1)) % _P
    c1 = (a0 * b1 + a1 * b0 + _W_EXT * (a2 * b3 + a3 * b2)) % _P
    c2 = (a0 * b2 + a1 * b1 + a2 * b0 + _W_EXT * a3 * b3) % _P
    c3 = (a0 * b3 + a1 * b2 + a2 * b1 + a3 * b0) % _P
    return [c0, c1, c2, c3]


def _outer_flat(a: np.ndarray, b: np.ndarray, length: int) -> np.ndarray:
    """Compute outer product, flatten, and truncate to length."""
    return np.outer(a, b).ravel()[:length]


def _outer_mod(a: np.ndarray, b: np.ndarray, length: int) -> np.ndarray:
    """Compute outer product, flatten, truncate, and reduce mod p."""
    return _outer_flat(a, b, length) % _P


def _precompute_z_powers_bsgs(z, degree: int):
    """Precompute z^0..z^{degree-1} as 4 coefficient arrays using BSGS."""
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

    c0 = _outer_mod(big_np[0], small_np[0], degree)
    w_sum = (_outer_flat(big_np[1], small_np[3], degree)
             + _outer_flat(big_np[2], small_np[2], degree)) % _P
    w_sum = (w_sum + _outer_mod(big_np[3], small_np[1], degree)) % _P
    c0 = (c0 + _W_EXT * w_sum) % _P

    t1 = (_outer_flat(big_np[0], small_np[1], degree)
          + _outer_flat(big_np[1], small_np[0], degree)) % _P
    w_sum1 = (_outer_flat(big_np[2], small_np[3], degree)
              + _outer_flat(big_np[3], small_np[2], degree)) % _P
    c1 = (t1 + _W_EXT * w_sum1) % _P

    t2 = (_outer_flat(big_np[0], small_np[2], degree)
          + _outer_flat(big_np[1], small_np[1], degree)) % _P
    t2 = (t2 + _outer_mod(big_np[2], small_np[0], degree)) % _P
    c2 = (t2 + _W_EXT * _outer_mod(big_np[3], small_np[3], degree)) % _P

    t3 = (_outer_flat(big_np[0], small_np[3], degree)
          + _outer_flat(big_np[1], small_np[2], degree)) % _P
    t3_2 = (_outer_flat(big_np[2], small_np[1], degree)
            + _outer_flat(big_np[3], small_np[0], degree)) % _P
    c3 = (t3 + t3_2) % _P

    return (c0, c1, c2, c3)


def eval_poly_ef4_batch(
    coeffs_per_col: list[list[int]],
    eval_point,
) -> list:
    """Evaluate multiple polynomials at a single EF4 point using BSGS.

    Args:
        coeffs_per_col: Polynomial coefficient vectors, all same degree.
        eval_point: EF4 scalar or [c0,c1,c2,c3] list.

    Returns:
        List of EF4 scalars, one per polynomial.
    """
    num_cols = len(coeffs_per_col)
    if num_cols == 0:
        return []
    degree = len(coeffs_per_col[0])
    if degree == 0:
        return [EF4.zero()] * num_cols

    if isinstance(eval_point, EF4):
        z = tuple(int(c) for c in eval_point._d[:4])
    else:
        z = tuple(int(c) for c in eval_point[:4])

    z_powers = _precompute_z_powers_bsgs(z, degree)

    coeff_mat = np.array(coeffs_per_col, dtype=np.int64)

    results = []
    for zp in z_powers:
        products = (coeff_mat * zp) % _P
        result_j = products.sum(axis=1) % _P
        results.append(result_j)

    out = np.stack(results, axis=1)
    return [EF4._wrap(np.array(row, dtype=np.int64)) for row in out]
