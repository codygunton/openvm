# Milestone 1: Standalone FRI Prover/Verifier — Design

## Problem Statement

We have Phase 1 (testing infrastructure) complete with golden test vectors for all
primitives and FRI protocol. Phase 2 (faithful translation) needs to begin. The
question is: what's the right first milestone that gives us a testable, self-contained
piece of the proving system?

## Decision: Standalone FRI + Primitives

**Milestone 1 = make all primitive tests + FRI tests pass.**

OpenVM uses vanilla Plonky3 FRI (p3-fri v0.4.1) with no modifications. The STARK
orchestration (stark-backend) sits on top. FRI + its primitive dependencies form a
natural, testable unit.

### Source Analysis

| Layer | Rust Source | Lines | Role |
|-------|-----------|-------|------|
| FRI core | `p3-fri-0.4.1/src/{prover,verifier}.rs` | ~780 | Fold, commit, query, verify |
| Folding strategy | `p3-fri-0.4.1/src/two_adic_pcs.rs:91-163` | ~70 | Two-adic fold formula |
| FRI config/proof | `p3-fri-0.4.1/src/{config,proof}.rs` | ~160 | Data structures |
| PCS wrapper | `p3-fri-0.4.1/src/two_adic_pcs.rs` | ~550 | **Deferred to milestone 2** |
| STARK orchestration | `stark-backend` | ~8,000 | **Deferred to milestone 2** |

Translation source: **pinned p3-fri v0.4.1** from cargo registry (at
`~/.cargo/registry/src/index.crates.io-.../p3-fri-0.4.1/`). This matches the
version used to generate test vectors.

### Key Design Decisions

1. **Primitives from pil2-proofman patterns**: Copy the structure/style from
   `/home/cody/pil2-proofman/executable-spec/primitives/` and adapt for BabyBear.
   Minimizes new design work, reuses proven patterns.

2. **FRI protocol fresh from Plonky3**: Translate directly from p3-fri v0.4.1 Rust
   source. Do not adapt pil2-proofman's FRI (it was translated from C++ pil2-proofman,
   different algorithm).

3. **Concrete implementations only**: No Python ABCs or trait hierarchies. Hardcode
   BabyBear + quartic extension + Poseidon2 Merkle. Matches SPEC_STYLE_GUIDE philosophy.

## Architecture

```
executable-spec/
├── primitives/
│   ├── field.py          BabyBear GF(p), p=2^31-2^27+1, quartic extension GF(p^4)
│   ├── poseidon2.py      Poseidon2 permutation (width-16) and compression
│   ├── merkle.py         Merkle tree construction and verification
│   ├── ntt.py            NTT/INTT over BabyBear two-adic subgroups
│   └── transcript.py     Fiat-Shamir challenger (Poseidon2-based)
├── protocol/
│   └── fri.py            FRI fold, commit_phase, verify_query, verify_fri
└── tests/
    ├── test_field.py          ← field.py
    ├── test_poseidon2.py      ← poseidon2.py
    ├── test_ntt.py            ← ntt.py
    ├── test_merkle.py         ← merkle.py
    └── test_fri.py            ← fri.py + transcript.py
```

### Dependency Graph

```
field.py (BabyBear base + quartic ext)
    ↓
poseidon2.py (hash permutation using field arithmetic)
    ↓
merkle.py (Poseidon2 commitments)     ntt.py (two-adic NTT over BabyBear)
    ↓                                     ↓
transcript.py (Fiat-Shamir with Poseidon2)
    ↓
fri.py (fold formula + verify_query + verify_fri)
```

## Primitives — Adaptation from pil2-proofman

| File | Source Pattern | Key Adaptations |
|------|---------------|-----------------|
| `field.py` | pil2-proofman `field.py` | Goldilocks (p=2^64-2^32+1) → BabyBear (p=2^31-2^27+1); cubic ext (deg 3) → quartic ext (deg 4); different irreducible polynomial; different roots of unity table |
| `poseidon2.py` | pil2-proofman pattern | Different round constants, BabyBear-specific parameters, width-16 permutation |
| `merkle.py` | pil2-proofman `merkle_tree.py` | Same structure, uses our Poseidon2; hash digest = 8 BabyBear elements |
| `ntt.py` | pil2-proofman `ntt.py` | Same algorithm, BabyBear roots of unity |
| `transcript.py` | pil2-proofman `transcript.py` | Same Fiat-Shamir pattern, BabyBear Poseidon2 |

## FRI Protocol — Fresh Translation from p3-fri v0.4.1

### Functions to Translate

From `prover.rs`:
- `prove_fri()` → Entry point: commit phase + query phase
- `commit_phase()` → Iterative folding with Merkle commitments
- `answer_query()` → Generate opening proof for a query index

From `verifier.rs`:
- `verify_fri()` → Replay transcript, verify all queries
- `verify_query()` → Verify single query chain: fold siblings, check Merkle

From `two_adic_pcs.rs` (folding only):
- `fold_row()` → Linear interpolation of two sibling evaluations at beta
- `fold_matrix()` → Batch folding: `(lo+hi)/2 + beta*(lo-hi)/(2*g^i)`

### The Folding Formula

Standard two-adic FRI fold. Given evaluations `e0 = f(x)` and `e1 = f(-x)`:

```
p_even(x^2) = (f(x) + f(-x)) / 2
p_odd(x^2)  = (f(x) - f(-x)) / (2x)
folded(x^2) = p_even(x^2) + beta * p_odd(x^2)
```

In the test vectors, evaluations are in natural order on the coset `g * <omega>`.

### Proof Structure

```python
FriProof:
    commit_phase_commits: list[MerkleRoot]    # One Merkle root per fold round
    final_poly: list[ExtFieldElem]            # Final low-degree polynomial
    query_proofs: list[QueryProof]            # One per query

QueryProof:
    commit_phase_openings: list[CommitPhaseStep]

CommitPhaseStep:
    sibling_value: ExtFieldElem               # Value at sibling index
    opening_proof: list[MerkleDigest]         # Merkle path
```

### Test Vector Details

**fri_folding.json** — Pure folding math (no crypto):
- Degree-15 polynomial over BinomialExtensionField<BabyBear, 4>
- Domain size 32 (log_poly_size=4, log_blowup=1)
- 4 folding rounds with deterministic challenges
- Tests: `fold_row` / `fold_matrix` produces expected output each round

**fri_verification.json** — Real FRI proof from Fibonacci STARK:
- Extracted from BabyBearPoseidon2Engine with 2 queries, 0 PoW bits
- Query indices derived from transcript replay (not stored in vectors)
- Tests: `verify_query` with Merkle verification + folding + final poly check

## What's Deferred to Milestone 2

- `open_input()` — Reduced opening computation `(f(z)-f(x))/(z-x)` with alpha batching
- `TwoAdicFriPcs` — Full polynomial commitment scheme (commit, open, verify)
- stark-backend STARK orchestration — Multi-AIR prover/verifier
- E2E tests (test_stark_e2e.py, test_verifier_e2e.py)

## Success Criteria

All of these tests pass:
- `test_field.py` — 5 tests (base add/mul/inv, extension ops, edge cases)
- `test_poseidon2.py` — 2 tests (permutation, compression)
- `test_ntt.py` — 2 tests (forward/inverse transform)
- `test_merkle.py` — 2 tests (construction, opening verification)
- `test_fri.py` — 3 tests (fold per round, final polynomial, query verification)

Run with: `./run-tests.sh primitives && ./run-tests.sh fri`
