"""FRI (Fast Reed-Solomon IOP of Proximity) protocol.

Implements FRI folding, verification, and proving matching Plonky3's FRI.

Reference:
    p3-fri-0.4.1/src/ (prover.rs, verifier.rs, two_adic_pcs.rs)
    crates/test-vectors/tests/verify_fri_vectors.rs
"""

from primitives.field import (
    BABYBEAR_PRIME,
    W,
    ff4,
    ff4_coeffs,
    ff4_from_base,
    get_omega,
)
from primitives.merkle import (
    build_merkle_tree,
    get_opening_proof,
    verify_opening_prehashed,
)
from primitives.ntt import intt
from primitives.poseidon2 import hash_to_digest
from primitives.transcript import Challenger


def fri_fold(
    evals: list[list[int]],
    challenge: list[int],
    log_domain_size: int,
    coset_shift: int,
) -> list[list[int]]:
    """Fold FRI evaluations with extension field challenge beta.

    Given evaluations of polynomial f on coset shift * <omega_N>,
    compute evaluations of the folded polynomial on shift^2 * <omega_{N/2}>.

    The folding decomposes f into even and odd parts:
        f(x) = f_even(x^2) + x * f_odd(x^2)
        folded(y) = f_even(y) + beta * f_odd(y)

    Evaluations are in natural order: first half at shift*omega^i,
    second half at shift*omega^(i+N/2) = -shift*omega^i.

    Args:
        evals: Extension field evaluations [[c0,c1,c2,c3], ...] on coset.
        challenge: Extension field folding challenge [c0,c1,c2,c3].
        log_domain_size: Log2 of current domain size.
        coset_shift: Current coset generator.

    Returns:
        Folded evaluations (half the input size).

    Reference:
        p3-fri-0.4.1/src/two_adic_pcs.rs (TwoAdicFriFolder::fold_row)
        crates/test-vectors/tests/verify_fri_vectors.rs
    """
    p = BABYBEAR_PRIME
    n = len(evals)
    half = n // 2
    beta = ff4(challenge)

    omega = get_omega(log_domain_size)
    two_inv = pow(2, p - 2, p)

    folded = []
    for i in range(half):
        f_pos = ff4(evals[i])           # f(x)  where x = shift * omega^i
        f_neg = ff4(evals[i + half])    # f(-x) where -x = shift * omega^(i+N/2)

        x = (coset_shift * pow(omega, i, p)) % p
        half_inv_x = pow(2 * x % p, p - 2, p)  # 1/(2x) mod p

        even = (f_pos + f_neg) * ff4_from_base(two_inv)
        odd = (f_pos - f_neg) * ff4_from_base(half_inv_x)
        result = even + beta * odd

        folded.append(ff4_coeffs(result))

    return folded


def reverse_bits_len(x: int, bit_len: int) -> int:
    """Reverse the lowest bit_len bits of x.

    Reference:
        p3-util-0.4.2/src/lib.rs
    """
    result = 0
    for _ in range(bit_len):
        result = (result << 1) | (x & 1)
        x >>= 1
    return result


def fold_row(
    index: int,
    log_height: int,
    beta,
    e0,
    e1,
):
    """Lagrange interpolation fold for the FRI verifier.

    Given evaluations e0, e1 at conjugate points xs0, xs1 in the two-adic
    coset, interpolate and evaluate at the folding challenge beta.

    Args:
        index: Parent index (after start_index >> 1).
        log_height: Log2 of the folded domain height.
        beta: Extension field folding challenge (FF4 element).
        e0: Evaluation at even position (FF4 element).
        e1: Evaluation at odd position (FF4 element).

    Returns:
        Folded evaluation (FF4 element).

    Reference:
        p3-fri-0.4.1/src/two_adic_pcs.rs (TwoAdicFriFolding::fold_row)
    """
    p = BABYBEAR_PRIME
    # xs0 = two_adic_generator(log_height + 1) ^ reverse_bits_len(index, log_height)
    subgroup_start = pow(W[log_height + 1],
                         reverse_bits_len(index, log_height),
                         p)
    xs0 = ff4_from_base(subgroup_start)
    xs1 = ff4_from_base((-subgroup_start) % p)  # = -xs0

    # Lagrange interpolation: e0 + (beta - xs0) * (e1 - e0) / (xs1 - xs0)
    inv_diff = ff4_from_base(pow((-2 * subgroup_start) % p, p - 2, p))
    return e0 + (beta - xs0) * (e1 - e0) * inv_diff


def hash_fri_leaf(e0, e1) -> list[int]:
    """Hash a pair of extension field elements as a FRI Merkle leaf.

    The FRI MMCS stores pairs of evaluations per tree leaf. The leaf
    hash is computed from the 8 base field coefficients (2 EF elements).

    Args:
        e0: Even-indexed evaluation (FF4 element).
        e1: Odd-indexed evaluation (FF4 element).

    Returns:
        8-element Poseidon2 digest.

    Reference:
        p3-merkle-tree-0.4.1/src/mmcs.rs (verify_batch leaf hashing)
    """
    return hash_to_digest(ff4_coeffs(e0) + ff4_coeffs(e1))


def fri_verify_query(
    commit_phase_commits: list[list[int]],
    betas: list[list[int]],
    query_index: int,
    query_proof: dict,
    reduced_opening: list[int],
    final_poly: list[list[int]],
    log_max_height: int,
    log_final_poly_len: int,
) -> list[int]:
    """Verify a single FRI query: fold chain + Merkle proofs + final poly.

    Starting from the reduced opening (initial folded evaluation), performs
    the FRI fold chain: at each round, verifies the Merkle proof for the
    sibling value, then folds via Lagrange interpolation. After all rounds,
    checks the result against the final polynomial evaluation.

    Args:
        commit_phase_commits: Merkle roots per round (8-element digests).
        betas: FRI folding challenges per round ([c0,c1,c2,c3] each).
        query_index: Starting query index.
        query_proof: Query proof data with commit_phase_openings.
        reduced_opening: Initial folded eval from PCS (4-element EF coeffs).
        final_poly: Final polynomial coefficients (list of 4-element EF coeffs).
        log_max_height: Log2 of max height (num_rounds + log_blowup + log_final_poly_len).
        log_final_poly_len: Log2 of final polynomial length.

    Returns:
        Final folded evaluation as [c0, c1, c2, c3].

    Reference:
        p3-fri-0.4.1/src/verifier.rs (verify_query)
    """
    num_rounds = len(commit_phase_commits)
    start_index = query_index
    folded_eval = ff4(reduced_opening)

    for round_idx in range(num_rounds):
        log_folded_height = log_max_height - 1 - round_idx
        opening = query_proof["commit_phase_openings"][round_idx]
        sibling = ff4(opening["sibling_value"])
        beta = ff4(betas[round_idx])

        # Arrange evals: e0 at even position, e1 at odd position
        index_sibling = start_index ^ 1
        if index_sibling % 2 == 0:
            e0, e1 = sibling, folded_eval
        else:
            e0, e1 = folded_eval, sibling

        # Parent index (matching p3-fri: start_index >>= 1 before verify_batch)
        parent_index = start_index >> 1

        # Verify Merkle proof for the pair of evaluations
        leaf_digest = hash_fri_leaf(e0, e1)
        assert verify_opening_prehashed(
            commit_phase_commits[round_idx],
            leaf_digest,
            parent_index,
            opening["opening_proof"],
        ), f"Merkle proof failed at round {round_idx}"

        # Fold via Lagrange interpolation
        folded_eval = fold_row(parent_index, log_folded_height, beta, e0, e1)

        # Advance to parent index for next round
        start_index = parent_index

    # Verify final polynomial evaluation
    if log_final_poly_len == 0:
        expected = ff4(final_poly[0])
    else:
        p = BABYBEAR_PRIME
        x = pow(W[log_final_poly_len],
                reverse_bits_len(start_index, log_final_poly_len), p)
        expected = ff4([0, 0, 0, 0])
        for i, c in enumerate(final_poly):
            expected = expected + ff4(c) * ff4_from_base(pow(x, i, p))

    assert ff4_coeffs(folded_eval) == ff4_coeffs(expected), (
        f"Final polynomial check failed: "
        f"{ff4_coeffs(folded_eval)} != {ff4_coeffs(expected)}"
    )

    return ff4_coeffs(folded_eval)


def verify_fri(
    commit_phase_commits: list[list[int]],
    final_poly: list[list[int]],
    query_proofs: list[dict],
    log_blowup: int,
    log_final_poly_len: int,
    num_queries: int,
) -> bool:
    """Verify FRI proof: transcript replay and structural consistency.

    Replays the Fiat-Shamir transcript to derive folding challenges (betas)
    and query indices, then verifies structural consistency of each query's
    opening proofs.

    Note: Full cryptographic verification of the fold chain requires
    reduced_openings from the PCS layer, which are not included in the
    standalone FRI test vectors. This function verifies:
    1. Transcript replay (observe commitments, sample challenges)
    2. Query index derivation (sample_bits from transcript)
    3. Structural consistency (proof lengths match tree heights)

    Args:
        commit_phase_commits: Merkle roots per round (8-element digests).
        final_poly: Final polynomial coefficients (extension field elements).
        query_proofs: Per-query opening data.
        log_blowup: Log2 of FRI blowup factor.
        log_final_poly_len: Log2 of final polynomial length.
        num_queries: Number of FRI queries.

    Returns:
        True if verification passes.

    Reference:
        p3-fri-0.4.1/src/verifier.rs
    """
    challenger = Challenger()

    num_rounds = len(commit_phase_commits)
    log_max_height = num_rounds + log_blowup + log_final_poly_len

    # Phase 1: Replay commit phase — observe roots and sample betas
    betas = []
    for commit in commit_phase_commits:
        challenger.observe_many(commit)
        beta = challenger.sample_ext()
        betas.append(beta)

    # Observe final polynomial coefficients
    for coeff in final_poly:
        challenger.observe_many(coeff)

    # Phase 2: Verify each query
    assert len(query_proofs) == num_queries, (
        f"Expected {num_queries} query proofs, got {len(query_proofs)}"
    )

    for qi, query_proof in enumerate(query_proofs):
        # Derive query index from transcript
        query_index = challenger.sample_bits(log_max_height)
        assert 0 <= query_index < (1 << log_max_height), (
            f"Query {qi}: index {query_index} out of range [0, {1 << log_max_height})"
        )

        # Verify opening structure
        openings = query_proof["commit_phase_openings"]
        assert len(openings) == num_rounds, (
            f"Query {qi}: expected {num_rounds} opening steps, got {len(openings)}"
        )

        # Verify each round's Merkle proof has correct length for tree height
        for round_idx, opening in enumerate(openings):
            log_folded_height = log_max_height - 1 - round_idx
            expected_proof_len = log_folded_height
            proof = opening["opening_proof"]
            assert len(proof) == expected_proof_len, (
                f"Query {qi}, round {round_idx}: "
                f"expected {expected_proof_len} Merkle siblings, got {len(proof)}"
            )

            # Verify sibling value is an extension field element (4 coefficients)
            assert len(opening["sibling_value"]) == 4, (
                f"Query {qi}, round {round_idx}: "
                f"sibling_value should have 4 components"
            )

            # Verify each Merkle sibling is a valid digest (8 elements)
            for si, sibling in enumerate(proof):
                assert len(sibling) == 8, (
                    f"Query {qi}, round {round_idx}, sibling {si}: "
                    f"digest should have 8 elements, got {len(sibling)}"
                )

    return True


# =========================================================================
# FRI Prover
# =========================================================================


def bit_reverse_list(lst: list) -> list:
    """Reorder list elements by bit-reversing their indices.

    Reference:
        p3-util-0.4.2/src/lib.rs (reverse_slice_index_bits)
    """
    n = len(lst)
    if n <= 1:
        return list(lst)
    log_n = n.bit_length() - 1
    return [lst[reverse_bits_len(i, log_n)] for i in range(n)]


def ef_idft(evals: list[list[int]]) -> list[list[int]]:
    """Inverse DFT for extension field evaluations.

    Applies INTT independently to each of the 4 base-field coefficient
    channels, matching Radix2DFTSmallBatch::idft_algebra.

    Args:
        evals: Extension field evaluations [[c0,c1,c2,c3], ...].

    Returns:
        Extension field polynomial coefficients.

    Reference:
        p3-dft-0.4.1/src/traits.rs (idft_algebra)
    """
    n = len(evals)
    if n == 1:
        return [list(evals[0])]
    # Transpose: extract each coefficient channel
    channels = [[evals[j][k] for j in range(n)] for k in range(4)]
    # INTT each channel independently
    channels_coeffs = [intt(ch) for ch in channels]
    # Transpose back
    return [[channels_coeffs[k][j] for k in range(4)] for j in range(n)]


def fold_matrix(
    evals_bit_reversed: list[list[int]],
    beta: list[int],
    log_height: int,
) -> list[list[int]]:
    """Fold bit-reversed evaluations with extension field challenge.

    Input evals are in bit-reversed order, so adjacent pairs
    (evals[2i], evals[2i+1]) are conjugate points (f(x), f(-x)).
    Pairs are folded using the standard decomposition into even/odd parts.

    Args:
        evals_bit_reversed: Extension field evaluations in bit-reversed order.
        beta: Extension field folding challenge [c0,c1,c2,c3].
        log_height: Log2 of the number of pairs (= log2(len(evals)/2)).

    Returns:
        Folded evaluations (half the input size), in bit-reversed order.

    Reference:
        p3-fri-0.4.1/src/two_adic_pcs.rs (TwoAdicFriFolding::fold_matrix)
    """
    p = BABYBEAR_PRIME
    beta_ef = ff4(beta)
    height = len(evals_bit_reversed) // 2

    # g_inv = two_adic_generator(log_height + 1)^(-1)
    g_inv = pow(W[log_height + 1], p - 2, p)

    # Precompute halve_inv_powers[i] = g_inv^i / 2 (before bit-reversal)
    two_inv = pow(2, p - 2, p)
    halve_inv_powers = []
    val = two_inv  # (1/2) * g_inv^0 = 1/2
    for _ in range(height):
        halve_inv_powers.append(val)
        val = (val * g_inv) % p

    # Bit-reverse the powers
    halve_inv_powers = bit_reverse_list(halve_inv_powers)

    folded = []
    for i in range(height):
        lo = ff4(evals_bit_reversed[2 * i])
        hi = ff4(evals_bit_reversed[2 * i + 1])
        # result = (lo + hi)/2 + (lo - hi) * beta * halve_inv_power
        result = (lo + hi) * ff4_from_base(two_inv) + \
                 (lo - hi) * beta_ef * ff4_from_base(halve_inv_powers[i])
        folded.append(ff4_coeffs(result))

    return folded


def commit_phase(
    evals_bit_reversed: list[list[int]],
    log_blowup: int,
    log_final_poly_len: int,
    challenger: Challenger,
) -> dict:
    """FRI commit phase: iterative folding with Merkle commitments.

    Args:
        evals_bit_reversed: Initial evaluations in bit-reversed order.
        log_blowup: Log2 of blowup factor.
        log_final_poly_len: Log2 of final polynomial length.
        challenger: Fiat-Shamir transcript.

    Returns:
        Dict with keys: commits, betas, final_poly, trees,
        folded_per_round, all_round_evals.

    Reference:
        p3-fri-0.4.1/src/prover.rs (commit_phase)
    """
    folded = list(evals_bit_reversed)
    commits = []
    betas = []
    trees = []
    folded_per_round = []
    all_round_evals = []  # Store evals before each fold (for query openings)
    blowup = 1 << log_blowup
    final_poly_len = 1 << log_final_poly_len

    while len(folded) > blowup * final_poly_len:
        height = len(folded) // 2
        log_height = height.bit_length() - 1

        # Store current evals for query phase
        all_round_evals.append(folded)

        # Build Merkle tree from pairs of evaluations.
        # Each leaf = hash of [lo_c0..lo_c3, hi_c0..hi_c3] (8 base field elements).
        leaves = []
        for i in range(0, len(folded), 2):
            leaves.append(folded[i] + folded[i + 1])
        root, tree = build_merkle_tree(leaves)

        # Observe commitment
        challenger.observe_many(root)
        commits.append(root)
        trees.append(tree)

        # Sample folding challenge
        beta = challenger.sample_ext()
        betas.append(beta)

        # Fold
        folded = fold_matrix(folded, beta, log_height)
        folded_per_round.append(folded)

    # Compute final polynomial via IDFT:
    # Truncate to final_poly_len, bit-reverse, IDFT.
    final_evals = folded[:final_poly_len]
    final_evals_natural = bit_reverse_list(final_evals)
    final_poly = ef_idft(final_evals_natural)

    # Observe final polynomial
    for coeff in final_poly:
        challenger.observe_many(coeff)

    return {
        "commits": commits,
        "betas": betas,
        "final_poly": final_poly,
        "trees": trees,
        "folded_per_round": folded_per_round,
        "all_round_evals": all_round_evals,
    }


def answer_query(
    trees: list,
    all_round_evals: list[list[list[int]]],
    start_index: int,
    num_rounds: int,
) -> list[dict]:
    """Generate FRI query opening proof for a single query index.

    For each round, opens the Merkle tree at the pair index and
    extracts the sibling value.

    Args:
        trees: Merkle trees from commit_phase (one per round).
        all_round_evals: Evaluations before each fold round.
        start_index: Starting query index.
        num_rounds: Number of FRI folding rounds.

    Returns:
        List of dicts with 'sibling_value' and 'opening_proof' per round.

    Reference:
        p3-fri-0.4.1/src/prover.rs (answer_query)
    """
    openings = []
    for i in range(num_rounds):
        index_i = start_index >> i
        index_i_sibling = index_i ^ 1
        index_pair = index_i >> 1

        # Get Merkle proof for the pair
        proof = get_opening_proof(trees[i], index_pair)

        # Sibling value from stored evaluations
        sibling_value = all_round_evals[i][index_i_sibling]

        openings.append({
            "sibling_value": sibling_value,
            "opening_proof": proof,
        })
    return openings


def prove_fri(
    evals_bit_reversed: list[list[int]],
    log_blowup: int,
    log_final_poly_len: int,
    num_queries: int,
    challenger: Challenger,
) -> dict:
    """Full FRI proof generation.

    Args:
        evals_bit_reversed: Extension field evaluations in bit-reversed order.
        log_blowup: Log2 of blowup factor.
        log_final_poly_len: Log2 of final polynomial length.
        num_queries: Number of query indices to sample.
        challenger: Fiat-Shamir transcript.

    Returns:
        FRI proof dict with commit_phase_commits, final_poly, query_proofs,
        betas, folded_per_round.

    Reference:
        p3-fri-0.4.1/src/prover.rs (prove_fri)
    """
    # Commit phase
    result = commit_phase(
        evals_bit_reversed, log_blowup, log_final_poly_len, challenger
    )

    # Query phase
    log_max_height = len(evals_bit_reversed).bit_length() - 1
    num_rounds = len(result["commits"])
    query_proofs = []

    for _ in range(num_queries):
        query_index = challenger.sample_bits(log_max_height)
        openings = answer_query(
            result["trees"],
            result["all_round_evals"],
            query_index,
            num_rounds,
        )
        query_proofs.append({
            "index": query_index,
            "commit_phase_openings": openings,
        })

    return {
        "commit_phase_commits": result["commits"],
        "final_poly": result["final_poly"],
        "query_proofs": query_proofs,
        "betas": result["betas"],
        "folded_per_round": result["folded_per_round"],
    }
