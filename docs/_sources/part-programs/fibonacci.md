(sec:fibonacci)=
# Fibonacci (Single-AIR)

The Fibonacci program is the simplest instantiation of the OpenVM STARK
protocol: a single AIR with 2 trace columns, 3 public values, and 5
constraints. It has no bus interactions (no LogUp phase).

## Trace Structure

The trace has 2 columns (*left*, *right*) and $N$ rows (power of 2)
({src}`witness.fibonacci.generate_fibonacci_trace`):

| Row | Left | Right |
|-----|------|-------|
| 0 | $a$ | $b$ |
| 1 | $b$ | $a + b$ |
| 2 | $a + b$ | $a + 2b$ |
| $\vdots$ | $\vdots$ | $\vdots$ |
| $N-1$ | $F_{N-1}$ | $F_N$ |

All values are reduced modulo $p$.

## Public Values

Three public values $[a, b, x]$ where
({src}`witness.fibonacci.fibonacci_public_values`):

- $a = \text{trace}[0][0]$ (initial left)
- $b = \text{trace}[0][1]$ (initial right)
- $x = \text{trace}[N-1][1]$ (final right = $N$-th Fibonacci number)

## Constraints

The AIR defines 5 constraint polynomials
({src}`constraints.fibonacci_air.FibonacciAir.eval_constraints`):

| # | Constraint | Selector | Purpose |
|---|-----------|----------|---------|
| 1 | $L_0 \cdot (\ell_0 - a)$ | `is_first_row` | First left $= a$ |
| 2 | $L_0 \cdot (\ell_1 - b)$ | `is_first_row` | First right $= b$ |
| 3 | $T \cdot (n_0 - \ell_1)$ | `is_transition` | Next left $=$ current right |
| 4 | $T \cdot (n_1 - \ell_0 - \ell_1)$ | `is_transition` | Next right $=$ sum |
| 5 | $L_{N-1} \cdot (\ell_1 - x)$ | `is_last_row` | Last right $= x$ |

Where:
- $\ell_0, \ell_1$: current row's left and right columns
- $n_0, n_1$: next row's left and right columns
- $L_0, L_{N-1}, T$: domain selectors (see {ref}`sec:notation`)

Each constraint has degree 2 (product of degree-1 selector and degree-1 expression).
The quotient degree is $d_Q = 2$, producing 2 quotient chunks.

## Protocol Walkthrough

For the test vector (`fibonacci_stark.json`, $N = 256$, $a = 1$, $b = 1$):

### Stage 1: Witness Commitment

1. Generate 256-row trace: `generate_fibonacci_trace(1, 1, 256)`.
2. Coset LDE with blowup 2: extend 256 → 512 evaluations.
3. Merkle commit → root $r_1$ (8 base field elements).
4. Absorb public values $[1, 1, F_{256}]$, preprocessed commits, $r_1$,
   $\log_2(256) = 8$ into transcript.

### Stage 2: After-Challenge (LogUp)

Skipped — Fibonacci has no bus interactions.

### Stage Q: Quotient

1. Sample $\alpha \leftarrow \mathcal{T}.\text{sample\_ext}()$.
2. Create disjoint quotient domain of size $256 \cdot 2 = 512$.
3. Extend trace to quotient domain.
4. Evaluate all 5 constraints at each quotient domain point.
5. Accumulate: $\text{acc}(x) = \alpha^4 C_1(x) + \alpha^3 C_2(x) + \alpha^2 C_3(x) + \alpha C_4(x) + C_5(x)$.
6. Divide by $Z_H(x)$ to get $Q(x)$.
7. Split into 2 chunks of 256 values each.
8. Merkle commit → root $r_Q$.

### Evaluation + FRI

1. Sample $\zeta \leftarrow \mathcal{T}.\text{sample\_ext}()$.
2. Open main trace and quotient chunks at $\zeta$ (and $\zeta\omega$ for main).
3. Run FRI commit + query phase.

### Verification

The verifier checks:
- $C_1(\zeta) = 0, \ldots, C_5(\zeta) = 0$ (after $\alpha$-folding)
- $\text{folded} \cdot Z_H(\zeta)^{-1} = Q(\zeta)$
- All Merkle proofs and FRI fold consistency.

## Test Vectors

The test suite verifies:
- **Prover**: Python produces byte-identical proof to Rust
  (`test_proof_binary_equivalence` in `test_stark_e2e.py`).
- **Verifier**: Python verifier accepts the Rust-generated proof
  (`test_verify_valid_proof` in `test_verifier_e2e.py`).
- **Failure detection**: Corrupted quotient/main/PV/VK all rejected.
