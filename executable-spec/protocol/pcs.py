"""PCS (Polynomial Commitment Scheme) for TwoAdicFriPcs.

Implements both prover and verifier sides of the FRI-based PCS.

Verifier: given committed polynomial batches opened at evaluation points,
reduces them to a single FRI verification problem.

Prover: commits polynomials via LDE + Merkle, evaluates at opening points,
computes reduced polynomials, and delegates to the FRI prover.

Reference:
    p3-fri-0.4.1/src/two_adic_pcs.rs  -- TwoAdicFriPcs::commit, open, verify
    p3-fri-0.4.1/src/verifier.rs      -- verify_fri, verify_query, open_input
    p3-fri-0.4.1/src/prover.rs        -- prove (FRI prover)
"""

from dataclasses import dataclass

from primitives.field import (
    BABYBEAR_PRIME,
    Digest,
    EF4Coeffs,
    EF4Vec,
    Fe,
    FF,
    FF4,
    GENERATOR,
    W,
    bit_reverse_list,
    ef4_mul,
    ef4v_add,
    ef4v_from_base,
    ef4v_from_scalar,
    ef4v_inv,
    ef4v_mul,
    ef4v_mul_scalar,
    ef4v_sub,
    ef4v_to_rows,
    ef4v_zeros,
    eval_poly_ef4_batch,
    ff4,
    ff4_coeffs,
    ff4_from_base,
    ff_column,
    get_omega,
    inv_mod,
    reverse_bits_len,
)
from primitives.merkle import get_opening_proof, verify_opening_prehashed
from primitives.ntt import coset_lde_batch as _coset_lde_batch_ffi, intt_batch as _intt_batch_ffi
from primitives.poseidon2 import compress, hash_to_digest, compress_batch, hash_batch
from primitives.transcript import Challenger, check_witness, grind
from protocol.domain import TwoAdicMultiplicativeCoset
from protocol.fri import answer_query, commit_phase as fri_commit_phase, fold_row, hash_fri_leaf
from protocol.proof import (
    BatchOpening,
    CommitPhaseProofStep,
    FriParameters,
    FriProof,
)

p = BABYBEAR_PRIME


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass
class PcsRound:
    """One round of PCS verification data.

    Corresponds to a single MMCS commitment (batch of matrices) and the
    domains/openings associated with it.

    Reference:
        p3-fri/src/two_adic_pcs.rs (CommitmentWithOpeningPoints type alias)
    """
    commitment: Digest
    domains_and_openings: list[tuple[TwoAdicMultiplicativeCoset, list[tuple[EF4Coeffs, list[EF4Coeffs]]]]]


# ---------------------------------------------------------------------------
# Merkle batch verification for input MMCS
# ---------------------------------------------------------------------------


def _verify_batch_opening(
    commitment: Digest,
    dimensions: list[int],
    index: int,
    batch_opening: BatchOpening,
) -> None:
    """Verify a batch Merkle opening for multiple matrices of varying heights.

    This implements the MerkleTreeMmcs::verify_batch logic from Plonky3.
    Matrices are sorted by height (tallest first). The Merkle tree has
    log_max_height levels. Shorter matrices are "rolled in" at the
    appropriate level by compressing their leaf hash with the running root.

    Args:
        commitment: The Merkle root commitment for this batch.
        dimensions: Heights of each matrix in the batch (in original order).
        index: The query index (at the tallest matrix's height).
        batch_opening: The opened values and Merkle proof.

    Reference:
        p3-merkle-tree-0.4.1/src/mmcs.rs (MerkleTreeMmcs::verify_batch, lines 230-338)
    """
    opened_values = batch_opening.opened_values
    opening_proof = batch_opening.opening_proof

    assert len(dimensions) == len(opened_values), (
        f"batch size mismatch: {len(dimensions)} dimensions vs {len(opened_values)} openings"
    )

    if not dimensions:
        raise ValueError("empty batch: no matrices to verify")

    # Sort matrices by height descending, keeping original indices.
    # This mirrors heights_tallest_first in the Rust code.
    sorted_entries = sorted(
        enumerate(dimensions), key=lambda x: -x[1]
    )

    max_height = sorted_entries[0][1]
    curr_height_padded = _next_power_of_two(max_height)
    log_max_height = curr_height_padded.bit_length() - 1

    assert len(opening_proof) == log_max_height, (
        f"wrong proof height: expected {log_max_height} siblings, got {len(opening_proof)}"
    )
    assert index < max_height, (
        f"index {index} out of bounds for max_height {max_height}"
    )

    # Consume entries from sorted_entries using a pointer, mimicking
    # Rust's peeking_take_while iterator pattern.
    entry_ptr = 0

    # Hash all matrix openings at the initial (tallest) padded height.
    initial_group = []
    while (entry_ptr < len(sorted_entries)
           and _next_power_of_two(sorted_entries[entry_ptr][1]) == curr_height_padded):
        initial_group.append(sorted_entries[entry_ptr])
        entry_ptr += 1

    root = _hash_matrix_rows(initial_group, opened_values)

    current_index = index

    for sibling in opening_proof:
        # Combine current root with sibling based on index parity
        if current_index & 1 == 0:
            root = compress(root, sibling)
        else:
            root = compress(sibling, root)
        current_index >>= 1
        curr_height_padded >>= 1

        # Check if there are new matrix rows to inject at the current level.
        # We peek at the next entry and check if its padded height matches.
        if (entry_ptr < len(sorted_entries)
                and _next_power_of_two(sorted_entries[entry_ptr][1]) == curr_height_padded):
            next_height = sorted_entries[entry_ptr][1]
            # Collect all entries with this exact height (peeking_take_while)
            inject_group = []
            while (entry_ptr < len(sorted_entries)
                   and sorted_entries[entry_ptr][1] == next_height):
                inject_group.append(sorted_entries[entry_ptr])
                entry_ptr += 1
            # Hash their rows and compress with current root
            inject_digest = _hash_matrix_rows(inject_group, opened_values)
            root = compress(root, inject_digest)

    assert root == commitment, "Merkle root mismatch in batch verification"


def _next_power_of_two(n: int) -> int:
    """Round up to the next power of two (or n if already a power of two)."""
    if n <= 0:
        return 1
    # Bit trick: if n is already a power of two, return n
    if n & (n - 1) == 0:
        return n
    return 1 << n.bit_length()


def _hash_matrix_rows(
    entries: list[tuple[int, int]],
    opened_values: list[list[Fe]],
) -> Digest:
    """Hash the opened row values for a group of matrices.

    Mimics PaddingFreeSponge::hash_iter_slices: concatenate all row data
    from the matrices (in the order given by entries) and hash.

    Args:
        entries: List of (original_index, height) pairs, all at the same padded height.
        opened_values: The full list of opened row values.

    Returns:
        A Poseidon2 digest of the concatenated row data.

    Reference:
        p3-symmetric PaddingFreeSponge::hash_iter_slices
    """
    all_values: list[Fe] = []
    for orig_idx, _ in entries:
        all_values.extend(opened_values[orig_idx])
    return hash_to_digest(all_values)


# ---------------------------------------------------------------------------
# Reduced opening computation (open_input)
# ---------------------------------------------------------------------------


def _open_input(
    fri_params: FriParameters,
    log_global_max_height: int,
    index: int,
    input_proof: list[BatchOpening],
    alpha: FF4,
    rounds: list[PcsRound],
) -> list[tuple[int, FF4]]:
    """Open input polynomials and combine into FRI reduced openings.

    For each batch commitment and its opening proof:
    1. Verify the Merkle batch opening
    2. For each matrix, compute x = g * omega^(bit_reverse(index >> bits_reduced, log_height))
    3. For each (point z, values) pair, compute (p(z) - p(x)) / (z - x) weighted by alpha

    Returns reduced openings as (log_height, reduced_value) pairs, sorted
    descending by log_height.

    Reference:
        p3-fri-0.4.1/src/verifier.rs (open_input, lines 343-455)
    """
    # For each log_height, store (alpha_pow, reduced_opening)
    reduced_openings: dict[int, tuple] = {}  # log_height -> (FF4_alpha_pow, FF4_reduced)

    alpha_one = ff4_from_base(1)
    alpha_zero = ff4_from_base(0)

    assert len(input_proof) == len(rounds), (
        f"input_proof length {len(input_proof)} != rounds length {len(rounds)}"
    )

    for batch_opening, round_data in zip(input_proof, rounds):
        mats = round_data.domains_and_openings

        # Compute the height of each matrix in the batch (domain size * blowup)
        batch_heights = [
            domain.size() << fri_params.log_blowup
            for domain, _ in mats
        ]

        # Compute the reduced index for this batch.
        # If the max height of the batch is smaller than the global max height,
        # shift the index down to compensate.
        if batch_heights:
            max_batch_height = max(batch_heights)
            log_max_batch = max_batch_height.bit_length() - 1
            reduced_index = index >> (log_global_max_height - log_max_batch)
        else:
            reduced_index = 0

        # Verify the Merkle batch opening
        _verify_batch_opening(
            round_data.commitment,
            batch_heights,
            reduced_index,
            batch_opening,
        )

        # For each matrix in the commitment
        for mat_idx, (mat_domain, mat_points_and_values) in enumerate(mats):
            mat_opening = batch_opening.opened_values[mat_idx]
            log_height = mat_domain.log_n + fri_params.log_blowup

            bits_reduced = log_global_max_height - log_height
            rev_reduced_index = reverse_bits_len(index >> bits_reduced, log_height)

            # x = GENERATOR * two_adic_generator(log_height)^rev_reduced_index
            # This is the evaluation point on the LDE coset g*H
            x = (GENERATOR * pow(get_omega(log_height), rev_reduced_index, p)) % p

            # Get or initialize the reduced opening accumulator for this log_height
            if log_height not in reduced_openings:
                reduced_openings[log_height] = (alpha_one, alpha_zero)

            alpha_pow_ef, ro_ef = reduced_openings[log_height]

            # For each (point z, claimed values) pair
            for z, ps_at_z in mat_points_and_values:
                # quotient = 1 / (z - x) in extension field
                z_ef = ff4(z)
                x_ef = ff4_from_base(x)
                quotient = (z_ef - x_ef) ** (-1)

                # For each column value
                for col_idx in range(len(mat_opening)):
                    p_at_x = ff4_from_base(mat_opening[col_idx])
                    p_at_z = ff4(ps_at_z[col_idx])

                    # ro += alpha_pow * (p_at_z - p_at_x) * quotient
                    ro_ef = ro_ef + alpha_pow_ef * (p_at_z - p_at_x) * quotient
                    alpha_pow_ef = alpha_pow_ef * alpha

            reduced_openings[log_height] = (alpha_pow_ef, ro_ef)

        # Check: if there's a reduced opening at log_height = log_blowup,
        # the polynomial must be constant, so the reduced opening must be zero
        if fri_params.log_blowup in reduced_openings:
            _, ro_check = reduced_openings[fri_params.log_blowup]
            assert ff4_coeffs(ro_check) == [0, 0, 0, 0], (
                "constant polynomial has non-zero reduced opening"
            )

    # Return reduced openings sorted descending by log_height
    result = []
    for log_h in sorted(reduced_openings.keys(), reverse=True):
        _, ro = reduced_openings[log_h]
        result.append((log_h, ro))
    return result


# ---------------------------------------------------------------------------
# Query verification (verify_query)
# ---------------------------------------------------------------------------


def _verify_query(
    fri_params: FriParameters,
    start_index: int,
    betas: list[FF4],
    commit_phase_commits: list[Digest],
    commit_phase_openings: list[CommitPhaseProofStep],
    reduced_openings: list[tuple[int, FF4]],
    log_global_max_height: int,
    log_final_height: int,
) -> tuple[FF4, int]:
    """Verify a single FRI query: fold chain with reduced openings rolled in.

    Starting from the initial reduced opening at log_global_max_height,
    performs FRI folds. At each step where a new reduced opening exists
    (for a smaller log_height), it is rolled in using beta^2.

    Returns (folded_eval, final_domain_index).

    Reference:
        p3-fri-0.4.1/src/verifier.rs (verify_query, lines 236-321)
    """
    ro_peeked = list(reduced_openings)
    ro_ptr = 0

    # The first reduced opening must be at log_global_max_height
    assert ro_ptr < len(ro_peeked), "no reduced openings"
    first_lh, first_ro = ro_peeked[ro_ptr]
    assert first_lh == log_global_max_height, (
        f"first reduced opening at log_height {first_lh}, expected {log_global_max_height}"
    )
    folded_eval = first_ro
    ro_ptr += 1

    num_commit_rounds = len(commit_phase_commits)
    assert len(betas) == num_commit_rounds
    assert len(commit_phase_openings) == num_commit_rounds

    domain_index = start_index

    # Fold from log_global_max_height - 1 down to log_final_height
    for round_idx in range(num_commit_rounds):
        log_folded_height = log_global_max_height - 1 - round_idx
        beta = betas[round_idx]
        comm = commit_phase_commits[round_idx]
        opening = commit_phase_openings[round_idx]

        # Get the sibling value
        index_sibling = domain_index ^ 1
        sibling = ff4(opening.sibling_value)

        # Arrange evals: evals[0] is at even position, evals[1] at odd
        if index_sibling % 2 == 0:
            e0, e1 = sibling, folded_eval
        else:
            e0, e1 = folded_eval, sibling

        # Parent index
        domain_index >>= 1

        # Verify Merkle proof for the FRI commitment
        # FRI commit phase uses pairs of extension field evaluations as leaves
        # Leaf = hash([e0_c0..e0_c3, e1_c0..e1_c3])
        leaf_digest = hash_fri_leaf(e0, e1)
        assert verify_opening_prehashed(
            comm,
            leaf_digest,
            domain_index,
            opening.opening_proof,
        ), f"FRI commit phase Merkle proof failed at round {round_idx}"

        # Fold
        folded_eval = fold_row(domain_index, log_folded_height, beta, e0, e1)

        # Roll in reduced openings at this folded height, if any
        if ro_ptr < len(ro_peeked) and ro_peeked[ro_ptr][0] == log_folded_height:
            _, ro_val = ro_peeked[ro_ptr]
            ro_ptr += 1
            # Use beta^2 as the random combination factor
            folded_eval = folded_eval + beta * beta * ro_val

    # All reduced openings must have been consumed
    assert ro_ptr == len(ro_peeked), (
        f"not all reduced openings consumed: {ro_ptr} of {len(ro_peeked)}"
    )

    return folded_eval, domain_index


# ---------------------------------------------------------------------------
# Main PCS verification
# ---------------------------------------------------------------------------


def pcs_verify(
    rounds: list[PcsRound],
    fri_proof: FriProof,
    challenger: Challenger,
    fri_params: FriParameters,
) -> None:
    """Verify PCS openings using FRI.

    This is the Python translation of TwoAdicFriPcs::verify followed by
    verify_fri. The algorithm:

    1. Sample FRI alpha from challenger (batch combination challenge)
    2. Replay FRI commit phase: for each commitment, observe it, check PoW,
       then sample beta
    3. Observe final polynomial
    4. Check query PoW
    5. For each query:
       a. Sample query index
       b. Compute reduced openings from the input proof (open_input)
       c. Verify FRI query (fold chain + Merkle proofs + final poly check)

    Args:
        rounds: List of PcsRound, one per batch commitment. Each contains
            the commitment digest and the domains/openings for that batch.
        fri_proof: The FRI proof containing commit phase commits, query proofs,
            final polynomial, and PoW witnesses.
        challenger: The Fiat-Shamir challenger (already has opened values observed).
        fri_params: FRI protocol parameters.

    Raises:
        AssertionError: If any verification check fails.

    Reference:
        p3-fri-0.4.1/src/two_adic_pcs.rs (TwoAdicFriPcs::verify, lines 523-554)
        p3-fri-0.4.1/src/verifier.rs (verify_fri, lines 54-204)
    """
    # --- Step 0: Observe all opened values (claimed polynomial evaluations) ---
    # The PCS verifier observes all evaluation claims into the transcript
    # before sampling the batch combination challenge.
    # Reference: two_adic_pcs.rs TwoAdicFriPcs::verify, lines 533-541
    for round_data in rounds:
        for _domain, points_and_values in round_data.domains_and_openings:
            for _point, values in points_and_values:
                for val in values:
                    challenger.observe_many(val)

    # --- Step 1: Sample batch combination challenge (alpha) ---
    alpha = ff4(challenger.sample_ext())

    # --- Step 2: Compute log_global_max_height ---
    # log_global_max_height = num_commit_rounds + log_blowup + log_final_poly_len
    num_commit_rounds = len(fri_proof.commit_phase_commits)
    log_global_max_height = (
        num_commit_rounds + fri_params.log_blowup + fri_params.log_final_poly_len
    )

    # --- Step 3: Validate proof shape ---
    assert len(fri_proof.commit_pow_witnesses) == num_commit_rounds, (
        f"commit_pow_witnesses length {len(fri_proof.commit_pow_witnesses)} != "
        f"commit_phase_commits length {num_commit_rounds}"
    )

    # --- Step 4: Replay commit phase — observe roots, check PoW, sample betas ---
    betas = []
    for round_idx in range(num_commit_rounds):
        comm = fri_proof.commit_phase_commits[round_idx]
        pow_witness = fri_proof.commit_pow_witnesses[round_idx]

        # Observe the commitment
        challenger.observe_many(comm)

        # Check per-round PoW
        assert check_witness(
            challenger, fri_params.commit_proof_of_work_bits, pow_witness
        ), f"commit phase PoW failed at round {round_idx}"

        # Sample folding challenge beta
        beta = ff4(challenger.sample_ext())
        betas.append(beta)

    # --- Step 5: Validate final polynomial length ---
    final_poly_len = 1 << fri_params.log_final_poly_len
    assert len(fri_proof.final_poly) == final_poly_len, (
        f"final poly length {len(fri_proof.final_poly)} != expected {final_poly_len}"
    )

    # --- Step 6: Observe final polynomial ---
    for coeff in fri_proof.final_poly:
        challenger.observe_many(coeff)

    # --- Step 7: Validate number of query proofs ---
    assert len(fri_proof.query_proofs) == fri_params.num_queries, (
        f"expected {fri_params.num_queries} query proofs, "
        f"got {len(fri_proof.query_proofs)}"
    )

    # --- Step 8: Check query-phase PoW ---
    assert check_witness(
        challenger, fri_params.query_proof_of_work_bits, fri_proof.query_pow_witness
    ), "query phase PoW failed"

    # --- Step 9: Process each query ---
    log_final_height = fri_params.log_blowup + fri_params.log_final_poly_len

    for qi, query_proof in enumerate(fri_proof.query_proofs):
        # Sample query index (extra_query_index_bits = 0 for TwoAdicFriFolding)
        query_index = challenger.sample_bits(log_global_max_height)

        # Compute reduced openings from the input proof
        reduced_openings = _open_input(
            fri_params,
            log_global_max_height,
            query_index,
            query_proof.input_proof,
            alpha,
            rounds,
        )

        # Verify FRI query: fold chain + Merkle proofs
        folded_eval, domain_index = _verify_query(
            fri_params,
            query_index,  # extra_query_index_bits = 0, so no shifting needed
            betas,
            fri_proof.commit_phase_commits,
            query_proof.commit_phase_openings,
            reduced_openings,
            log_global_max_height,
            log_final_height,
        )

        # --- Step 10: Verify final polynomial evaluation ---
        # The final polynomial is evaluated at x where
        # x = two_adic_generator(log_global_max_height)^reverse_bits_len(domain_index, log_global_max_height)
        #
        # However, after the fold chain, domain_index has been shifted down by
        # num_commit_rounds. The remaining index corresponds to the final domain.
        # We evaluate at x = W[log_global_max_height]^reverse_bits_len(domain_index, log_global_max_height)
        #
        # But wait: domain_index after folding has log_final_height bits.
        # The Rust code uses log_global_max_height for the bit reversal, but domain_index
        # only has log_final_height significant bits at this point. Let's match the Rust exactly.
        x_base = pow(
            W[log_global_max_height],
            reverse_bits_len(domain_index, log_global_max_height),
            p,
        )

        # Evaluate the final polynomial at x using Horner's method
        # final_poly is in coefficient form: f(x) = c0 + c1*x + c2*x^2 + ...
        eval_result = ff4_from_base(0)
        for coeff in reversed(fri_proof.final_poly):
            eval_result = eval_result * ff4_from_base(x_base) + ff4(coeff)

        assert ff4_coeffs(folded_eval) == ff4_coeffs(eval_result), (
            f"Query {qi}: final polynomial mismatch: "
            f"folded={ff4_coeffs(folded_eval)}, expected={ff4_coeffs(eval_result)}"
        )


# ===========================================================================
# PCS Prover
# ===========================================================================


@dataclass
class CommittedData:
    """Prover data for a single MMCS commitment (batch of matrices).

    Reference:
        p3-merkle-tree MerkleTreeMmcs::commit
    """
    root: Digest
    tree: list[list[Digest]]  # Merkle tree levels
    # Per-matrix LDE rows (bit-reversed): [mat][row][col]
    lde_rows: list[list[list[Fe]]]
    # Per-matrix coefficient form: [mat][col][coeff]
    coeffs: list[list[list[Fe]]]
    # Per-matrix domain
    domains: list[TwoAdicMultiplicativeCoset]


# ---------------------------------------------------------------------------
# PCS commit
# ---------------------------------------------------------------------------


def pcs_commit(
    evaluations: list[tuple[TwoAdicMultiplicativeCoset, list[list[Fe]]]],
    log_blowup: int,
) -> CommittedData:
    """Commit to a batch of matrices via LDE + Merkle tree.

    For each matrix on its domain:
    1. INTT each column to coefficients (treating as subgroup evaluations).
    2. Zero-pad to LDE size (n * 2^log_blowup).
    3. Coset-shift coefficients: c'[j] = c[j] * (GENERATOR / domain.shift)^j.
    4. NTT on the extended subgroup.
    5. Bit-reverse the rows.
    Then build a single Merkle tree from the concatenated rows.

    Args:
        evaluations: List of (domain, matrix) where matrix is list of rows.
        log_blowup: Log2 of the FRI blowup factor.

    Returns:
        CommittedData with Merkle tree, LDE, and coefficients.

    Reference:
        p3-fri-0.4.1/src/two_adic_pcs.rs TwoAdicFriPcs::commit
    """
    all_lde_rows: list[list[list[Fe]]] = []
    all_coeffs: list[list[list[Fe]]] = []
    all_domains: list[TwoAdicMultiplicativeCoset] = []

    for domain, matrix in evaluations:
        n = domain.size()
        assert len(matrix) == n
        n_ext = n << log_blowup
        num_cols = len(matrix[0]) if matrix else 0

        # LDE shift = GENERATOR / domain.shift
        lde_shift = (GENERATOR * inv_mod(domain.shift)) % p

        # Extract columns
        columns = [[matrix[r][c] for r in range(n)] for c in range(num_cols)]

        # Compute coefficients via batch INTT (Rust FFI, parallelized)
        coeffs_columns = _intt_batch_ffi(columns) if columns else []
        coeffs_per_col = [list(c) for c in coeffs_columns]

        # Coset LDE via Rust FFI (INTT + pad + shift + NTT, all parallelized)
        # Note: coset_lde_batch does its own INTT internally, so we pass evals
        lde_columns = list(_coset_lde_batch_ffi(columns, lde_shift, log_blowup)) if columns else []

        # Transpose to rows and bit-reverse
        lde_matrix = [
            [lde_columns[c][r] for c in range(num_cols)]
            for r in range(n_ext)
        ]
        lde_matrix = bit_reverse_list(lde_matrix)

        all_lde_rows.append(lde_matrix)
        all_coeffs.append(coeffs_per_col)
        all_domains.append(domain)

    # Build Merkle tree using Plonky3's multi-height MMCS protocol.
    # Tallest matrices go at the leaf level; shorter matrices are "injected"
    # at higher tree levels via compress(node, hash(shorter_rows)).
    # This matches _verify_batch_opening (p3-merkle-tree mmcs.rs verify_batch).
    if not all_lde_rows:
        return CommittedData(
            root=[0] * 8, tree=[], lde_rows=[], coeffs=[], domains=[]
        )

    root, tree = _build_mmcs_tree(all_lde_rows)

    return CommittedData(
        root=root,
        tree=tree,
        lde_rows=all_lde_rows,
        coeffs=all_coeffs,
        domains=all_domains,
    )


def _build_mmcs_tree(
    all_lde_rows: list[list[list[Fe]]],
) -> tuple[Digest, list[list[Digest]]]:
    """Build multi-height MMCS Merkle tree matching Plonky3.

    Tallest matrices are hashed at the leaf level. Shorter matrices are
    "injected" at higher levels by compressing their row hashes with tree
    nodes. This mirrors the verify_batch logic in _verify_batch_opening.

    Reference:
        p3-merkle-tree-0.4.1/src/mmcs.rs MerkleTreeMmcs::commit
    """
    # Group matrices by padded height, sorted tallest first.
    heights = [(i, len(rows)) for i, rows in enumerate(all_lde_rows)]
    sorted_entries = sorted(heights, key=lambda x: -x[1])

    max_height = sorted_entries[0][1]
    curr_height_padded = _next_power_of_two(max_height)

    # Collect tallest group (all matrices whose padded height == curr_height_padded)
    entry_ptr = 0
    tallest_group: list[int] = []  # matrix indices
    while (entry_ptr < len(sorted_entries)
           and _next_power_of_two(sorted_entries[entry_ptr][1]) == curr_height_padded):
        tallest_group.append(sorted_entries[entry_ptr][0])
        entry_ptr += 1

    # Hash leaf rows from tallest matrices only (batch parallel)
    leaf_inputs: list[list[Fe]] = []
    for row_idx in range(max_height):
        leaf_data: list[Fe] = []
        for mat_idx in tallest_group:
            leaf_data.extend(all_lde_rows[mat_idx][row_idx])
        leaf_inputs.append(leaf_data)

    # Pad to power-of-two if needed
    pad_count = curr_height_padded - max_height
    for _ in range(pad_count):
        leaf_inputs.append([])

    leaf_digests: list[Digest] = hash_batch(leaf_inputs)

    # Build tree bottom-up, injecting shorter matrices at appropriate levels
    tree_levels: list[list[Digest]] = [leaf_digests]
    current_level = leaf_digests
    level_height = curr_height_padded

    while level_height > 1:
        # Pair siblings to form parent level (batch parallel)
        lefts = current_level[0::2]
        rights = current_level[1::2]
        next_level: list[Digest] = compress_batch(lefts, rights)
        level_height >>= 1

        # Check if shorter matrices should be injected at this level
        if (entry_ptr < len(sorted_entries)
                and _next_power_of_two(sorted_entries[entry_ptr][1]) == level_height):
            inject_height = sorted_entries[entry_ptr][1]
            inject_group: list[int] = []  # matrix indices
            while (entry_ptr < len(sorted_entries)
                   and sorted_entries[entry_ptr][1] == inject_height):
                inject_group.append(sorted_entries[entry_ptr][0])
                entry_ptr += 1

            # Batch: collect all inject data, hash in parallel, then compress
            inject_inputs: list[list[Fe]] = []
            inject_positions: list[int] = []
            for pos in range(len(next_level)):
                row_idx = pos if pos < inject_height else pos % inject_height
                inject_data: list[Fe] = []
                for mat_idx in inject_group:
                    if row_idx < len(all_lde_rows[mat_idx]):
                        inject_data.extend(all_lde_rows[mat_idx][row_idx])
                if inject_data:
                    inject_inputs.append(inject_data)
                    inject_positions.append(pos)

            if inject_inputs:
                inject_digests = hash_batch(inject_inputs)
                # Batch compress: node with inject_digest
                node_lefts = [next_level[pos] for pos in inject_positions]
                inject_results = compress_batch(node_lefts, inject_digests)
                for pos, result in zip(inject_positions, inject_results):
                    next_level[pos] = result

        current_level = next_level
        tree_levels.append(current_level)

    root = list(current_level[0]) if current_level else hash_to_digest([])
    return root, tree_levels


# ---------------------------------------------------------------------------
# Polynomial evaluation from coefficients
# ---------------------------------------------------------------------------


def _eval_poly_ef4(
    coeffs: list[Fe],
    point: EF4Coeffs,
    domain_shift: Fe,
) -> EF4Coeffs:
    """Evaluate polynomial at an extension field point.

    The coefficients come from INTT on the domain subgroup. The original
    polynomial p(x) satisfies p(domain_shift * omega^k) = evals[k].
    So the INTT polynomial f(x) = p(domain_shift * x) and
    p(point) = f(point / domain_shift).

    Args:
        coeffs: INTT coefficients of one column.
        point: Extension field evaluation point.
        domain_shift: Domain's coset shift.

    Returns:
        p(point) as EF4Coeffs.

    Reference:
        Horner's method over BinomialExtensionField
    """
    # eval_point = point / domain_shift
    if domain_shift == 1:
        eval_point = ff4(point)
    else:
        eval_point = ff4(point) * ff4_from_base(inv_mod(domain_shift))

    result = ff4_from_base(0)
    for i in range(len(coeffs) - 1, -1, -1):
        result = result * eval_point + ff4_from_base(coeffs[i])

    return ff4_coeffs(result)


def _compute_x_array(log_height: int, x_arrays: dict[int, FF]) -> FF:
    """Compute and cache bit-reversed domain points x_i for a given log_height.

    Args:
        log_height: Log2 of the LDE height.
        x_arrays: Mutable cache mapping log_height to precomputed x arrays.

    Returns:
        Array of x_i = GENERATOR * omega^{bit_reverse(i)} for i in [0, 2^log_height).
    """
    if log_height not in x_arrays:
        height = 1 << log_height
        omega = get_omega(log_height)
        x_arr = ff_column([
            (GENERATOR * pow(omega, reverse_bits_len(i, log_height), p)) % p
            for i in range(height)
        ])
        x_arrays[log_height] = x_arr
    return x_arrays[log_height]


def _compute_inv_diff(
    log_height: int,
    point: EF4Coeffs,
    x_arrays: dict[int, FF],
    inv_diff_cache: dict[tuple, EF4Vec],
) -> EF4Vec:
    """Compute and cache (z - x_i)^{-1} for all domain points x_i.

    Args:
        log_height: Log2 of the LDE height.
        point: Extension field evaluation point z.
        x_arrays: Mutable cache for precomputed x arrays (passed to _compute_x_array).
        inv_diff_cache: Mutable cache mapping (log_height, point) to inverse differences.

    Returns:
        EF4Vec of (z - x_i)^{-1} for each domain point x_i.
    """
    key = (log_height, tuple(point))
    if key not in inv_diff_cache:
        height = 1 << log_height
        x_arr = _compute_x_array(log_height, x_arrays)
        z_v = ef4v_from_scalar(point, height)
        diff = ef4v_sub(z_v, ef4v_from_base(x_arr))
        inv_diff_cache[key] = ef4v_inv(diff)
    return inv_diff_cache[key]


# ---------------------------------------------------------------------------
# PCS open (prover side)
# ---------------------------------------------------------------------------


@dataclass
class PcsOpeningRound:
    """One round of PCS opening data for the prover.

    Reference:
        p3-fri-0.4.1/src/two_adic_pcs.rs TwoAdicFriPcs::open
    """
    committed: CommittedData
    # Per-matrix list of opening points
    points_per_mat: list[list[EF4Coeffs]]


def pcs_open(
    rounds: list[PcsOpeningRound],
    challenger: Challenger,
    fri_params: FriParameters,
) -> tuple[list[list[list[list[EF4Coeffs]]]], dict, list[int]]:
    """Open committed polynomials at specified points.

    Prover-side PCS open: evaluates polynomials, computes reduced
    polynomials, runs FRI, and generates Merkle openings.

    Args:
        rounds: Per-commitment opening data.
        challenger: The Fiat-Shamir challenger (quotient already observed).
        fri_params: FRI protocol parameters.

    Returns:
        (all_opened_values, fri_proof_data, query_indices) where:
        - all_opened_values[round][mat][point] = list[EF4Coeffs]
        - fri_proof_data contains the FRI proof components
        - query_indices for Merkle opening generation

    Reference:
        p3-fri-0.4.1/src/two_adic_pcs.rs TwoAdicFriPcs::open lines 286-440
    """
    # --- Step A: Evaluate polynomials at opening points ---

    all_opened_values: list[list[list[list[EF4Coeffs]]]] = []

    for rnd_idx, rnd in enumerate(rounds):
        round_values: list[list[list[EF4Coeffs]]] = []
        for mat_idx in range(len(rnd.committed.domains)):
            domain = rnd.committed.domains[mat_idx]
            coeffs_per_col = rnd.committed.coeffs[mat_idx]
            points = rnd.points_per_mat[mat_idx]
            degree = len(coeffs_per_col[0]) if coeffs_per_col else 0

            mat_values: list[list[EF4Coeffs]] = []
            for point in points:
                # Compute eval_point = point / domain_shift
                if domain.shift == 1:
                    eval_pt = list(point)
                else:
                    eval_pt = ff4_coeffs(
                        ff4(point) * ff4_from_base(inv_mod(domain.shift))
                    )

                if degree >= 256 and len(coeffs_per_col) > 0:
                    col_values = eval_poly_ef4_batch(
                        coeffs_per_col, eval_pt,
                    )
                else:
                    col_values = [
                        _eval_poly_ef4(col_coeffs, point, domain.shift)
                        for col_coeffs in coeffs_per_col
                    ]
                mat_values.append(col_values)
            round_values.append(mat_values)
        all_opened_values.append(round_values)

    # --- Step B: Observe all opened values ---
    for round_values in all_opened_values:
        for mat_values in round_values:
            for point_values in mat_values:
                for val in point_values:
                    challenger.observe_many(val)

    # --- Step C: Sample FRI alpha ---
    alpha = ff4(challenger.sample_ext())

    # --- Step D: Compute reduced polynomials per height ---
    # Group by LDE height, accumulate alpha-weighted quotients.
    reduced_evals_np: dict[int, EF4Vec] = {}  # log_height → EF4Vec
    # Cache (z - x_i)^{-1} per (log_height, z_tuple) to avoid recomputation
    inv_diff_cache: dict[tuple, EF4Vec] = {}

    # Pre-compute bit-reversed domain points x_i per log_height
    x_arrays: dict[int, FF] = {}

    # Per-height alpha_pow accumulators (matching Rust's num_reduced[log_height]).
    # Each height independently tracks alpha^k for its k-th column.
    # Reference: p3-fri two_adic_pcs.rs lines 226,253,271
    alpha_coeffs = ff4_coeffs(alpha)
    alpha_pow_per_height: dict[int, EF4Coeffs] = {}

    for rnd_idx, rnd in enumerate(rounds):
        for mat_idx in range(len(rnd.committed.domains)):
            domain = rnd.committed.domains[mat_idx]
            lde_rows = rnd.committed.lde_rows[mat_idx]
            points = rnd.points_per_mat[mat_idx]
            log_height = domain.log_n + fri_params.log_blowup
            height = 1 << log_height

            if log_height not in reduced_evals_np:
                reduced_evals_np[log_height] = ef4v_zeros(height)

            if log_height not in alpha_pow_per_height:
                alpha_pow_per_height[log_height] = [1, 0, 0, 0]

            num_cols = len(lde_rows[0]) if lde_rows else 0
            mat_opened_values = all_opened_values[rnd_idx][mat_idx]

            # Pre-extract LDE columns as field arrays for this matrix
            lde_cols_np = [
                ff_column([lde_rows[i][c] for i in range(height)])
                for c in range(num_cols)
            ]

            for pt_idx, point in enumerate(points):
                inv_diff = _compute_inv_diff(log_height, point, x_arrays, inv_diff_cache)
                point_values = mat_opened_values[pt_idx]

                for col_idx in range(num_cols):
                    # p_at_z is scalar EF4, p_at_x is base field array
                    p_at_z_v = ef4v_from_scalar(point_values[col_idx], height)
                    p_at_x_v = ef4v_from_base(lde_cols_np[col_idx])

                    # (p_at_z - p_at_x) * inv_diff
                    quotient = ef4v_mul(ef4v_sub(p_at_z_v, p_at_x_v), inv_diff)

                    # alpha_pow * quotient (per-height alpha_pow)
                    scaled = ef4v_mul_scalar(quotient, alpha_pow_per_height[log_height])

                    # Accumulate
                    re = reduced_evals_np[log_height]
                    reduced_evals_np[log_height] = ef4v_add(re, scaled)

                    # Advance this height's alpha_pow
                    alpha_pow_per_height[log_height] = ef4_mul(
                        alpha_pow_per_height[log_height], alpha_coeffs
                    )
    # Convert EF4Vec back to list-of-EF4Coeffs for FRI
    reduced_evals: dict[int, list[EF4Coeffs]] = {}
    for log_h, ev in reduced_evals_np.items():
        reduced_evals[log_h] = ef4v_to_rows(ev)

    # --- Step E: FRI prove ---
    # Collect reduced evaluations in descending height order
    sorted_heights = sorted(reduced_evals.keys(), reverse=True)
    log_global_max_height = sorted_heights[0] if sorted_heights else 0

    # Tallest height goes directly to FRI; shorter heights are rolled in
    reduced_br = reduced_evals[sorted_heights[0]]
    reduced_openings_by_height = None
    if len(sorted_heights) > 1:
        reduced_openings_by_height = {
            log_h: reduced_evals[log_h]
            for log_h in sorted_heights[1:]
        }

    # FRI commit phase
    fri_result = fri_commit_phase(
        reduced_br,
        fri_params.log_blowup,
        fri_params.log_final_poly_len,
        challenger,
        fri_params.commit_proof_of_work_bits,
        reduced_openings_by_height=reduced_openings_by_height,
    )

    # Query PoW
    query_pow_witness = grind(challenger, fri_params.query_proof_of_work_bits)

    # --- Step F: Query phase ---
    num_fri_rounds = len(fri_result.commits)
    query_indices: list[int] = []
    fri_query_proofs: list[tuple[int, list]] = []

    for _ in range(fri_params.num_queries):
        query_index = challenger.sample_bits(log_global_max_height)
        query_indices.append(query_index)

        # FRI commit phase openings
        fri_openings = answer_query(
            fri_result.trees,
            fri_result.all_round_evals,
            query_index,
            num_fri_rounds,
        )
        fri_query_proofs.append((query_index, fri_openings))

    # Package FRI proof data
    fri_proof_data = {
        "commit_phase_commits": fri_result.commits,
        "final_poly": fri_result.final_poly,
        "commit_pow_witnesses": fri_result.commit_pow_witnesses,
        "query_pow_witness": query_pow_witness,
        "fri_query_proofs": fri_query_proofs,
    }

    return all_opened_values, fri_proof_data, query_indices


# ---------------------------------------------------------------------------
# Batch opening generation for query proofs
# ---------------------------------------------------------------------------


def _generate_batch_opening(
    committed: CommittedData,
    query_index: int,
    log_global_max_height: int,
) -> BatchOpening:
    """Generate a BatchOpening for a single query index.

    Extracts the row values and Merkle proof from the committed data.

    Args:
        committed: The committed batch data.
        query_index: The global query index.
        log_global_max_height: Log2 of the global max LDE height.

    Returns:
        BatchOpening with row values and Merkle proof.

    Reference:
        p3-merkle-tree mmcs.rs MerkleTreeMmcs::open_batch
    """
    # Determine the max height of matrices in this batch
    max_mat_height = max(len(rows) for rows in committed.lde_rows)
    log_max_mat = max_mat_height.bit_length() - 1

    # Reduce query index to this batch's height
    reduced_index = query_index >> (log_global_max_height - log_max_mat)

    # Extract row values for each matrix
    opened_values: list[list[Fe]] = []
    for mat_idx in range(len(committed.lde_rows)):
        mat_height = len(committed.lde_rows[mat_idx])
        if mat_height == max_mat_height:
            row = committed.lde_rows[mat_idx][reduced_index]
        else:
            # For shorter matrices, further reduce the index
            log_mat = mat_height.bit_length() - 1
            mat_index = reduced_index >> (log_max_mat - log_mat)
            row = committed.lde_rows[mat_idx][mat_index]
        opened_values.append(list(row))

    # Merkle proof at the reduced index
    proof = get_opening_proof(committed.tree, reduced_index)

    return BatchOpening(
        opened_values=opened_values,
        opening_proof=proof,
    )
