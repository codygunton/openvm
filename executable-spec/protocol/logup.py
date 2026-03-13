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

import numpy as np

from primitives.field import (
    BABYBEAR_PRIME,
    FF4,
    FF,
    Fe,
)
from protocol.constraints import eval_dag_all_rows
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

    Unlike the OOD evaluator (constraints.py) which works in FF4, this evaluator
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


def generate_betas(beta: FF4, interactions: list[Interaction]) -> list[FF4]:
    """Generate [beta^0, beta^1, ..., beta^{max_msg_len}].

    Reference:
        stark-backend/src/interaction/utils.rs generate_betas
    """
    max_msg_len = max((len(inter.message) for inter in interactions), default=0)
    betas: list[FF4] = [FF4.one()]
    current = FF4.one()
    for _ in range(max_msg_len):
        current = current * FF4(beta)
        betas.append(current)
    return betas


# ---------------------------------------------------------------------------
# Interaction chunking
# ---------------------------------------------------------------------------


def _max_field_degree(
    interaction_idx: int,
    interactions: list[Interaction],
    dag: SymbolicExpressionDag,
) -> int:
    """Max degree among message fields for an interaction."""
    return max(
        (node_degree(dag.nodes[msg_idx]) for msg_idx in interactions[interaction_idx].message),
        default=0,
    )


def _count_degree(
    interaction_idx: int,
    interactions: list[Interaction],
    dag: SymbolicExpressionDag,
) -> int:
    """Degree of the count field for an interaction."""
    return node_degree(dag.nodes[interactions[interaction_idx].count])


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

    # Sort by ascending (max_field_degree, count_degree)
    interaction_idxs = list(range(len(interactions)))
    interaction_idxs.sort(
        key=lambda i: (_max_field_degree(i, interactions, dag), _count_degree(i, interactions, dag))
    )

    # Greedily pack into chunks
    running_sum_field_degree = 0
    numerator_max_degree = 0
    interaction_partitions: list[list[int]] = []
    cur_chunk: list[int] = []

    for interaction_idx in interaction_idxs:
        field_deg = _max_field_degree(interaction_idx, interactions, dag)
        count_deg = _count_degree(interaction_idx, interactions, dag)

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


# <doc-anchor id="compute-logup">
def compute_after_challenge_trace(
    interactions: list[Interaction],
    interaction_partitions: list[list[int]],
    dag: SymbolicExpressionDag,
    partitioned_main: list[list[list[Fe]]],
    preprocessed: list[list[Fe]] | None,
    public_values: list[Fe],
    alpha: FF4,
    beta: FF4,
    height: int,
) -> tuple[list[list[list[int]]], list[int]]:
    """Compute FriLogUp after-challenge trace and cumulative sum.

    Processes ALL rows simultaneously using vectorized field arrays.

    Args:
        interactions: List of Interaction (message/count are DAG node indices).
        interaction_partitions: Chunking of interaction indices.
        dag: Symbolic expression DAG.
        partitioned_main: [part_index][rows][cols].
        preprocessed: [rows][cols] or None.
        public_values: Public input values.
        alpha: First interaction challenge (FF4).
        beta: Second interaction challenge (FF4).
        height: Trace height.

    Returns:
        (perm_trace, cumulative_sum) where:
        - perm_trace: [height][perm_width] of FF4 values (as list[int])
        - perm_width = len(interaction_partitions) + 1
        - cumulative_sum: FF4 value (as list[int])

    Reference:
        stark-backend/src/interaction/fri_log_up.rs
        generate_after_challenge_trace (lines 299-437)
    """
    betas = generate_betas(beta, interactions)

    # Step 1: Evaluate full DAG at all rows (vectorized)
    node_values = eval_dag_all_rows(
        dag, partitioned_main, preprocessed, public_values, height
    )

    # Step 2: Compute FF4 denominators for all interactions, all rows
    alpha_v = FF4.broadcast(FF4(alpha), height)

    all_denoms = []
    for interaction in interactions:
        msg = interaction.message
        denom = alpha_v + FF4.from_base(node_values[msg[0]])
        for j in range(1, len(msg)):
            beta_j = FF4.broadcast(betas[j], height)
            denom = denom + beta_j.mul_base(node_values[msg[j]])
        beta_last = FF4.broadcast(betas[len(msg)], height)
        bus_val = FF(np.full(height, (interaction.bus_index + 1) % p))
        denom = denom + beta_last.mul_base(bus_val)
        all_denoms.append(denom)

    # Step 3: Batch invert all denominators (vectorized)
    all_reciprocals = [d.inv() for d in all_denoms]

    # Step 4: Compute chunk values for all rows
    perm_chunks = []
    for partition in interaction_partitions:
        chunk_sum = FF4.zeros(height)
        for interaction_idx in partition:
            count_vals = node_values[interactions[interaction_idx].count]
            term = all_reciprocals[interaction_idx].mul_base(count_vals)
            chunk_sum = chunk_sum + term
        perm_chunks.append(chunk_sum)

    # Compute phi (row sum of all chunks)
    phi = FF4.zeros(height)
    for chunk in perm_chunks:
        phi = phi + chunk

    # Step 5: Convert phi to running sum (prefix sum)
    running_sum = phi.cumsum()

    # Convert back to list-of-lists format expected by caller
    chunk_rows = [chunk.to_rows() for chunk in perm_chunks]
    running_sum_rows = running_sum.to_rows()

    perm_trace = []
    for row in range(height):
        row_data = [chunk_rows[c][row] for c in range(len(perm_chunks))]
        row_data.append(running_sum_rows[row])
        perm_trace.append(row_data)

    cumulative_sum = running_sum_rows[height - 1]

    return perm_trace, cumulative_sum
