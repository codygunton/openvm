(app:fri)=
# Appendix: FRI Protocol

## FRI Polynomial Construction

The FRI input polynomial is a batch combination of all committed polynomials.
For each committed matrix at each opening point $z$, the PCS computes the
quotient of $(p(z) - p(x)) / (z - x)$ and accumulates with $\alpha$-powers
({src}`protocol.pcs.pcs_open`).

The result is one reduced polynomial per distinct log-height.

### Per-Height Alpha Tracking

For multi-AIR proofs where committed matrices have different heights, the
$\alpha$-powers are tracked **per height**
({src}`protocol.pcs._open_input`):

```
alpha_pow_per_height: dict[int, FF4]
```

Each log-height maintains its own running $\alpha^j$ counter. This is critical
for multi-height correctness; single-height proofs work either way.

## Folding Formula

Each FRI round halves the polynomial degree. Given evaluations $F_k$ of degree
$< 2^{b_k}$, the fold produces $F_{k+1}$ of degree $< 2^{b_k - 1}$.

**Verifier fold** ({src}`protocol.fri.fold_row`):

Given two evaluations $(e_0, e_1)$ at conjugate points $(\omega^j, -\omega^j)$
and challenge $\beta$:

$$
F_{k+1}(\omega^{2j}) = \frac{e_0 + e_1}{2} + \beta \cdot \frac{e_0 - e_1}{2\omega^j}
$$

Equivalently, as Lagrange interpolation at $\beta$:

$$
F_{k+1} = e_0 + (\beta - \omega^j) \cdot \frac{e_1 - e_0}{-2\omega^j}
$$

**Prover fold** ({src}`protocol.fri.fold_matrix`):

The prover operates on bit-reversed evaluations. For the full matrix:

1. Split into even-indexed (lo) and odd-indexed (hi) rows.
2. Precompute halved inverse powers: $h_i = \omega_k^{-i} / 2$ (then bit-reverse).
3. Fold vectorized: $\text{result}_i = (lo_i + hi_i) / 2 + \beta \cdot (lo_i - hi_i) \cdot h_i$.

## Commit Phase

The commit phase iteratively folds and commits
({src}`protocol.fri.commit_phase`):

1. Set $F_0 = $ initial evaluations (bit-reversed).
2. While $|F_k| > \text{blowup} \cdot \text{final\_poly\_len}$:
   a. Pair adjacent evaluations as Merkle leaves.
   b. Build Merkle tree, extract root $r_k^{\text{FRI}}$.
   c. Observe $r_k^{\text{FRI}}$ into transcript.
   d. Grind for PoW (if configured).
   e. Sample $\beta_k \leftarrow \mathcal{T}.\text{sample\_ext}()$.
   f. Fold: $F_{k+1} = \Fold(F_k, \beta_k)$.
   g. Roll in reduced openings at folded height (if any):
      $F_{k+1} \mathrel{+}= \beta_k^2 \cdot \text{ro}$.
3. Compute final polynomial: truncate, bit-reverse, IDFT to coefficients.
4. Observe final polynomial coefficients.

## Query Phase

For each query ({src}`protocol.fri.answer_query`):

1. Derive query index $q$ from transcript.
2. For each FRI round $k$:
   a. Compute sibling index: $q_k^{\text{sib}} = q_k \oplus 1$.
   b. Compute pair index: $q_k^{\text{pair}} = q_k \gg 1$.
   c. Extract sibling evaluation from round $k$ evaluations.
   d. Get Merkle proof at pair index.
   e. $q_{k+1} = q_k \gg 1$.

The verifier checks each query by recomputing the fold and verifying Merkle
proofs (see {ref}`sec:verification`).

## FRI Parameters

| Parameter | Symbol | Description | Python |
|-----------|--------|-------------|--------|
| Log blowup | $\log_2 \beta$ | Reed-Solomon rate parameter | `fri_params.log_blowup` |
| Num queries | $Q_{\text{queries}}$ | Number of random query repetitions | `fri_params.num_queries` |
| PoW bits | $b_{\text{pow}}$ | Proof-of-work difficulty per commit round | `fri_params.proof_of_work_bits` |
| Final poly len | $|F_K|$ | Length of final polynomial (log) | `fri_params.log_final_poly_len` |

The security level depends on all four parameters. Increasing `num_queries`
or `proof_of_work_bits` increases security at the cost of proof size or
prover time, respectively.

## FRI Leaf Hashing

A FRI Merkle leaf is the hash of two adjacent extension field evaluations
({src}`protocol.fri.hash_fri_leaf`):

$$
\text{leaf} = \Poseidon(e_0[0], e_0[1], e_0[2], e_0[3], e_1[0], e_1[1], e_1[2], e_1[3])
$$

where $e_0, e_1 \in \Fext$ are represented as 4 base field elements each.
