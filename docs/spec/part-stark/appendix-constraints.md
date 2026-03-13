(app:constraints)=
# Appendix: Constraint System

## Symbolic Expression DAG

Constraints are represented as a directed acyclic graph (DAG) of symbolic
expressions ({src}`protocol.proof.SymbolicExpressionDag`). Each node has:

- **kind**: One of `VARIABLE`, `CONSTANT`, `IS_FIRST_ROW`, `IS_LAST_ROW`,
  `IS_TRANSITION`, `ADD`, `SUB`, `MUL`, `NEG`.
- **operands**: Indices into the DAG node array (for binary/unary operations).
- **degree_multiple**: The constraint degree for variable nodes.

The DAG is stored in topological order: every node's operands have smaller
indices than the node itself.

### Node Types

| Kind | Arity | Description |
|------|-------|-------------|
| `VARIABLE` | 0 | Reference to a trace column at a specific offset |
| `CONSTANT` | 0 | Literal field element |
| `IS_FIRST_ROW` | 0 | Lagrange selector $L_0(\zeta)$ |
| `IS_LAST_ROW` | 0 | Lagrange selector $L_{N-1}(\zeta)$ |
| `IS_TRANSITION` | 0 | $\zeta/s - \omega^{-1}$ (nonzero on non-last rows) |
| `ADD` | 2 | $a + b$ |
| `SUB` | 2 | $a - b$ |
| `MUL` | 2 | $a \cdot b$ |
| `NEG` | 1 | $-a$ |

### Variables

A `SymbolicVariable` ({src}`protocol.proof.SymbolicVariable`) references a
specific trace column:

- **entry_type**: `PREPROCESSED`, `MAIN`, `PERMUTATION`, `PUBLIC`, `CHALLENGE`,
  `EXPOSED_AFTER_CHALLENGE`.
- **column_index**: Column within the partition.
- **offset**: Row offset (0 = current row, 1 = next row).

## DAG Evaluation

### At a Point (Verifier)

The verifier evaluates the DAG at the OOD point $\zeta$ using the opened
polynomial values ({src}`protocol.constraints.eval_symbolic_dag`):

1. Walk nodes in topological order.
2. For each node, dispatch on kind:
   - `VARIABLE`: look up the opened value from the appropriate trace partition.
   - `CONSTANT`: wrap as FF4.
   - Selector nodes: fetch from precomputed `DomainSelectors`.
   - Arithmetic nodes: compute from previously evaluated operands.
3. Extract the constraint output indices: `dag.constraint_idx`.

### Over the Full Trace (Prover)

The prover evaluates the DAG at every point of the quotient domain
({src}`protocol.quotient.eval_symbolic_expression_dag_full`). The same
DAG walk applies, but each node result is a column (array) rather than a scalar.

## Constraint Folding

Given $k$ constraint evaluations $C_0, C_1, \ldots, C_{k-1}$ and the challenge
$\alpha$, the folded constraint value is computed by Horner's method
({src}`protocol.constraints.fold_constraints`):

$$
\text{folded} = C_0 \cdot \alpha^{k-1} + C_1 \cdot \alpha^{k-2} + \cdots + C_{k-1}
$$

Equivalently, accumulate right-to-left:

```
accumulator = 0
for i in range(k):
    accumulator = accumulator * alpha + C_i
```

## Quotient Relationship

The quotient polynomial $Q(X)$ encodes the constraint satisfaction:

$$
Q(X) = \frac{\sum_i \alpha^i \cdot C_i(X)}{Z_H(X)}
$$

where $Z_H(X) = (X/s)^N - 1$ is the vanishing polynomial of the trace domain.

The verifier checks this at $\zeta$:

$$
\text{folded} \cdot Z_H(\zeta)^{-1} = Q(\zeta)
$$

where $Q(\zeta)$ is reconstructed from the quotient chunk openings
({src}`protocol.constraints.reconstruct_quotient`).

## Domain Selectors

The selectors at a point $\zeta$ outside the trace domain $H = \{s \cdot \omega^i\}$
are ({src}`protocol.domain.get_selectors_at_point`):

Let $u = \zeta / s$ (the "unshifted" point).

$$
\begin{aligned}
Z_H(\zeta) &= u^N - 1 \\
\text{is\_first\_row} &= \frac{Z_H(\zeta)}{u - 1} \\
\text{is\_last\_row} &= \frac{Z_H(\zeta)}{u - \omega^{-1}} \\
\text{is\_transition} &= u - \omega^{-1} \\
\text{inv\_zeroifier} &= Z_H(\zeta)^{-1}
\end{aligned}
$$

These are **unnormalized** selectors (as in Plonky3). The normalization factor
cancels in the quotient relationship.
