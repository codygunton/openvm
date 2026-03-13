(sec:multi-air)=
# Multi-AIR (RV32IM)

The RV32IM Fibonacci program demonstrates the full multi-AIR STARK protocol
with LogUp bus interactions, matrices of different heights, and per-height
alpha tracking.

## Multi-AIR Structure

Unlike the single-AIR Fibonacci, a multi-AIR proof covers multiple AIRs
simultaneously. Each AIR has its own:

- Trace matrix (possibly different number of rows)
- Constraint DAG
- Preprocessed trace (optional)
- Bus interactions (optional)

All AIRs share a single FRI proof.

## Different Heights

In a multi-AIR proof, trace matrices can have different heights ($N_a \ne N_b$).
This affects the protocol in several ways:

1. **MMCS**: The mixed-matrix commitment scheme
   ({src}`protocol.pcs._build_mmcs_tree`) builds a single Merkle tree
   where shorter matrices are injected at higher tree levels.

2. **Per-height alpha tracking**: During PCS opening, the $\alpha$-power
   accumulator is maintained separately for each distinct $\log_2(\text{height})$:
   ```
   alpha_pow_per_height: dict[int, FF4]
   ```
   This is critical — using a single global accumulator produces incorrect
   proofs for multi-height cases, even though it works for single-height proofs.

3. **FRI reduced openings**: The FRI commit phase receives reduced openings
   keyed by height. During folding, reduced openings at the current folded
   height are "rolled in": $F_{k+1} \mathrel{+}= \beta_k^2 \cdot \text{ro}_h$.

## LogUp Bus Interactions

Multi-AIR programs use LogUp to enforce consistency between AIRs via shared
buses ({src}`protocol.logup.compute_after_challenge_trace`).

### Interaction Definition

Each interaction specifies:
- **Bus ID**: which bus this interaction connects to.
- **Message fields**: column indices defining the message.
- **Count**: the multiplicity (positive for sends, negative for receives).

### After-Challenge Computation

1. **Beta powers**: $\beta^0, \beta^1, \ldots$ generated from the interaction
   challenge $\beta_{\text{lu}}$.

2. **Denominators**: For each interaction at each row:
   $$
   d = \alpha_{\text{lu}} + m_0 + \beta_{\text{lu}} \cdot m_1 + \cdots + \beta_{\text{lu}}^{|m|} \cdot (\text{bus\_id} + 1)
   $$

3. **Reciprocals**: Batch invert all denominators.

4. **Chunk sums**: Weighted sum of reciprocals by interaction counts.

5. **Running sum $\phi$**: Prefix sum of chunk sums. The final value
   $\phi[N-1]$ is the *exposed value* (cumulative sum).

### Cumulative Sum Check

The sum of exposed values across all AIRs must equal zero:

$$
\sum_{a \in \text{AIRs}} \phi_a[N_a - 1] = 0
$$

This enforces that every bus send has a corresponding receive.

## Test Vectors

The `rv32im_fibonacci` test vector exercises:
- Multiple AIRs with different trace heights
- LogUp interactions across AIRs
- Per-height alpha tracking in PCS
- Multi-height MMCS tree construction

The test suite caches the proof to avoid the ~25-minute generation time.
Both prover (byte-for-byte match) and verifier tests pass.
