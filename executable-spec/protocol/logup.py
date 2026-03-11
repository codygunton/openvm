"""FriLogUp auxiliary column computation.

Computes the after-challenge trace for the FRI LogUp protocol, which implements
the log-derivative argument for bus interactions in multi-AIR STARKs.

Given interaction definitions from the VK, main/preprocessed trace values, and
random challenges (alpha, beta), this module computes:
1. Reciprocal columns for interaction denominators
2. Chunk sums (bundled reciprocals weighted by counts)
3. Running sum (phi) column for cumulative sum verification

Reference:
    stark-backend/src/interaction/fri_log_up.rs  — generate_after_challenge_trace,
                                                    find_interaction_chunks
    stark-backend/src/interaction/trace.rs       — Evaluator (row-level DAG evaluator)
    stark-backend/src/interaction/utils.rs       — generate_betas
"""

from __future__ import annotations

from primitives.field import (
    BABYBEAR_PRIME,
    EF4Coeffs,
    Fe,
    ef4_add,
    ef4_batch_inverse,
    ef4_from_base,
    ef4_mul,
    ef4_mul_base,
)
from protocol.proof import (
    EntryType,
    Interaction,
    StarkVerifyingKey,
    SymbolicExpressionDag,
    SymbolicExpressionNode,
    SymbolicNodeKind,
)

p = BABYBEAR_PRIME


# ---------------------------------------------------------------------------
# Node degree computation
# ---------------------------------------------------------------------------


def node_degree(node: SymbolicExpressionNode) -> int:
    """Return the degree_multiple of a DAG node.

    Reference:
        stark-backend symbolic_expression.rs SymbolicExpression::degree_multiple
    """
    kind = node.kind
    if kind == SymbolicNodeKind.VARIABLE:
        entry_kind = node.variable.entry.kind
        if entry_kind in (EntryType.MAIN, EntryType.PREPROCESSED, EntryType.PERMUTATION):
            return 1
        return 0  # PUBLIC, CHALLENGE, EXPOSED
    if kind == SymbolicNodeKind.CONSTANT:
        return 0
    if kind == SymbolicNodeKind.IS_FIRST_ROW:
        return 1
    if kind == SymbolicNodeKind.IS_LAST_ROW:
        return 1
    if kind == SymbolicNodeKind.IS_TRANSITION:
        return 0
    # ADD, SUB, MUL, NEG — stored in the node
    return node.degree_multiple


# ---------------------------------------------------------------------------
# Row-level DAG evaluator (base field)
# ---------------------------------------------------------------------------


def eval_dag_at_row(
    dag: SymbolicExpressionDag,
    partitioned_main: list[list[list[Fe]]],
    preprocessed: list[list[Fe]] | None,
    public_values: list[Fe],
    height: int,
    row_idx: int,
) -> list[Fe]:
    """Evaluate all DAG nodes at a specific trace row in the base field.

    Unlike the OOD evaluator (constraints.py) which works in EF4, this evaluator
    works in the base field since we're evaluating at actual trace domain points.

    IsFirstRow/IsLastRow/IsTransition are set to 0 since interaction expressions
    never reference them (the Rust Evaluator marks them as unreachable).

    Args:
        dag: The symbolic expression DAG.
        partitioned_main: [part_index][rows][cols] — cached mains first, then common main.
        preprocessed: [rows][cols] or None.
        public_values: Public input values.
        height: Trace height.
        row_idx: Current row index.

    Returns:
        Evaluated base field values for every node in the DAG.

    Reference:
        stark-backend/src/interaction/trace.rs Evaluator::eval_var
    """
    node_values: list[Fe] = [0] * len(dag.nodes)

    for i, node in enumerate(dag.nodes):
        kind = node.kind

        if kind == SymbolicNodeKind.VARIABLE:
            var = node.variable
            entry = var.entry
            if entry.kind == EntryType.MAIN:
                row = (row_idx + entry.offset) % height
                node_values[i] = partitioned_main[entry.part_index][row][var.index] % p
            elif entry.kind == EntryType.PREPROCESSED:
                row = (row_idx + entry.offset) % height
                node_values[i] = preprocessed[row][var.index] % p
            elif entry.kind == EntryType.PUBLIC:
                node_values[i] = public_values[var.index] % p
            elif entry.kind in (EntryType.PERMUTATION, EntryType.CHALLENGE, EntryType.EXPOSED):
                # These are used by constraint expressions sharing the same DAG,
                # not by interaction message/count paths. Set to 0 so DAG evaluation
                # proceeds (matching IsFirstRow/IsLastRow treatment above).
                node_values[i] = 0
            else:
                raise ValueError(f"Unexpected entry kind in interaction DAG: {entry.kind}")

        elif kind == SymbolicNodeKind.CONSTANT:
            node_values[i] = node.constant_value % p

        elif kind in (
            SymbolicNodeKind.IS_FIRST_ROW,
            SymbolicNodeKind.IS_LAST_ROW,
            SymbolicNodeKind.IS_TRANSITION,
        ):
            # Not used by interaction expressions. Set to 0 so DAG evaluation
            # proceeds for shared nodes used only by constraints.
            node_values[i] = 0

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
# Beta power generation
# ---------------------------------------------------------------------------


def generate_betas(beta: EF4Coeffs, interactions: list[Interaction]) -> list[EF4Coeffs]:
    """Generate [beta^0, beta^1, ..., beta^{max_msg_len}].

    Reference:
        stark-backend/src/interaction/utils.rs generate_betas
    """
    max_msg_len = max((len(inter.message) for inter in interactions), default=0)
    betas: list[EF4Coeffs] = [[1, 0, 0, 0]]
    current: EF4Coeffs = [1, 0, 0, 0]
    for _ in range(max_msg_len):
        current = ef4_mul(current, beta)
        betas.append(current)
    return betas


# ---------------------------------------------------------------------------
# Interaction chunking
# ---------------------------------------------------------------------------


def find_interaction_chunks(
    interactions: list[Interaction],
    dag: SymbolicExpressionDag,
    max_constraint_degree: int,
) -> list[list[int]]:
    """Partition interactions into chunks respecting max constraint degree.

    Returns list of lists of interaction indices.
    Width of after_challenge trace = len(partitions) + 1 (extra phi column).

    Algorithm:
    1. Sort interaction indices by ascending (max_field_degree, count_degree)
    2. Greedily pack: add to current chunk if degree constraint allows
    3. Seal chunk and start new one when degree would be exceeded

    Reference:
        stark-backend/src/interaction/fri_log_up.rs find_interaction_chunks (lines 573-643)
    """
    if not interactions:
        return []

    def max_field_degree(i: int) -> int:
        return max(
            (node_degree(dag.nodes[msg_idx]) for msg_idx in interactions[i].message),
            default=0,
        )

    def count_degree(i: int) -> int:
        return node_degree(dag.nodes[interactions[i].count])

    # Sort by ascending (max_field_degree, count_degree)
    interaction_idxs = list(range(len(interactions)))
    interaction_idxs.sort(key=lambda i: (max_field_degree(i), count_degree(i)))

    # Greedily pack into chunks
    running_sum_field_degree = 0
    numerator_max_degree = 0
    interaction_partitions: list[list[int]] = []
    cur_chunk: list[int] = []

    for interaction_idx in interaction_idxs:
        field_deg = max_field_degree(interaction_idx)
        count_deg = count_degree(interaction_idx)

        new_num_max_degree = max(
            numerator_max_degree + field_deg,
            count_deg + running_sum_field_degree,
        )
        new_denom_degree = running_sum_field_degree + field_deg

        if max(new_num_max_degree, new_denom_degree + 1) <= max_constraint_degree:
            # Include in current chunk
            cur_chunk.append(interaction_idx)
            numerator_max_degree = new_num_max_degree
            running_sum_field_degree += field_deg
        else:
            # Seal current chunk and start new one
            if cur_chunk:
                interaction_partitions.append(cur_chunk)
                cur_chunk = []
            cur_chunk.append(interaction_idx)
            numerator_max_degree = count_deg
            running_sum_field_degree = field_deg

    # Seal the last chunk
    if cur_chunk:
        interaction_partitions.append(cur_chunk)

    return interaction_partitions


# ---------------------------------------------------------------------------
# Max constraint degree
# ---------------------------------------------------------------------------


def compute_max_constraint_degree(per_air_vks: list[StarkVerifyingKey]) -> int:
    """Compute max constraint degree across all AIRs from their DAGs.

    This is the maximum degree_multiple of any constraint output node across all AIRs.

    Reference:
        stark-backend/src/prover/coordinator.rs max_constraint_degree computation
    """
    max_deg = 0
    for svk in per_air_vks:
        dag = svk.symbolic_constraints.constraints
        for constraint_idx in dag.constraint_idx:
            deg = node_degree(dag.nodes[constraint_idx])
            max_deg = max(max_deg, deg)
    return max_deg


# ---------------------------------------------------------------------------
# Core: after-challenge trace computation
# ---------------------------------------------------------------------------


_NUMPY_THRESHOLD = 256  # Use numpy vectorization for traces larger than this


def compute_after_challenge_trace(
    interactions: list[Interaction],
    interaction_partitions: list[list[int]],
    dag: SymbolicExpressionDag,
    partitioned_main: list[list[list[Fe]]],
    preprocessed: list[list[Fe]] | None,
    public_values: list[Fe],
    alpha: EF4Coeffs,
    beta: EF4Coeffs,
    height: int,
) -> tuple[list[list[EF4Coeffs]], EF4Coeffs]:
    """Compute FriLogUp after-challenge trace and cumulative sum.

    For each row:
    1. Evaluate full DAG at the row (base field)
    2. Compute denominator per interaction:
       denom_i = alpha + msg[0] + beta*msg[1] + ... + beta^{|msg|}*(bus_index+1)
    3. Batch-invert all denominators
    4. For each chunk: perm[chunk] = sum(reciprocal_i * count_i)
    5. phi = sum of all chunk values (row sum)
    Then convert phi column to running sum (prefix sum).

    Args:
        interactions: List of Interaction (message/count are DAG node indices).
        interaction_partitions: Chunking of interaction indices.
        dag: Symbolic expression DAG.
        partitioned_main: [part_index][rows][cols].
        preprocessed: [rows][cols] or None.
        public_values: Public input values.
        alpha: First interaction challenge (EF4).
        beta: Second interaction challenge (EF4).
        height: Trace height.

    Returns:
        (perm_trace, cumulative_sum) where:
        - perm_trace: [height][perm_width] of EF4 values
        - perm_width = len(interaction_partitions) + 1
        - cumulative_sum: EF4 value

    Reference:
        stark-backend/src/interaction/fri_log_up.rs
        generate_after_challenge_trace (lines 299-437)
    """
    if height >= _NUMPY_THRESHOLD:
        from protocol.vectorized import compute_after_challenge_trace_numpy
        return compute_after_challenge_trace_numpy(
            interactions, interaction_partitions, dag,
            partitioned_main, preprocessed, public_values,
            alpha, beta, height,
        )

    perm_width = len(interaction_partitions) + 1  # +1 for phi column
    betas = generate_betas(beta, interactions)

    zero_ef4: EF4Coeffs = [0, 0, 0, 0]
    perm_trace: list[list[EF4Coeffs]] = [
        [list(zero_ef4) for _ in range(perm_width)] for _ in range(height)
    ]

    for n in range(height):
        # Step 1: Evaluate full DAG at this row
        dag_vals = eval_dag_at_row(
            dag, partitioned_main, preprocessed, public_values, height, n
        )

        # Step 2: Compute denominators for all interactions
        denoms: list[EF4Coeffs] = []
        for interaction in interactions:
            msg = interaction.message
            # denom = alpha + eval(msg[0]) + betas[1]*eval(msg[1]) + ...
            #       + betas[msg_len] * (bus_index + 1)
            denom = ef4_add(alpha, ef4_from_base(dag_vals[msg[0]]))
            for j in range(1, len(msg)):
                denom = ef4_add(denom, ef4_mul_base(betas[j], dag_vals[msg[j]]))
            denom = ef4_add(denom, ef4_mul_base(betas[len(msg)], interaction.bus_index + 1))
            denoms.append(denom)

        # Step 3: Batch invert
        reciprocals = ef4_batch_inverse(denoms)

        # Step 4: Compute chunk values and row sum
        row_sum: EF4Coeffs = list(zero_ef4)
        for chunk_idx, partition in enumerate(interaction_partitions):
            perm_val: EF4Coeffs = list(zero_ef4)
            for interaction_idx in partition:
                count_val = dag_vals[interactions[interaction_idx].count]
                interaction_val = ef4_mul_base(reciprocals[interaction_idx], count_val)
                perm_val = ef4_add(perm_val, interaction_val)
            perm_trace[n][chunk_idx] = perm_val
            row_sum = ef4_add(row_sum, perm_val)

        # phi column = row sum (will become running sum below)
        perm_trace[n][perm_width - 1] = row_sum

    # Step 5: Convert phi column to running sum (prefix sum)
    phi: EF4Coeffs = list(zero_ef4)
    for n in range(height):
        phi = ef4_add(phi, perm_trace[n][perm_width - 1])
        perm_trace[n][perm_width - 1] = list(phi)

    cumulative_sum = perm_trace[height - 1][perm_width - 1]

    return perm_trace, cumulative_sum
