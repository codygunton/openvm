# Part I: STARK Protocol

This part specifies the generic FRI-STARK proving system used by OpenVM.
The protocol is parameterized by the AIR (Algebraic Intermediate Representation),
which defines the constraint system and execution trace structure.

## Protocol Overview

```{mermaid}
flowchart LR
    W[Witness<br>Generation] --> S1[Stage 1:<br>Main Trace<br>Commitment]
    S1 --> LU{LogUp?}
    LU -->|yes| S2[Stage 2:<br>After-Challenge<br>Trace]
    LU -->|no| SQ
    S2 --> SQ[Stage Q:<br>Quotient<br>Polynomial]
    SQ --> EV[Evaluation<br>at ζ]
    EV --> PCS[PCS<br>Batch Opening]
    PCS --> FRI[FRI<br>Commit + Query]
    FRI --> V{Verify}

    style W fill:#e8f4e8
    style S1 fill:#dae8fc
    style S2 fill:#dae8fc
    style SQ fill:#dae8fc
    style EV fill:#fff2cc
    style PCS fill:#f8cecc
    style FRI fill:#f8cecc
    style V fill:#e1d5e7
```

## Prover Flow

1. **Stage 1**: Commit main trace columns via coset LDE + Merkle tree.
2. **Stage 2** *(if interactions)*: Compute LogUp auxiliary columns, commit.
3. **Stage Q**: Compute quotient polynomial $Q(x) = \sum \alpha^i C_i(x) / Z_H(x)$,
   split into chunks, commit.
4. **Evaluation**: Open all polynomials at OOD point $\zeta$ and $\zeta\omega$.
5. **PCS/FRI**: Prove opened values are consistent with commitments.

## Verifier Flow

1. Replay the Fiat-Shamir transcript (same observations as prover).
2. Verify constraint equation at $\zeta$: $\text{folded} / Z_H(\zeta) = Q(\zeta)$.
3. Verify PCS: Merkle proofs + FRI fold consistency.
4. If LogUp: verify cumulative sum is zero.

## Pages

```{toctree}
:maxdepth: 2

notation
building-blocks
commitment-phase
verification
full-protocol
appendix-constraints
appendix-fri
glossary
```
