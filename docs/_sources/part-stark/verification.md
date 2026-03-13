(sec:verification)=
# Verification Phase

This section describes the verifier's checks: constraint verification at the
OOD point, PCS verification, and FRI verification.

## Overview

The verifier ({src}`protocol.stark.verify_stark`) performs four main checks:

1. **Transcript replay**: Reconstruct the Fiat-Shamir transcript identically
   to the prover (same observations, same challenges).
2. **Constraint check**: Verify that the opened values satisfy the constraint
   equations at $\zeta$.
3. **PCS verification**: Verify that the opened values are consistent with the
   committed polynomials via Merkle proofs and FRI.
4. **LogUp check**: If interactions exist, verify the cumulative sum is zero.

## Constraint Check at OOD Point

For each AIR, the verifier checks that the quotient polynomial $Q(\zeta)$ is
consistent with the constraint evaluations
({src}`protocol.constraints.verify_single_rap_constraints`):

### Step 1: Compute Selectors

Evaluate the domain selectors at the OOD point $\zeta$
({src}`protocol.domain.get_selectors_at_point`):

$$
\text{is\_first\_row}(\zeta), \quad
\text{is\_last\_row}(\zeta), \quad
\text{is\_transition}(\zeta), \quad
Z_H(\zeta)^{-1}
$$

### Step 2: Evaluate Constraint DAG

Walk the symbolic expression DAG in topological order, dispatching on each
node type ({src}`protocol.constraints.eval_symbolic_dag`):

| Node Kind | Computation |
|-----------|-------------|
| `VARIABLE` | Look up opened value from the appropriate trace partition |
| `CONSTANT` | Wrap constant as FF4 |
| `IS_FIRST_ROW` | Fetch selector value |
| `IS_LAST_ROW` | Fetch selector value |
| `IS_TRANSITION` | Fetch selector value |
| `ADD`, `SUB`, `MUL`, `NEG` | Compute from prior DAG results using FF4 operators |

This produces a list of constraint evaluations $C_0(\zeta), C_1(\zeta), \ldots, C_{k-1}(\zeta)$.

### Step 3: Fold Constraints

Combine constraint evaluations using Horner's method with the challenge $\alpha$
({src}`protocol.constraints.fold_constraints`):

$$
\text{folded} = C_0 \cdot \alpha^{k-1} + C_1 \cdot \alpha^{k-2} + \cdots + C_{k-1}
$$

Equivalently: `accumulator = accumulator * alpha + C_i` for each constraint.

### Step 4: Reconstruct Quotient

Reconstruct $Q(\zeta)$ from the opened quotient chunks $Q_0, Q_1, \ldots, Q_{d_Q-1}$.
Each chunk was evaluated at $\zeta$; the full quotient is reconstructed using
the quotient chunk domains
({src}`protocol.constraints.reconstruct_quotient`).

### Step 5: Verify

Assert the quotient relationship holds:

$$
\text{folded} \cdot Z_H(\zeta)^{-1} = Q(\zeta)
$$

If this fails, raise `OodEvaluationMismatch`.

## PCS Verification

The PCS verifier checks that the opened polynomial values are consistent with
the Merkle commitments ({src}`protocol.pcs.pcs_verify`):

### Step A: Observe and Sample

1. Observe all opened values (claimed polynomial evaluations) into the transcript.
2. Sample the batch combination challenge $\alpha_{\text{pcs}}$.

### Step B: Replay FRI Commit Phase

For each FRI commit round:
1. Observe the round commitment.
2. Check the round's PoW witness.
3. Sample the folding challenge $\beta_i$.

Observe the final polynomial coefficients.

### Step C: Verify Queries

For each query (sampled from the transcript):

1. **Compute reduced openings** ({src}`protocol.pcs._open_input`):
   For each batch commitment, at the query index:

   a. Verify the Merkle batch opening proof.

   b. For each matrix in the batch, at each opening point $z$:
      - Compute the evaluation point $x$ on the coset domain from the query
        index (accounting for bit reversal).
      - Accumulate the $\alpha$-weighted quotient:
        $$
        \text{reduced} \mathrel{+}= \alpha^j \cdot \frac{p(z) - p(x)}{z - x}
        $$
        where $p(z)$ is the claimed opened value and $p(x)$ is the Merkle-opened
        value.

   The result is a per-height reduced opening: one FF4 value per distinct
   $\log_2(\text{height})$.

   :::{note}
   For multi-AIR proofs with matrices of different heights, reduced openings
   are tracked **per height** via `alpha_pow_per_height`, not globally.
   This is critical for correctness.
   :::

2. **Verify FRI query** ({src}`protocol.pcs._verify_query`):
   Starting from the reduced opening at the maximum height, fold down through
   each FRI round:

   a. Extract the sibling evaluation from the proof.
   b. Arrange $(e_0, e_1)$ by index parity.
   c. Verify the Merkle proof for the FRI leaf hash of $(e_0, e_1)$.
   d. Fold: $\text{folded} = \Fold(e_0, e_1, \beta_i)$ via Lagrange interpolation
      ({src}`protocol.fri.fold_row`).
   e. Roll in next reduced opening if one exists at the folded height:
      $\text{folded} \mathrel{+}= \beta_i^2 \cdot \text{ro}$.

3. **Check final polynomial**: evaluate the final polynomial at the folded
   domain point and verify it matches the folded evaluation.

## FRI Verification

The FRI verifier ({src}`protocol.fri.verify_fri`) checks:

1. **Commit phase replay**: Observe each commitment, sample folding challenges
   $\beta_0, \beta_1, \ldots$ from the transcript.

2. **Final polynomial**: Observe its coefficients and verify its degree.

3. **Query phase**: For each query index (sampled from the transcript), verify
   structural consistency of the opening proofs (Merkle proof lengths, sibling
   value dimensions).

The actual folding consistency check is performed within the PCS query
verification above.

## LogUp Verification

For multi-AIR proofs with bus interactions, the verifier additionally checks
({src}`protocol.stark._partially_verify_fri_log_up`):

1. **PoW verification**: Check the interaction-phase proof-of-work witness.
2. **Sample interaction challenges**: $\alpha_{\text{lu}}, \beta_{\text{lu}}$
   (identical to prover).
3. **Observe exposed values**: The cumulative sums per AIR.
4. **Observe after-challenge commitment**.
5. **Cumulative sum check**: Verify that the total cumulative sum across all
   AIRs equals zero. A nonzero sum indicates a bus interaction mismatch.

:::{important}
The LogUp error (`ChallengePhaseError`) is raised **only after** the constraint
verification succeeds. This matches the Rust reference implementation's error
precedence: `OodEvaluationMismatch` takes priority over `ChallengePhaseError`.
:::

## Proof-of-Work Checks

The protocol includes two PoW grinding steps:

| PoW | When | Purpose |
|-----|------|---------|
| LogUp PoW | Before interaction challenges | Security parameter for LogUp phase |
| DEEP PoW | After quotient commitment | Security parameter for evaluation phase |

The verifier checks these by cloning the transcript state, observing the witness,
and verifying leading bits are zero
({src}`primitives.transcript.Challenger.check_witness`).
