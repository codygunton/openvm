"""FRI (Fast Reed-Solomon IOP of Proximity) protocol.

Implements FRI folding, verification, and proving matching Plonky3's FRI.

Reference:
    p3-fri-0.4.1/src/ (prover.rs, verifier.rs, two_adic_pcs.rs)
"""

from dataclasses import dataclass

import numpy as np

from primitives.field import (
    BABYBEAR_PRIME,
    DIGEST_WIDTH,
    Digest,
    FF4,
    FIELD_EXTENSION_DEGREE,
    FF,
    Fe,
    MerklePath,
    TWO_INV,
    W,
    bit_reverse_list,
    get_omega,
    inv_mod,
    reverse_bits_len,
)
from primitives.merkle import (
    build_merkle_tree,
    get_opening_proof,
    verify_opening_prehashed,
)
from primitives.ntt import ef4_idft
from primitives.poseidon2 import hash_to_digest
from primitives.transcript import Challenger, grind

p = BABYBEAR_PRIME


# --- Data Structures ---


@dataclass
class CommitPhaseResult:
    """Output of FRI commit phase."""
    commits: list[Digest]
    betas: list[list[int]]
    final_poly: list[list[int]]
    trees: list[list[list[Digest]]]
    folded_per_round: list[list[list[int]]]
    all_round_evals: list[list[list[int]]]
    commit_pow_witnesses: list[int]


@dataclass
class FriQueryStep:
    """One round of a FRI query opening."""
    sibling_value: list[int]
    opening_proof: MerklePath


@dataclass
class FriQueryResult:
    """FRI query proof for a single query index."""
    index: int
    commit_phase_openings: list[FriQueryStep]


@dataclass
class CommitPhaseOutput:
    """Complete FRI proof."""
    commit_phase_commits: list[Digest]
    final_poly: list[list[int]]
    query_proofs: list[FriQueryResult]
    betas: list[list[int]]
    folded_per_round: list[list[list[int]]]


# --- Verifier: Natural-Order Folding ---


def fri_fold(
    evals: list[list[int]],
    challenge: list[int],
    log_domain_size: int,
    coset_shift: Fe,
) -> list[list[int]]:
    """Fold evaluations on coset: f_even(y) + beta * f_odd(y).

    Reference:
        p3-fri two_adic_pcs.rs (TwoAdicFriFolder::fold_row)
    """
    n = len(evals)
    half = n // 2
    beta = FF4(challenge)

    omega = get_omega(log_domain_size)

    folded = []
    for i in range(half):
        f_pos = FF4(evals[i])           # f(x)  where x = shift * omega^i
        f_neg = FF4(evals[i + half])    # f(-x) where -x = shift * omega^(i+N/2)

        x = (coset_shift * pow(omega, i, p)) % p
        half_inv_x = inv_mod((2 * x) % p)  # 1/(2x) mod p

        even = (f_pos + f_neg).mul_base(TWO_INV)
        odd = (f_pos - f_neg).mul_base(half_inv_x)
        result = even + beta * odd

        folded.append(result.to_list())

    return folded


# --- Verifier: Query Verification ---


# <doc-anchor id="fri-fold">
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
    beta, e0, e1 = FF4(beta), FF4(e0), FF4(e1)
    # x_even = two_adic_generator(log_height + 1) ^ reverse_bits_len(index, log_height)
    subgroup_start = pow(W[log_height + 1],
                         reverse_bits_len(index, log_height),
                         p)
    x_even = FF4(subgroup_start)

    # Lagrange interpolation: e0 + (beta - x_even) * (e1 - e0) / (x_odd - x_even)
    inv_diff = FF4(inv_mod((-2 * subgroup_start) % p))
    return e0 + (beta - x_even) * (e1 - e0) * inv_diff


# <doc-anchor id="hash-fri-leaf">
def hash_fri_leaf(e0: FF4, e1: FF4) -> Digest:
    """Hash pair of extension field evaluations as FRI Merkle leaf.

    Reference:
        p3-merkle-tree mmcs.rs (verify_batch leaf hashing)
    """
    e0, e1 = FF4(e0), FF4(e1)
    return hash_to_digest(e0.to_list() + e1.to_list())


def ef4_pairs_to_leaves(evals: list[list[int]]) -> list[list[int]]:
    """Pair consecutive FF4 elements into 8-element Merkle leaves."""
    return [evals[i] + evals[i + 1] for i in range(0, len(evals), 2)]


def fri_verify_query(
    commit_phase_commits: list[Digest],
    betas: list[list[int]],
    query_index: int,
    query_proof: dict,
    reduced_opening: list[int],
    final_poly: list[list[int]],
    log_max_height: int,
    log_final_poly_len: int,
) -> list[int]:
    """Verify single FRI query: fold chain + Merkle proofs + final poly check.

    Reference:
        p3-fri verifier.rs (verify_query)
    """
    num_rounds = len(commit_phase_commits)
    start_index = query_index
    folded_eval = FF4(reduced_opening)

    for round_idx in range(num_rounds):
        log_folded_height = log_max_height - 1 - round_idx
        opening = query_proof["commit_phase_openings"][round_idx]
        sibling = FF4(opening["sibling_value"])
        beta = FF4(betas[round_idx])

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
        expected = FF4(final_poly[0])
    else:
        x = pow(W[log_final_poly_len],
                reverse_bits_len(start_index, log_final_poly_len), p)
        expected = FF4.zero()
        for i, c in enumerate(final_poly):
            expected = expected + FF4(c) * FF4(pow(x, i, p))

    assert folded_eval.to_list() == expected.to_list(), (
        f"Final polynomial check failed: "
        f"{folded_eval.to_list()} != {expected.to_list()}"
    )

    return folded_eval.to_list()


# <doc-anchor id="verify-fri">
def verify_fri(
    commit_phase_commits: list[Digest],
    final_poly: list[list[int]],
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

            # Verify sibling value is an extension field element
            assert len(opening["sibling_value"]) == FIELD_EXTENSION_DEGREE, (
                f"Query {qi}, round {round_idx}: "
                f"sibling_value should have {FIELD_EXTENSION_DEGREE} components"
            )

            # Verify each Merkle sibling is a valid digest
            for si, sibling in enumerate(proof):
                assert len(sibling) == DIGEST_WIDTH, (
                    f"Query {qi}, round {round_idx}, sibling {si}: "
                    f"digest should have {DIGEST_WIDTH} elements, got {len(sibling)}"
                )

    return True


# --- Prover: Bit-Reversed Folding ---


# <doc-anchor id="fold-matrix">
def fold_matrix(
    evals_bit_reversed: list[list[int]],
    beta: FF4,
    log_height: int,
) -> list[list[int]]:
    """Fold bit-reversed evaluations: adjacent pairs are conjugates.

    Reference:
        p3-fri two_adic_pcs.rs (TwoAdicFriFolding::fold_matrix)
    """
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

    # Vectorized fold
    beta = FF4(beta)
    lo = FF4.from_rows(evals_bit_reversed[0::2])
    hi = FF4.from_rows(evals_bit_reversed[1::2])

    hip = FF(halve_inv_powers)
    two_inv_col = FF(np.full(height, TWO_INV))

    # result = (lo + hi) * TWO_INV + (lo - hi) * beta * halve_inv_power
    sum_half = (lo + hi).mul_base(two_inv_col)
    diff_beta = (lo - hi) * beta
    diff_beta_hip = diff_beta.mul_base(hip)
    result = sum_half + diff_beta_hip

    return result.to_rows()


# --- Prover: Commit Phase ---


# <doc-anchor id="commit-phase">
def commit_phase(
    evals_bit_reversed: list[list[int]],
    log_blowup: int,
    log_final_poly_len: int,
    challenger: Challenger,
    commit_pow_bits: int = 0,
    reduced_openings_by_height: dict[int, list[list[int]]] | None = None,
) -> CommitPhaseResult:
    """FRI commit phase: iterative folding with Merkle commitments.

    For multi-height FRI, reduced_openings_by_height maps log_height to
    bit-reversed reduced evaluations at that height.  After folding to a
    given height, the corresponding reduced opening is rolled in using
    beta^2 as the combination factor (matching the verifier).

    Reference:
        p3-fri prover.rs (commit_phase)
    """
    folded = list(evals_bit_reversed)
    commits: list[Digest] = []
    betas: list[list[int]] = []
    trees: list[list[list[Digest]]] = []
    folded_per_round: list[list[list[int]]] = []
    all_round_evals: list[list[list[int]]] = []
    commit_pow_witnesses: list[int] = []
    blowup = 1 << log_blowup
    final_poly_len = 1 << log_final_poly_len
    while len(folded) > blowup * final_poly_len:
        height = len(folded) // 2
        log_height = height.bit_length() - 1

        # Store current evals for query phase
        all_round_evals.append(folded)

        # Build Merkle tree from pairs of evaluations.
        # Each leaf = hash of [lo_c0..lo_c3, hi_c0..hi_c3] (8 base field elements).
        leaves = ef4_pairs_to_leaves(folded)
        root, tree = build_merkle_tree(leaves)

        # Observe commitment
        challenger.observe_many(root)
        commits.append(root)
        trees.append(tree)

        # Per-round PoW
        pow_witness = grind(challenger, commit_pow_bits)
        commit_pow_witnesses.append(pow_witness)

        # Sample folding challenge
        beta = challenger.sample_ext()
        betas.append(beta.to_list())

        # Fold
        folded = fold_matrix(folded, beta, log_height)

        # Roll in reduced openings at this folded height, if any.
        # This mirrors the verifier's: folded += beta^2 * reduced_opening
        # Reference: p3-fri verifier.rs verify_query (line ~310)
        if reduced_openings_by_height and log_height in reduced_openings_by_height:
            roll_in_data = reduced_openings_by_height[log_height]
            assert len(roll_in_data) == len(folded), (
                f"Roll-in size mismatch at log_height {log_height}: "
                f"{len(roll_in_data)} vs {len(folded)}"
            )
            beta_sq = beta * beta
            fv = FF4.from_rows(folded)
            rv = FF4.from_rows(roll_in_data)
            result = fv + rv * beta_sq
            folded = result.to_rows()

        folded_per_round.append(folded)

    # Compute final polynomial via IDFT:
    # Truncate to final_poly_len, bit-reverse, IDFT.
    final_evals = folded[:final_poly_len]
    final_evals_natural = bit_reverse_list(final_evals)
    final_poly = ef4_idft(final_evals_natural)

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
        commit_pow_witnesses=commit_pow_witnesses,
    )


# --- Prover: Query Phase ---


# <doc-anchor id="answer-query">
def answer_query(
    trees: list[list[list[Digest]]],
    all_round_evals: list[list[list[int]]],
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


# <doc-anchor id="prove-fri">
def prove_fri(
    evals_bit_reversed: list[list[int]],
    log_blowup: int,
    log_final_poly_len: int,
    num_queries: int,
    challenger: Challenger,
) -> CommitPhaseOutput:
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
        query_proofs.append(FriQueryResult(
            index=query_index,
            commit_phase_openings=openings,
        ))

    return CommitPhaseOutput(
        commit_phase_commits=result.commits,
        final_poly=result.final_poly,
        query_proofs=query_proofs,
        betas=result.betas,
        folded_per_round=result.folded_per_round,
    )
