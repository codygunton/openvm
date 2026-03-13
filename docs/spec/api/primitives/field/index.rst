primitives.field
================

.. py:module:: primitives.field

.. autoapi-nested-parse::

   BabyBear field GF(p) and quartic extension GF(p^4).

   Uses galois library for base field GF(p) arithmetic.
   Uses custom FF4 class for fast quartic extension field arithmetic.

   Type Discipline
   ---------------
   FF (galois GF(p)):
       Base field columns and scalars. Fast jit-compiled numpy operations.

   FF4:
       Extension field GF(p^4) scalars and columns. Stores coefficients as int64 arrays.
       Supports operator overloads: +, -, *, /, **, negation.
       Scalars: shape (4,). Columns: shape (N, 4).
       Coefficients in ascending degree: c0 + c1*x + c2*x^2 + c3*x^3 as [c0, c1, c2, c3].



Attributes
----------

.. autoapisummary::

   primitives.field.BABYBEAR_PRIME
   primitives.field.FIELD_EXTENSION_DEGREE
   primitives.field.DIGEST_WIDTH
   primitives.field.TWO_ADICITY
   primitives.field.GENERATOR
   primitives.field.TWO_INV
   primitives.field.MONTY_R
   primitives.field.MONTY_RINV
   primitives.field.FF
   primitives.field.GaloisFF4
   primitives.field.GaloisFF4Poly
   primitives.field.FFPoly
   primitives.field.HashOutput
   primitives.field.Fe
   primitives.field.FF4Coeffs
   primitives.field.Digest
   primitives.field.MerklePath
   primitives.field.FF4Vec
   primitives.field.W
   primitives.field.W_INV


Classes
-------

.. autoapisummary::

   primitives.field.FF4


Functions
---------

.. autoapisummary::

   primitives.field.ff4_coeffs
   primitives.field.ff4
   primitives.field.ff4_from_base
   primitives.field.ff4_array
   primitives.field.ff4_from_json
   primitives.field.ff4_to_json
   primitives.field.ef4_from_json
   primitives.field.ef4_to_json
   primitives.field.ef4_from_base
   primitives.field.ef4_mul
   primitives.field.ef4_mul_base
   primitives.field.ef4_add
   primitives.field.ef4_sub
   primitives.field.ef4_neg
   primitives.field.ef4_inv
   primitives.field.ef4_div
   primitives.field.ef4_pow
   primitives.field.ef4_exp_power_of_2
   primitives.field.ef4v_add
   primitives.field.ef4v_sub
   primitives.field.ef4v_neg
   primitives.field.ef4v_mul
   primitives.field.ef4v_mul_base
   primitives.field.ef4v_from_base
   primitives.field.ef4v_from_scalar
   primitives.field.ef4v_inv
   primitives.field.ef4v_mul_scalar
   primitives.field.ef4v_zeros
   primitives.field.ef4v_from_rows
   primitives.field.ef4v_to_rows
   primitives.field.ef4v_roll
   primitives.field.ef4v_cumsum
   primitives.field.ff_column
   primitives.field.ff_zeros
   primitives.field.ff_constant
   primitives.field.ff_roll
   primitives.field.get_omega
   primitives.field.get_omega_inv
   primitives.field.inv_mod
   primitives.field.to_monty
   primitives.field.from_monty
   primitives.field.reverse_bits_len
   primitives.field.bit_reverse_list
   primitives.field.batch_inverse
   primitives.field.ef4_batch_inverse
   primitives.field.batch_inverse_base
   primitives.field.eval_poly_ef4_batch


Module Contents
---------------

.. py:data:: BABYBEAR_PRIME
   :value: 2013265921


.. py:data:: FIELD_EXTENSION_DEGREE
   :value: 4


.. py:data:: DIGEST_WIDTH
   :value: 8


.. py:data:: TWO_ADICITY
   :value: 27


.. py:data:: GENERATOR
   :value: 31


.. py:data:: TWO_INV

.. py:data:: MONTY_R

.. py:data:: MONTY_RINV

.. py:data:: FF
   :value: None


   Base field GF(p) - BabyBear prime field.


.. py:class:: FF4(data=None)

   Extension field GF(p^4) = GF(p)[x]/(x^4 - 11) element or column.

   Internal storage: int64 ndarray of shape (4,) for scalar or (N, 4) for column.
   Coefficients in ascending degree: c0 + c1*x + c2*x^2 + c3*x^3 stored as [c0, c1, c2, c3].

   Supports operator overloads for clean mathematical notation:
       c = a + b          # addition
       c = a - b          # subtraction
       c = a * b          # polynomial multiplication
       c = a / b          # division (multiply by inverse)
       c = a ** n         # exponentiation (n can be -1)
       c = -a             # negation


   .. py:method:: zeros(n: int) -> FF4
      :classmethod:


      Zero column of length n.



   .. py:method:: one() -> FF4
      :classmethod:


      Multiplicative identity scalar.



   .. py:method:: zero() -> FF4
      :classmethod:


      Additive identity scalar.



   .. py:method:: from_base(base) -> FF4
      :classmethod:


      Lift base field values to FF4 as (val, 0, 0, 0).

      Args:
          base: int, FF array, list[int], or ndarray of base field elements.



   .. py:method:: from_rows(rows: list) -> FF4
      :classmethod:


      Create from list of [c0, c1, c2, c3] coefficient lists.



   .. py:method:: broadcast(scalar: FF4, n: int) -> FF4
      :classmethod:


      Broadcast a scalar FF4 to a column of length n.



   .. py:property:: is_scalar
      :type: bool



   .. py:property:: coeffs
      :type: tuple


      Return coefficients as tuple (c0, c1, c2, c3) for a scalar.



   .. py:property:: c0
      :type: int


      Constant coefficient (base field component).



   .. py:method:: to_rows() -> list

      Convert to list of [c0, c1, c2, c3] coefficient lists.



   .. py:method:: to_list() -> list

      Convert scalar to [c0, c1, c2, c3] list (backward compat with FF4Coeffs).



   .. py:method:: inv() -> FF4

      Multiplicative inverse via tower decomposition.



   .. py:method:: mul_base(base_vals) -> FF4

      Multiply each element by base field values (coefficient-wise scaling).

      More efficient than a * FF4.from_base(b) because it avoids
      the full polynomial multiply (only 4 multiplications vs 16).



   .. py:method:: roll(shift: int) -> FF4

      Circular shift (for columns).



   .. py:method:: cumsum() -> FF4

      Prefix sum (for columns). Safe for height <= 2^27.



.. py:data:: GaloisFF4

.. py:data:: GaloisFF4Poly

.. py:data:: FFPoly
   :value: None


.. py:data:: HashOutput

.. py:data:: Fe

.. py:data:: FF4Coeffs

.. py:data:: Digest

.. py:data:: MerklePath

.. py:function:: ff4_coeffs(elem) -> list[int]

   Extract ascending-order coefficients [a0, a1, a2, a3] from FF4 element.


.. py:function:: ff4(coeffs) -> GaloisFF4

   Construct galois FF4 scalar from ascending-order coefficients [a0, a1, a2, a3].

   Also accepts FF4 objects for backward compat.


.. py:function:: ff4_from_base(val: int) -> GaloisFF4

   Embed base field element into galois FF4 as (val, 0, 0, 0).


.. py:function:: ff4_array(c0: list[int], c1: list[int], c2: list[int], c3: list[int]) -> GaloisFF4

   Construct galois FF4 array from parallel coefficient lists.


.. py:function:: ff4_from_json(json_arr: list[list[int]]) -> GaloisFF4

   Parse JSON [[c0,c1,c2,c3],...] to galois FF4 array.


.. py:function:: ff4_to_json(arr) -> list[list[int]]

   Convert galois FF4 array to JSON [[c0,c1,c2,c3],...] format.


.. py:function:: ef4_from_json(json_arr: list[list[int]]) -> FF4

   Parse JSON [[c0,c1,c2,c3],...] to FF4 column.


.. py:function:: ef4_to_json(ef4_col: FF4) -> list[list[int]]

   Convert FF4 column/scalar to JSON [[c0,c1,c2,c3],...] format.


.. py:function:: ef4_from_base(x: int) -> FF4

   Embed base field element into extension field as (x, 0, 0, 0).

   DEPRECATED: Use FF4(x) or FF4.from_base(x) instead.


.. py:function:: ef4_mul(a, b) -> FF4

   Multiply two extension field elements.

   DEPRECATED: Use a * b instead.


.. py:function:: ef4_mul_base(a, b: int) -> FF4

   Multiply extension field element by a base field element.

   DEPRECATED: Use a.mul_base(b) instead.


.. py:function:: ef4_add(a, b) -> FF4

   Add two extension field elements.

   DEPRECATED: Use a + b instead.


.. py:function:: ef4_sub(a, b) -> FF4

   Subtract two extension field elements.

   DEPRECATED: Use a - b instead.


.. py:function:: ef4_neg(a) -> FF4

   Negate an extension field element.

   DEPRECATED: Use -a instead.


.. py:function:: ef4_inv(x) -> FF4

   Multiplicative inverse in extension field.

   DEPRECATED: Use x ** -1 instead.


.. py:function:: ef4_div(a, b) -> FF4

   Division in extension field: a / b.

   DEPRECATED: Use a / b instead.


.. py:function:: ef4_pow(x, n: int) -> FF4

   Exponentiation in extension field.

   DEPRECATED: Use x ** n instead.


.. py:function:: ef4_exp_power_of_2(x, log_power: int) -> FF4

   Compute x^(2^log_power) by repeated squaring.

   DEPRECATED: Use x ** (2 ** log_power) instead.


.. py:data:: FF4Vec

.. py:function:: ef4v_add(a: FF4, b: FF4) -> FF4

   DEPRECATED: Use a + b.


.. py:function:: ef4v_sub(a: FF4, b: FF4) -> FF4

   DEPRECATED: Use a - b.


.. py:function:: ef4v_neg(a: FF4) -> FF4

   DEPRECATED: Use -a.


.. py:function:: ef4v_mul(a: FF4, b: FF4) -> FF4

   DEPRECATED: Use a * b.


.. py:function:: ef4v_mul_base(a: FF4, b) -> FF4

   DEPRECATED: Use a.mul_base(b).


.. py:function:: ef4v_from_base(b) -> FF4

   DEPRECATED: Use FF4.from_base(b).


.. py:function:: ef4v_from_scalar(coeffs, n: int) -> FF4

   DEPRECATED: Use FF4.broadcast(scalar, n).


.. py:function:: ef4v_inv(a: FF4) -> FF4

   DEPRECATED: Use a.inv() or a ** -1.


.. py:function:: ef4v_mul_scalar(a: FF4, s) -> FF4

   DEPRECATED: Use a * s.


.. py:function:: ef4v_zeros(n: int) -> FF4

   DEPRECATED: Use FF4.zeros(n).


.. py:function:: ef4v_from_rows(rows: list) -> FF4

   DEPRECATED: Use FF4.from_rows(rows).


.. py:function:: ef4v_to_rows(v: FF4) -> list

   DEPRECATED: Use v.to_rows().


.. py:function:: ef4v_roll(v: FF4, shift: int) -> FF4

   DEPRECATED: Use v.roll(shift).


.. py:function:: ef4v_cumsum(v: FF4) -> FF4

   DEPRECATED: Use v.cumsum().


.. py:function:: ff_column(data) -> FF

   DEPRECATED: Use FF(data) directly.


.. py:function:: ff_zeros(n: int) -> FF

   DEPRECATED: Use FF.Zeros(n) directly.


.. py:function:: ff_constant(val: int, n: int) -> FF

   DEPRECATED: Use FF(np.full(n, val % BABYBEAR_PRIME)) directly.


.. py:function:: ff_roll(arr, shift: int)

   DEPRECATED: Use np.roll(arr, shift) directly.


.. py:data:: W
   :type:  list[int]
   :value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]


.. py:data:: W_INV
   :type:  list[int]

.. py:function:: get_omega(n_bits: int) -> int

   Return primitive 2^n_bits-th root of unity.


.. py:function:: get_omega_inv(n_bits: int) -> int

   Return inverse of primitive 2^n_bits-th root of unity.


.. py:function:: inv_mod(x: int) -> int

   Multiplicative inverse of x modulo BABYBEAR_PRIME.


.. py:function:: to_monty(x: int) -> int

   Convert a canonical integer to BabyBear Montgomery form.


.. py:function:: from_monty(x: int) -> int

   Convert a BabyBear Montgomery-form value to canonical form.


.. py:function:: reverse_bits_len(x: int, bit_len: int) -> int

   Reverse the lowest bit_len bits of x.


.. py:function:: bit_reverse_list(lst: list) -> list

   Reorder list elements by bit-reversing their indices.


.. py:function:: batch_inverse(values)

   Montgomery batch inversion for any galois array.

   Converts N field inversions into 3N-3 multiplications + 1 inversion.


.. py:function:: ef4_batch_inverse(values: list) -> list

   Montgomery batch inversion for FF4 elements.

   Converts N extension field inversions into 3N-3 multiplications + 1 inversion.
   Accepts list of FF4 scalars or list of FF4Coeffs (list[int]).


.. py:function:: batch_inverse_base(values: list) -> list

   Montgomery batch inversion in the base field.


.. py:function:: eval_poly_ef4_batch(coeffs_per_col: list[list[int]], eval_point) -> list

   Evaluate multiple polynomials at a single FF4 point using BSGS.

   Args:
       coeffs_per_col: Polynomial coefficient vectors, all same degree.
       eval_point: FF4 scalar or [c0,c1,c2,c3] list.

   Returns:
       List of FF4 scalars, one per polynomial.


