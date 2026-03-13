(sec:glossary)=
# Glossary of Notation

## Fields and Domains

| Symbol | Meaning | Python Name |
|--------|---------|-------------|
| $p = 2^{31} - 2^{27} + 1$ | BabyBear prime | `BABYBEAR_PRIME` |
| $\F$ | Base field $\text{GF}(p)$ | `FF` |
| $\Fext$ | Quartic extension $\text{GF}(p^4)$, $x^4 = 11$ | `FF4` |
| $g = 31$ | Multiplicative generator of $\F^*$ | `GENERATOR` |
| $N = 2^n$ | Trace size (rows per AIR) | `domain.size()` |
| $\omega$ | Primitive $N$-th root of unity | `get_omega(log_n)` |
| $H$ | Trace domain $\{s \cdot \omega^i\}$ | `TwoAdicMultiplicativeCoset` |
| $s$ | Coset shift | `domain.shift` |
| $H^*$ | Extended (LDE) domain | coset of size $N \cdot \beta$ |
| $\beta$ | Blowup factor $2^{\text{log\_blowup}}$ | `fri_params.log_blowup` |

## Vanishing and Selectors

| Symbol | Meaning | Python Name |
|--------|---------|-------------|
| $Z_H(X) = (X/s)^N - 1$ | Vanishing polynomial | `domain.zp_at_point(x)` |
| $Z_H(\zeta)^{-1}$ | Inverse vanishing at OOD point | `selectors.inv_zeroifier` |
| $L_0(\zeta)$ | First-row selector | `selectors.is_first_row` |
| $L_{N-1}(\zeta)$ | Last-row selector | `selectors.is_last_row` |
| $\zeta/s - \omega^{-1}$ | Transition selector | `selectors.is_transition` |

## Challenges

| Symbol | Meaning | Python Name |
|--------|---------|-------------|
| $\alpha_{\text{lu}}, \beta_{\text{lu}}$ | LogUp interaction challenges (FF4) | `alpha_lu`, `beta_lu` |
| $\alpha$ | Constraint folding challenge (FF4) | `alpha` |
| $\zeta$ | OOD evaluation point (FF4) | `zeta` |
| $\alpha_{\text{pcs}}$ | PCS batch combination challenge (FF4) | `alpha` in `pcs_open`/`pcs_verify` |
| $\beta_k$ | FRI fold challenge for round $k$ (FF4) | `betas[k]` |

## Polynomials

| Symbol | Meaning | Python Name |
|--------|---------|-------------|
| $f_1, \ldots, f_m$ | Witness polynomials (main trace) | columns of main trace matrix |
| $h_1, \ldots, h_k$ | After-challenge polynomials (LogUp) | `after_challenge_trace` |
| $Q_0, \ldots, Q_{d_Q-1}$ | Quotient polynomial chunks | `quotient_chunks` |
| $F$ | FRI input (reduced polynomial) | `reduced_evals` in `pcs_open` |
| $F_K$ | Final FRI polynomial (coefficients) | `final_poly` |
| $C_i$ | Constraint polynomial $i$ | result of `eval_symbolic_dag` |

## Commitments

| Symbol | Meaning | Python Name |
|--------|---------|-------------|
| $r_{\text{pre}}$ | Preprocessed trace commitment(s) | `vk.preprocessed_commit` |
| $r_1$ | Main trace commitment | `commitments.main_trace` |
| $r_2$ | After-challenge trace commitment | `commitments.after_challenge` |
| $r_Q$ | Quotient commitment | `commitments.quotient_commit` |
| $r_k^{\text{FRI}}$ | FRI round $k$ commitment | `commit_phase_commits[k]` |

## LogUp

| Symbol | Meaning | Python Name |
|--------|---------|-------------|
| $d_i$ | Interaction denominator | computed in `compute_after_challenge_trace` |
| $\phi$ | Running sum (prefix sum of chunks) | `phi` column |
| $\phi[N-1]$ | Cumulative sum (exposed value) | `cumulative_sum` |

## Proof-of-Work

| Symbol | Meaning | Python Name |
|--------|---------|-------------|
| $\eta_{\text{lu}}$ | LogUp PoW witness | `pow_witness` |
| $\eta_{\text{deep}}$ | DEEP PoW witness | `pow_witness` |
| $b_{\text{pow}}$ | PoW difficulty (leading zero bits) | `proof_of_work_bits` |

## FRI Parameters

| Symbol | Meaning | Python Name |
|--------|---------|-------------|
| $K$ | Number of FRI folding rounds | `num_rounds` (derived) |
| $Q_{\text{queries}}$ | Number of query repetitions | `fri_params.num_queries` |
| $\log_2 \beta$ | Log blowup factor | `fri_params.log_blowup` |
| $b_{\text{pow}}$ | Per-round PoW difficulty | `fri_params.proof_of_work_bits` |
| $|F_K|$ | Final polynomial length (log) | `fri_params.log_final_poly_len` |

## Transcript Operations

| Operation | Meaning | Python Name |
|-----------|---------|-------------|
| $\mathcal{T}.\text{observe}(v)$ | Absorb field element | `challenger.observe(v)` |
| $\mathcal{T}.\text{observe\_many}(\mathbf{v})$ | Absorb multiple / FF4 | `challenger.observe_many(v)` |
| $\mathcal{T}.\text{sample}()$ | Squeeze base field element | `challenger.sample()` |
| $\mathcal{T}.\text{sample\_ext}()$ | Squeeze extension field element | `challenger.sample_ext()` |
| $\mathcal{T}.\text{sample\_bits}(b)$ | Squeeze $b$-bit integer | `challenger.sample_bits(b)` |

## Defined Terms

**Verifying Key (VK)**
: Contains per-AIR parameters: preprocessed commitment, constraint DAG,
  interaction definitions, quotient degree. The VK's `pre_hash` is the first
  thing absorbed into the transcript
  ({src}`protocol.proof.MultiStarkVerifyingKey`).

**MMCS (Mixed-Matrix Commitment Scheme)**
: A Merkle tree that commits multiple matrices of different heights in a single
  tree. Shorter matrices are injected at higher levels
  ({src}`protocol.pcs._build_mmcs_tree`).

**Air ID**
: Integer identifier for each AIR in a multi-AIR proof. Must be unique and
  sorted in the proof.

**Exposed Values**
: The cumulative sums from the LogUp after-challenge computation, revealed to
  the verifier. Their sum across all AIRs must equal zero.
