(sec:building-blocks)=
# Building Blocks

## Witness Generation

In Part I, witness generation is treated as a black box.
This part focuses on the STARK protocol parameterized by the AIR
(Algebraic Intermediate Representation), which defines the constraint system
and execution trace structure.

Part II (Programs) covers concrete witness generation for Fibonacci (single-AIR)
and multi-AIR (RV32IM) programs.

(sec:poseidon2)=
## Poseidon2 Hash

The hash function is Poseidon2 over BabyBear with width $w = 16$
({src}`primitives.poseidon2.permute`).

**Permutation.** The core primitive is a permutation $\pi : \F^{16} \to \F^{16}$
that applies the Poseidon2 round function to a 16-element state vector.

**Compression.** Two 8-element digests are compressed into one by concatenating
them into a 16-element vector, applying $\pi$, and truncating to the first 8
elements ({src}`primitives.poseidon2.compress`):

$$
\text{compress}(L, R) = \pi(L \| R)[0{:}8].
$$

**Leaf hashing.** Input data is hashed to an 8-element digest using a
padding-free sponge construction ({src}`primitives.poseidon2.hash_to_digest`).

Both compression and hashing support batch operations parallelized via rayon
({src}`primitives.poseidon2.compress_batch`, {src}`primitives.poseidon2.hash_batch`).

(sec:merkle)=
## Merkle Tree

A binary Merkle tree commits a list of leaves to a single 8-element digest root.

**Build.** Given $n$ leaves (each a list of field elements):
1. Hash each leaf to a digest: $d_i = \Hash(\ell_i)$ (batch-parallel)
2. Build the tree bottom-up: each internal node is
   $\text{compress}(d_{\text{left}}, d_{\text{right}})$ (batch-parallel)
3. The root is the single digest at the top level

({src}`primitives.merkle.build_merkle_tree`)

**Opening proof.** To prove that leaf $j$ is in the tree, the prover reveals
the sibling digests along the path from leaf to root
({src}`primitives.merkle.get_opening_proof`).

**Verification.** The verifier recomputes the root by walking the path:
at each level, compress the current digest with the sibling (ordering determined
by the index bit), and check that the final result matches the committed root
({src}`primitives.merkle.verify_opening_prehashed`).

(sec:transcript)=
## Fiat-Shamir Transcript

The transcript is a Poseidon2 duplex sponge matching Plonky3's
`DuplexChallenger<BabyBear, Poseidon2<16>, 16, 8>`
({src}`primitives.transcript.Challenger`).

The sponge state is 16 field elements, partitioned into:

```
┌─────────────────────────┬─────────────────────────┐
│     Rate (8 elements)   │   Capacity (8 elements) │
└─────────────────────────┴─────────────────────────┘
```

**Operations.**

- $\mathcal{T}.\text{observe}(v)$:
  Absorb a single field element into the rate portion.
  When the rate buffer is full (8 elements), apply the Poseidon2 permutation
  ({src}`primitives.transcript.Challenger.observe`).

- $\mathcal{T}.\text{observe\_many}(v_1, \ldots, v_k)$:
  Absorb multiple elements. Also accepts an `FF4` scalar (absorbs its 4 coefficients)
  ({src}`primitives.transcript.Challenger.observe_many`).

- $\mathcal{T}.\text{sample}() \to c \in \F$:
  Squeeze one base field element from the sponge output.
  Clears the input buffer and applies a permutation if the output buffer is empty
  ({src}`primitives.transcript.Challenger.sample`).

- $\mathcal{T}.\text{sample\_ext}() \to c \in \Fext$:
  Squeeze four base field elements and interpret as an extension field element
  $c_0 + c_1 x + c_2 x^2 + c_3 x^3$
  ({src}`primitives.transcript.Challenger.sample_ext`).

- $\mathcal{T}.\text{sample\_bits}(b) \to n \in [0, 2^b)$:
  Squeeze base field elements and extract $b$ bits
  ({src}`primitives.transcript.Challenger.sample_bits`).

- $\mathcal{T}.\text{check\_witness}(w)$:
  Verify a proof-of-work witness by sampling bits and checking the leading bits
  are zero ({src}`primitives.transcript.Challenger.check_witness`).

- $\mathcal{T}.\text{grind}(b) \to w$:
  Find a PoW witness by brute-force trial (prover only)
  ({src}`primitives.transcript.Challenger.grind`).

(sec:ntt)=
## NTT and Coset LDE

**Forward NTT.** Transforms polynomial coefficients to evaluations at roots of
unity ({src}`primitives.ntt.ntt`):

$$
\hat{f}_i = \sum_{j=0}^{N-1} f_j \cdot \omega^{ij}, \qquad i = 0, \ldots, N-1.
$$

**Inverse NTT.** Recovers coefficients from evaluations ({src}`primitives.ntt.intt`):

$$
f_j = \frac{1}{N} \sum_{i=0}^{N-1} \hat{f}_i \cdot \omega^{-ij}.
$$

**Coset LDE (Low-Degree Extension).** Extends a polynomial from its trace
domain to a larger evaluation domain for commitment
({src}`primitives.ntt.coset_lde_batch`):

1. Compute coefficients: $\mathbf{c} = \INTT_N(\mathbf{f})$
2. Zero-pad to $N_{\text{ext}}$ coefficients
3. Apply coset shift: $c_i \leftarrow c_i \cdot s^i$ (where $s$ is the LDE shift)
4. Evaluate: $\tilde{\mathbf{f}} = \NTT_{N_{\text{ext}}}(\mathbf{c})$

The LDE shift is computed as $s_{\text{lde}} = g / s_{\text{trace}}$ where
$g = 31$ is the multiplicative generator and $s_{\text{trace}}$ is the trace
domain shift.

All NTT operations use Plonky3's SIMD-optimized Radix2DitParallel DFT via Rust
FFI for performance.

(sec:poly-eval)=
## Polynomial Evaluation

**Multi-point evaluation** of a polynomial (given in coefficient form) at
multiple extension field points uses the Baby-Step Giant-Step (BSGS) optimization
for efficiency ({src}`primitives.field.eval_poly_ef4_batch`).

This is used during the prover's evaluation stage to open polynomials at the OOD
point $\zeta$ and at $\zeta \omega$ (the next-row point).
