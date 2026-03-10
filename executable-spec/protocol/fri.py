"""FRI (Fast Reed-Solomon IOP of Proximity) protocol.

Implements FRI folding and verification matching Plonky3's FRI implementation.

Reference:
    p3-fri-0.4.1/src/ (verifier.rs, two_adic_pcs.rs)
    crates/test-vectors/tests/verify_fri_vectors.rs
"""

from primitives.field import (
    BABYBEAR_PRIME,
    ff4,
    ff4_coeffs,
    ff4_from_base,
    get_omega,
)
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
