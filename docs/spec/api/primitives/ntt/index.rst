primitives.ntt
==============

.. py:module:: primitives.ntt

.. autoapi-nested-parse::

   Number Theoretic Transform (NTT) over BabyBear.

   Uses Plonky3's SIMD-optimized Radix2DitParallel DFT via Rust FFI.
   Fallback to galois library if FFI is unavailable.

   Reference:
       p3-dft (Plonky3's DFT implementation for BabyBear).



Functions
---------

.. autoapisummary::

   primitives.ntt.ntt
   primitives.ntt.coset_lde
   primitives.ntt.ef4_idft


Module Contents
---------------

.. py:function:: ntt(coeffs: list[int]) -> list[int]

   Forward NTT: coefficient form -> evaluation form.

   Uses Plonky3's SIMD-optimized Radix2DitParallel DFT (~50x faster than galois).

   Args:
       coeffs: Polynomial coefficients [a0, a1, ..., a_{n-1}].
               Length must be a power of 2.

   Returns:
       Evaluations at the n-th roots of unity.


.. py:function:: coset_lde(evals: list[primitives.field.Fe], shift: primitives.field.Fe, log_blowup: int) -> list[primitives.field.Fe]

   Low-degree extension onto a coset: INTT, zero-pad, coset shift, NTT.

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


.. py:function:: ef4_idft(evals: list[primitives.field.FF4Coeffs]) -> list[primitives.field.FF4Coeffs]

   Inverse DFT for extension field evaluations (channel-wise INTT).

   Decomposes each FF4 element into 4 base-field channels, applies INTT
   to each channel independently, then recombines into FF4 coefficients.

   Reference:
       p3-dft traits.rs (idft_algebra)

   Args:
       evals: Extension field evaluations, each a 4-element list [c0, c1, c2, c3].
              Length must be a power of 2.

   Returns:
       Polynomial coefficients in extension field form.


