"""Prover-side quotient polynomial computation.

Computes Q(x) = (sum of alpha-weighted constraints at x) / Z_H(x) for each
point x in the quotient domain.  The quotient domain is a coset disjoint from
the trace domain with size = trace_size * quotient_degree.

The output is split into `quotient_degree` chunks, each of trace_domain.size()
rows, with extension field elements flattened to base field coefficients for
Merkle commitment.

Reference:
    stark-backend/src/prover/cpu/quotient/single.rs  — compute_single_rap_quotient_values
    stark-backend/src/prover/cpu/quotient/mod.rs      — QuotientCommitter
    stark-backend/src/prover/cpu/quotient/evaluator.rs — ProverConstraintEvaluator
    p3-commit-0.4.1/src/domain.rs                      — selectors_on_coset
    p3-dft-0.4.1/src/traits.rs                         — coset_lde_batch
"""

from __future__ import annotations

from dataclasses import dataclass

from primitives.field import (
    BABYBEAR_PRIME,
    EF4Coeffs,
    EF4Vec,
    FF,
    Fe,
    GENERATOR,
    batch_inverse_base,
    ef4_add,
    ef4_from_base,
    ef4_mul,
    ef4_mul_base,
    ef4_neg,
    ef4_sub,
    ef4v_add,
    ef4v_from_base,
    ef4v_from_rows,
    ef4v_from_scalar,
    ef4v_mul,
    ef4v_mul_base,
    ef4v_mul_scalar,
    ef4v_neg,
    ef4v_roll,
    ef4v_sub,
    ef4v_to_rows,
    ef4v_zeros,
    ff_column,
    ff_roll,
    get_omega,
    inv_mod,
)
from primitives.ntt import coset_lde
from protocol.domain import (
    TwoAdicMultiplicativeCoset,
)
from protocol.proof import (
    SymbolicExpressionDag,
    SymbolicNodeKind,
    EntryType,
)

p = BABYBEAR_PRIME


# ---------------------------------------------------------------------------
# Coset LDE: extend trace to quotient domain
# ---------------------------------------------------------------------------


def coset_lde_column(
    evals: list[Fe],
    trace_domain: TwoAdicMultiplicativeCoset,
    quotient_domain: TwoAdicMultiplicativeCoset,
) -> list[Fe]:
    """Extend polynomial from trace domain to quotient domain via coset LDE.

    Reference:
        p3-dft-0.4.1/src/traits.rs coset_lde_batch (lines 226-249)
        p3-dft-0.4.1/src/util.rs   coset_shift_cols (lines 28-36)

    Args:
        evals: Column evaluations on the trace domain (natural order).
        trace_domain: The trace domain (shift=1 subgroup).
        quotient_domain: The quotient domain (disjoint coset).

    Returns:
        Column evaluations on the quotient domain (natural order).
    """
    log_blowup = quotient_domain.log_n - trace_domain.log_n
    lde_shift = (GENERATOR * inv_mod(trace_domain.shift)) % p
    return coset_lde(evals, lde_shift, log_blowup)


def extend_trace_to_quotient_domain(
    trace: list[list[Fe]],
    trace_domain: TwoAdicMultiplicativeCoset,
    quotient_domain: TwoAdicMultiplicativeCoset,
) -> list[list[Fe]]:
    """Extend the full trace matrix from trace domain to quotient domain.

    Input: trace as list of rows [[col0, col1, ...], ...] on the trace domain.
    Output: evaluations on the quotient domain, as list of rows.

    Each column is independently extended via coset LDE.

    Reference:
        stark-backend/src/prover/cpu/quotient/mod.rs single_rap_quotient_values
        (trace LDE is done externally but the math is the same)

    Args:
        trace: Trace matrix as list of rows, each row a list of base field elements.
        trace_domain: Trace domain (shift=1 subgroup).
        quotient_domain: Quotient domain (disjoint coset, larger).

    Returns:
        Trace evaluations on quotient domain, as list of rows.
    """
    n_trace = trace_domain.size()
    n_quot = quotient_domain.size()
    assert len(trace) == n_trace
    if n_trace == 0:
        return [[] for _ in range(n_quot)]

    num_cols = len(trace[0])

    # Transpose: extract columns
    columns = [[trace[r][c] for r in range(n_trace)] for c in range(num_cols)]

    # LDE each column
    lde_columns = [
        coset_lde_column(col, trace_domain, quotient_domain)
        for col in columns
    ]

    # Transpose back to rows
    return [[lde_columns[c][r] for c in range(num_cols)] for r in range(n_quot)]


# ---------------------------------------------------------------------------
# Selectors on coset (batch computation for prover efficiency)
# ---------------------------------------------------------------------------


@dataclass
class CosetSelectors:
    """Precomputed Lagrange selectors at every point of a coset.

    These are base-field-valued selectors for the trace domain evaluated at
    every point of the quotient domain (a coset).

    Reference:
        p3-commit-0.4.1/src/domain.rs selectors_on_coset (lines 252-292)
    """
    is_first_row: list[Fe]
    is_last_row: list[Fe]
    is_transition: list[Fe]
    inv_zeroifier: list[Fe]


def _single_point_selector(
    exponent: int,
    xs: list[Fe],
    z_h_short: list[Fe],
    rate: int,
    quot_size: int,
    trace_domain: TwoAdicMultiplicativeCoset,
) -> list[Fe]:
    """Compute the selector for the trace domain point omega^exponent.

    selector[i] = Z_H(x_i) / (x_i - omega^exponent)

    Reference:
        p3-commit-0.4.1/src/domain.rs lines 268-278
    """
    coset_point = pow(trace_domain.gen(), exponent, p)
    # Compute denominators: x_i - coset_point
    denoms = [(x - coset_point) % p for x in xs]
    # Batch invert
    inv_denoms = batch_inverse_base(denoms)
    # Multiply by Z_H (which cycles with period rate)
    return [(z_h_short[i % rate] * inv_denoms[i]) % p for i in range(quot_size)]


def selectors_on_coset(
    trace_domain: TwoAdicMultiplicativeCoset,
    quotient_domain: TwoAdicMultiplicativeCoset,
) -> CosetSelectors:
    """Compute Lagrange selectors of the trace domain at every quotient domain point.

    This is a batch computation that avoids per-point extension field arithmetic.
    All arithmetic is in the base field because quotient domain points are base
    field elements (the coset shift is a base field element).

    Algorithm from p3-commit selectors_on_coset:
    1. Compute Z_H(x) = (shift^n * rate_gen^i) - 1 for each i in [0, rate),
       where rate = quotient_size / trace_size. These values cycle with period
       `rate` over the quotient domain.
    2. For single_point_selector(j):
       - Compute denoms[i] = x_i - omega^j for each quotient domain point x_i
       - Batch invert the denoms
       - Multiply: selector[i] = Z_H(x_i) * denom_inv[i]
    3. is_first_row = single_point_selector(0)
       is_last_row = single_point_selector(n-1) where n = trace_size
    4. is_transition[i] = x_i - omega^{-1} (omega = trace generator)
    5. inv_zeroifier = batch_inverse(Z_H) cycled over quotient_size

    Reference:
        p3-commit-0.4.1/src/domain.rs selectors_on_coset (lines 252-292)

    Args:
        trace_domain: The trace domain (must have shift=1).
        quotient_domain: A coset disjoint from trace domain.

    Returns:
        CosetSelectors with base field values at each quotient domain point.
    """
    assert trace_domain.shift == 1, "trace domain must be unshifted subgroup"
    assert quotient_domain.shift != 1, "quotient domain must be a proper coset"
    assert quotient_domain.log_n >= trace_domain.log_n

    trace_size = trace_domain.size()
    quot_size = quotient_domain.size()
    rate_bits = quotient_domain.log_n - trace_domain.log_n
    rate = 1 << rate_bits

    # --- Z_H values ---
    # Z_H(x) for the trace subgroup H at points of the quotient domain.
    # The trace domain has shift=1, so Z_H(x) = x^n - 1.
    # For x = quot_shift * quot_gen^i, we have x^n = quot_shift^n * quot_gen^(ni).
    # Since quot_gen is a 2^quot_log_n-th root of unity and n = 2^trace_log_n,
    # quot_gen^n = rate_gen (the 2^rate_bits-th root of unity).
    # So Z_H(x_i) depends only on (i mod rate).
    s_pow_n = pow(quotient_domain.shift, trace_size, p)
    rate_gen = get_omega(rate_bits)
    z_h_short = []
    rate_pow = 1  # rate_gen^i
    for i in range(rate):
        z_h_val = (s_pow_n * rate_pow - 1) % p
        z_h_short.append(z_h_val)
        rate_pow = (rate_pow * rate_gen) % p

    # --- Quotient domain points ---
    # x_i = quot_shift * quot_gen^i
    quot_gen = quotient_domain.gen()
    xs = []
    x = quotient_domain.shift
    for i in range(quot_size):
        xs.append(x)
        x = (x * quot_gen) % p

    is_first_row = _single_point_selector(0, xs, z_h_short, rate, quot_size, trace_domain)
    is_last_row = _single_point_selector(
        trace_size - 1, xs, z_h_short, rate, quot_size, trace_domain,
    )

    # --- is_transition ---
    # is_transition[i] = x_i - omega^{-1}
    subgroup_last = inv_mod(trace_domain.gen())
    is_transition = [(x - subgroup_last) % p for x in xs]

    # --- inv_zeroifier ---
    # inv(Z_H) for the short cycle, then extend
    inv_z_h_short = batch_inverse_base(z_h_short)
    # Cycle over the full quotient domain
    inv_zeroifier = [inv_z_h_short[i % rate] for i in range(quot_size)]

    return CosetSelectors(
        is_first_row=is_first_row,
        is_last_row=is_last_row,
        is_transition=is_transition,
        inv_zeroifier=inv_zeroifier,
    )


# ---------------------------------------------------------------------------
# Symbolic constraint evaluation
# ---------------------------------------------------------------------------


def eval_symbolic_expression_dag(
    dag: SymbolicExpressionDag,
    local_row: list[Fe],
    next_row: list[Fe],
    public_values: list[Fe],
    is_first_row: Fe,
    is_last_row: Fe,
    is_transition: Fe,
) -> list[Fe]:
    """Evaluate all nodes of a symbolic expression DAG at a given row.

    Evaluates in topological order (nodes reference only earlier nodes).
    All arithmetic is in the base field.

    This is the simplified version for AIRs without preprocessed traces,
    permutation arguments, or challenges (e.g., FibonacciAir).

    Reference:
        stark-backend/src/prover/cpu/quotient/evaluator.rs
        ProverConstraintEvaluator::eval_nodes_mut (lines 175-217)

    Args:
        dag: The symbolic expression DAG.
        local_row: Trace values at the current row.
        next_row: Trace values at the next row.
        public_values: Public input values.
        is_first_row: Selector value for first row.
        is_last_row: Selector value for last row.
        is_transition: Selector value for transition.

    Returns:
        Evaluated values for every node in the DAG (base field).
    """
    node_values: list[Fe] = [0] * len(dag.nodes)

    for i, node in enumerate(dag.nodes):
        kind = node.kind

        if kind == SymbolicNodeKind.VARIABLE:
            var = node.variable
            assert var is not None
            entry = var.entry
            if entry.kind == EntryType.MAIN:
                if entry.offset == 0:
                    node_values[i] = local_row[var.index] % p
                elif entry.offset == 1:
                    node_values[i] = next_row[var.index] % p
                else:
                    raise ValueError(f"Unsupported main offset: {entry.offset}")
            elif entry.kind == EntryType.PUBLIC:
                node_values[i] = public_values[var.index] % p
            elif entry.kind == EntryType.PREPROCESSED:
                # For preprocessed traces — not supported in simple AIRs
                raise NotImplementedError("Preprocessed traces not yet supported")
            elif entry.kind == EntryType.PERMUTATION:
                raise NotImplementedError("Permutation traces not yet supported")
            elif entry.kind == EntryType.CHALLENGE:
                raise NotImplementedError("Challenge variables not yet supported")
            elif entry.kind == EntryType.EXPOSED:
                raise NotImplementedError("Exposed values not yet supported")
            else:
                raise ValueError(f"Unknown entry kind: {entry.kind}")

        elif kind == SymbolicNodeKind.CONSTANT:
            node_values[i] = node.constant_value % p

        elif kind == SymbolicNodeKind.IS_FIRST_ROW:
            node_values[i] = is_first_row % p

        elif kind == SymbolicNodeKind.IS_LAST_ROW:
            node_values[i] = is_last_row % p

        elif kind == SymbolicNodeKind.IS_TRANSITION:
            node_values[i] = is_transition % p

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


def eval_symbolic_expression_dag_full(
    dag: SymbolicExpressionDag,
    partitioned_local: list[list[Fe]],
    partitioned_next: list[list[Fe]],
    public_values: list[Fe],
    is_first_row: Fe,
    is_last_row: Fe,
    is_transition: Fe,
    preprocessed_local: list[Fe],
    preprocessed_next: list[Fe],
    after_challenge_local: list[EF4Coeffs] | None = None,
    after_challenge_next: list[EF4Coeffs] | None = None,
    challenges: list[list[EF4Coeffs]] | None = None,
    exposed_values: list[list[EF4Coeffs]] | None = None,
) -> list[EF4Coeffs]:
    """Evaluate all DAG nodes for multi-AIR quotient computation, in EF4.

    Handles all entry types: PREPROCESSED, MAIN, PUBLIC, PERMUTATION,
    CHALLENGE, EXPOSED.  All arithmetic is in EF4 since PERMUTATION/CHALLENGE/
    EXPOSED variables are inherently extension field.

    For MAIN, partitioned_local[part_index][col_index] gives the base field
    value for the current row.  part_index selects the trace partition
    (cached mains first, then common main).

    For PERMUTATION, after_challenge_local[index] and after_challenge_next[index]
    are EF4 values indexed by extension field column number.

    Reference:
        stark-backend/src/prover/cpu/quotient/evaluator.rs
        ProverConstraintEvaluator (eval_var + eval_nodes_mut)
    """
    node_values: list[EF4Coeffs] = [[0, 0, 0, 0]] * len(dag.nodes)

    for i, node in enumerate(dag.nodes):
        kind = node.kind

        if kind == SymbolicNodeKind.VARIABLE:
            var = node.variable
            entry = var.entry
            if entry.kind == EntryType.MAIN:
                if entry.offset == 0:
                    node_values[i] = ef4_from_base(partitioned_local[entry.part_index][var.index])
                else:
                    node_values[i] = ef4_from_base(partitioned_next[entry.part_index][var.index])
            elif entry.kind == EntryType.PREPROCESSED:
                if entry.offset == 0:
                    node_values[i] = ef4_from_base(preprocessed_local[var.index])
                else:
                    node_values[i] = ef4_from_base(preprocessed_next[var.index])
            elif entry.kind == EntryType.PUBLIC:
                node_values[i] = ef4_from_base(public_values[var.index])
            elif entry.kind == EntryType.PERMUTATION:
                if entry.offset == 0:
                    node_values[i] = list(after_challenge_local[var.index])
                else:
                    node_values[i] = list(after_challenge_next[var.index])
            elif entry.kind == EntryType.CHALLENGE:
                node_values[i] = list(challenges[0][var.index])
            elif entry.kind == EntryType.EXPOSED:
                node_values[i] = list(exposed_values[0][var.index])
            else:
                raise ValueError(f"Unknown entry kind: {entry.kind}")

        elif kind == SymbolicNodeKind.CONSTANT:
            node_values[i] = ef4_from_base(node.constant_value)

        elif kind == SymbolicNodeKind.IS_FIRST_ROW:
            node_values[i] = ef4_from_base(is_first_row)

        elif kind == SymbolicNodeKind.IS_LAST_ROW:
            node_values[i] = ef4_from_base(is_last_row)

        elif kind == SymbolicNodeKind.IS_TRANSITION:
            node_values[i] = ef4_from_base(is_transition)

        elif kind == SymbolicNodeKind.ADD:
            node_values[i] = ef4_add(node_values[node.left_idx], node_values[node.right_idx])

        elif kind == SymbolicNodeKind.SUB:
            node_values[i] = ef4_sub(node_values[node.left_idx], node_values[node.right_idx])

        elif kind == SymbolicNodeKind.MUL:
            node_values[i] = ef4_mul(node_values[node.left_idx], node_values[node.right_idx])

        elif kind == SymbolicNodeKind.NEG:
            node_values[i] = ef4_neg(node_values[node.idx])

        else:
            raise ValueError(f"Unknown node kind: {kind}")

    return node_values


def accumulate_constraints_ef4(
    dag: SymbolicExpressionDag,
    node_values: list[EF4Coeffs],
    alpha: EF4Coeffs,
) -> EF4Coeffs:
    """Fold constraint values using powers of alpha (EF4 variant).

    Same as accumulate_constraints but node_values are already EF4.

    Reference:
        stark-backend/src/prover/cpu/quotient/evaluator.rs accumulate (lines 229-247)
    """
    num_constraints = len(dag.constraint_idx)
    alpha_powers: list[EF4Coeffs] = []
    current = [1, 0, 0, 0]
    for _ in range(num_constraints):
        alpha_powers.append(current)
        current = ef4_mul(current, alpha)

    accumulator: EF4Coeffs = [0, 0, 0, 0]
    for alpha_pow, node_idx in zip(alpha_powers, reversed(dag.constraint_idx)):
        constraint_val = node_values[node_idx]
        term = ef4_mul(alpha_pow, constraint_val)
        accumulator = ef4_add(accumulator, term)

    return accumulator


def accumulate_constraints(
    dag: SymbolicExpressionDag,
    node_values: list[Fe],
    alpha: EF4Coeffs,
) -> EF4Coeffs:
    """Fold constraint values using powers of alpha.

    Computes sum_{k} alpha^k * constraint_value[k], where the constraints
    are indexed by dag.constraint_idx in reverse order (highest power first)
    to match the Rust evaluator.

    Reference:
        stark-backend/src/prover/cpu/quotient/evaluator.rs
        ProverConstraintEvaluator::accumulate (lines 229-247)

    Args:
        dag: The symbolic expression DAG.
        node_values: Evaluated values for all DAG nodes (from eval_symbolic_expression_dag).
        alpha: The alpha challenge (extension field element).

    Returns:
        The folded constraint value (extension field element).
    """
    num_constraints = len(dag.constraint_idx)

    # Precompute alpha powers: alpha^0, alpha^1, ..., alpha^{num_constraints-1}
    alpha_powers: list[EF4Coeffs] = []
    current = [1, 0, 0, 0]
    for _ in range(num_constraints):
        alpha_powers.append(current)
        current = ef4_mul(current, alpha)

    # Accumulate: alpha_powers[k] is paired with constraint_idx in reverse order
    # This matches the Rust code: zip(alpha_powers, constraint_idx.iter().rev())
    accumulator: EF4Coeffs = [0, 0, 0, 0]
    for alpha_pow, node_idx in zip(alpha_powers, reversed(dag.constraint_idx)):
        constraint_val = node_values[node_idx]
        # constraint_val is base field; lift to EF4 and multiply by alpha_pow (EF4)
        term = ef4_mul_base(alpha_pow, constraint_val)
        accumulator = ef4_add(accumulator, term)

    return accumulator


# ---------------------------------------------------------------------------
# Main quotient computation
# ---------------------------------------------------------------------------


def compute_quotient_values(
    trace_on_quotient_domain: list[list[Fe]],
    constraints_dag: SymbolicExpressionDag,
    public_values: list[Fe],
    alpha: EF4Coeffs,
    quotient_domain: TwoAdicMultiplicativeCoset,
    trace_domain: TwoAdicMultiplicativeCoset,
    preprocessed_on_quot: list[list[Fe]] | None = None,
    after_challenge_on_quot: list[list[EF4Coeffs]] | None = None,
    challenges: list[list[EF4Coeffs]] | None = None,
    exposed_values: list[list[EF4Coeffs]] | None = None,
    partitioned_trace_on_quot: list[list[list[Fe]]] | None = None,
) -> list[EF4Coeffs]:
    """Compute quotient polynomial evaluations on the quotient domain.

    For each point x in the quotient domain:
    1. Look up precomputed selectors (is_first_row, is_last_row, is_transition).
    2. Get the local row and next row from the extended trace.
       The "next" row uses the trace-domain step size within the quotient domain:
       next[i] = trace[(i + quotient_size / trace_size) % quotient_size].
    3. Evaluate all symbolic constraints at (local, next, public_values, selectors).
    4. Fold constraints: accumulator = sum(alpha^k * C_k).
    5. Divide by vanishing polynomial: quotient[x] = accumulator * inv_zeroifier[x].

    When preprocessed/after_challenge/challenges/exposed_values are provided,
    uses the full EF4 evaluator to handle PERMUTATION/CHALLENGE/EXPOSED variables.
    Otherwise, uses the base field evaluator for simple AIRs.

    Reference:
        stark-backend/src/prover/cpu/quotient/single.rs
        compute_single_rap_quotient_values (lines 60-346)

    Args:
        trace_on_quotient_domain: Extended trace as list of rows on quotient domain.
        constraints_dag: Symbolic expression DAG for the AIR constraints.
        public_values: Public input values.
        alpha: Alpha challenge for constraint folding (extension field element).
        quotient_domain: The quotient domain (disjoint coset).
        trace_domain: The trace domain (shift=1 subgroup).
        preprocessed_on_quot: Preprocessed trace on quotient domain (optional).
        after_challenge_on_quot: After-challenge trace on quotient domain as
            [rows][perm_width] of EF4Coeffs (optional).
        challenges: Challenges per phase (optional).
        exposed_values: Exposed values per phase (optional).

    Returns:
        List of EF4 values, one per quotient domain point.
    """
    quot_size = quotient_domain.size()
    trace_size = trace_domain.size()
    step = quot_size // trace_size  # quotient_degree

    use_full_evaluator = after_challenge_on_quot is not None
    if use_full_evaluator:
        # Vectorized EF4 evaluator for multi-AIR with interactions
        return _compute_quotient_values_vectorized(
            trace_on_quotient_domain, constraints_dag, public_values, alpha,
            quotient_domain, trace_domain,
            preprocessed_on_quot, after_challenge_on_quot,
            challenges, exposed_values,
            partitioned_trace_on_quot,
        )

    # Base field evaluator for simple AIRs (e.g., fibonacci_stark)
    sels = selectors_on_coset(trace_domain, quotient_domain)

    quotient_values: list[EF4Coeffs] = []
    for i in range(quot_size):
        local_row = trace_on_quotient_domain[i]
        next_idx = (i + step) % quot_size
        next_row = trace_on_quotient_domain[next_idx]

        node_values = eval_symbolic_expression_dag(
            constraints_dag,
            local_row,
            next_row,
            public_values,
            sels.is_first_row[i],
            sels.is_last_row[i],
            sels.is_transition[i],
        )
        accumulated = accumulate_constraints(constraints_dag, node_values, alpha)

        quotient_val = ef4_mul_base(accumulated, sels.inv_zeroifier[i])
        quotient_values.append(quotient_val)

    return quotient_values


def _prepare_trace_columns(
    partitioned_trace_on_quot: list[list[list[Fe]]] | None,
    trace_on_quotient_domain: list[list[Fe]],
    preprocessed_on_quot: list[list[Fe]] | None,
    after_challenge_on_quot: list[list[EF4Coeffs]] | None,
    quot_size: int,
    step: int,
) -> tuple[
    list[list[FF]],
    list[list[FF]],
    list[FF] | None,
    list[FF] | None,
    list[EF4Vec] | None,
    list[EF4Vec] | None,
]:
    """Convert row-major traces to column-major FF/EF4Vec arrays with next-row shifts.

    Returns:
        (ff_parts_local, ff_parts_next,
         ff_prep_local, ff_prep_next,
         ef4_ac_local, ef4_ac_next)
    """
    # --- Convert partitioned main traces to FF column arrays ---
    ff_parts_local: list[list[FF]] = []
    ff_parts_next: list[list[FF]] = []

    if partitioned_trace_on_quot is not None:
        for part in partitioned_trace_on_quot:
            cols = len(part[0]) if part else 0
            local_cols: list[FF] = []
            next_cols: list[FF] = []
            for c in range(cols):
                col_data = ff_column([part[r][c] for r in range(quot_size)])
                local_cols.append(col_data)
                next_cols.append(ff_roll(col_data, -step))
            ff_parts_local.append(local_cols)
            ff_parts_next.append(next_cols)
    else:
        cols = len(trace_on_quotient_domain[0]) if trace_on_quotient_domain else 0
        local_cols = []
        next_cols = []
        for c in range(cols):
            col_data = ff_column(
                [trace_on_quotient_domain[r][c] for r in range(quot_size)]
            )
            local_cols.append(col_data)
            next_cols.append(ff_roll(col_data, -step))
        ff_parts_local.append(local_cols)
        ff_parts_next.append(next_cols)

    # --- Convert preprocessed trace ---
    ff_prep_local: list[FF] | None = None
    ff_prep_next: list[FF] | None = None
    if preprocessed_on_quot is not None:
        prep_cols = len(preprocessed_on_quot[0]) if preprocessed_on_quot else 0
        ff_prep_local = []
        ff_prep_next = []
        for c in range(prep_cols):
            col_data = ff_column(
                [preprocessed_on_quot[r][c] for r in range(quot_size)]
            )
            ff_prep_local.append(col_data)
            ff_prep_next.append(ff_roll(col_data, -step))

    # --- Convert after_challenge trace (EF4 columns) ---
    ef4_ac_local: list[EF4Vec] | None = None
    ef4_ac_next: list[EF4Vec] | None = None
    if after_challenge_on_quot is not None:
        perm_width = len(after_challenge_on_quot[0])
        ef4_ac_local = []
        ef4_ac_next = []
        for col in range(perm_width):
            ef4_col = ef4v_from_rows(
                [after_challenge_on_quot[r][col] for r in range(quot_size)]
            )
            ef4_ac_local.append(ef4_col)
            ef4_ac_next.append(ef4v_roll(ef4_col, -step))

    return (
        ff_parts_local, ff_parts_next,
        ff_prep_local, ff_prep_next,
        ef4_ac_local, ef4_ac_next,
    )


def _eval_dag_vectorized(
    dag: SymbolicExpressionDag,
    ff_parts_local: list[list[FF]],
    ff_parts_next: list[list[FF]],
    ff_prep_local: list[FF] | None,
    ff_prep_next: list[FF] | None,
    ef4_ac_local: list[EF4Vec] | None,
    ef4_ac_next: list[EF4Vec] | None,
    is_first_row_ff: FF,
    is_last_row_ff: FF,
    is_transition_ff: FF,
    public_values: list[Fe],
    challenges: list[list[EF4Coeffs]] | None,
    exposed_values: list[list[EF4Coeffs]] | None,
    quot_size: int,
) -> list[EF4Vec | FF | None]:
    """Evaluate DAG nodes vectorized: all quotient domain points at once.

    Returns per-node column arrays (EF4Vec or FF).
    """
    nodes = dag.nodes
    node_values: list[EF4Vec | FF | None] = [None] * len(nodes)

    for i, node in enumerate(nodes):
        kind = node.kind

        if kind == SymbolicNodeKind.VARIABLE:
            var = node.variable
            entry = var.entry
            if entry.kind == EntryType.MAIN:
                if entry.offset == 0:
                    node_values[i] = ef4v_from_base(ff_parts_local[entry.part_index][var.index])
                else:
                    node_values[i] = ef4v_from_base(ff_parts_next[entry.part_index][var.index])
            elif entry.kind == EntryType.PREPROCESSED:
                if entry.offset == 0:
                    node_values[i] = ef4v_from_base(ff_prep_local[var.index])
                else:
                    node_values[i] = ef4v_from_base(ff_prep_next[var.index])
            elif entry.kind == EntryType.PUBLIC:
                node_values[i] = ef4v_from_scalar(
                    [public_values[var.index] % p, 0, 0, 0], quot_size
                )
            elif entry.kind == EntryType.PERMUTATION:
                if entry.offset == 0:
                    node_values[i] = ef4_ac_local[var.index]
                else:
                    node_values[i] = ef4_ac_next[var.index]
            elif entry.kind == EntryType.CHALLENGE:
                node_values[i] = ef4v_from_scalar(challenges[0][var.index], quot_size)
            elif entry.kind == EntryType.EXPOSED:
                node_values[i] = ef4v_from_scalar(exposed_values[0][var.index], quot_size)
            else:
                raise ValueError(f"Unknown entry kind: {entry.kind}")

        elif kind == SymbolicNodeKind.CONSTANT:
            node_values[i] = ef4v_from_scalar(
                [node.constant_value % p, 0, 0, 0], quot_size
            )

        elif kind == SymbolicNodeKind.IS_FIRST_ROW:
            node_values[i] = ef4v_from_base(is_first_row_ff)

        elif kind == SymbolicNodeKind.IS_LAST_ROW:
            node_values[i] = ef4v_from_base(is_last_row_ff)

        elif kind == SymbolicNodeKind.IS_TRANSITION:
            node_values[i] = ef4v_from_base(is_transition_ff)

        elif kind == SymbolicNodeKind.ADD:
            node_values[i] = ef4v_add(node_values[node.left_idx], node_values[node.right_idx])

        elif kind == SymbolicNodeKind.SUB:
            node_values[i] = ef4v_sub(node_values[node.left_idx], node_values[node.right_idx])

        elif kind == SymbolicNodeKind.MUL:
            node_values[i] = ef4v_mul(node_values[node.left_idx], node_values[node.right_idx])

        elif kind == SymbolicNodeKind.NEG:
            node_values[i] = ef4v_neg(node_values[node.idx])

        else:
            raise ValueError(f"Unknown node kind: {kind}")

    return node_values


def _accumulate_and_divide(
    dag: SymbolicExpressionDag,
    node_values: list[EF4Vec | FF | None],
    alpha: EF4Coeffs,
    inv_zeroifier_ff: FF,
    quot_size: int,
) -> list[EF4Coeffs]:
    """Alpha-weighted accumulation of constraint values and vanishing poly division.

    Returns quotient values as a list of EF4Coeffs rows.
    """
    num_constraints = len(dag.constraint_idx)
    alpha_powers: list[EF4Coeffs] = []
    current_alpha: EF4Coeffs = [1, 0, 0, 0]
    for _ in range(num_constraints):
        alpha_powers.append(current_alpha)
        current_alpha = ef4_mul(current_alpha, alpha)

    acc = ef4v_zeros(quot_size)
    for alpha_pow, node_idx in zip(alpha_powers, reversed(dag.constraint_idx)):
        term = ef4v_mul_scalar(node_values[node_idx], alpha_pow)
        acc = ef4v_add(acc, term)

    # Divide by vanishing polynomial
    result = ef4v_mul_base(acc, inv_zeroifier_ff)

    return ef4v_to_rows(result)


def _compute_quotient_values_vectorized(
    trace_on_quotient_domain: list[list[Fe]],
    constraints_dag: SymbolicExpressionDag,
    public_values: list[Fe],
    alpha: EF4Coeffs,
    quotient_domain: TwoAdicMultiplicativeCoset,
    trace_domain: TwoAdicMultiplicativeCoset,
    preprocessed_on_quot: list[list[Fe]] | None,
    after_challenge_on_quot: list[list[EF4Coeffs]] | None,
    challenges: list[list[EF4Coeffs]] | None,
    exposed_values: list[list[EF4Coeffs]] | None,
    partitioned_trace_on_quot: list[list[list[Fe]]] | None,
) -> list[EF4Coeffs]:
    """Vectorized quotient computation for multi-AIR with interactions.

    Evaluates the constraint DAG at ALL quotient domain points simultaneously.
    """
    quot_size = quotient_domain.size()
    trace_size = trace_domain.size()
    step = quot_size // trace_size

    # --- Precompute selectors ---
    sels = selectors_on_coset(trace_domain, quotient_domain)
    is_first_row_ff = ff_column(sels.is_first_row)
    is_last_row_ff = ff_column(sels.is_last_row)
    is_transition_ff = ff_column(sels.is_transition)
    inv_zeroifier_ff = ff_column(sels.inv_zeroifier)

    # --- Convert traces to column-major arrays ---
    (
        ff_parts_local, ff_parts_next,
        ff_prep_local, ff_prep_next,
        ef4_ac_local, ef4_ac_next,
    ) = _prepare_trace_columns(
        partitioned_trace_on_quot, trace_on_quotient_domain,
        preprocessed_on_quot, after_challenge_on_quot,
        quot_size, step,
    )

    # --- Evaluate DAG nodes ---
    node_values = _eval_dag_vectorized(
        constraints_dag,
        ff_parts_local, ff_parts_next,
        ff_prep_local, ff_prep_next,
        ef4_ac_local, ef4_ac_next,
        is_first_row_ff, is_last_row_ff, is_transition_ff,
        public_values, challenges, exposed_values,
        quot_size,
    )

    # --- Accumulate and divide by vanishing polynomial ---
    return _accumulate_and_divide(
        constraints_dag, node_values, alpha, inv_zeroifier_ff, quot_size,
    )


# ---------------------------------------------------------------------------
# Split quotient into chunks for commitment
# ---------------------------------------------------------------------------


def quotient_values_to_chunks(
    quotient_values: list[EF4Coeffs],
    num_chunks: int,
) -> list[list[list[Fe]]]:
    """Split quotient evaluations into chunks and flatten to base field.

    The quotient evaluations on the quotient domain (a coset) are "vertically
    strided" into num_chunks sub-cosets.  Row i of the full quotient maps to
    chunk (i % num_chunks), row (i // num_chunks) of that chunk.

    Each chunk row has 4 base field elements (the EF4 coefficients), matching
    the Rust representation where extension field elements are transmuted to
    base field columns.

    Reference:
        p3-commit-0.4.1/src/domain.rs split_evals (lines 188-221)
        stark-backend/src/prover/cpu/quotient/single.rs lines 336-343

    The ordering follows the Rust pattern:
        quotient_values[chunk_idx + row_idx * quotient_degree]
    goes into chunk[chunk_idx][row_idx].

    Args:
        quotient_values: Flat list of EF4 quotient values on the quotient domain.
        num_chunks: Number of chunks (= quotient_degree, a power of 2).

    Returns:
        List of chunks. Each chunk is a list of rows, each row being
        [c0, c1, c2, c3] (the 4 base field coefficients of the EF4 element).
    """
    quot_size = len(quotient_values)
    assert quot_size % num_chunks == 0
    rows_per_chunk = quot_size // num_chunks

    chunks: list[list[list[Fe]]] = [[] for _ in range(num_chunks)]
    for row_idx in range(rows_per_chunk):
        for chunk_idx in range(num_chunks):
            # The quotient domain index for this (chunk_idx, row_idx) pair
            # matches the "vertically strided" layout from Plonky3:
            # full_idx = chunk_idx + row_idx * num_chunks
            full_idx = chunk_idx + row_idx * num_chunks
            ef4_val = quotient_values[full_idx]
            # Flatten EF4 to 4 base field elements
            chunks[chunk_idx].append(list(ef4_val))

    return chunks


# ---------------------------------------------------------------------------
# Complete quotient computation pipeline
# ---------------------------------------------------------------------------


def extend_after_challenge_to_quotient_domain(
    after_challenge_trace: list[list[EF4Coeffs]],
    trace_domain: TwoAdicMultiplicativeCoset,
    quotient_domain: TwoAdicMultiplicativeCoset,
) -> list[list[EF4Coeffs]]:
    """Extend after-challenge trace (EF4 values) to the quotient domain.

    Each EF4 element has 4 base field coefficients. We LDE each coefficient
    column independently, then reconstruct EF4 values at quotient domain points.

    Args:
        after_challenge_trace: [rows][perm_width] of EF4Coeffs on trace domain.
        trace_domain: Trace domain (shift=1 subgroup).
        quotient_domain: Quotient domain (disjoint coset).

    Returns:
        [quot_rows][perm_width] of EF4Coeffs on quotient domain.
    """
    n_trace = trace_domain.size()
    n_quot = quotient_domain.size()
    perm_width = len(after_challenge_trace[0])

    # Flatten: extract 4 base field columns per EF4 column
    # Total base field columns = perm_width * 4
    num_base_cols = perm_width * 4
    base_columns: list[list[Fe]] = [[] for _ in range(num_base_cols)]
    for row in range(n_trace):
        for col in range(perm_width):
            ef4_val = after_challenge_trace[row][col]
            for coeff_idx in range(4):
                base_columns[col * 4 + coeff_idx].append(ef4_val[coeff_idx])

    # LDE each base field column
    lde_columns = [
        coset_lde_column(col, trace_domain, quotient_domain)
        for col in base_columns
    ]

    # Reconstruct EF4 values at quotient domain points
    result: list[list[EF4Coeffs]] = []
    for row in range(n_quot):
        ef4_row: list[EF4Coeffs] = []
        for col in range(perm_width):
            ef4_val = [
                lde_columns[col * 4 + ci][row]
                for ci in range(4)
            ]
            ef4_row.append(ef4_val)
        result.append(ef4_row)

    return result


def compute_quotient_chunks(
    trace: list[list[Fe]],
    constraints_dag: SymbolicExpressionDag,
    public_values: list[Fe],
    alpha: EF4Coeffs,
    trace_domain: TwoAdicMultiplicativeCoset,
    quotient_degree: int,
    preprocessed_trace: list[list[Fe]] | None = None,
    after_challenge_trace: list[list[EF4Coeffs]] | None = None,
    challenges: list[list[EF4Coeffs]] | None = None,
    exposed_values: list[list[EF4Coeffs]] | None = None,
    partitioned_traces: list[list[list[Fe]]] | None = None,
) -> tuple[list[list[list[Fe]]], list[TwoAdicMultiplicativeCoset]]:
    """End-to-end quotient computation: trace -> quotient chunks ready for commitment.

    This combines:
    1. Create the quotient domain (disjoint from trace domain)
    2. LDE the trace to the quotient domain
    3. Compute quotient values on the quotient domain
    4. Split into chunks

    Reference:
        stark-backend/src/prover/cpu/quotient/mod.rs
        QuotientCommitter::single_rap_quotient_values (lines 84-130)

    Args:
        trace: Trace matrix as list of rows on the trace domain.
        constraints_dag: Symbolic expression DAG for the AIR constraints.
        public_values: Public input values.
        alpha: Alpha challenge (extension field element).
        trace_domain: Trace domain (shift=1 subgroup).
        quotient_degree: Factor multiplying trace degree to get quotient degree.
        preprocessed_trace: Preprocessed trace matrix [rows][cols] (optional).
        after_challenge_trace: After-challenge trace [rows][perm_width] of EF4 (optional).
        challenges: Challenges per phase (optional).
        exposed_values: Exposed values per phase (optional).

    Returns:
        Tuple of (chunks, chunk_domains) where:
        - chunks: list of num_chunks matrices, each a list of rows of 4 Fe elements.
        - chunk_domains: the sub-coset domains for each chunk.
    """
    from protocol.domain import create_disjoint_domain

    # Step 1: Create quotient domain
    quotient_domain = create_disjoint_domain(
        trace_domain,
        trace_domain.size() * quotient_degree,
    )

    # Step 2: LDE trace to quotient domain
    trace_on_quot = extend_trace_to_quotient_domain(
        trace, trace_domain, quotient_domain,
    )

    # Step 2b: LDE preprocessed trace if present
    preprocessed_on_quot = None
    if preprocessed_trace is not None:
        preprocessed_on_quot = extend_trace_to_quotient_domain(
            preprocessed_trace, trace_domain, quotient_domain,
        )

    # Step 2c: LDE after_challenge trace if present
    after_challenge_on_quot = None
    if after_challenge_trace is not None:
        after_challenge_on_quot = extend_after_challenge_to_quotient_domain(
            after_challenge_trace, trace_domain, quotient_domain,
        )

    # Step 2d: LDE partitioned traces if present
    partitioned_trace_on_quot = None
    if partitioned_traces is not None:
        partitioned_trace_on_quot = [
            extend_trace_to_quotient_domain(part, trace_domain, quotient_domain)
            for part in partitioned_traces
        ]

    # Step 3: Compute quotient values
    quotient_values = compute_quotient_values(
        trace_on_quot,
        constraints_dag,
        public_values,
        alpha,
        quotient_domain,
        trace_domain,
        preprocessed_on_quot=preprocessed_on_quot,
        after_challenge_on_quot=after_challenge_on_quot,
        challenges=challenges,
        exposed_values=exposed_values,
        partitioned_trace_on_quot=partitioned_trace_on_quot,
    )

    # Step 4: Split into chunks
    chunks = quotient_values_to_chunks(quotient_values, quotient_degree)

    # Step 5: Compute chunk domains
    chunk_domains = quotient_domain.split_domains(quotient_degree)

    return chunks, chunk_domains
