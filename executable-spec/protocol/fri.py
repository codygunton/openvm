"""FRI (Fast Reed-Solomon IOP of Proximity) protocol.

Implements FRI folding, verification, and proving matching Plonky3's FRI.

Reference:
    p3-fri-0.4.1/src/ (prover.rs, verifier.rs, two_adic_pcs.rs)
"""

from dataclasses import dataclass

from primitives.field import (
    BABYBEAR_PRIME,
    EF4Coeffs,
    Digest,
    Fe,
    FF4,
    MerklePath,
    TWO_INV,
    W,
    bit_reverse_list,
    ff4,
    ff4_coeffs,
    ff4_from_base,
    get_omega,
    inv_mod,
    reverse_bits_len,
)
from primitives.merkle import (
    build_merkle_tree,
    get_opening_proof,
    verify_opening_prehashed,
)
from primitives.ntt import intt
from primitives.poseidon2 import hash_to_digest
from primitives.transcript import Challenger

p = BABYBEAR_PRIME


# --- Data Structures ---


@dataclass
class CommitPhaseResult:
    """Output of FRI commit phase."""
    commits: list[Digest]
    betas: list[EF4Coeffs]
    final_poly: list[EF4Coeffs]
    trees: list
    folded_per_round: list[list[EF4Coeffs]]
    all_round_evals: list[list[EF4Coeffs]]


@dataclass
class FriQueryStep:
    """One round of a FRI query opening."""
    sibling_value: EF4Coeffs
    opening_proof: MerklePath


@dataclass
class FriQueryProof:
    """FRI query proof for a single query index."""
    index: int
    commit_phase_openings: list[FriQueryStep]


@dataclass
class FriProof:
    """Complete FRI proof."""
    commit_phase_commits: list[Digest]
    final_poly: list[EF4Coeffs]
    query_proofs: list[FriQueryProof]
    betas: list[EF4Coeffs]
    folded_per_round: list[list[EF4Coeffs]]


# --- Verifier: Natural-Order Folding ---


def fri_fold(
    evals: list[EF4Coeffs],
    challenge: EF4Coeffs,
    log_domain_size: int,
    coset_shift: Fe,
) -> list[EF4Coeffs]:
    """Fold evaluations on coset: f_even(y) + beta * f_odd(y).

    Reference:
        p3-fri two_adic_pcs.rs (TwoAdicFriFolder::fold_row)
    """
    n = len(evals)
    half = n // 2
    beta = ff4(challenge)

    omega = get_omega(log_domain_size)

    folded = []
    for i in range(half):
        f_pos = ff4(evals[i])           # f(x)  where x = shift * omega^i
        f_neg = ff4(evals[i + half])    # f(-x) where -x = shift * omega^(i+N/2)

        x = (coset_shift * pow(omega, i, p)) % p
        half_inv_x = inv_mod((2 * x) % p)  # 1/(2x) mod p

        even = (f_pos + f_neg) * ff4_from_base(TWO_INV)
        odd = (f_pos - f_neg) * ff4_from_base(half_inv_x)
        result = even + beta * odd

        folded.append(ff4_coeffs(result))

    return folded


# --- Verifier: Query Verification ---


def fold_row(
    index: int,
    log_height: int,
    beta: FF4,
    e0: FF4,
    e1: FF4,
) -> FF4:
    """Lagrange interpolation fold at challenge beta.

    Reference:
        p3-fri two_adic_pcs.rs (TwoAdicFriFolding::fold_row)
    """
    # xs0 = two_adic_generator(log_height + 1) ^ reverse_bits_len(index, log_height)
    subgroup_start = pow(W[log_height + 1],
                         reverse_bits_len(index, log_height),
                         p)
    xs0 = ff4_from_base(subgroup_start)

    # Lagrange interpolation: e0 + (beta - xs0) * (e1 - e0) / (xs1 - xs0)
    inv_diff = ff4_from_base(inv_mod((-2 * subgroup_start) % p))
    return e0 + (beta - xs0) * (e1 - e0) * inv_diff


def hash_fri_leaf(e0: FF4, e1: FF4) -> Digest:
    """Hash pair of extension field evaluations as FRI Merkle leaf.

    Reference:
        p3-merkle-tree mmcs.rs (verify_batch leaf hashing)
    """
    return hash_to_digest(ff4_coeffs(e0) + ff4_coeffs(e1))


def fri_verify_query(
    commit_phase_commits: list[Digest],
    betas: list[EF4Coeffs],
    query_index: int,
    query_proof: dict,
    reduced_opening: EF4Coeffs,
    final_poly: list[EF4Coeffs],
    log_max_height: int,
    log_final_poly_len: int,
) -> EF4Coeffs:
    """Verify single FRI query: fold chain + Merkle proofs + final poly check.

    Reference:
        p3-fri verifier.rs (verify_query)
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
    commit_phase_commits: list[Digest],
    final_poly: list[EF4Coeffs],
    query_proofs: list[dict],
    log_blowup: int,
    log_final_poly_len: int,
    num_queries: int,
) -> bool:
    """Verify FRI proof: transcript replay and structural consistency.

    Note: Full fold-chain verification requires reduced_openings from PCS.
    This function verifies transcript replay, query index derivation, and
    proof structure (lengths, digest sizes).

    Reference:
        p3-fri verifier.rs (verify_fri)
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


# --- Prover: Bit-Reversed Folding ---


def ef_idft(evals: list[EF4Coeffs]) -> list[EF4Coeffs]:
    """Inverse DFT for extension field evaluations (channel-wise INTT).

    Reference:
        p3-dft traits.rs (idft_algebra)
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
    evals_bit_reversed: list[EF4Coeffs],
    beta: EF4Coeffs,
    log_height: int,
) -> list[EF4Coeffs]:
    """Fold bit-reversed evaluations: adjacent pairs are conjugates.

    Reference:
        p3-fri two_adic_pcs.rs (TwoAdicFriFolding::fold_matrix)
    """
    beta_ef = ff4(beta)
    height = len(evals_bit_reversed) // 2

    # g_inv = two_adic_generator(log_height + 1)^(-1)
    g_inv = inv_mod(W[log_height + 1])

    # Precompute halve_inv_powers[i] = g_inv^i / 2 (before bit-reversal)
    halve_inv_powers = []
    val = TWO_INV  # (1/2) * g_inv^0 = 1/2
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
        result = (lo + hi) * ff4_from_base(TWO_INV) + \
                 (lo - hi) * beta_ef * ff4_from_base(halve_inv_powers[i])
        folded.append(ff4_coeffs(result))

    return folded


# --- Prover: Commit Phase ---


def commit_phase(
    evals_bit_reversed: list[EF4Coeffs],
    log_blowup: int,
    log_final_poly_len: int,
    challenger: Challenger,
) -> CommitPhaseResult:
    """FRI commit phase: iterative folding with Merkle commitments.

    Reference:
        p3-fri prover.rs (commit_phase)
    """
    folded = list(evals_bit_reversed)
    commits: list[Digest] = []
    betas: list[EF4Coeffs] = []
    trees: list = []
    folded_per_round: list[list[EF4Coeffs]] = []
    all_round_evals: list[list[EF4Coeffs]] = []
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

    return CommitPhaseResult(
        commits=commits,
        betas=betas,
        final_poly=final_poly,
        trees=trees,
        folded_per_round=folded_per_round,
        all_round_evals=all_round_evals,
    )


# --- Prover: Query Phase ---


def answer_query(
    trees: list,
    all_round_evals: list[list[EF4Coeffs]],
    start_index: int,
    num_rounds: int,
) -> list[FriQueryStep]:
    """Generate FRI query opening for a single query index.

    Reference:
        p3-fri prover.rs (answer_query)
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

        openings.append(FriQueryStep(
            sibling_value=sibling_value,
            opening_proof=proof,
        ))
    return openings


def prove_fri(
    evals_bit_reversed: list[EF4Coeffs],
    log_blowup: int,
    log_final_poly_len: int,
    num_queries: int,
    challenger: Challenger,
) -> FriProof:
    """Full FRI proof generation.

    Reference:
        p3-fri prover.rs (prove)
    """
    # Commit phase
    result = commit_phase(
        evals_bit_reversed, log_blowup, log_final_poly_len, challenger
    )

    # Query phase
    log_max_height = len(evals_bit_reversed).bit_length() - 1
    num_rounds = len(result.commits)
    query_proofs = []

    for _ in range(num_queries):
        query_index = challenger.sample_bits(log_max_height)
        openings = answer_query(
            result.trees,
            result.all_round_evals,
            query_index,
            num_rounds,
        )
        query_proofs.append(FriQueryProof(
            index=query_index,
            commit_phase_openings=openings,
        ))

    return FriProof(
        commit_phase_commits=result.commits,
        final_poly=result.final_poly,
        query_proofs=query_proofs,
        betas=result.betas,
        folded_per_round=result.folded_per_round,
    )
