(sec:commitment-phase)=
# Commitment Phase

This section describes the prover's commitment rounds: transcript seeding,
witness commitment (Stage 1), after-challenge trace via LogUp (Stage 2),
quotient computation (Stage Q), and the evaluation stage.

## Transcript Seeding

Before any commitment, the prover initializes the Fiat-Shamir transcript
({src}`protocol.stark.prove_stark`):

1. Create a fresh `Challenger()`.
2. Observe the verifying key's `pre_hash` (8 base field elements).
3. Observe the number of AIRs as a single field element.
4. Observe each `air_id` individually.

The verifier performs identical seeding
({src}`protocol.stark.verify_stark`).

## Stage 1: Witness Commitment

The prover commits the main execution trace:

1. **Generate witness trace.** For each AIR, produce a matrix of $N$ rows and
   $m$ columns, where each column is a witness polynomial $f_j$ evaluated at
   the trace domain $H$ (program-specific; see Part II).

2. **Commit via PCS.** Extend each column to the LDE domain via coset LDE,
   build an MMCS Merkle tree, and extract the root commitment $r_1$
   ({src}`protocol.pcs.pcs_commit`).

3. **Absorb into transcript.** The prover observes (in order):
   - Public values for each AIR
   - Flattened preprocessed trace commitments (from the verifying key)
   - Main trace commitment $r_1$
   - $\log_2(\text{degree})$ for each AIR

$$
\mathcal{T}.\text{observe}(\text{pv}_1, \ldots, \text{pv}_k, \; r_{\text{pre}}, \; r_1, \; \log_2 N_1, \ldots)
$$

## Stage 2: After-Challenge Trace (LogUp)

If any AIR has bus interactions (lookups, permutations), the prover computes
auxiliary columns using the FRI LogUp protocol.

1. **Grind for LogUp PoW.** Compute a proof-of-work witness for the
   interaction phase.

2. **Sample interaction challenges.** Draw two extension field challenges:
   - $\alpha_{\text{lu}} \leftarrow \mathcal{T}.\text{sample\_ext}()$
   - $\beta_{\text{lu}} \leftarrow \mathcal{T}.\text{sample\_ext}()$

3. **Compute after-challenge trace.** For each AIR with interactions, compute
   the log-derivative auxiliary columns
   ({src}`protocol.logup.compute_after_challenge_trace`):

   a. **Generate beta powers** from $\beta_{\text{lu}}$ for each interaction.

   b. **Evaluate constraint DAG** at all rows to get field values needed by
      the interaction expressions.

   c. **Compute denominators** for each interaction at each row:
      $$
      d_i = \alpha_{\text{lu}} + m_0 + \beta_{\text{lu}} \cdot m_1 + \cdots + \beta_{\text{lu}}^{|m|} \cdot (\text{bus\_id} + 1)
      $$
      where $m_0, m_1, \ldots$ are the interaction message fields.

   d. **Batch invert** all denominators, then compute chunk sums (weighted by
      interaction counts).

   e. **Running sum** $\phi$: prefix sum of chunk sums. The cumulative sum
      (exposed value) must equal zero for a valid trace.

4. **Observe exposed values** (cumulative sums) into the transcript.

5. **Commit after-challenge trace** via PCS. Flatten FF4 columns to 4 base
   field columns each. Observe the commitment into the transcript.

## Stage Q: Quotient Polynomial

The prover computes and commits the quotient polynomial.

1. **Sample $\alpha$** from the transcript: $\alpha \leftarrow \mathcal{T}.\text{sample\_ext}()$.
   This challenge combines all constraint polynomials via random linear combination.

2. **Compute quotient chunks** for each AIR
   ({src}`protocol.quotient.compute_quotient_chunks`):

   a. Create a disjoint quotient domain of size $N \cdot d_Q$, where $d_Q$ is
      the quotient degree.

   b. Extend all traces (main, preprocessed, after-challenge) to the quotient
      domain via coset LDE.

   c. Evaluate all constraints at every point of the quotient domain, accumulate
      with $\alpha$-powers using Horner's method:
      $$
      \text{accumulated}(x) = \sum_{i} \alpha^i \cdot C_i(x)
      $$

   d. Divide by the vanishing polynomial:
      $$
      Q(x) = \frac{\text{accumulated}(x)}{Z_H(x)}
      $$

   e. Split the quotient evaluations into $d_Q$ chunks of $N$ values each,
      producing $d_Q$ separate trace matrices.

3. **Commit quotient chunks** via PCS. Observe the commitment $r_Q$ into the
   transcript.

## DEEP Proof-of-Work

After the quotient commitment, the prover grinds for a DEEP proof-of-work
witness (separate from the LogUp PoW).

## Evaluation Stage

1. **Sample $\zeta$** from the transcript:
   $\zeta \leftarrow \mathcal{T}.\text{sample\_ext}()$.
   This is the out-of-domain (OOD) evaluation point.

2. **Open all polynomials** at $\zeta$ and $\zeta \omega$ (the next-row point).
   For each committed matrix, the prover evaluates its columns at both points
   using polynomial evaluation from coefficients
   ({src}`protocol.pcs.pcs_open`):

   | Round | Matrices | Points |
   |-------|----------|--------|
   | Preprocessed | Per-AIR constant columns | $\zeta$, $\zeta\omega$ |
   | Cached main | Per-partition trace columns | $\zeta$, $\zeta\omega$ |
   | Common main | Shared trace columns | $\zeta$, $\zeta\omega$ |
   | After-challenge | LogUp auxiliary columns | $\zeta$, $\zeta\omega$ |
   | Quotient | Quotient chunks | $\zeta$ only |

3. **Observe opened values** into the transcript (done inside `pcs_open`).

4. **Compute FRI reduced polynomial** and run the FRI protocol to prove the
   opened values are consistent with the commitments
   ({src}`protocol.fri.prove_fri`).
