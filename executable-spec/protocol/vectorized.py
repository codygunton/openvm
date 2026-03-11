"""Numpy-vectorized hot paths for the STARK prover.

Provides vectorized versions of the logup DAG evaluation and EF4 batch
operations that process ALL rows simultaneously using numpy arrays instead
of Python loops. This gives 100-1000x speedups for large traces.

All arithmetic is BabyBear modular arithmetic on numpy int64 arrays.
EF4 elements are represented as 4 separate numpy int64 arrays (one per
coefficient of the quartic extension x^4 - 11).
"""

from __future__ import annotations

import numpy as np

from primitives.field import BABYBEAR_PRIME

p = BABYBEAR_PRIME
W = 11  # x^4 - W = 0, the extension polynomial constant


# ---------------------------------------------------------------------------
# Vectorized base field arithmetic on numpy int64 arrays
# ---------------------------------------------------------------------------


def _mod(a: np.ndarray) -> np.ndarray:
    """Reduce numpy int64 array mod p."""
    return a % p


def _mul(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """Multiply two numpy int64 arrays mod p."""
    return (a * b) % p


def _add(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """Add two numpy int64 arrays mod p."""
    return (a + b) % p


def _sub(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """Subtract two numpy int64 arrays mod p."""
    return (a - b) % p


def _neg(a: np.ndarray) -> np.ndarray:
    """Negate numpy int64 array mod p."""
    return (p - a) % p


def _modpow(base: np.ndarray, exp: int) -> np.ndarray:
    """Modular exponentiation for numpy int64 arrays: base^exp mod p.

    Uses square-and-multiply. Each step is a vectorized numpy operation.
    For exp = p-2, this takes ~31 iterations.
    """
    result = np.ones_like(base)
    base = base % p
    while exp > 0:
        if exp & 1:
            result = (result * base) % p
        base = (base * base) % p
        exp >>= 1
    return result


def _inv(a: np.ndarray) -> np.ndarray:
    """Modular inverse for numpy int64 arrays: a^{-1} mod p via Fermat."""
    return _modpow(a, p - 2)


# ---------------------------------------------------------------------------
# Vectorized EF4 arithmetic
# EF4 element = (c0, c1, c2, c3) where element = c0 + c1*x + c2*x^2 + c3*x^3
# and x^4 = W = 11.
# Each coefficient is a numpy int64 array of shape (n,).
# ---------------------------------------------------------------------------

EF4Vec = tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]


def ef4v_add(a: EF4Vec, b: EF4Vec) -> EF4Vec:
    """Add two vectorized EF4 elements."""
    return (_add(a[0], b[0]), _add(a[1], b[1]),
            _add(a[2], b[2]), _add(a[3], b[3]))


def ef4v_sub(a: EF4Vec, b: EF4Vec) -> EF4Vec:
    """Subtract two vectorized EF4 elements."""
    return (_sub(a[0], b[0]), _sub(a[1], b[1]),
            _sub(a[2], b[2]), _sub(a[3], b[3]))


def ef4v_neg(a: EF4Vec) -> EF4Vec:
    """Negate a vectorized EF4 element."""
    return (_neg(a[0]), _neg(a[1]), _neg(a[2]), _neg(a[3]))


def ef4v_mul(a: EF4Vec, b: EF4Vec) -> EF4Vec:
    """Multiply two vectorized EF4 elements.

    (a0+a1*x+a2*x^2+a3*x^3) * (b0+b1*x+b2*x^2+b3*x^3) mod (x^4 - W)
    """
    a0, a1, a2, a3 = a
    b0, b1, b2, b3 = b

    # Standard schoolbook with reduction x^4 = W
    c0 = _add(_mul(a0, b0), _mul(np.int64(W), _add(_add(_mul(a1, b3), _mul(a2, b2)), _mul(a3, b1))))
    c1 = _add(_add(_mul(a0, b1), _mul(a1, b0)), _mul(np.int64(W), _add(_mul(a2, b3), _mul(a3, b2))))
    c2 = _add(_add(_add(_mul(a0, b2), _mul(a1, b1)), _mul(a2, b0)), _mul(np.int64(W), _mul(a3, b3)))
    c3 = _add(_add(_add(_mul(a0, b3), _mul(a1, b2)), _mul(a2, b1)), _mul(a3, b0))
    return (c0, c1, c2, c3)


def ef4v_mul_base(a: EF4Vec, b: np.ndarray) -> EF4Vec:
    """Multiply vectorized EF4 by base field array."""
    return (_mul(a[0], b), _mul(a[1], b), _mul(a[2], b), _mul(a[3], b))


def ef4v_from_base(b: np.ndarray) -> EF4Vec:
    """Embed base field array into EF4: (b, 0, 0, 0)."""
    z = np.zeros_like(b)
    return (b % p, z, z, z)


def ef4v_from_scalar(coeffs: list[int], n: int) -> EF4Vec:
    """Broadcast a scalar EF4 element to arrays of length n."""
    return (
        np.full(n, coeffs[0] % p, dtype=np.int64),
        np.full(n, coeffs[1] % p, dtype=np.int64),
        np.full(n, coeffs[2] % p, dtype=np.int64),
        np.full(n, coeffs[3] % p, dtype=np.int64),
    )


def ef4v_inv(a: EF4Vec) -> EF4Vec:
    """Batch inverse of vectorized EF4 elements using tower decomposition.

    F_{p^4} = F_{p^2}[v]/(v^2 - u) where u^2 = W in F_{p^2}.
    Element a = (a0 + a2*u) + (a1 + a3*u)*v = A + B*v.
    Inverse: (A - B*v) / (A^2 - B^2*u).
    """
    a0, a1, a2, a3 = a

    # Tower: A = (a0, a2) in F_{p^2}, B = (a1, a3) in F_{p^2}
    # F_{p^2} multiplication: (x0+x1*u)(y0+y1*u) = (x0*y0 + W*x1*y1, x0*y1 + x1*y0)

    # A^2 in F_{p^2}
    a2_0 = _add(_mul(a0, a0), _mul(np.int64(W), _mul(a2, a2)))
    a2_1 = _mul(np.int64(2), _mul(a0, a2))

    # B^2 in F_{p^2}
    b2_0 = _add(_mul(a1, a1), _mul(np.int64(W), _mul(a3, a3)))
    b2_1 = _mul(np.int64(2), _mul(a1, a3))

    # B^2 * u in F_{p^2}: (c0+c1*u)*u = W*c1 + c0*u
    b2u_0 = _mul(np.int64(W), b2_1)
    b2u_1 = b2_0

    # D = A^2 - B^2*u in F_{p^2}
    d0 = _sub(a2_0, b2u_0)
    d1 = _sub(a2_1, b2u_1)

    # norm = D_0^2 - W * D_1^2 in F_p
    norm = _sub(_mul(d0, d0), _mul(np.int64(W), _mul(d1, d1)))

    # norm_inv = norm^{-1} in F_p
    ni = _inv(norm)

    # D_inv = (D_0, -D_1) * norm_inv in F_{p^2}
    e0 = _mul(d0, ni)
    e1 = _neg(_mul(d1, ni))

    # Result = (A - B*v) * D_inv
    # = A*E - B*E*v where E = D_inv in F_{p^2}
    # A*E in F_{p^2}: (a0*e0 + W*a2*e1, a0*e1 + a2*e0)
    ae_0 = _add(_mul(a0, e0), _mul(np.int64(W), _mul(a2, e1)))
    ae_1 = _add(_mul(a0, e1), _mul(a2, e0))

    # B*E in F_{p^2}: (a1*e0 + W*a3*e1, a1*e1 + a3*e0)
    be_0 = _add(_mul(a1, e0), _mul(np.int64(W), _mul(a3, e1)))
    be_1 = _add(_mul(a1, e1), _mul(a3, e0))

    # Convert back: result = (AE)_0 + (-(BE)_0)*x + (AE)_1*x^2 + (-(BE)_1)*x^3
    return (ae_0, _neg(be_0), ae_1, _neg(be_1))


# ---------------------------------------------------------------------------
# Vectorized DAG evaluation for logup (base field, all rows at once)
# ---------------------------------------------------------------------------


def eval_dag_all_rows(dag, partitioned_main, preprocessed, public_values, height):
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
    from protocol.proof import EntryType, SymbolicNodeKind

    # Pre-convert trace matrices to numpy column arrays for fast lookup
    # partitioned_main[part][row][col] → np_parts[part][col] = array(height)
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
                    # next row: shift by offset
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
                # PERMUTATION, CHALLENGE, EXPOSED — not used by interaction paths
                node_values[i] = zero.copy()

        elif kind == SymbolicNodeKind.CONSTANT:
            node_values[i] = np.full(height, node.constant_value % p, dtype=np.int64)

        elif kind in (SymbolicNodeKind.IS_FIRST_ROW,
                      SymbolicNodeKind.IS_LAST_ROW,
                      SymbolicNodeKind.IS_TRANSITION):
            node_values[i] = zero.copy()

        elif kind == SymbolicNodeKind.ADD:
            node_values[i] = _add(node_values[node.left_idx], node_values[node.right_idx])

        elif kind == SymbolicNodeKind.SUB:
            node_values[i] = _sub(node_values[node.left_idx], node_values[node.right_idx])

        elif kind == SymbolicNodeKind.MUL:
            node_values[i] = _mul(node_values[node.left_idx], node_values[node.right_idx])

        elif kind == SymbolicNodeKind.NEG:
            node_values[i] = _neg(node_values[node.idx])

        else:
            raise ValueError(f"Unknown node kind: {kind}")

    return node_values


# ---------------------------------------------------------------------------
# Vectorized after-challenge trace computation
# ---------------------------------------------------------------------------


def compute_after_challenge_trace_numpy(
    interactions, interaction_partitions, dag,
    partitioned_main, preprocessed, public_values,
    alpha, beta, height,
):
    """Vectorized version of compute_after_challenge_trace.

    Processes ALL rows simultaneously using numpy arrays. Same semantics
    as the scalar version in logup.py.

    Returns:
        (perm_trace, cumulative_sum) — same format as scalar version:
        perm_trace[height][perm_width] of EF4Coeffs (list[int]),
        cumulative_sum is EF4Coeffs.
    """
    from protocol.logup import generate_betas

    perm_width = len(interaction_partitions) + 1
    betas = generate_betas(beta, interactions)

    # Step 1: Evaluate full DAG at all rows (vectorized)
    node_values = eval_dag_all_rows(
        dag, partitioned_main, preprocessed, public_values, height
    )

    # Step 2: Compute EF4 denominators for all interactions, all rows
    # denom_i = alpha + msg[0] + beta*msg[1] + ... + beta^{|msg|}*(bus_index+1)
    alpha_v = ef4v_from_scalar(alpha, height)

    all_denoms = []  # list of EF4Vec, one per interaction
    for interaction in interactions:
        msg = interaction.message
        # Start with alpha + msg[0]
        denom = ef4v_add(alpha_v, ef4v_from_base(node_values[msg[0]]))
        for j in range(1, len(msg)):
            beta_j = ef4v_from_scalar(betas[j], height)
            denom = ef4v_add(denom, ef4v_mul_base(beta_j, node_values[msg[j]]))
        # + beta^{|msg|} * (bus_index + 1)
        beta_last = ef4v_from_scalar(betas[len(msg)], height)
        bus_val = np.full(height, (interaction.bus_index + 1) % p, dtype=np.int64)
        denom = ef4v_add(denom, ef4v_mul_base(beta_last, bus_val))
        all_denoms.append(denom)

    # Step 3: Batch invert all denominators (vectorized)
    all_reciprocals = [ef4v_inv(d) for d in all_denoms]

    # Step 4: Compute chunk values for all rows
    # perm_trace_v[chunk_idx] = EF4Vec (sum of reciprocal * count for each interaction in chunk)
    perm_chunks = []  # list of EF4Vec, one per chunk
    for partition in interaction_partitions:
        chunk_sum = ef4v_from_scalar([0, 0, 0, 0], height)
        for interaction_idx in partition:
            count_node = interactions[interaction_idx].count
            count_vals = node_values[count_node]
            term = ef4v_mul_base(all_reciprocals[interaction_idx], count_vals)
            chunk_sum = ef4v_add(chunk_sum, term)
        perm_chunks.append(chunk_sum)

    # Compute phi (row sum of all chunks)
    phi = ef4v_from_scalar([0, 0, 0, 0], height)
    for chunk in perm_chunks:
        phi = ef4v_add(phi, chunk)

    # Step 5: Convert phi to running sum (prefix sum — sequential)
    phi_0 = np.cumsum(phi[0].astype(np.int64)) % p
    phi_1 = np.cumsum(phi[1].astype(np.int64)) % p
    phi_2 = np.cumsum(phi[2].astype(np.int64)) % p
    phi_3 = np.cumsum(phi[3].astype(np.int64)) % p

    # Convert back to list-of-lists format expected by caller
    perm_trace = []
    for row in range(height):
        row_data = []
        for chunk in perm_chunks:
            row_data.append([int(chunk[0][row]), int(chunk[1][row]),
                             int(chunk[2][row]), int(chunk[3][row])])
        row_data.append([int(phi_0[row]), int(phi_1[row]),
                         int(phi_2[row]), int(phi_3[row])])
        perm_trace.append(row_data)

    cumulative_sum = [int(phi_0[height - 1]), int(phi_1[height - 1]),
                      int(phi_2[height - 1]), int(phi_3[height - 1])]

    return perm_trace, cumulative_sum


# ---------------------------------------------------------------------------
# Vectorized EF4 scalar multiply (scalar EF4 × vectorized EF4)
# ---------------------------------------------------------------------------


def ef4v_mul_scalar(a: EF4Vec, s: list[int]) -> EF4Vec:
    """Multiply vectorized EF4 by a scalar EF4 element.

    More efficient than broadcasting s to full arrays and calling ef4v_mul,
    since it avoids creating 4 full arrays for the scalar.
    """
    s0, s1, s2, s3 = np.int64(s[0] % p), np.int64(s[1] % p), np.int64(s[2] % p), np.int64(s[3] % p)
    a0, a1, a2, a3 = a
    Wc = np.int64(W)
    c0 = _add(_mul(a0, s0), _mul(Wc, _add(_add(_mul(a1, s3), _mul(a2, s2)), _mul(a3, s1))))
    c1 = _add(_add(_mul(a0, s1), _mul(a1, s0)), _mul(Wc, _add(_mul(a2, s3), _mul(a3, s2))))
    c2 = _add(_add(_add(_mul(a0, s2), _mul(a1, s1)), _mul(a2, s0)), _mul(Wc, _mul(a3, s3)))
    c3 = _add(_add(_add(_mul(a0, s3), _mul(a1, s2)), _mul(a2, s1)), _mul(a3, s0))
    return (c0, c1, c2, c3)


# ---------------------------------------------------------------------------
# Vectorized quotient computation (full EF4 evaluator)
# ---------------------------------------------------------------------------


def compute_quotient_values_numpy(
    trace_on_quotient_domain, constraints_dag, public_values, alpha,
    quotient_domain, trace_domain,
    preprocessed_on_quot, after_challenge_on_quot,
    challenges, exposed_values,
    partitioned_trace_on_quot,
):
    """Numpy-vectorized version of compute_quotient_values for the full EF4 evaluator.

    Evaluates the constraint DAG at ALL quotient domain points simultaneously,
    handling all entry types (MAIN, PREPROCESSED, PERMUTATION, CHALLENGE, EXPOSED).

    Returns:
        List of EF4Coeffs, one per quotient domain point.
    """
    from primitives.field import ef4_mul as scalar_ef4_mul
    from protocol.proof import EntryType, SymbolicNodeKind
    from protocol.quotient import selectors_on_coset

    quot_size = quotient_domain.size()
    trace_size = trace_domain.size()
    step = quot_size // trace_size

    # --- Precompute selectors (scalar, then convert to numpy) ---
    sels = selectors_on_coset(trace_domain, quotient_domain)
    is_first_row_np = np.array(sels.is_first_row, dtype=np.int64)
    is_last_row_np = np.array(sels.is_last_row, dtype=np.int64)
    is_transition_np = np.array(sels.is_transition, dtype=np.int64)
    inv_zeroifier_np = np.array(sels.inv_zeroifier, dtype=np.int64)

    # --- Convert partitioned main traces to numpy column arrays ---
    # np_parts_local[part][col] = array(quot_size), np_parts_next = rolled version
    np_parts_local = []
    np_parts_next = []

    if partitioned_trace_on_quot is not None:
        for part in partitioned_trace_on_quot:
            cols = len(part[0]) if part else 0
            local_cols = []
            next_cols = []
            for c in range(cols):
                col_data = np.array([part[r][c] for r in range(quot_size)], dtype=np.int64)
                local_cols.append(col_data)
                next_cols.append(np.roll(col_data, -step))
            np_parts_local.append(local_cols)
            np_parts_next.append(next_cols)
    else:
        cols = len(trace_on_quotient_domain[0]) if trace_on_quotient_domain else 0
        local_cols = []
        next_cols = []
        for c in range(cols):
            col_data = np.array(
                [trace_on_quotient_domain[r][c] for r in range(quot_size)], dtype=np.int64
            )
            local_cols.append(col_data)
            next_cols.append(np.roll(col_data, -step))
        np_parts_local.append(local_cols)
        np_parts_next.append(next_cols)

    # --- Convert preprocessed trace ---
    np_prep_local = None
    np_prep_next = None
    if preprocessed_on_quot is not None:
        prep_cols = len(preprocessed_on_quot[0]) if preprocessed_on_quot else 0
        np_prep_local = []
        np_prep_next = []
        for c in range(prep_cols):
            col_data = np.array(
                [preprocessed_on_quot[r][c] for r in range(quot_size)], dtype=np.int64
            )
            np_prep_local.append(col_data)
            np_prep_next.append(np.roll(col_data, -step))

    # --- Convert after_challenge trace (EF4 columns) ---
    # after_challenge_on_quot[row][perm_col] = EF4Coeffs
    # → np_ac[perm_col] = {local: EF4Vec, next: EF4Vec}
    np_ac_local = None
    np_ac_next = None
    if after_challenge_on_quot is not None:
        perm_width = len(after_challenge_on_quot[0])
        np_ac_local = []
        np_ac_next = []
        for col in range(perm_width):
            c0 = np.array([after_challenge_on_quot[r][col][0] for r in range(quot_size)], dtype=np.int64)
            c1 = np.array([after_challenge_on_quot[r][col][1] for r in range(quot_size)], dtype=np.int64)
            c2 = np.array([after_challenge_on_quot[r][col][2] for r in range(quot_size)], dtype=np.int64)
            c3 = np.array([after_challenge_on_quot[r][col][3] for r in range(quot_size)], dtype=np.int64)
            np_ac_local.append((c0 % p, c1 % p, c2 % p, c3 % p))
            np_ac_next.append((
                np.roll(c0, -step) % p,
                np.roll(c1, -step) % p,
                np.roll(c2, -step) % p,
                np.roll(c3, -step) % p,
            ))

    # --- Evaluate DAG nodes (all in EF4, all quotient points at once) ---
    nodes = constraints_dag.nodes
    node_values = [None] * len(nodes)

    zero_np = np.zeros(quot_size, dtype=np.int64)

    for i, node in enumerate(nodes):
        kind = node.kind

        if kind == SymbolicNodeKind.VARIABLE:
            var = node.variable
            entry = var.entry
            if entry.kind == EntryType.MAIN:
                if entry.offset == 0:
                    node_values[i] = ef4v_from_base(np_parts_local[entry.part_index][var.index] % p)
                else:
                    node_values[i] = ef4v_from_base(np_parts_next[entry.part_index][var.index] % p)
            elif entry.kind == EntryType.PREPROCESSED:
                if entry.offset == 0:
                    node_values[i] = ef4v_from_base(np_prep_local[var.index] % p)
                else:
                    node_values[i] = ef4v_from_base(np_prep_next[var.index] % p)
            elif entry.kind == EntryType.PUBLIC:
                node_values[i] = ef4v_from_scalar(
                    [public_values[var.index] % p, 0, 0, 0], quot_size
                )
            elif entry.kind == EntryType.PERMUTATION:
                if entry.offset == 0:
                    node_values[i] = np_ac_local[var.index]
                else:
                    node_values[i] = np_ac_next[var.index]
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
            node_values[i] = ef4v_from_base(is_first_row_np)

        elif kind == SymbolicNodeKind.IS_LAST_ROW:
            node_values[i] = ef4v_from_base(is_last_row_np)

        elif kind == SymbolicNodeKind.IS_TRANSITION:
            node_values[i] = ef4v_from_base(is_transition_np)

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

    # --- Accumulate constraints with alpha powers ---
    num_constraints = len(constraints_dag.constraint_idx)
    alpha_powers = []
    current_alpha = [1, 0, 0, 0]
    for _ in range(num_constraints):
        alpha_powers.append(current_alpha)
        current_alpha = scalar_ef4_mul(current_alpha, alpha)

    acc = (zero_np.copy(), zero_np.copy(), zero_np.copy(), zero_np.copy())
    for alpha_pow, node_idx in zip(alpha_powers, reversed(constraints_dag.constraint_idx)):
        term = ef4v_mul_scalar(node_values[node_idx], alpha_pow)
        acc = ef4v_add(acc, term)

    # --- Divide by vanishing polynomial: multiply by inv_zeroifier (base field) ---
    result = ef4v_mul_base(acc, inv_zeroifier_np)

    # --- Convert back to list format ---
    result_arr = np.stack(result, axis=1)  # shape: (quot_size, 4)
    return result_arr.tolist()


# ---------------------------------------------------------------------------
# Vectorized FRI fold_matrix
# ---------------------------------------------------------------------------


def fold_matrix_numpy(
    evals_bit_reversed: list[list[int]],
    beta: list[int],
    log_height: int,
    halve_inv_powers_br: list[int],
) -> list[list[int]]:
    """Numpy-vectorized version of FRI fold_matrix.

    Folds bit-reversed evaluations: (lo + hi)/2 + (lo - hi) * beta * hip.

    Args:
        evals_bit_reversed: Flat list of EF4Coeffs (pairs of conjugates).
        beta: Folding challenge (EF4).
        log_height: Log2 of the folded height.
        halve_inv_powers_br: Bit-reversed halve_inv_powers (base field).

    Returns:
        Folded list of EF4Coeffs (half the size).
    """
    height = len(evals_bit_reversed) // 2

    # Extract lo/hi interleaved arrays
    arr = np.array(evals_bit_reversed, dtype=np.int64)  # shape (2*height, 4)
    lo = (arr[0::2, 0], arr[0::2, 1], arr[0::2, 2], arr[0::2, 3])
    hi = (arr[1::2, 0], arr[1::2, 1], arr[1::2, 2], arr[1::2, 3])

    hip = np.array(halve_inv_powers_br, dtype=np.int64)
    two_inv = np.int64(p + 1) // np.int64(2) % p  # (p+1)/2 mod p

    # result = (lo + hi) * TWO_INV + (lo - hi) * beta * halve_inv_power
    sum_half = ef4v_mul_base(ef4v_add(lo, hi), np.full(height, two_inv, dtype=np.int64))
    diff_beta = ef4v_mul_scalar(ef4v_sub(lo, hi), beta)
    diff_beta_hip = ef4v_mul_base(diff_beta, hip)
    result = ef4v_add(sum_half, diff_beta_hip)

    # Convert back
    out = np.stack(result, axis=1)  # (height, 4)
    return out.tolist()


def roll_in_numpy(
    folded: list[list[int]],
    roll_in: list[list[int]],
    beta_sq: list[int],
) -> list[list[int]]:
    """Numpy-vectorized roll-in: folded[i] += beta_sq * roll_in[i]."""
    f = np.array(folded, dtype=np.int64)
    r = np.array(roll_in, dtype=np.int64)
    fv = (f[:, 0], f[:, 1], f[:, 2], f[:, 3])
    rv = (r[:, 0], r[:, 1], r[:, 2], r[:, 3])
    result = ef4v_add(fv, ef4v_mul_scalar(rv, beta_sq))
    out = np.stack(result, axis=1)
    return out.tolist()


def leaves_from_folded_numpy(folded: list[list[int]]) -> list[list[int]]:
    """Build Merkle leaves by pairing consecutive EF4 elements.

    Each leaf = [lo_c0..lo_c3, hi_c0..hi_c3] (8 base field elements).
    """
    arr = np.array(folded, dtype=np.int64)  # shape (2*N, 4)
    return arr.reshape(-1, 8).tolist()


# ---------------------------------------------------------------------------
# Batch polynomial evaluation at a single EF4 point
# ---------------------------------------------------------------------------


def eval_poly_ef4_batch(
    coeffs_per_col: list[list[int]],
    eval_point: list[int],
) -> list[list[int]]:
    """Evaluate multiple polynomials at the same EF4 point using numpy.

    Uses baby-step/giant-step to precompute z-powers in O(sqrt(D)) EF4 muls,
    then uses numpy matrix multiply for O(D * num_cols) base field muls.

    Args:
        coeffs_per_col: List of polynomial coefficient arrays (one per column).
            Each is a list of base field elements.
        eval_point: EF4 evaluation point [c0, c1, c2, c3].

    Returns:
        List of EF4 evaluation results (one per column), each [c0, c1, c2, c3].
    """
    num_cols = len(coeffs_per_col)
    if num_cols == 0:
        return []
    degree = len(coeffs_per_col[0])
    if degree == 0:
        return [[0, 0, 0, 0]] * num_cols

    z = tuple(int(c) for c in eval_point)

    # --- Precompute z-powers using baby-step/giant-step ---
    z_powers = _precompute_z_powers_bsgs(z, degree)  # 4 arrays of shape (degree,)

    # --- Batch evaluate: result[col] = sum_i coeffs[col][i] * z^i ---
    coeff_mat = np.array(coeffs_per_col, dtype=np.int64)  # (num_cols, degree)

    results = []
    for zp in z_powers:
        # Element-wise multiply: each product < p^2 ≈ 2^62 < int64_max
        products = (coeff_mat * zp) % p  # (num_cols, degree), each in [0, p)
        # Sum: degree values each < p; total < degree * p ≈ 2^50 < int64_max
        result_j = products.sum(axis=1) % p  # (num_cols,)
        results.append(result_j)

    out = np.stack(results, axis=1)  # (num_cols, 4)
    return out.tolist()


def _precompute_z_powers_bsgs(
    z: tuple[int, int, int, int],
    degree: int,
) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    """Precompute z^0, z^1, ..., z^{degree-1} using baby-step/giant-step.

    Returns 4 numpy arrays of shape (degree,), one per EF4 component.
    Total cost: O(sqrt(degree)) EF4 muls + O(degree) numpy operations.
    """
    if degree <= 0:
        return tuple(np.empty(0, dtype=np.int64) for _ in range(4))

    # Choose block size B ≈ sqrt(degree)
    B = max(1, int(degree ** 0.5))
    num_blocks = (degree + B - 1) // B

    # Baby step: compute z^0, z^1, ..., z^{B-1} sequentially
    small = [(1, 0, 0, 0)]  # z^0
    cur = (1, 0, 0, 0)
    for _ in range(B - 1):
        cur = _ef4_mul_raw(cur, z)
        small.append(cur)

    # Giant step base: z^B
    z_B = _ef4_mul_raw(cur, z)  # cur = z^{B-1}, so cur * z = z^B

    # Giant step: compute (z^B)^0, (z^B)^1, ..., (z^B)^{num_blocks-1}
    big = [(1, 0, 0, 0)]
    cur = (1, 0, 0, 0)
    for _ in range(num_blocks - 1):
        cur = _ef4_mul_raw(cur, z_B)
        big.append(cur)

    # Convert to numpy component arrays
    small_np = [np.array([s[c] for s in small], dtype=np.int64) for c in range(4)]
    big_np = [np.array([b[c] for b in big], dtype=np.int64) for c in range(4)]

    # Compute all z-powers via outer products:
    # z^{kB + j} = big[k] * small[j]  (EF4 multiply)
    # Component 0: b0*s0 + W*(b1*s3 + b2*s2 + b3*s1)
    # Component 1: b0*s1 + b1*s0 + W*(b2*s3 + b3*s2)
    # Component 2: b0*s2 + b1*s1 + b2*s0 + W*b3*s3
    # Component 3: b0*s3 + b1*s2 + b2*s1 + b3*s0

    def _outer_flat(a: np.ndarray, b: np.ndarray) -> np.ndarray:
        """Outer product (num_blocks, B) flattened to (num_blocks*B,), truncated to degree."""
        return np.outer(a, b).ravel()[:degree]

    def _outer_mod(a: np.ndarray, b: np.ndarray) -> np.ndarray:
        """Outer product reduced mod p."""
        return _outer_flat(a, b) % p

    # Build each component. Each product b_a[k]*s_b[j] < p^2 ≈ 2^62 fits int64.
    # Sum of 2 products < 2*p^2 ≈ 2^63 — right at the limit. Reduce after sum of 2.
    W_val = W  # 11

    # Component 0: b0*s0 + W*(b1*s3 + b2*s2 + b3*s1)
    t0 = _outer_mod(big_np[0], small_np[0])
    w_sum = (_outer_flat(big_np[1], small_np[3]) + _outer_flat(big_np[2], small_np[2])) % p
    w_sum = (w_sum + _outer_mod(big_np[3], small_np[1])) % p
    c0 = (t0 + W_val * w_sum) % p

    # Component 1: b0*s1 + b1*s0 + W*(b2*s3 + b3*s2)
    t1 = (_outer_flat(big_np[0], small_np[1]) + _outer_flat(big_np[1], small_np[0])) % p
    w_sum1 = (_outer_flat(big_np[2], small_np[3]) + _outer_flat(big_np[3], small_np[2])) % p
    c1 = (t1 + W_val * w_sum1) % p

    # Component 2: b0*s2 + b1*s1 + b2*s0 + W*b3*s3
    t2 = (_outer_flat(big_np[0], small_np[2]) + _outer_flat(big_np[1], small_np[1])) % p
    t2 = (t2 + _outer_mod(big_np[2], small_np[0])) % p
    c2 = (t2 + W_val * _outer_mod(big_np[3], small_np[3])) % p

    # Component 3: b0*s3 + b1*s2 + b2*s1 + b3*s0
    t3 = (_outer_flat(big_np[0], small_np[3]) + _outer_flat(big_np[1], small_np[2])) % p
    t3_2 = (_outer_flat(big_np[2], small_np[1]) + _outer_flat(big_np[3], small_np[0])) % p
    c3 = (t3 + t3_2) % p

    return (c0, c1, c2, c3)


def _ef4_mul_raw(
    a: tuple[int, int, int, int],
    b: tuple[int, int, int, int],
) -> tuple[int, int, int, int]:
    """Raw EF4 multiply using Python ints. No class overhead."""
    a0, a1, a2, a3 = a
    b0, b1, b2, b3 = b
    c0 = (a0 * b0 + W * (a1 * b3 + a2 * b2 + a3 * b1)) % p
    c1 = (a0 * b1 + a1 * b0 + W * (a2 * b3 + a3 * b2)) % p
    c2 = (a0 * b2 + a1 * b1 + a2 * b0 + W * a3 * b3) % p
    c3 = (a0 * b3 + a1 * b2 + a2 * b1 + a3 * b0) % p
    return (c0, c1, c2, c3)
