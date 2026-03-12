"""Constraint DAG evaluation and STARK constraint verification.

Evaluates the SymbolicExpressionDag at the OOD (out-of-domain) point and checks
that folded_constraints * inv_zeroifier == quotient for each AIR.

Reference:
    stark-backend/src/verifier/constraints.rs   -- verify_single_rap_constraints
    stark-backend/src/verifier/folder.rs         -- GenericVerifierConstraintFolder
    stark-backend/src/air_builders/symbolic/symbolic_expression.rs -- SymbolicEvaluator::eval_nodes
    stark-backend/src/air_builders/symbolic/dag.rs -- SymbolicExpressionNode
"""

from __future__ import annotations

import numpy as np

from primitives.field import (
    BABYBEAR_PRIME,
    EF4Coeffs,
    Fe,
    ef4_add,
    ef4_div,
    ef4_from_base,
    ef4_mul,
    ef4_neg,
    ef4_sub,
)
from protocol.domain import (
    DomainSelectors,
    TwoAdicMultiplicativeCoset,
)
from protocol.proof import (
    AdjacentOpenedValues,
    EntryType,
    SymbolicExpressionDag,
    SymbolicNodeKind,
    SymbolicVariable,
)

EF4_ZERO: EF4Coeffs = [0, 0, 0, 0]
EF4_ONE: EF4Coeffs = [1, 0, 0, 0]


# ---------------------------------------------------------------------------
# Unflatten: reconstitute extension field elements from flattened base field
# ---------------------------------------------------------------------------


def unflatten_ext_values(flattened: list[EF4Coeffs]) -> list[EF4Coeffs]:
    """Reconstitute extension field elements from flattened challenge values.

    In the Rust verifier, after_challenge (permutation) trace values are stored
    as "flattened" extension field elements: each EF4 element is stored as 4
    consecutive Challenge (== EF4) values where only the base field component
    is meaningful.  This function groups them back into proper EF4 elements.

    Given D=4 (extension degree), each group of 4 consecutive flattened values
    [c0, c1, c2, c3] (each an EF4Coeffs with only coeffs[0] used) is combined
    using the monomial basis:
        result = c0 * 1 + c1 * x + c2 * x^2 + c3 * x^3

    Since each c_i is already an EF4Coeffs, multiplying by x^e_i is the same
    as shifting coefficients. But in the verifier, each c_i is treated as a
    full EF4 element, and monomial(e_i) * c_i uses extension field multiplication.

    For BabyBear extension degree 4 with irreducible x^4 - 11:
        monomial(0) = [1,0,0,0]
        monomial(1) = [0,1,0,0]
        monomial(2) = [0,0,1,0]
        monomial(3) = [0,0,0,1]

    Reference:
        stark-backend/src/verifier/constraints.rs lines 65-75 (unflatten closure)
    """
    ext_degree = 4
    assert len(flattened) % ext_degree == 0, (
        f"Flattened length {len(flattened)} not divisible by ext degree {ext_degree}"
    )

    result = []
    for i in range(0, len(flattened), ext_degree):
        chunk = flattened[i : i + ext_degree]
        # Each chunk[e_i] is an EF4Coeffs.
        # Multiply chunk[e_i] by monomial(e_i) and sum.
        # monomial(e_i) = unit vector with 1 at position e_i.
        # For the verifier, chunk[e_i] * monomial(e_i) means:
        #   ef4_mul(chunk[e_i], [0,...,1,...0]) with 1 at position e_i.
        # But monomial(e_i) as EF4 coeffs is just the basis vector.
        acc = list(EF4_ZERO)
        for e_i in range(ext_degree):
            monomial = [0, 0, 0, 0]
            monomial[e_i] = 1
            term = ef4_mul(chunk[e_i], monomial)
            acc = ef4_add(acc, term)
        result.append(acc)
    return result


# ---------------------------------------------------------------------------
# DAG evaluation
# ---------------------------------------------------------------------------


def eval_symbolic_dag(
    dag: SymbolicExpressionDag,
    selectors: DomainSelectors,
    preprocessed_local: list[EF4Coeffs],
    preprocessed_next: list[EF4Coeffs],
    partitioned_main_values: list[AdjacentOpenedValues],
    after_challenge_values: list[AdjacentOpenedValues],
    challenges: list[list[EF4Coeffs]],
    public_values: list[Fe],
    exposed_values_after_challenge: list[list[EF4Coeffs]],
) -> list[EF4Coeffs]:
    """Evaluate the symbolic expression DAG and return constraint values.

    Walks the DAG nodes in topological order (they are already sorted).
    For each node, computes the extension field value based on the node kind.

    Returns the values at the constraint output indices (dag.constraint_idx).

    Reference:
        stark-backend/src/air_builders/symbolic/symbolic_expression.rs
            SymbolicEvaluator::eval_nodes (lines 364-396)
        stark-backend/src/verifier/folder.rs
            GenericVerifierConstraintFolder impl of SymbolicEvaluator (lines 74-123)
    """
    results: list[EF4Coeffs] = []

    for node in dag.nodes:
        value: EF4Coeffs

        if node.kind == SymbolicNodeKind.VARIABLE:
            value = _lookup_variable(
                node.variable,
                preprocessed_local,
                preprocessed_next,
                partitioned_main_values,
                after_challenge_values,
                challenges,
                public_values,
                exposed_values_after_challenge,
            )
        elif node.kind == SymbolicNodeKind.CONSTANT:
            # Embed base field constant into extension field
            value = ef4_from_base(node.constant_value)
        elif node.kind == SymbolicNodeKind.IS_FIRST_ROW:
            value = list(selectors.is_first_row)
        elif node.kind == SymbolicNodeKind.IS_LAST_ROW:
            value = list(selectors.is_last_row)
        elif node.kind == SymbolicNodeKind.IS_TRANSITION:
            value = list(selectors.is_transition)
        elif node.kind == SymbolicNodeKind.ADD:
            value = ef4_add(results[node.left_idx], results[node.right_idx])
        elif node.kind == SymbolicNodeKind.SUB:
            value = ef4_sub(results[node.left_idx], results[node.right_idx])
        elif node.kind == SymbolicNodeKind.MUL:
            value = ef4_mul(results[node.left_idx], results[node.right_idx])
        elif node.kind == SymbolicNodeKind.NEG:
            value = ef4_neg(results[node.idx])
        else:
            raise ValueError(f"Unknown SymbolicNodeKind: {node.kind}")

        results.append(value)

    # Extract constraint values at the specified indices
    return [list(results[idx]) for idx in dag.constraint_idx]


def _lookup_variable(
    var: SymbolicVariable,
    preprocessed_local: list[EF4Coeffs],
    preprocessed_next: list[EF4Coeffs],
    partitioned_main_values: list[AdjacentOpenedValues],
    after_challenge_values: list[AdjacentOpenedValues],
    challenges: list[list[EF4Coeffs]],
    public_values: list[Fe],
    exposed_values_after_challenge: list[list[EF4Coeffs]],
) -> EF4Coeffs:
    """Look up a symbolic variable's value from the opened proof values.

    Maps each Entry type to the appropriate opened values slice:
    - Preprocessed{offset} -> preprocessed local (offset=0) or next (offset=1)
    - Main{part_index, offset} -> partitioned_main[part_index].local/next[index]
    - Public -> public_values[index] embedded into EF4
    - Permutation{offset} -> after_challenge[0].local/next[index] (always phase 0)
    - Challenge -> challenges[0][index] (always phase 0)
    - Exposed -> exposed_values_after_challenge[0][index] (always phase 0)

    Reference:
        stark-backend/src/verifier/folder.rs lines 95-121
        (GenericVerifierConstraintFolder::eval_var)
    """
    entry = var.entry
    index = var.index

    if entry.kind == EntryType.PREPROCESSED:
        if entry.offset == 0:
            return list(preprocessed_local[index])
        else:
            return list(preprocessed_next[index])

    elif entry.kind == EntryType.MAIN:
        part = partitioned_main_values[entry.part_index]
        if entry.offset == 0:
            return list(part.local[index])
        else:
            return list(part.next[index])

    elif entry.kind == EntryType.PUBLIC:
        # Public values are base field elements, embed into extension field
        return ef4_from_base(public_values[index])

    elif entry.kind == EntryType.PERMUTATION:
        # Permutation = after_challenge phase 0 (always .first() in Rust)
        # NOTE: after_challenge_values here have already been unflattened
        part = after_challenge_values[0]
        if entry.offset == 0:
            return list(part.local[index])
        else:
            return list(part.next[index])

    elif entry.kind == EntryType.CHALLENGE:
        # Challenge phase 0 (always .first() in Rust)
        return list(challenges[0][index])

    elif entry.kind == EntryType.EXPOSED:
        # Exposed values after challenge phase 0 (always .first() in Rust)
        return list(exposed_values_after_challenge[0][index])

    else:
        raise ValueError(f"Unknown EntryType: {entry.kind}")


# ---------------------------------------------------------------------------
# Constraint folding
# ---------------------------------------------------------------------------


def fold_constraints(
    constraint_evals: list[EF4Coeffs],
    alpha: EF4Coeffs,
) -> EF4Coeffs:
    """Compute random linear combination of constraint evaluations.

    The Rust verifier uses Horner's method: starting with accumulator = 0,
    for each constraint C_i:
        accumulator = accumulator * alpha + C_i

    This produces: C_0 * alpha^{n-1} + C_1 * alpha^{n-2} + ... + C_{n-1}

    Reference:
        stark-backend/src/verifier/folder.rs lines 56-72
        (GenericVerifierConstraintFolder::eval_constraints + assert_zero)
    """
    accumulator = list(EF4_ZERO)
    for c_eval in constraint_evals:
        # accumulator *= alpha
        accumulator = ef4_mul(accumulator, alpha)
        # accumulator += c_eval
        accumulator = ef4_add(accumulator, c_eval)
    return accumulator


# ---------------------------------------------------------------------------
# Quotient reconstruction
# ---------------------------------------------------------------------------


def reconstruct_quotient(
    quotient_chunks: list[list[EF4Coeffs]],
    qc_domains: list[TwoAdicMultiplicativeCoset],
    zeta: EF4Coeffs,
) -> EF4Coeffs:
    """Recompute the full quotient polynomial value at zeta from chunks.

    For each chunk domain D_i, compute:
        zps[i] = prod_{j != i} Z_{D_j}(zeta) / Z_{D_j}(first_point(D_i))

    Then the full quotient at zeta is:
        Q(zeta) = sum_i zps[i] * sum_e (monomial(e) * chunk[i][e])

    Each quotient_chunks[i] has D=4 elements, which are the coefficients
    in the extension field basis. So quotient_chunks[i] is treated as a
    single EF4 element by combining with the monomial basis.

    Reference:
        stark-backend/src/verifier/constraints.rs lines 38-63
    """
    num_chunks = len(qc_domains)
    assert len(quotient_chunks) == num_chunks

    # Compute zps[i] = prod_{j!=i} Z_{D_j}(zeta) / Z_{D_j}(D_i.first_point())
    zps: list[EF4Coeffs] = []
    for i in range(num_chunks):
        prod = list(EF4_ONE)
        for j in range(num_chunks):
            if j != i:
                zp_at_zeta = qc_domains[j].vanishing_poly_at_point(zeta)
                first_pt = ef4_from_base(qc_domains[i].first_point())
                zp_at_first = qc_domains[j].vanishing_poly_at_point(first_pt)
                factor = ef4_div(zp_at_zeta, zp_at_first)
                prod = ef4_mul(prod, factor)
        zps.append(prod)

    # Reconstruct: Q(zeta) = sum_i zps[i] * (sum_e monomial(e) * chunk[i][e])
    result = list(EF4_ZERO)
    for ch_i in range(num_chunks):
        chunk = quotient_chunks[ch_i]
        # Combine chunk values using monomial basis:
        # sum_e monomial(e) * chunk[e]
        # monomial(e) is the unit vector with 1 at position e
        chunk_sum = list(EF4_ZERO)
        for e_i, c in enumerate(chunk):
            monomial = [0, 0, 0, 0]
            monomial[e_i] = 1
            term = ef4_mul(c, monomial)
            chunk_sum = ef4_add(chunk_sum, term)
        # zps[ch_i] * chunk_sum
        term = ef4_mul(zps[ch_i], chunk_sum)
        result = ef4_add(result, term)

    return result


# ---------------------------------------------------------------------------
# Vectorized DAG evaluation (all rows at once, base field)
# ---------------------------------------------------------------------------

p = BABYBEAR_PRIME


def eval_dag_all_rows(
    dag: SymbolicExpressionDag,
    partitioned_main: list[list[list[Fe]]],
    preprocessed: list[list[Fe]] | None,
    public_values: list[Fe],
    height: int,
) -> list:
    """Evaluate full DAG at ALL rows simultaneously using numpy arrays.

    Each DAG node evaluates to a numpy int64 array of shape (height,).

    Args:
        dag: SymbolicExpressionDag
        partitioned_main: [part_index][rows][cols] — list of trace matrices
        preprocessed: [rows][cols] or None
        public_values: list of Fe
        height: trace height

    Returns:
        List of numpy int64 arrays, one per DAG node.
    """
    # Pre-convert trace matrices to numpy column arrays for fast lookup
    np_parts = []
    for part in partitioned_main:
        cols = len(part[0]) if part else 0
        np_cols = [
            np.array([part[r][c] for r in range(height)], dtype=np.int64)
            for c in range(cols)
        ]
        np_parts.append(np_cols)

    np_prep = None
    if preprocessed is not None:
        prep_cols = len(preprocessed[0]) if preprocessed else 0
        np_prep = [
            np.array([preprocessed[r][c] for r in range(height)], dtype=np.int64)
            for c in range(prep_cols)
        ]

    nodes = dag.nodes
    node_values = [None] * len(nodes)
    zero = np.zeros(height, dtype=np.int64)

    for i, node in enumerate(nodes):
        kind = node.kind

        if kind == SymbolicNodeKind.VARIABLE:
            var = node.variable
            entry = var.entry
            if entry.kind == EntryType.MAIN:
                if entry.offset == 0:
                    node_values[i] = np_parts[entry.part_index][var.index] % p
                else:
                    col = np_parts[entry.part_index][var.index]
                    node_values[i] = np.roll(col, -entry.offset) % p
            elif entry.kind == EntryType.PREPROCESSED:
                if entry.offset == 0:
                    node_values[i] = np_prep[var.index] % p
                else:
                    col = np_prep[var.index]
                    node_values[i] = np.roll(col, -entry.offset) % p
            elif entry.kind == EntryType.PUBLIC:
                node_values[i] = np.full(height, public_values[var.index] % p, dtype=np.int64)
            else:
                node_values[i] = zero.copy()

        elif kind == SymbolicNodeKind.CONSTANT:
            node_values[i] = np.full(height, node.constant_value % p, dtype=np.int64)

        elif kind in (SymbolicNodeKind.IS_FIRST_ROW,
                      SymbolicNodeKind.IS_LAST_ROW,
                      SymbolicNodeKind.IS_TRANSITION):
            node_values[i] = zero.copy()

        elif kind == SymbolicNodeKind.ADD:
            node_values[i] = (node_values[node.left_idx] + node_values[node.right_idx]) % p

        elif kind == SymbolicNodeKind.SUB:
            node_values[i] = (node_values[node.left_idx] - node_values[node.right_idx]) % p

        elif kind == SymbolicNodeKind.MUL:
            node_values[i] = (node_values[node.left_idx] * node_values[node.right_idx]) % p

        elif kind == SymbolicNodeKind.NEG:
            node_values[i] = (-node_values[node.idx]) % p

        else:
            raise ValueError(f"Unknown node kind: {kind}")

    return node_values


# ---------------------------------------------------------------------------
# Verification error
# ---------------------------------------------------------------------------


class VerificationError(Exception):
    """Raised when STARK constraint verification fails.

    Reference:
        stark-backend/src/verifier/error.rs (enum VerificationError)
    """
    pass


class OodEvaluationMismatch(VerificationError):
    """Out-of-domain evaluation mismatch: constraints(zeta) != quotient(zeta) * Z_H(zeta).

    Reference:
        stark-backend/src/verifier/error.rs (VerificationError::OodEvaluationMismatch)
    """
    pass


# ---------------------------------------------------------------------------
# Main per-AIR verification function
# ---------------------------------------------------------------------------


def verify_single_rap_constraints(
    constraints: SymbolicExpressionDag,
    preprocessed_values: AdjacentOpenedValues | None,
    partitioned_main_values: list[AdjacentOpenedValues],
    after_challenge_values: list[AdjacentOpenedValues],
    quotient_chunks: list[list[EF4Coeffs]],
    domain: TwoAdicMultiplicativeCoset,
    qc_domains: list[TwoAdicMultiplicativeCoset],
    zeta: EF4Coeffs,
    alpha: EF4Coeffs,
    challenges: list[list[EF4Coeffs]],
    public_values: list[Fe],
    exposed_values_after_challenge: list[list[EF4Coeffs]],
) -> None:
    """Verify constraints for a single RAP (AIR with interactions).

    Steps:
    1. Compute selectors at zeta (is_first_row, is_last_row, is_transition, inv_zeroifier).
    2. Unflatten after_challenge values from base-field-flattened to EF4 elements.
    3. Evaluate constraint DAG at opened values.
    4. Fold constraints with alpha (random linear combination).
    5. Reconstruct quotient from chunks.
    6. Assert: folded_constraints * inv_zeroifier == quotient_value.

    Raises OodEvaluationMismatch if the check fails.

    Reference:
        stark-backend/src/verifier/constraints.rs lines 21-140
        (verify_single_rap_constraints)
    """
    # Step 1: compute selectors at zeta
    selectors = domain.selectors_at_point(zeta)

    # Step 2: extract preprocessed local/next
    if preprocessed_values is not None:
        preprocessed_local = preprocessed_values.local
        preprocessed_next = preprocessed_values.next
    else:
        preprocessed_local = []
        preprocessed_next = []

    # Step 3: unflatten after_challenge values
    # In Rust, after_challenge opened values are stored as flattened extension
    # field elements (each EF4 element = 4 consecutive Challenge values).
    # We reconstitute them before passing to the DAG evaluator.
    unflattened_after_challenge: list[AdjacentOpenedValues] = []
    for ac_vals in after_challenge_values:
        local_unflat = unflatten_ext_values(ac_vals.local)
        next_unflat = unflatten_ext_values(ac_vals.next)
        unflattened_after_challenge.append(
            AdjacentOpenedValues(local=local_unflat, next=next_unflat)
        )

    # Step 4: evaluate constraint DAG
    constraint_evals = eval_symbolic_dag(
        dag=constraints,
        selectors=selectors,
        preprocessed_local=preprocessed_local,
        preprocessed_next=preprocessed_next,
        partitioned_main_values=partitioned_main_values,
        after_challenge_values=unflattened_after_challenge,
        challenges=challenges,
        public_values=public_values,
        exposed_values_after_challenge=exposed_values_after_challenge,
    )

    # Step 5: fold constraints
    folded_constraints = fold_constraints(constraint_evals, alpha)

    # Step 6: reconstruct quotient
    quotient = reconstruct_quotient(quotient_chunks, qc_domains, zeta)

    # Step 7: check folded_constraints * inv_zeroifier == quotient
    lhs = ef4_mul(folded_constraints, selectors.inv_zeroifier)
    if lhs != quotient:
        raise OodEvaluationMismatch(
            f"OOD evaluation mismatch: "
            f"folded_constraints * inv_zeroifier = {lhs} != quotient = {quotient}"
        )
