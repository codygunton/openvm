(sec:full-protocol)=
# The Full Protocol, Rolled Out

This section presents the complete OpenVM STARK protocol
as a single self-contained reference covering the **multi-AIR** setting.
The proof covers $n$ AIRs, each with its own trace height $N_a$,
constraint set, and optional bus interactions.
All notation follows {ref}`sec:notation`.

```{list-table}
:header-rows: 1
:class: protocol-table

* - **Prover**
  -
  - **Verifier**
* - ***Setup: Transcript seeding*** ({src}`protocol/stark.py#seed-transcript`)
  -
  -
* - Create fresh transcript $\mathcal{T}$ (Poseidon2 duplex sponge).
    <br>$\mathcal{T}.\text{observe}(\text{vk.pre\_hash})$ (8 base field elements).
    <br>$\mathcal{T}.\text{observe}(n)$ — number of AIRs in this proof.
    <br>For $a = 0, \ldots, n-1$: $\mathcal{T}.\text{observe}(\text{air\_id}_a)$.
  -
  - Create fresh $\mathcal{T}$; identical observations ({src}`protocol/stark.py#verify-stark`).
* - ***Round 1: Witness commitment*** ({src}`protocol/stark.py#commit-main-trace`)
  -
  -
* - For each AIR $a$ ($a = 0, \ldots, n-1$):
    <br>Generate witness trace $T_a$ with $N_a$ rows and $m_a$ columns.
    <br>Heights may differ: $N_a \ne N_b$ in general.
    <br>Determine domain: $D_a = \langle \omega_{N_a} \rangle$ ({src}`protocol.domain.natural_domain_for_degree`).
  -
  -
* - **Preprocessed traces** (per-AIR, committed individually):
    <br>For each AIR $a$ with preprocessed data:
    <br>$\quad r_{\text{pre},a} := \MT(\text{prep}_a)$. Verify $r_{\text{pre},a}$ matches VK.
    <br>**Cached main traces** (per-partition, committed individually):
    <br>For each AIR and each cached-main partition:
    <br>$\quad r_{\text{cached}} := \MT(\text{cached partition})$.
    <br>**Common main trace** (all AIRs, one MMCS commitment):
    <br>Collect $\{(D_a, T_a)\}$ for AIRs with common main columns.
    <br>Build mixed-matrix Merkle tree ({src}`protocol/pcs.py#build-mmcs-tree`):
    shorter matrices are injected at higher tree levels.
    <br>$r_1 := \MT(\{T_a\})$ ({src}`protocol/pcs.py#pcs-commit`).
  -
  -
* - $\mathcal{T}.\text{observe}(\text{pv}_a)$ for each AIR $a$.
    <br>$\mathcal{T}.\text{observe}(r_{\text{pre}})$ — flattened preprocessed commitments from VK.
    <br>$\mathcal{T}.\text{observe}(r_{\text{cached}_0}), \ldots, \mathcal{T}.\text{observe}(r_1)$ — all main commitments in order.
    <br>$\mathcal{T}.\text{observe}(\log_2 N_a)$ for each AIR $a$.
  - $=$
  - Identical observations of $\text{pv}_a$, preprocessed commits, main commits, $\log_2 N_a$.
* - ***Round 2: After-challenge trace (LogUp)*** ({src}`protocol/stark.py#commit-after-challenge`)
  -
  - *(only if any AIR has bus interactions)*
* - Grind for LogUp PoW witness $\eta_{\text{lu}}$.
  -
  -
* - $\eta_{\text{lu}}$
  - $\longrightarrow$
  - **Check:** PoW witness valid ({src}`primitives.transcript.Challenger.check_witness`).
* - $(\alpha_{\text{lu}}, \beta_{\text{lu}})$
  - $\longleftarrow$
  - $\alpha_{\text{lu}} \leftarrow \mathcal{T}.\text{sample\_ext}()$, $\beta_{\text{lu}} \leftarrow \mathcal{T}.\text{sample\_ext}()$.
* - For each AIR $a$ with interactions ({src}`protocol/logup.py#compute-logup`):
    <br>For each interaction (bus_id $b$, message fields $m_0, \ldots, m_{k-1}$, count $c$):
    <br>$\quad d = \alpha_{\text{lu}} + m_0 + \beta_{\text{lu}} m_1 + \cdots + \beta_{\text{lu}}^k (\text{b}+1)$.
    <br>Batch invert all denominators; compute chunk sums weighted by counts.
    <br>Running sum $\phi_a$: prefix sum of chunk sums over $N_a$ rows.
    <br>Exposed value $= \phi_a[N_a - 1]$ (cumulative sum for AIR $a$).
  -
  -
* - $\mathcal{T}.\text{observe}(\text{exposed values per AIR})$ (each is FF4: 4 base elements).
    <br>Flatten FF4 columns to base field. Extend each via coset LDE over $D_a$.
    <br>$r_2 := \MT(\text{after-challenge columns for all AIRs})$ ({src}`protocol/pcs.py#pcs-commit`).
    <br>$\mathcal{T}.\text{observe}(r_2)$.
  - $=$
  - Observe exposed values, $r_2$.
    <br>**Check:** $\displaystyle\sum_{a=0}^{n-1} \text{exposed\_value}_a = 0$ — bus sends and receives balance globally.
* - ***Round Q: Quotient polynomial*** ({src}`protocol/stark.py#commit-quotient`)
  -
  -
* - $\alpha$
  - $\longleftarrow$
  - $\alpha \leftarrow \mathcal{T}.\text{sample\_ext}()$ (constraint folding challenge).
* - For each AIR $a$ ({src}`protocol/quotient.py#compute-quotient-chunks`):
    <br>Create disjoint quotient domain of size $N_a \cdot d_{Q,a}$ ({src}`protocol.domain.create_disjoint_domain`).
    <br>Extend all of AIR $a$'s traces (preprocessed, main, after-challenge) to quotient domain.
    <br>Evaluate AIR $a$'s constraints at each quotient domain point.
    <br>$\text{acc}_a(x) = \sum_i \alpha^i C_{a,i}(x)$ (Horner's method).
    <br>$Q_a(x) = \text{acc}_a(x) / Z_{H_a}(x)$ where $Z_{H_a}$ is the vanishing polynomial for $D_a$.
    <br>Split into $d_{Q,a}$ chunks: $Q_{a,0}, Q_{a,1}, \ldots$
  -
  -
* - Collect all quotient chunks across AIRs, each at its own height.
    <br>$r_Q := \MT(\{Q_{a,j}\})$ ({src}`protocol/pcs.py#pcs-commit`) — single MMCS tree over all chunks.
    <br>$\mathcal{T}.\text{observe}(r_Q)$.
  - $=$
  - $\mathcal{T}.\text{observe}(r_Q)$.
* - ***DEEP Proof-of-Work***
  -
  -
* - Grind for DEEP PoW witness $\eta_{\text{deep}}$.
  -
  -
* - $\eta_{\text{deep}}$
  - $\longrightarrow$
  - **Check:** PoW witness valid.
* - ***Evaluation stage*** ({src}`protocol/stark.py#open-at-zeta`)
  -
  -
* - $\zeta$
  - $\longleftarrow$
  - $\zeta \leftarrow \mathcal{T}.\text{sample\_ext}()$ (OOD evaluation point).
* - For each committed matrix, evaluate columns at $\zeta$ and $\zeta\omega_a$
    ({src}`protocol/pcs.py#pcs-open`):
    <br>$e_{p,\text{local}} = p(\zeta)$, $e_{p,\text{next}} = p(\zeta\omega_a)$.
    <br>Note: $\omega_a$ is the trace domain generator for AIR $a$'s height.
    <br>(Quotient chunks: opened at $\zeta$ only, no next.)
  -
  -
* - $\{e_{p,o}\}$
  - $\longrightarrow$
  - $\{e_{p,o}\}$; $\mathcal{T}.\text{observe}(\{e_{p,o}\})$.
* - ***PCS batch opening*** ({src}`protocol/pcs.py#pcs-open`)
  -
  -
* - $\alpha_{\text{pcs}}$
  - $\longleftarrow$
  - $\alpha_{\text{pcs}} \leftarrow \mathcal{T}.\text{sample\_ext}()$ (batch combination challenge).
* - **Reduced polynomial per height** ({src}`protocol/pcs.py#per-height-alpha`):
    <br>Group all committed matrices by $\log_2(\text{height})$.
    <br>For each matrix at height $2^h$, for each opening point $z \in \{\zeta, \zeta\omega_a\}$:
    <br>$\quad \text{reduced}_h \mathrel{+}= \alpha_{\text{pcs},h}^j \cdot \frac{p(z) - p(x)}{z - x}$
    <br>where $x$ ranges over the LDE domain and $j$ is the column index.
    <br>**Critical:** $\alpha_{\text{pcs}}$ powers are tracked **per height** $h$,
    not globally — matrices at different heights have independent power sequences.
    A single global accumulator produces incorrect proofs for multi-height cases.
  -
  -
* - ***FRI commit phase*** ({src}`protocol/fri.py#commit-phase`)
  -
  -
* - Collect reduced polynomials keyed by $\log_2(\text{height})$.
    <br>Set $F_0 = \text{reduced polynomial at max height}$ (bit-reversed evaluations).
  -
  -
* - Pair evaluations as leaves, $r_0^{\text{FRI}} = \MT(F_0)$ ({src}`protocol/fri.py#hash-fri-leaf`).
  - $\longrightarrow$
  - $r_0^{\text{FRI}}$; $\mathcal{T}.\text{observe}(r_0^{\text{FRI}})$.
* - PoW grind per round (if configured).
  - $\longrightarrow$
  -
* - $\beta_0$
  - $\longleftarrow$
  - $\beta_0 \leftarrow \mathcal{T}.\text{sample\_ext}()$.
* - $\Fold$: $F_1 = \Fold(F_0, \beta_0)$ ({src}`protocol/fri.py#fold-matrix`).
    <br>If a reduced opening $\text{ro}_h$ exists at the folded height $h$:
    <br>$\quad F_1 \mathrel{+}= \beta_0^2 \cdot \text{ro}_h$ — "roll in" the shorter matrix's contribution.
    <br>This is how matrices at different heights enter the FRI proof.
  -
  -
* - $\vdots$ (repeat for $K$ rounds; at each fold, roll in any $\text{ro}_h$ at that height)
  -
  - $\vdots$
* - Final polynomial $F_K$ (truncated, bit-reversed, IDFT to coefficients).
  - $\longrightarrow$
  - $F_K$; $\mathcal{T}.\text{observe}(F_K)$.
* - ***Query-phase PoW***
  -
  -
* - PoW witness for query phase.
  - $\longrightarrow$
  - **Check:** PoW valid.
* - ***Query phase*** ({src}`protocol/fri.py#answer-query`)
  -
  -
* - For each query $i = 1, \ldots, Q_{\text{queries}}$:
  - $=$
  - $q_i \leftarrow \mathcal{T}.\text{sample\_bits}(\log_2 N_{\text{ext}})$.
* - Merkle openings at $q_i$ for every committed tree.
    <br>FRI sibling values and Merkle proofs per round.
  - $\longrightarrow$
  - For each query $q$:
* -
  -
  - **Merkle check:** verify opening proofs for all committed trees
    ({src}`protocol/pcs.py#verify-batch-opening`).
    <br>MMCS trees with multiple heights: verify at the appropriate tree level for each matrix.
* -
  -
  - **Reduced opening** ({src}`protocol/pcs.py#reduce-to-fri-input`):
    <br>For each $\log_2(\text{height})$ $h$, compute $\text{reduced}_h$ from opened leaf values
    and claimed evaluations $\{e_{p,o}\}$.
    <br>$\text{reduced}_h = \sum_j \alpha_{\text{pcs},h}^j \cdot \frac{e_{p,o} - p(x_q)}{z - x_q}$
    <br>where $x_q = g \cdot \omega_{\text{ext}}^{\text{rev}(q)}$ (per-height alpha tracking).
* -
  -
  - **FRI fold verification** ({src}`protocol/pcs.py#verify-fri-query`):
    <br>Start with $\text{reduced}_{h_{\max}}$. For each round $k$:
    <br>$(e_0, e_1)$ from sibling + current, ordered by index parity.
    <br>Verify Merkle proof for $\Hash(e_0 \| e_1)$ ({src}`protocol/fri.py#hash-fri-leaf`).
    <br>$\text{folded} = \Fold(e_0, e_1, \beta_k)$ ({src}`protocol/fri.py#fri-fold`).
    <br>Roll in: $\text{folded} \mathrel{+}= \beta_k^2 \cdot \text{ro}_h$ (if reduced opening exists at current height $h$).
* -
  -
  - **Final polynomial check:** evaluate $F_K$ at folded domain point,
    verify $= \text{folded}$.
* - ***Constraint check*** ({src}`protocol/stark.py#verify-constraints`)
  -
  -
* -
  -
  - For each AIR $a$, with domain $D_a$ and constraint set $\{C_{a,i}\}$
    ({src}`protocol.constraints.verify_single_rap_constraints`):
    <br>Compute selectors at $\zeta$ relative to $D_a$: `is_first_row`, `is_last_row`, `is_transition`, $Z_{H_a}(\zeta)^{-1}$
    ({src}`protocol/domain.py#get-selectors`).
    <br>Evaluate AIR $a$'s constraint DAG from $\{e_{p,o}\}$ ({src}`protocol/constraints.py#eval-symbolic-dag`).
    <br>$\text{folded}_a = \alpha^{k-1} C_{a,0}(\zeta) + \cdots + C_{a,k-1}(\zeta)$
    ({src}`protocol/constraints.py#fold-constraints`).
    <br>Reconstruct $Q_a(\zeta)$ from quotient chunk openings
    ({src}`protocol/constraints.py#reconstruct-quotient`).
    <br>**Check:** $\text{folded}_a \cdot Z_{H_a}(\zeta)^{-1} = Q_a(\zeta)$.
* -
  -
  - **Accept** iff all checks pass for all $n$ AIRs.
```

## Proof Structure

$$\pi = \bigl(r_{\text{cached}}, r_1, [r_2], r_Q, \{e_{p,o}\}, \eta_{\text{lu}}, \eta_{\text{deep}}, \{r_k^{\text{FRI}}\}_{k=0}^{K-1}, F_K, \{\text{query proofs}\}\bigr)$$

## Multi-AIR Key Points

**Different heights.**
AIR traces may have different numbers of rows ($N_a \ne N_b$).
This propagates through the entire protocol: domain selectors are per-AIR,
the vanishing polynomial $Z_{H_a}$ is per-AIR, and $\omega_a$ (the next-row
step in the "next" opening) differs per AIR.

**MMCS commitments.**
When multiple matrices of different heights are committed together
(common main trace, quotient chunks), they share a single Merkle tree
via the mixed-matrix commitment scheme ({src}`protocol/pcs.py#build-mmcs-tree`).
Shorter matrices are injected at higher tree levels.

**Per-height alpha tracking.**
During PCS opening, the $\alpha_{\text{pcs}}$ power accumulator is maintained
separately for each distinct $\log_2(\text{height})$
({src}`protocol/pcs.py#per-height-alpha`).
Using a single global accumulator works for single-height proofs but produces
incorrect results for multi-height cases.

**FRI roll-in.**
Reduced openings from shorter matrices enter the FRI polynomial at the
folding round where the FRI domain size matches their height.
At each fold: $F_{k+1} \mathrel{+}= \beta_k^2 \cdot \text{ro}_h$.

**Error precedence.** `OodEvaluationMismatch` (constraint check failure) takes
priority over `ChallengePhaseError` (LogUp cumulative sum nonzero). The verifier
raises the LogUp error only after the constraint check succeeds.
