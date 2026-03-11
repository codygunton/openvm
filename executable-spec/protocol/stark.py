"""STARK verifier orchestration.

Top-level verifier that coordinates transcript operations, PCS verification,
and constraint checking for a multi-AIR STARK proof.

The Fiat-Shamir transcript operations must match the Rust verifier EXACTLY in
order: any deviation produces wrong challenges and breaks verification.

Reference:
    stark-backend/src/verifier/mod.rs  -- MultiTraceStarkVerifier::verify_raps
    stark-backend/src/verifier/error.rs -- VerificationError enum
    stark-backend/src/interaction/fri_log_up.rs -- FriLogUpPhase::partially_verify
"""

from __future__ import annotations

from typing import Optional

from primitives.field import BABYBEAR_PRIME, Digest, EF4Coeffs, Fe, ef4_add
from primitives.transcript import Challenger, check_witness, grind
from protocol.constraints import (
    VerificationError,
    verify_single_rap_constraints,
)
from protocol.domain import (
    TwoAdicMultiplicativeCoset,
    create_disjoint_domain,
    natural_domain_for_degree,
)
from protocol.pcs import PcsRound, pcs_verify
from protocol.proof import (
    AdjacentOpenedValues,
    AirProofData,
    BatchOpening,
    Commitments,
    CommitPhaseProofStep,
    FriLogUpPartialProof,
    FriParameters,
    FriProof as ProofFriProof,
    MultiStarkVerifyingKey,
    OpenedValues,
    OpeningProof,
    Proof,
    QueryProof,
    StarkVerifyingKey,
)

p = BABYBEAR_PRIME

# Number of challenges and exposed values for FRI LogUp phase.
# Reference: stark-backend/src/interaction/fri_log_up.rs lines 240-241
STARK_LU_NUM_CHALLENGES = 2
STARK_LU_NUM_EXPOSED_VALUES = 1

# Extension field degree (BabyBear quartic extension).
EXT_DEGREE = 4


# ---------------------------------------------------------------------------
# Verification errors
# ---------------------------------------------------------------------------
# VerificationError and OodEvaluationMismatch are defined in protocol.constraints
# and re-exported here (imported above) to avoid circular imports.


class InvalidProofShape(VerificationError):
    """Proof structure does not match the verifying key.

    Reference:
        stark-backend/src/verifier/error.rs (VerificationError::InvalidProofShape)
    """

    def __init__(self, message: str = "") -> None:
        super().__init__(f"InvalidProofShape: {message}" if message else "InvalidProofShape")


class InvalidOpeningArgument(VerificationError):
    """PCS opening verification failed.

    Reference:
        stark-backend/src/verifier/error.rs (VerificationError::InvalidOpeningArgument)
    """

    def __init__(self, message: str = "") -> None:
        super().__init__(f"InvalidOpeningArgument: {message}" if message else "InvalidOpeningArgument")


class ChallengePhaseError(VerificationError):
    """RAP challenge phase verification failed (e.g. non-zero cumulative sum).

    Reference:
        stark-backend/src/verifier/error.rs (VerificationError::ChallengePhaseError)
    """

    def __init__(self, message: str = "") -> None:
        super().__init__(f"ChallengePhaseError: {message}" if message else "ChallengePhaseError")


class InvalidDeepPowWitness(VerificationError):
    """DEEP proof-of-work witness is invalid.

    Reference:
        stark-backend/src/verifier/error.rs (VerificationError::InvalidDeepPowWitness)
    """

    def __init__(self, message: str = "") -> None:
        super().__init__(f"InvalidDeepPowWitness: {message}" if message else "InvalidDeepPowWitness")


# ---------------------------------------------------------------------------
# Helper: VK view (analogous to MultiStarkVerifyingKeyView)
# ---------------------------------------------------------------------------


def _get_vk_view(
    vk: MultiStarkVerifyingKey,
    air_ids: list[int],
) -> list[StarkVerifyingKey]:
    """Index into the VK's per_air list by air_id.

    Reference:
        stark-backend/src/keygen/view.rs (MultiStarkVerifyingKey::view)
    """
    return [vk.inner.per_air[air_id] for air_id in air_ids]


def _has_common_main(svk: StarkVerifyingKey) -> bool:
    """Check if this AIR has a common main trace.

    Reference:
        stark-backend/src/keygen/types.rs (StarkVerifyingKey::has_common_main)
    """
    return svk.params.width.common_main != 0


def _has_interaction(svk: StarkVerifyingKey) -> bool:
    """Check if this AIR has bus interactions.

    Reference:
        stark-backend/src/keygen/types.rs (StarkVerifyingKey::has_interaction)
    """
    return len(svk.symbolic_constraints.interactions) > 0


def _num_cached_mains(svk: StarkVerifyingKey) -> int:
    """Number of cached main trace partitions.

    Reference:
        stark-backend/src/keygen/types.rs (StarkVerifyingKey::num_cached_mains)
    """
    return len(svk.params.width.cached_mains)


def _num_phases(per_air_vks: list[StarkVerifyingKey]) -> int:
    """Maximum number of challenge phases across all AIRs.

    Reference:
        stark-backend/src/keygen/view.rs (MultiStarkVerifyingKeyView::num_phases)
    """
    if not per_air_vks:
        return 0
    return max(len(vk.params.width.after_challenge) for vk in per_air_vks)


def _flattened_preprocessed_commits(per_air_vks: list[StarkVerifyingKey]) -> list[Digest]:
    """Return all non-None preprocessed commitments.

    Reference:
        stark-backend/src/keygen/view.rs
        (MultiStarkVerifyingKeyView::flattened_preprocessed_commits)
    """
    commits = []
    for vk in per_air_vks:
        if vk.preprocessed_data is not None:
            commits.append(vk.preprocessed_data.commit)
    return commits


def _preprocessed_commits(per_air_vks: list[StarkVerifyingKey]) -> list[Optional[Digest]]:
    """Return preprocessed commit for each AIR (None if not present).

    Reference:
        stark-backend/src/keygen/view.rs
        (MultiStarkVerifyingKeyView::preprocessed_commits)
    """
    result = []
    for vk in per_air_vks:
        if vk.preprocessed_data is not None:
            result.append(vk.preprocessed_data.commit)
        else:
            result.append(None)
    return result


# ---------------------------------------------------------------------------
# Helper: RAP phase partial verification (FRI LogUp)
# ---------------------------------------------------------------------------


def _partially_verify_fri_log_up(
    challenger: Challenger,
    partial_proof: Optional[FriLogUpPartialProof],
    exposed_values_per_air_per_phase: list[list[list[EF4Coeffs]]],
    commitments_per_phase: list,  # list[Digest]
    log_up_pow_bits: int,
) -> tuple[list[list[EF4Coeffs]], Optional[str]]:
    """Partially verify the FRI LogUp challenge phase.

    Returns (challenges_per_phase, error_or_none).

    The challenges_per_phase is always returned (even on error) because the Rust
    verifier continues past ChallengePhaseError to check OodEvaluationMismatch.

    Reference:
        stark-backend/src/interaction/fri_log_up.rs
        (FriLogUpPhase::partially_verify, lines 153-237)
    """
    # Check if any AIR has exposed values (i.e., has interactions)
    has_any = any(
        len(ev_per_phase) > 0
        for ev_per_phase in exposed_values_per_air_per_phase
    )

    if not has_any:
        # No interactions at all: no challenges, no error
        return [], None

    # There are interactions, so we need a partial proof
    if partial_proof is None:
        return [], "MissingPartialProof"

    # PoW check for LogUp security
    # Reference: fri_log_up.rs lines 181-189
    if not check_witness(challenger, log_up_pow_bits, partial_proof.logup_pow_witness):
        return [], "InvalidPowWitness"

    # Sample interaction challenges (2 for FRI LogUp)
    # Reference: fri_log_up.rs lines 191-192
    challenges: list[EF4Coeffs] = []
    for _ in range(STARK_LU_NUM_CHALLENGES):
        challenges.append(challenger.sample_ext())

    # Observe exposed values (cumulative sums)
    # Reference: fri_log_up.rs lines 194-200
    for exposed_values_per_phase in exposed_values_per_air_per_phase:
        if exposed_values_per_phase:
            exposed_values = exposed_values_per_phase[0]  # .first()
            for exposed_value in exposed_values:
                # exposed_value is EF4Coeffs (4 base field elements)
                # challenger.observe_slice(exposed_value.as_base_slice())
                challenger.observe_many(exposed_value)

    # Observe after_challenge commitment
    # Reference: fri_log_up.rs line 202
    challenger.observe_many(commitments_per_phase[0])

    # Check cumulative sum == 0
    # Reference: fri_log_up.rs lines 204-232
    # Sum all cumulative sums (exposed_values[0][0] for each AIR that has them)
    cumulative_sum = [0, 0, 0, 0]
    for exposed_values_per_phase in exposed_values_per_air_per_phase:
        if exposed_values_per_phase and exposed_values_per_phase[0]:
            assert len(exposed_values_per_phase) <= 1, (
                "Verifier does not support more than 1 challenge phase"
            )
            assert len(exposed_values_per_phase[0]) == 1, (
                "Only exposed value should be cumulative sum"
            )
            csum = exposed_values_per_phase[0][0]
            cumulative_sum = ef4_add(cumulative_sum, csum)

    error = None
    if cumulative_sum != [0, 0, 0, 0]:
        error = "NonZeroCumulativeSum"

    return [challenges], error


# ---------------------------------------------------------------------------
# Helper: Build PCS rounds from proof data
# ---------------------------------------------------------------------------


def _build_trace_domain_and_openings(
    domain: TwoAdicMultiplicativeCoset,
    zeta: EF4Coeffs,
    values: AdjacentOpenedValues,
) -> tuple[TwoAdicMultiplicativeCoset, list[tuple]]:
    """Build (domain, [(zeta, local_values), (next_point, next_values)]).

    Reference:
        stark-backend/src/verifier/mod.rs lines 221-226
        (trace_domain_and_openings closure)
    """
    next_point = domain.next_point(zeta)
    return (
        domain,
        [
            (zeta, values.local),
            (next_point, values.next),
        ],
    )


# ---------------------------------------------------------------------------
# Main verifier
# ---------------------------------------------------------------------------


def verify_stark(
    vk: MultiStarkVerifyingKey,
    proof: Proof,
    fri_params: FriParameters,
) -> None:
    """Verify a multi-AIR STARK proof.

    This is the top-level verification function. It must reproduce the Rust
    verifier's Fiat-Shamir transcript operations in EXACTLY the same order.

    Args:
        vk: The multi-AIR verifying key.
        proof: The complete STARK proof.
        fri_params: FRI protocol parameters.

    Raises:
        VerificationError: If any verification check fails.

    Reference:
        stark-backend/src/verifier/mod.rs
        (MultiTraceStarkVerifier::verify + verify_raps, lines 38-430)
    """
    # --- Phase 1: Setup ---
    # Reference: mod.rs lines 46-47
    air_ids = [ap.air_id for ap in proof.per_air]
    per_air_vks = _get_vk_view(vk, air_ids)
    num_airs = len(air_ids)

    # --- Phase 2: Transcript initialization ---
    challenger = Challenger()

    # (1) Observe VK pre_hash (8 u32s, each observed individually)
    # Reference: mod.rs line 66
    challenger.observe_many(vk.pre_hash)

    # (2) Observe number of AIRs
    # Reference: mod.rs line 69
    # Rust: challenger.observe(Val::from_usize(num_airs)) puts Montgomery form into sponge.
    # Python: our Poseidon2 FFI converts canonical -> Montgomery internally, so we
    # observe the canonical integer value. The FFI equivalence is:
    #   Rust sponge[i] = to_monty(x)
    #   Python sponge[i] = x, then FFI does BabyBear::new(x) = to_monty(x) before permute.
    challenger.observe(num_airs)

    # (3) Observe each air_id
    # Reference: mod.rs lines 70-72
    # Same canonical-form reasoning as num_airs above.
    for air_id in air_ids:
        challenger.observe(air_id)

    # --- Trace height constraint checks ---
    # Reference: mod.rs lines 74-83
    for constraint in vk.inner.trace_height_constraints:
        total = sum(
            constraint.coefficients[ap.air_id] * ap.degree
            for ap in proof.per_air
        )
        if total >= constraint.threshold:
            raise InvalidProofShape(
                f"trace height constraint violated: {total} >= {constraint.threshold}"
            )

    # --- Check air_ids are unique and sorted ---
    # Reference: mod.rs lines 85-93
    sorted_ids = sorted(air_ids)
    for i in range(len(sorted_ids) - 1):
        if sorted_ids[i] >= sorted_ids[i + 1]:
            raise VerificationError("DuplicateAirs: duplicate air_id found")

    # --- Get public values per AIR ---
    # Reference: mod.rs lines 96-104
    public_values = [ap.public_values for ap in proof.per_air]
    for pvs, svk in zip(public_values, per_air_vks):
        if len(pvs) != svk.params.num_public_values:
            raise InvalidProofShape(
                f"public values count mismatch: {len(pvs)} vs {svk.params.num_public_values}"
            )

    # (4) Observe public values for each AIR
    # Reference: mod.rs lines 106-108
    for pis in public_values:
        challenger.observe_many(pis)

    # (5) Observe preprocessed commitments
    # Reference: mod.rs lines 110-112
    for preprocessed_commit in _flattened_preprocessed_commits(per_air_vks):
        challenger.observe_many(preprocessed_commit)

    # --- Validate main trace commit count ---
    # Reference: mod.rs lines 115-125
    num_cached_mains = sum(
        len(svk.params.width.cached_mains) for svk in per_air_vks
    )
    num_main_commits = num_cached_mains + 1  # always 1 common main
    if len(proof.commitments.main_trace) != num_main_commits:
        raise InvalidProofShape(
            f"main trace commit count mismatch: "
            f"{len(proof.commitments.main_trace)} vs {num_main_commits}"
        )

    # (6) Observe main trace commitments
    # Reference: mod.rs line 127
    # challenger.observe_slice(&proof.commitments.main_trace)
    # Each commitment is a Digest (8 u32s), and observe(Hash) observes each element.
    for commit in proof.commitments.main_trace:
        challenger.observe_many(commit)

    # (7) Observe log2(degree) for each AIR
    # Reference: mod.rs lines 128-134
    # Same canonical-form reasoning as num_airs above.
    for ap in proof.per_air:
        log_degree = ap.degree.bit_length() - 1
        assert 1 << log_degree == ap.degree, f"degree must be power of 2: {ap.degree}"
        challenger.observe(log_degree)

    # --- Phase 3: RAP (interaction) challenge phase ---
    # Reference: mod.rs lines 136-184

    exposed_values_per_air_per_phase = [
        ap.exposed_values_after_challenge for ap in proof.per_air
    ]

    num_phases = _num_phases(per_air_vks)
    if num_phases != len(proof.commitments.after_challenge) or num_phases > 1:
        raise InvalidProofShape(
            f"after_challenge phase count mismatch: "
            f"num_phases={num_phases}, commits={len(proof.commitments.after_challenge)}"
        )

    # Validate exposed_values shape (T01c)
    # Reference: mod.rs lines 166-172
    for ev_per_phase, svk in zip(exposed_values_per_air_per_phase, per_air_vks):
        if len(ev_per_phase) != len(svk.params.num_exposed_values_after_challenge):
            raise InvalidProofShape("exposed_values_after_challenge phase count mismatch")
        for ev, n in zip(ev_per_phase, svk.params.num_exposed_values_after_challenge):
            if len(ev) != n:
                raise InvalidProofShape(
                    f"exposed_values count mismatch: {len(ev)} vs {n}"
                )

    # Call RAP phase partial verification
    # Reference: mod.rs lines 174-184
    challenges_per_phase, rap_phase_error = _partially_verify_fri_log_up(
        challenger,
        proof.rap_phase_seq_proof,
        exposed_values_per_air_per_phase,
        proof.commitments.after_challenge,
        vk.inner.log_up_pow_bits,
    )
    # Don't bail on error yet -- OodEvaluationMismatch takes precedence
    # Reference: mod.rs lines 181-184

    # --- Phase 4: Quotient challenge ---

    # Sample alpha (constraint combination challenge)
    # Reference: mod.rs lines 187-188
    alpha: EF4Coeffs = challenger.sample_ext()

    # Observe quotient commitment (8 u32s)
    # Reference: mod.rs line 191
    challenger.observe_many(proof.commitments.quotient)

    # DEEP proof-of-work check
    # Reference: mod.rs lines 193-198
    deep_pow_bits = vk.inner.deep_pow_bits
    if not check_witness(challenger, deep_pow_bits, proof.opening.deep_pow_witness):
        raise InvalidDeepPowWitness("DEEP proof-of-work witness is invalid")

    # Sample zeta (OOD evaluation point)
    # Reference: mod.rs lines 199-201
    zeta: EF4Coeffs = challenger.sample_ext()

    # --- Phase 5: Build domains ---
    # Reference: mod.rs lines 204-218
    domains: list[TwoAdicMultiplicativeCoset] = []
    quotient_chunks_domains: list[list[TwoAdicMultiplicativeCoset]] = []

    for svk, ap in zip(per_air_vks, proof.per_air):
        degree = ap.degree
        quotient_degree = svk.quotient_degree
        domain = natural_domain_for_degree(degree)
        quotient_domain = create_disjoint_domain(domain, degree * quotient_degree)
        qc_doms = quotient_domain.split_domains(quotient_degree)
        domains.append(domain)
        quotient_chunks_domains.append(qc_doms)

    # --- Phase 6: Build PCS rounds and observe opened values ---
    opened_values = proof.opening.values

    # 1. Preprocessed trace openings
    # Reference: mod.rs lines 231-258
    preprocessed_widths = [
        svk.params.width.preprocessed
        for svk in per_air_vks
        if svk.params.width.preprocessed is not None
    ]
    if len(preprocessed_widths) != len(opened_values.preprocessed):
        raise InvalidProofShape(
            f"preprocessed count mismatch: "
            f"{len(preprocessed_widths)} vs {len(opened_values.preprocessed)}"
        )
    for w, ov in zip(preprocessed_widths, opened_values.preprocessed):
        if w != len(ov.local) or w != len(ov.next):
            raise InvalidProofShape("preprocessed width mismatch")

    rounds: list[PcsRound] = []

    # Build preprocessed rounds: each AIR with preprocessed gets its own round
    prep_commits = _preprocessed_commits(per_air_vks)
    prep_idx = 0
    for i_air in range(num_airs):
        commit = prep_commits[i_air]
        if commit is not None:
            domain = domains[i_air]
            values = opened_values.preprocessed[prep_idx]
            dom_and_openings = _build_trace_domain_and_openings(domain, zeta, values)
            rounds.append(PcsRound(
                commitment=commit,
                domains_and_openings=[dom_and_openings],
            ))
            prep_idx += 1

    # 2. Main trace openings
    # Reference: mod.rs lines 260-303
    num_main_commits_ov = len(opened_values.main)
    if num_main_commits_ov != len(proof.commitments.main_trace):
        raise InvalidProofShape("main opened values count != main trace commits")

    main_commit_idx = 0

    # Cached main traces (all commits except the last)
    # Reference: mod.rs lines 268-276
    for svk, domain in zip(per_air_vks, domains):
        for cached_main_width in svk.params.width.cached_mains:
            commit = proof.commitments.main_trace[main_commit_idx]
            if len(opened_values.main[main_commit_idx]) != 1:
                raise InvalidProofShape("cached main should have exactly 1 matrix")
            value = opened_values.main[main_commit_idx][0]
            if cached_main_width != len(value.local) or cached_main_width != len(value.next):
                raise InvalidProofShape("cached main width mismatch")
            dom_and_openings = _build_trace_domain_and_openings(domain, zeta, value)
            rounds.append(PcsRound(
                commitment=commit,
                domains_and_openings=[dom_and_openings],
            ))
            main_commit_idx += 1

    # Common main trace (last commit, multiple matrices for different AIRs)
    # Reference: mod.rs lines 283-303
    values_per_mat = opened_values.main[main_commit_idx]
    commit = proof.commitments.main_trace[main_commit_idx]
    common_main_domains_and_openings = []
    mat_idx = 0
    for svk, domain in zip(per_air_vks, domains):
        if _has_common_main(svk):
            if mat_idx >= len(values_per_mat):
                raise InvalidProofShape("not enough common main matrices")
            values = values_per_mat[mat_idx]
            width = svk.params.width.common_main
            if width != len(values.local) or width != len(values.next):
                raise InvalidProofShape("common main width mismatch")
            dom_and_openings = _build_trace_domain_and_openings(domain, zeta, values)
            common_main_domains_and_openings.append(dom_and_openings)
            mat_idx += 1
    if len(common_main_domains_and_openings) != len(values_per_mat):
        raise InvalidProofShape("common main matrix count mismatch")
    rounds.append(PcsRound(
        commitment=commit,
        domains_and_openings=common_main_domains_and_openings,
    ))

    # 3. After-challenge trace openings (at most 1 phase)
    # Reference: mod.rs lines 305-340
    has_any_interaction = any(_has_interaction(svk) for svk in per_air_vks)

    if not has_any_interaction:
        if (proof.commitments.after_challenge
                or opened_values.after_challenge):
            raise InvalidProofShape("no interactions but after_challenge data present")
        assert num_phases == 0
    else:
        if num_phases != 1 or len(opened_values.after_challenge) != 1:
            raise InvalidProofShape("after_challenge shape mismatch")
        after_challenge_commit = proof.commitments.after_challenge[0]
        ac_domains_and_openings = []
        ac_mat_idx = 0
        for svk, domain in zip(per_air_vks, domains):
            if _has_interaction(svk):
                if ac_mat_idx >= len(opened_values.after_challenge[0]):
                    raise InvalidProofShape("not enough after_challenge matrices")
                values = opened_values.after_challenge[0][ac_mat_idx]
                width = svk.params.width.after_challenge[0] * EXT_DEGREE
                if width != len(values.local) or width != len(values.next):
                    raise InvalidProofShape("after_challenge width mismatch")
                dom_and_openings = _build_trace_domain_and_openings(
                    domain, zeta, values
                )
                ac_domains_and_openings.append(dom_and_openings)
                ac_mat_idx += 1
        if len(ac_domains_and_openings) != len(opened_values.after_challenge[0]):
            raise InvalidProofShape("after_challenge matrix count mismatch")
        rounds.append(PcsRound(
            commitment=after_challenge_commit,
            domains_and_openings=ac_domains_and_openings,
        ))

    # 4. Quotient openings
    # Reference: mod.rs lines 341-366
    if len(opened_values.quotient) != num_airs:
        raise InvalidProofShape(
            f"quotient count mismatch: {len(opened_values.quotient)} vs {num_airs}"
        )
    for per_air_q, svk in zip(opened_values.quotient, per_air_vks):
        if len(per_air_q) != svk.quotient_degree:
            raise InvalidProofShape("quotient chunk count mismatch")
        for chunk in per_air_q:
            if len(chunk) != EXT_DEGREE:
                raise InvalidProofShape("quotient chunk width mismatch")

    quotient_domains_and_openings = []
    for air_chunks, qc_doms in zip(opened_values.quotient, quotient_chunks_domains):
        for chunk_values, qc_domain in zip(air_chunks, qc_doms):
            quotient_domains_and_openings.append(
                (qc_domain, [(zeta, chunk_values)])
            )
    rounds.append(PcsRound(
        commitment=proof.commitments.quotient,
        domains_and_openings=quotient_domains_and_openings,
    ))

    # --- Phase 6b: PCS verification ---
    # Reference: mod.rs lines 362-363 (pcs.verify)
    try:
        pcs_verify(rounds, proof.opening.proof, challenger, fri_params)
    except (AssertionError, Exception) as e:
        raise InvalidOpeningArgument(str(e)) from e

    # --- Phase 7: Constraint verification ---
    # Reference: mod.rs lines 365-424

    preprocessed_idx = 0
    after_challenge_idx = [0] * num_phases
    cached_main_commit_idx = 0
    common_main_matrix_idx = 0

    for i_air in range(num_airs):
        domain = domains[i_air]
        qc_doms = quotient_chunks_domains[i_air]
        quotient_chunks = opened_values.quotient[i_air]
        svk = per_air_vks[i_air]
        air_proof = proof.per_air[i_air]

        # Preprocessed values
        # Reference: mod.rs lines 378-382
        preprocessed_values: Optional[AdjacentOpenedValues] = None
        if svk.preprocessed_data is not None:
            preprocessed_values = opened_values.preprocessed[preprocessed_idx]
            preprocessed_idx += 1

        # Partitioned main values
        # Reference: mod.rs lines 383-397
        partitioned_main_values: list[AdjacentOpenedValues] = []
        for _ in range(_num_cached_mains(svk)):
            partitioned_main_values.append(
                opened_values.main[cached_main_commit_idx][0]
            )
            cached_main_commit_idx += 1
        if _has_common_main(svk):
            partitioned_main_values.append(
                opened_values.main[-1][common_main_matrix_idx]
            )
            common_main_matrix_idx += 1

        # After challenge values
        # Reference: mod.rs lines 394-410
        after_challenge_values: list[AdjacentOpenedValues] = []
        if _has_interaction(svk):
            for phase_idx in range(num_phases):
                matrix_idx = after_challenge_idx[phase_idx]
                after_challenge_idx[phase_idx] += 1
                after_challenge_values.append(
                    opened_values.after_challenge[phase_idx][matrix_idx]
                )

        # Verify constraints for this AIR
        # Reference: mod.rs lines 405-418
        verify_single_rap_constraints(
            constraints=svk.symbolic_constraints.constraints,
            preprocessed_values=preprocessed_values,
            partitioned_main_values=partitioned_main_values,
            after_challenge_values=after_challenge_values,
            quotient_chunks=quotient_chunks,
            domain=domain,
            qc_domains=qc_doms,
            zeta=zeta,
            alpha=alpha,
            challenges=challenges_per_phase,
            public_values=air_proof.public_values,
            exposed_values_after_challenge=air_proof.exposed_values_after_challenge,
        )

    # If we made it this far without OodEvaluationMismatch, check the RAP phase result
    # Reference: mod.rs lines 422-428
    if rap_phase_error is not None:
        raise ChallengePhaseError(rap_phase_error)


# ===========================================================================
# STARK Prover
# ===========================================================================


def prove_stark(
    vk: MultiStarkVerifyingKey,
    traces: list[list[list[Fe]]],
    public_values_per_air: list[list[Fe]],
    fri_params: FriParameters,
    preprocessed_traces: Optional[list[Optional[list[list[Fe]]]]] = None,
    cached_main_traces: Optional[list[list[list[list[Fe]]]]] = None,
    air_ids: Optional[list[int]] = None,
) -> Proof:
    """Generate a multi-AIR STARK proof.

    Prover counterpart to verify_stark. Reproduces the same Fiat-Shamir
    transcript operations so that the resulting proof is accepted by the
    verifier.

    Args:
        vk: The multi-AIR verifying key.
        traces: Per-AIR common main trace matrices (list of rows).
        public_values_per_air: Per-AIR public values.
        fri_params: FRI protocol parameters.
        preprocessed_traces: Per-AIR preprocessed trace matrices (None if absent).
        cached_main_traces: Per-AIR list of cached main trace matrices.
        air_ids: Maps proof index to VK AIR index. If None, assumes identity
            mapping (all VK AIRs have data).

    Returns:
        A Proof matching the Rust prover output.

    Reference:
        stark-backend/src/prover/coordinator.rs prove
    """
    import time as _time
    _t0 = _time.time()
    def _log(msg):
        print(f"  [prove_stark {_time.time()-_t0:6.1f}s] {msg}", flush=True)

    from protocol.pcs import (
        CommittedData,
        PcsOpeningRound,
        _generate_batch_opening,
        pcs_commit,
        pcs_open,
    )
    from protocol.quotient import compute_quotient_chunks

    if air_ids is None:
        air_ids = list(range(len(vk.inner.per_air)))
    per_air_vks = _get_vk_view(vk, air_ids)
    num_airs = len(air_ids)

    # --- Phase 1: Transcript initialization ---
    challenger = Challenger()

    # (1) Observe VK pre_hash
    challenger.observe_many(vk.pre_hash)

    # (2) Observe number of AIRs
    challenger.observe(num_airs)

    # (3) Observe each air_id
    for air_id in air_ids:
        challenger.observe(air_id)

    # --- Phase 2: Commit traces ---
    # Build domains for each AIR
    domains: list[TwoAdicMultiplicativeCoset] = []
    for i_air in range(num_airs):
        # Determine height from available trace data
        if traces[i_air]:
            height = len(traces[i_air])
        elif cached_main_traces and cached_main_traces[i_air]:
            height = len(cached_main_traces[i_air][0])
        elif preprocessed_traces and preprocessed_traces[i_air] is not None:
            height = len(preprocessed_traces[i_air])
        else:
            raise ValueError(f"AIR {i_air} has no trace data to determine height")
        domain = natural_domain_for_degree(height)
        domains.append(domain)

    _log(f"start: {num_airs} AIRs, air_ids={air_ids}")

    # (a) Commit preprocessed traces (each AIR gets its own commitment).
    # Re-commit from raw data so we have Merkle trees for PCS openings.
    # Verify roots match VK to ensure consistency.
    preprocessed_committed_list: list[tuple[int, CommittedData]] = []
    for i_air in range(num_airs):
        svk = per_air_vks[i_air]
        if svk.preprocessed_data is not None:
            prep = preprocessed_traces[i_air] if preprocessed_traces else None
            assert prep is not None, (
                f"VK expects preprocessed for AIR {i_air} but none provided"
            )
            committed = pcs_commit([(domains[i_air], prep)], fri_params.log_blowup)
            assert committed.root == svk.preprocessed_data.commit, (
                f"Preprocessed commitment mismatch for AIR {i_air}"
            )
            preprocessed_committed_list.append((i_air, committed))

    _log(f"preprocessed committed ({len(preprocessed_committed_list)} AIRs)")

    # (b) Commit cached main traces (each partition gets its own commitment).
    # Order: for each AIR, for each cached main partition -> separate commit.
    cached_committed_list: list[CommittedData] = []
    main_commits: list[Digest] = []
    for i_air in range(num_airs):
        svk = per_air_vks[i_air]
        num_cached = len(svk.params.width.cached_mains)
        if num_cached > 0:
            air_cached = cached_main_traces[i_air] if cached_main_traces else []
            assert len(air_cached) == num_cached, (
                f"AIR {i_air}: expected {num_cached} cached mains, got {len(air_cached)}"
            )
            for cached_matrix in air_cached:
                committed = pcs_commit(
                    [(domains[i_air], cached_matrix)], fri_params.log_blowup
                )
                cached_committed_list.append(committed)
                main_commits.append(committed.root)

    _log(f"cached mains committed ({len(cached_committed_list)})")

    # (c) Commit common main trace (all AIRs in one batch).
    common_main_evals = []
    for i_air in range(num_airs):
        if _has_common_main(per_air_vks[i_air]):
            common_main_evals.append((domains[i_air], traces[i_air]))
    main_committed = pcs_commit(common_main_evals, fri_params.log_blowup)
    main_commits.append(main_committed.root)

    _log(f"common main committed ({len(common_main_evals)} mats)")

    # --- Phase 3: Observe into transcript ---
    # (4) Observe public values
    for pis in public_values_per_air:
        challenger.observe_many(pis)

    # (5) Observe preprocessed commitments (from VK, not re-committed roots)
    for commit in _flattened_preprocessed_commits(per_air_vks):
        challenger.observe_many(commit)

    # (6) Observe main trace commitments (cached + common)
    for commit in main_commits:
        challenger.observe_many(commit)

    # (7) Observe log_degree for each AIR
    for i_air in range(num_airs):
        degree = domains[i_air].size()
        log_degree = degree.bit_length() - 1
        challenger.observe(log_degree)

    # --- Phase 4: RAP phase (interactions) ---
    has_any_interaction = any(_has_interaction(svk) for svk in per_air_vks)
    rap_phase_seq_proof = None
    challenges_per_phase: list[list[EF4Coeffs]] = []
    after_challenge_commits: list[Digest] = []
    after_challenge_per_air: list[Optional[list[list[EF4Coeffs]]]] = [None] * num_airs
    exposed_values_per_air: list[list[list[EF4Coeffs]]] = [[] for _ in range(num_airs)]
    ac_committed: Optional[CommittedData] = None

    _log("starting RAP phase")

    if has_any_interaction:
        from protocol.logup import (
            compute_after_challenge_trace,
            compute_max_constraint_degree,
            find_interaction_chunks,
        )

        max_cd = compute_max_constraint_degree(vk.inner.per_air)

        # (a) LogUp PoW grinding
        logup_pow_witness = grind(challenger, vk.inner.log_up_pow_bits)

        # (b) Sample 2 interaction challenges
        interaction_challenges: list[EF4Coeffs] = [
            challenger.sample_ext() for _ in range(STARK_LU_NUM_CHALLENGES)
        ]
        alpha_lu, beta_lu = interaction_challenges[0], interaction_challenges[1]

        # (c) Compute after_challenge trace per AIR
        for i_air in range(num_airs):
            svk = per_air_vks[i_air]
            interactions = svk.symbolic_constraints.interactions
            if not interactions:
                continue

            dag = svk.symbolic_constraints.constraints
            partitions = find_interaction_chunks(interactions, dag, max_cd)

            # Build partitioned_main: [cached_mains..., common_main]
            partitioned_main: list[list[list[Fe]]] = []
            if cached_main_traces and cached_main_traces[i_air]:
                for cm in cached_main_traces[i_air]:
                    partitioned_main.append(cm)
            if _has_common_main(svk):
                partitioned_main.append(traces[i_air])

            prep = None
            if preprocessed_traces and preprocessed_traces[i_air] is not None:
                prep = preprocessed_traces[i_air]

            height = domains[i_air].size()
            _log(f"  logup AIR {air_ids[i_air]}: h={height}, inters={len(interactions)}")
            perm_trace, cum_sum = compute_after_challenge_trace(
                interactions, partitions, dag,
                partitioned_main, prep,
                public_values_per_air[i_air],
                alpha_lu, beta_lu, height,
            )
            after_challenge_per_air[i_air] = perm_trace
            exposed_values_per_air[i_air] = [[cum_sum]]

        # (d) Observe exposed values (cumulative sums)
        for i_air in range(num_airs):
            evs = exposed_values_per_air[i_air]
            if evs:
                for ev in evs[0]:  # phase 0
                    challenger.observe_many(ev)

        # (e) Flatten after_challenge traces to base field and commit.
        # Each EF4 element → 4 base field columns.
        ac_evals = []
        for i_air in range(num_airs):
            if after_challenge_per_air[i_air] is not None:
                domain = domains[i_air]
                perm_trace = after_challenge_per_air[i_air]
                height = len(perm_trace)
                perm_width = len(perm_trace[0])
                base_field_rows: list[list[Fe]] = []
                for row_idx in range(height):
                    row: list[Fe] = []
                    for col in range(perm_width):
                        row.extend(perm_trace[row_idx][col])
                    base_field_rows.append(row)
                ac_evals.append((domain, base_field_rows))

        _log(f"committing after_challenge ({len(ac_evals)} mats)")
        ac_committed = pcs_commit(ac_evals, fri_params.log_blowup)

        # (f) Observe after_challenge commitment
        challenger.observe_many(ac_committed.root)

        rap_phase_seq_proof = FriLogUpPartialProof(
            logup_pow_witness=logup_pow_witness
        )
        challenges_per_phase = [interaction_challenges]
        after_challenge_commits = [ac_committed.root]

    _log("starting quotient phase")

    # --- Phase 5: Sample alpha and compute quotient ---
    alpha: EF4Coeffs = challenger.sample_ext()

    # Compute quotient for each AIR
    all_quotient_chunks: list[list[list[list[Fe]]]] = []
    quotient_chunk_domains: list[list[TwoAdicMultiplicativeCoset]] = []

    for i_air in range(num_airs):
        svk = per_air_vks[i_air]
        domain = domains[i_air]

        # Build partitioned traces for quotient evaluation
        partitioned_traces_list: Optional[list[list[list[Fe]]]] = None
        num_cached = _num_cached_mains(svk)
        if num_cached > 0 or (preprocessed_traces and preprocessed_traces[i_air]):
            # Multi-partition or preprocessed: need partitioned traces
            parts: list[list[list[Fe]]] = []
            if cached_main_traces and cached_main_traces[i_air]:
                for cm in cached_main_traces[i_air]:
                    parts.append(cm)
            if _has_common_main(svk):
                parts.append(traces[i_air])
            partitioned_traces_list = parts

        # Prepare challenges and exposed values for quotient
        chall = None
        exp_vals = None
        if has_any_interaction and _has_interaction(svk):
            chall = [challenges_per_phase[0]]
            evs = exposed_values_per_air[i_air]
            exp_vals = evs if evs else None

        prep = None
        if preprocessed_traces and preprocessed_traces[i_air] is not None:
            prep = preprocessed_traces[i_air]

        _log(f"  quotient AIR {air_ids[i_air]}: h={domain.size()}, qd={svk.quotient_degree}")
        chunks, qc_doms = compute_quotient_chunks(
            trace=traces[i_air],
            constraints_dag=svk.symbolic_constraints.constraints,
            public_values=public_values_per_air[i_air],
            alpha=alpha,
            trace_domain=domain,
            quotient_degree=svk.quotient_degree,
            preprocessed_trace=prep,
            after_challenge_trace=after_challenge_per_air[i_air],
            challenges=chall,
            exposed_values=exp_vals,
            partitioned_traces=partitioned_traces_list,
        )
        all_quotient_chunks.append(chunks)
        quotient_chunk_domains.append(qc_doms)

    # --- Phase 6: Commit quotient ---
    quotient_evals = []
    for i_air in range(num_airs):
        for chunk_idx, chunk_matrix in enumerate(all_quotient_chunks[i_air]):
            qc_domain = quotient_chunk_domains[i_air][chunk_idx]
            quotient_evals.append((qc_domain, chunk_matrix))

    _log(f"committing quotient ({len(quotient_evals)} mats)")
    quotient_committed = pcs_commit(quotient_evals, fri_params.log_blowup)

    # (8) Observe quotient commitment
    challenger.observe_many(quotient_committed.root)

    # --- Phase 7: DEEP PoW ---
    deep_pow_witness = grind(challenger, vk.inner.deep_pow_bits)

    # --- Phase 8: Sample zeta ---
    zeta: EF4Coeffs = challenger.sample_ext()

    _log("starting PCS open phase")

    # --- Phase 9: Build PCS opening rounds ---
    # Round order must match the verifier exactly:
    # preprocessed (each own round) → cached mains (each own round) →
    # common main (one round) → after_challenge (one round) → quotient (one round)
    opening_rounds: list[PcsOpeningRound] = []
    next_points = [domains[i_air].next_point(zeta) for i_air in range(num_airs)]

    # (a) Preprocessed traces (each in its own round)
    for i_air, committed in preprocessed_committed_list:
        opening_rounds.append(PcsOpeningRound(
            committed=committed,
            points_per_mat=[[zeta, next_points[i_air]]],
        ))

    # (b) Cached main traces (each in its own round)
    cached_idx = 0
    for i_air in range(num_airs):
        svk = per_air_vks[i_air]
        for _ in svk.params.width.cached_mains:
            committed = cached_committed_list[cached_idx]
            opening_rounds.append(PcsOpeningRound(
                committed=committed,
                points_per_mat=[[zeta, next_points[i_air]]],
            ))
            cached_idx += 1

    # (c) Common main trace (one round, multiple matrices)
    common_main_points = []
    for i_air in range(num_airs):
        if _has_common_main(per_air_vks[i_air]):
            common_main_points.append([zeta, next_points[i_air]])
    opening_rounds.append(PcsOpeningRound(
        committed=main_committed,
        points_per_mat=common_main_points,
    ))

    # (d) After-challenge traces (one round if present)
    if has_any_interaction and ac_committed is not None:
        ac_points = []
        for i_air in range(num_airs):
            if _has_interaction(per_air_vks[i_air]):
                ac_points.append([zeta, next_points[i_air]])
        opening_rounds.append(PcsOpeningRound(
            committed=ac_committed,
            points_per_mat=ac_points,
        ))

    # (e) Quotient chunks (one round)
    quotient_points = []
    for i_air in range(num_airs):
        for _ in all_quotient_chunks[i_air]:
            quotient_points.append([zeta])
    opening_rounds.append(PcsOpeningRound(
        committed=quotient_committed,
        points_per_mat=quotient_points,
    ))

    # --- Phase 10: PCS open ---
    all_opened_values, fri_proof_data, query_indices = pcs_open(
        opening_rounds, challenger, fri_params
    )

    _log("PCS open complete, building proof structure")

    # --- Phase 11: Build proof structure ---
    # Extract opened values from all_opened_values[round_idx][mat_idx][point_idx]
    round_idx = 0

    # Preprocessed opened values
    preprocessed_ov: list[AdjacentOpenedValues] = []
    for _ in preprocessed_committed_list:
        mat_values = all_opened_values[round_idx][0]  # single matrix
        preprocessed_ov.append(AdjacentOpenedValues(
            local=mat_values[0],
            next=mat_values[1],
        ))
        round_idx += 1

    # Main trace opened values: [cached_0, ..., cached_n, common_main_batch]
    main_ov: list[list[AdjacentOpenedValues]] = []

    # Cached mains (each has 1 matrix)
    for _ in cached_committed_list:
        mat_values = all_opened_values[round_idx][0]
        main_ov.append([AdjacentOpenedValues(
            local=mat_values[0],
            next=mat_values[1],
        )])
        round_idx += 1

    # Common main (multiple matrices, one per AIR with common main)
    common_main_round = all_opened_values[round_idx]
    common_main_avs: list[AdjacentOpenedValues] = []
    for mat_values in common_main_round:
        common_main_avs.append(AdjacentOpenedValues(
            local=mat_values[0],
            next=mat_values[1],
        ))
    main_ov.append(common_main_avs)
    round_idx += 1

    # After-challenge opened values
    after_challenge_ov: list[list[AdjacentOpenedValues]] = []
    if has_any_interaction and ac_committed is not None:
        ac_round = all_opened_values[round_idx]
        ac_avs: list[AdjacentOpenedValues] = []
        for mat_values in ac_round:
            ac_avs.append(AdjacentOpenedValues(
                local=mat_values[0],
                next=mat_values[1],
            ))
        after_challenge_ov.append(ac_avs)
        round_idx += 1

    # Quotient opened values
    quotient_round = all_opened_values[round_idx]
    quotient_ov: list[list[list[EF4Coeffs]]] = []
    chunk_idx = 0
    for i_air in range(num_airs):
        air_chunks_ov: list[list[EF4Coeffs]] = []
        for _ in all_quotient_chunks[i_air]:
            air_chunks_ov.append(quotient_round[chunk_idx][0])
            chunk_idx += 1
        quotient_ov.append(air_chunks_ov)

    opened_values = OpenedValues(
        preprocessed=preprocessed_ov,
        main=main_ov,
        after_challenge=after_challenge_ov,
        quotient=quotient_ov,
    )

    # Build query proofs
    log_global_max_height = (
        len(fri_proof_data["commit_phase_commits"])
        + fri_params.log_blowup
        + fri_params.log_final_poly_len
    )

    query_proofs: list[QueryProof] = []
    for qi in range(fri_params.num_queries):
        query_index = query_indices[qi]
        _, fri_openings = fri_proof_data["fri_query_proofs"][qi]

        input_proof: list[BatchOpening] = []
        for rnd in opening_rounds:
            batch_opening = _generate_batch_opening(
                rnd.committed, query_index, log_global_max_height
            )
            input_proof.append(batch_opening)

        commit_phase_openings: list[CommitPhaseProofStep] = []
        for step in fri_openings:
            commit_phase_openings.append(CommitPhaseProofStep(
                sibling_value=step.sibling_value,
                opening_proof=step.opening_proof,
            ))

        query_proofs.append(QueryProof(
            input_proof=input_proof,
            commit_phase_openings=commit_phase_openings,
        ))

    # Build FRI proof
    fri_proof = ProofFriProof(
        commit_phase_commits=fri_proof_data["commit_phase_commits"],
        query_proofs=query_proofs,
        final_poly=fri_proof_data["final_poly"],
        commit_pow_witnesses=fri_proof_data["commit_pow_witnesses"],
        query_pow_witness=fri_proof_data["query_pow_witness"],
    )

    opening_proof = OpeningProof(
        proof=fri_proof,
        values=opened_values,
        deep_pow_witness=deep_pow_witness,
    )

    # Build per-AIR proof data
    per_air_proof_data: list[AirProofData] = []
    for i_air in range(num_airs):
        per_air_proof_data.append(AirProofData(
            air_id=air_ids[i_air],
            degree=domains[i_air].size(),
            exposed_values_after_challenge=exposed_values_per_air[i_air],
            public_values=public_values_per_air[i_air],
        ))

    commitments = Commitments(
        main_trace=main_commits,
        after_challenge=after_challenge_commits,
        quotient=quotient_committed.root,
    )

    _log("proof complete")

    return Proof(
        commitments=commitments,
        opening=opening_proof,
        per_air=per_air_proof_data,
        rap_phase_seq_proof=rap_phase_seq_proof,
    )
