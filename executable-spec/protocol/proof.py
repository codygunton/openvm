"""STARK proof and verifying key data structures.

Faithful Python translation of the Rust proof types from the stark-backend and
Plonky3 FRI libraries, plus JSON parsing functions for deserialization from
serde_json output.

Proof types reference:
    stark-backend/src/proof.rs — Proof, Commitments, OpeningProof, OpenedValues,
                                  AdjacentOpenedValues, AirProofData
    p3-fri/src/proof.rs        — FriProof, QueryProof, CommitPhaseProofStep
    p3-fri/src/two_adic_pcs.rs — BatchOpening

VK types reference:
    stark-backend/src/keygen/types.rs — MultiStarkVerifyingKey, StarkVerifyingKey,
                                         StarkVerifyingParams, TraceWidth
    stark-backend/src/air_builders/symbolic/dag.rs — SymbolicExpressionDag,
                                                      SymbolicExpressionNode
    stark-backend/src/air_builders/symbolic/symbolic_variable.rs — SymbolicVariable, Entry
    stark-backend/src/interaction/mod.rs — Interaction
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum, auto
from typing import Optional

from primitives.field import Digest, EF4Coeffs, Fe, MerklePath, from_monty, to_monty


# ---------------------------------------------------------------------------
# Montgomery-to-canonical conversion for serde-parsed values.
#
# BabyBear (MontyField31) serializes in Montgomery form: serde writes the
# internal `value` field which is `x * 2^32 mod p`.  Our Python Poseidon2
# FFI and galois-based field arithmetic operate on canonical integers.
# Therefore we convert at parse time so the rest of the codebase can be
# representation-agnostic.
# ---------------------------------------------------------------------------

def _monty_fe(x: int) -> Fe:
    """Convert a single serde-serialized BabyBear value from Montgomery to canonical."""
    return from_monty(x)


def _monty_list(xs: list[int]) -> list[int]:
    """Convert a list of serde-serialized BabyBear values from Montgomery to canonical."""
    return [from_monty(x) for x in xs]


# ---------------------------------------------------------------------------
# Proof data structures
# ---------------------------------------------------------------------------


@dataclass
class Commitments:
    """All commitments to a multi-matrix STARK (not preprocessed).

    Reference:
        stark-backend/src/proof.rs (struct Commitments<Com>)
    """
    main_trace: list[Digest]
    after_challenge: list[Digest]
    quotient: Digest


@dataclass
class AdjacentOpenedValues:
    """Opened values at zeta and zeta * g for one trace matrix.

    Reference:
        stark-backend/src/proof.rs (struct AdjacentOpenedValues<Challenge>)
    """
    local: list[EF4Coeffs]
    next: list[EF4Coeffs]


@dataclass
class OpenedValues:
    """All opened values across preprocessed, main, after-challenge, and quotient.

    Reference:
        stark-backend/src/proof.rs (struct OpenedValues<Challenge>)
    """
    # For each preprocessed trace commitment, the opened values
    preprocessed: list[AdjacentOpenedValues]
    # For each main trace commitment, for each matrix in commitment
    main: list[list[AdjacentOpenedValues]]
    # For each phase after challenge, for each matrix in commitment
    after_challenge: list[list[AdjacentOpenedValues]]
    # For each AIR, for each quotient chunk, the opened values
    quotient: list[list[list[EF4Coeffs]]]


@dataclass
class CommitPhaseProofStep:
    """One round of a FRI query: sibling value + Merkle proof.

    Reference:
        p3-fri/src/proof.rs (struct CommitPhaseProofStep<F, M>)
    """
    sibling_value: EF4Coeffs
    opening_proof: MerklePath


@dataclass
class BatchOpening:
    """Opened values from a single MMCS batch at a query index.

    Reference:
        p3-fri/src/two_adic_pcs.rs (struct BatchOpening<Val, InputMmcs>)
    """
    opened_values: list[list[Fe]]
    opening_proof: MerklePath


@dataclass
class QueryProof:
    """FRI query proof for a single query index.

    Reference:
        p3-fri/src/proof.rs (struct QueryProof<F, M, InputProof>)
    """
    # InputProof = Vec<BatchOpening<Val, InputMmcs>> for TwoAdicFriPcs
    input_proof: list[BatchOpening]
    commit_phase_openings: list[CommitPhaseProofStep]


@dataclass
class FriProof:
    """Complete FRI proof.

    Reference:
        p3-fri/src/proof.rs (struct FriProof<F, M, Witness, InputProof>)
    """
    commit_phase_commits: list[Digest]
    query_proofs: list[QueryProof]
    final_poly: list[EF4Coeffs]
    # Per commit-phase round, the PoW witness (u64 in Rust, int here)
    commit_pow_witnesses: list[int]
    # Single PoW witness for query phase
    query_pow_witness: int


@dataclass
class OpeningProof:
    """PCS opening proof with opened values.

    Reference:
        stark-backend/src/proof.rs (struct OpeningProof<PcsProof, Challenge>)
    """
    proof: FriProof
    values: OpenedValues
    # PoW witness for DEEP quotient (logup pow)
    deep_pow_witness: int


@dataclass
class AirProofData:
    """Proof data for a single AIR.

    Reference:
        stark-backend/src/proof.rs (struct AirProofData<Val, Challenge>)
    """
    air_id: int
    degree: int  # height of trace matrix
    # For each challenge phase with trace, the values to expose to the verifier
    exposed_values_after_challenge: list[list[EF4Coeffs]]
    public_values: list[Fe]


@dataclass
class FriLogUpPartialProof:
    """Partial proof for the FRI LogUp challenge phase.

    Reference:
        stark-backend/src/interaction/fri_log_up.rs (struct FriLogUpPartialProof<Witness>)
    """
    logup_pow_witness: Fe


@dataclass
class Proof:
    """Full multi-AIR STARK proof.

    Reference:
        stark-backend/src/proof.rs (struct Proof<SC>)
    """
    commitments: Commitments
    opening: OpeningProof
    per_air: list[AirProofData]
    # Partial proof for RAP phase, if it exists (currently always None for simple AIRs)
    rap_phase_seq_proof: Optional[FriLogUpPartialProof] = None


# ---------------------------------------------------------------------------
# Verifying key data structures
# ---------------------------------------------------------------------------


class EntryType(Enum):
    """Kind of symbolic variable entry.

    Reference:
        stark-backend/src/air_builders/symbolic/symbolic_variable.rs (enum Entry)
    """
    PREPROCESSED = auto()
    MAIN = auto()
    PERMUTATION = auto()
    PUBLIC = auto()
    CHALLENGE = auto()
    EXPOSED = auto()


@dataclass
class Entry:
    """Symbolic variable entry with kind and offset/part_index.

    Reference:
        stark-backend/src/air_builders/symbolic/symbolic_variable.rs (enum Entry)
    """
    kind: EntryType
    # Offset into the trace window (0 = local, 1 = next). None for Public/Challenge/Exposed.
    offset: Optional[int] = None
    # Part index for Main entries (which main trace partition).
    part_index: Optional[int] = None


@dataclass
class SymbolicVariable:
    """A variable within the evaluation window (column reference).

    Reference:
        stark-backend/src/air_builders/symbolic/symbolic_variable.rs
        (struct SymbolicVariable<F>)
    """
    entry: Entry
    index: int


class SymbolicNodeKind(Enum):
    """Kind of symbolic expression node.

    Reference:
        stark-backend/src/air_builders/symbolic/dag.rs (enum SymbolicExpressionNode)
    """
    VARIABLE = auto()
    IS_FIRST_ROW = auto()
    IS_LAST_ROW = auto()
    IS_TRANSITION = auto()
    CONSTANT = auto()
    ADD = auto()
    SUB = auto()
    NEG = auto()
    MUL = auto()


@dataclass
class SymbolicExpressionNode:
    """A node in the symbolic expression DAG.

    Reference:
        stark-backend/src/air_builders/symbolic/dag.rs
        (enum SymbolicExpressionNode<F>)

    Fields vary by kind:
    - VARIABLE: variable is set
    - CONSTANT: constant_value is set
    - ADD/SUB/MUL: left_idx, right_idx, degree_multiple are set
    - NEG: idx, degree_multiple are set
    - IS_FIRST_ROW/IS_LAST_ROW/IS_TRANSITION: no extra fields
    """
    kind: SymbolicNodeKind
    # For VARIABLE
    variable: Optional[SymbolicVariable] = None
    # For CONSTANT
    constant_value: Optional[Fe] = None
    # For ADD, SUB, MUL
    left_idx: Optional[int] = None
    right_idx: Optional[int] = None
    # For NEG
    idx: Optional[int] = None
    # For ADD, SUB, MUL, NEG
    degree_multiple: Optional[int] = None


@dataclass
class SymbolicExpressionDag:
    """DAG of symbolic expressions in topological order.

    Reference:
        stark-backend/src/air_builders/symbolic/dag.rs
        (struct SymbolicExpressionDag<F>)
    """
    nodes: list[SymbolicExpressionNode]
    # Indices of nodes that represent constraints (assert == 0)
    constraint_idx: list[int]


@dataclass
class Interaction:
    """A bus interaction.

    Reference:
        stark-backend/src/interaction/mod.rs (struct Interaction<Expr>)

    In the DAG form, message and count are node indices (int).
    """
    message: list[int]
    count: int
    bus_index: int
    count_weight: int


@dataclass
class SymbolicConstraintsDag:
    """Complete symbolic constraints for a single AIR.

    Reference:
        stark-backend/src/air_builders/symbolic/dag.rs
        (struct SymbolicConstraintsDag<F>)
    """
    constraints: SymbolicExpressionDag
    interactions: list[Interaction]


@dataclass
class TraceWidth:
    """Widths of different parts of a trace matrix.

    Reference:
        stark-backend/src/keygen/types.rs (struct TraceWidth)
    """
    preprocessed: Optional[int]
    cached_mains: list[int]
    common_main: int
    # Widths counted by extension field elements (not base field elements)
    after_challenge: list[int]


@dataclass
class StarkVerifyingParams:
    """Verification parameters for a single STARK.

    Reference:
        stark-backend/src/keygen/types.rs (struct StarkVerifyingParams)
    """
    width: TraceWidth
    num_public_values: int
    num_exposed_values_after_challenge: list[int]
    num_challenges_to_sample: list[int]


class RapPhaseSeqKind(Enum):
    """Supported challenge phase protocols.

    Reference:
        stark-backend/src/interaction/mod.rs (enum RapPhaseSeqKind)
    """
    FRI_LOG_UP = auto()


@dataclass
class VerifierSinglePreprocessedData:
    """Verifier data for preprocessed trace for a single AIR.

    Reference:
        stark-backend/src/keygen/types.rs (struct VerifierSinglePreprocessedData<Com>)
    """
    commit: Digest


@dataclass
class StarkVerifyingKey:
    """Verifying key for a single STARK (single AIR).

    Reference:
        stark-backend/src/keygen/types.rs (struct StarkVerifyingKey<Val, Com>)
    """
    preprocessed_data: Optional[VerifierSinglePreprocessedData]
    params: StarkVerifyingParams
    symbolic_constraints: SymbolicConstraintsDag
    quotient_degree: int
    rap_phase_seq_kind: RapPhaseSeqKind


@dataclass
class LinearConstraint:
    """Linear constraint on trace heights.

    Reference:
        stark-backend/src/keygen/types.rs (struct LinearConstraint)
    """
    coefficients: list[int]
    threshold: int


@dataclass
class MultiStarkVerifyingKey0:
    """Inner verifying key data (without pre_hash).

    Reference:
        stark-backend/src/keygen/types.rs (struct MultiStarkVerifyingKey0<SC>)
    """
    per_air: list[StarkVerifyingKey]
    trace_height_constraints: list[LinearConstraint]
    log_up_pow_bits: int
    deep_pow_bits: int


@dataclass
class MultiStarkVerifyingKey:
    """Complete multi-AIR verifying key.

    Reference:
        stark-backend/src/keygen/types.rs (struct MultiStarkVerifyingKey<SC>)
    """
    inner: MultiStarkVerifyingKey0
    pre_hash: Digest


# ---------------------------------------------------------------------------
# FRI parameters
# ---------------------------------------------------------------------------


@dataclass
class FriParameters:
    """FRI protocol parameters.

    Reference:
        stark-sdk/src/config/mod.rs (struct FriParameters)
    """
    log_blowup: int
    log_final_poly_len: int
    num_queries: int
    query_proof_of_work_bits: int
    commit_proof_of_work_bits: int


# ---------------------------------------------------------------------------
# JSON parsing functions
# ---------------------------------------------------------------------------


def _parse_digest(data: dict | list) -> Digest:
    """Parse a Poseidon2 digest from serde JSON (Montgomery -> canonical).

    Handles both serde formats:
    - Direct array: [u32; 8] (from serde Hash serialization of Vec<Hash>)
    - Dict: {"value": [u32; 8], "_marker": null} (from serde Hash serialization)

    Both formats contain Montgomery-form BabyBear values since they come from
    serde serialization of Hash<BabyBear, _, 8>.  We convert to canonical.
    """
    if isinstance(data, list):
        return _monty_list(data)
    return _monty_list(data["value"])


def _parse_ef4(data: dict | list) -> EF4Coeffs:
    """Parse an extension field element from serde JSON (Montgomery -> canonical).

    Handles both serde formats:
    - Direct array: [c0, c1, c2, c3]
    - Dict: {"value": [c0,c1,c2,c3], "_phantom": null}

    Both contain Montgomery-form BabyBear coefficients.
    """
    if isinstance(data, list):
        return _monty_list(data)
    return _monty_list(data["value"])


def _parse_merkle_proof(data: list) -> MerklePath:
    """Parse a Merkle opening proof from JSON (list of digests)."""
    return [_parse_digest(d) for d in data]


def _parse_adjacent_opened_values(data: dict) -> AdjacentOpenedValues:
    """Parse AdjacentOpenedValues from JSON.

    Reference:
        stark-backend/src/proof.rs (struct AdjacentOpenedValues<Challenge>)
    """
    return AdjacentOpenedValues(
        local=[_parse_ef4(v) for v in data["local"]],
        next=[_parse_ef4(v) for v in data["next"]],
    )


def _parse_batch_opening(data: dict) -> BatchOpening:
    """Parse a BatchOpening from serde JSON (Montgomery -> canonical).

    Reference:
        p3-fri/src/two_adic_pcs.rs (struct BatchOpening<Val, InputMmcs>)
    """
    return BatchOpening(
        opened_values=[_monty_list(row) for row in data["opened_values"]],
        opening_proof=_parse_merkle_proof(data["opening_proof"]),
    )


def _parse_commit_phase_proof_step(data: dict) -> CommitPhaseProofStep:
    """Parse a CommitPhaseProofStep from JSON.

    Reference:
        p3-fri/src/proof.rs (struct CommitPhaseProofStep<F, M>)
    """
    return CommitPhaseProofStep(
        sibling_value=_parse_ef4(data["sibling_value"]),
        opening_proof=_parse_merkle_proof(data["opening_proof"]),
    )


def _parse_query_proof(data: dict) -> QueryProof:
    """Parse a QueryProof from JSON.

    Reference:
        p3-fri/src/proof.rs (struct QueryProof<F, M, InputProof>)
    """
    return QueryProof(
        input_proof=[_parse_batch_opening(bp) for bp in data["input_proof"]],
        commit_phase_openings=[
            _parse_commit_phase_proof_step(step)
            for step in data["commit_phase_openings"]
        ],
    )


def _parse_fri_proof(data: dict) -> FriProof:
    """Parse a FriProof from serde JSON (Montgomery -> canonical).

    Reference:
        p3-fri/src/proof.rs (struct FriProof<F, M, Witness, InputProof>)
    """
    return FriProof(
        commit_phase_commits=[_parse_digest(c) for c in data["commit_phase_commits"]],
        query_proofs=[_parse_query_proof(qp) for qp in data["query_proofs"]],
        final_poly=[_parse_ef4(c) for c in data["final_poly"]],
        commit_pow_witnesses=_monty_list(data["commit_pow_witnesses"]),
        query_pow_witness=_monty_fe(data["query_pow_witness"]),
    )


def _parse_opened_values(data: dict) -> OpenedValues:
    """Parse OpenedValues from JSON.

    Reference:
        stark-backend/src/proof.rs (struct OpenedValues<Challenge>)
    """
    return OpenedValues(
        preprocessed=[
            _parse_adjacent_opened_values(av) for av in data["preprocessed"]
        ],
        main=[
            [_parse_adjacent_opened_values(av) for av in commitment_matrices]
            for commitment_matrices in data["main"]
        ],
        after_challenge=[
            [_parse_adjacent_opened_values(av) for av in commitment_matrices]
            for commitment_matrices in data["after_challenge"]
        ],
        quotient=[
            [
                [_parse_ef4(v) for v in chunk]
                for chunk in air_chunks
            ]
            for air_chunks in data["quotient"]
        ],
    )


def _parse_opening_proof(data: dict) -> OpeningProof:
    """Parse OpeningProof from serde JSON (Montgomery -> canonical).

    Reference:
        stark-backend/src/proof.rs (struct OpeningProof<PcsProof, Challenge>)
    """
    return OpeningProof(
        proof=_parse_fri_proof(data["proof"]),
        values=_parse_opened_values(data["values"]),
        deep_pow_witness=_monty_fe(data["deep_pow_witness"]),
    )


def _parse_air_proof_data(data: dict) -> AirProofData:
    """Parse AirProofData from serde JSON (Montgomery -> canonical).

    Reference:
        stark-backend/src/proof.rs (struct AirProofData<Val, Challenge>)
    """
    return AirProofData(
        air_id=data["air_id"],
        degree=data["degree"],
        exposed_values_after_challenge=[
            [_parse_ef4(v) for v in phase_values]
            for phase_values in data["exposed_values_after_challenge"]
        ],
        public_values=_monty_list(data["public_values"]),
    )


def parse_proof_json(data: dict) -> Proof:
    """Parse a Proof from a JSON dict (serde_json format).

    Handles the JSON produced by serde_json::to_vec(&proof) for
    Proof<BabyBearPoseidon2Config>.

    Reference:
        stark-backend/src/proof.rs (struct Proof<SC>)
    """
    commitments_data = data["commitments"]
    commitments = Commitments(
        main_trace=[_parse_digest(c) for c in commitments_data["main_trace"]],
        after_challenge=[_parse_digest(c) for c in commitments_data["after_challenge"]],
        quotient=_parse_digest(commitments_data["quotient"]),
    )

    opening = _parse_opening_proof(data["opening"])
    per_air = [_parse_air_proof_data(a) for a in data["per_air"]]

    rap_phase_seq_proof = None
    rp_data = data.get("rap_phase_seq_proof")
    if rp_data is not None:
        rap_phase_seq_proof = FriLogUpPartialProof(
            logup_pow_witness=_monty_fe(rp_data["logup_pow_witness"]),
        )

    return Proof(
        commitments=commitments,
        opening=opening,
        per_air=per_air,
        rap_phase_seq_proof=rap_phase_seq_proof,
    )


# ---------------------------------------------------------------------------
# VK JSON parsing
# ---------------------------------------------------------------------------


def _parse_entry(data: dict) -> Entry:
    """Parse an Entry from serde_json tagged-enum format.

    serde_json serializes Rust enums with internal data as:
    {"Preprocessed": {"offset": 0}} or "Public" (unit variants).

    Reference:
        stark-backend/src/air_builders/symbolic/symbolic_variable.rs (enum Entry)
    """
    if isinstance(data, str):
        kind_map = {
            "Public": EntryType.PUBLIC,
            "Challenge": EntryType.CHALLENGE,
            "Exposed": EntryType.EXPOSED,
        }
        return Entry(kind=kind_map[data])

    if "Preprocessed" in data:
        inner = data["Preprocessed"]
        return Entry(
            kind=EntryType.PREPROCESSED,
            offset=inner["offset"],
        )
    if "Main" in data:
        inner = data["Main"]
        return Entry(
            kind=EntryType.MAIN,
            offset=inner["offset"],
            part_index=inner["part_index"],
        )
    if "Permutation" in data:
        inner = data["Permutation"]
        return Entry(
            kind=EntryType.PERMUTATION,
            offset=inner["offset"],
        )

    raise ValueError(f"Unknown Entry variant: {data}")


def _parse_symbolic_variable(data: dict) -> SymbolicVariable:
    """Parse a SymbolicVariable from JSON.

    Reference:
        stark-backend/src/air_builders/symbolic/symbolic_variable.rs
        (struct SymbolicVariable<F>)
    """
    return SymbolicVariable(
        entry=_parse_entry(data["entry"]),
        index=data["index"],
    )


def _parse_symbolic_expression_node(data: dict | str) -> SymbolicExpressionNode:
    """Parse a SymbolicExpressionNode from serde_json tagged-enum format.

    Reference:
        stark-backend/src/air_builders/symbolic/dag.rs
        (enum SymbolicExpressionNode<F>)
    """
    # Unit variants are serialized as strings
    if isinstance(data, str):
        kind_map = {
            "IsFirstRow": SymbolicNodeKind.IS_FIRST_ROW,
            "IsLastRow": SymbolicNodeKind.IS_LAST_ROW,
            "IsTransition": SymbolicNodeKind.IS_TRANSITION,
        }
        return SymbolicExpressionNode(kind=kind_map[data])

    if "Variable" in data:
        return SymbolicExpressionNode(
            kind=SymbolicNodeKind.VARIABLE,
            variable=_parse_symbolic_variable(data["Variable"]),
        )
    if "Constant" in data:
        return SymbolicExpressionNode(
            kind=SymbolicNodeKind.CONSTANT,
            constant_value=_monty_fe(data["Constant"]),
        )
    if "Add" in data:
        inner = data["Add"]
        return SymbolicExpressionNode(
            kind=SymbolicNodeKind.ADD,
            left_idx=inner["left_idx"],
            right_idx=inner["right_idx"],
            degree_multiple=inner["degree_multiple"],
        )
    if "Sub" in data:
        inner = data["Sub"]
        return SymbolicExpressionNode(
            kind=SymbolicNodeKind.SUB,
            left_idx=inner["left_idx"],
            right_idx=inner["right_idx"],
            degree_multiple=inner["degree_multiple"],
        )
    if "Mul" in data:
        inner = data["Mul"]
        return SymbolicExpressionNode(
            kind=SymbolicNodeKind.MUL,
            left_idx=inner["left_idx"],
            right_idx=inner["right_idx"],
            degree_multiple=inner["degree_multiple"],
        )
    if "Neg" in data:
        inner = data["Neg"]
        return SymbolicExpressionNode(
            kind=SymbolicNodeKind.NEG,
            idx=inner["idx"],
            degree_multiple=inner["degree_multiple"],
        )

    raise ValueError(f"Unknown SymbolicExpressionNode variant: {data}")


def _parse_symbolic_expression_dag(data: dict) -> SymbolicExpressionDag:
    """Parse a SymbolicExpressionDag from JSON.

    Reference:
        stark-backend/src/air_builders/symbolic/dag.rs
        (struct SymbolicExpressionDag<F>)
    """
    return SymbolicExpressionDag(
        nodes=[_parse_symbolic_expression_node(n) for n in data["nodes"]],
        constraint_idx=list(data["constraint_idx"]),
    )


def _parse_interaction(data: dict) -> Interaction:
    """Parse an Interaction from JSON.

    In DAG form, message entries and count are node indices (usize).

    Reference:
        stark-backend/src/interaction/mod.rs (struct Interaction<Expr>)
    """
    return Interaction(
        message=list(data["message"]),
        count=data["count"],
        bus_index=data["bus_index"],
        count_weight=data["count_weight"],
    )


def _parse_symbolic_constraints_dag(data: dict) -> SymbolicConstraintsDag:
    """Parse a SymbolicConstraintsDag from JSON.

    Reference:
        stark-backend/src/air_builders/symbolic/dag.rs
        (struct SymbolicConstraintsDag<F>)
    """
    return SymbolicConstraintsDag(
        constraints=_parse_symbolic_expression_dag(data["constraints"]),
        interactions=[_parse_interaction(i) for i in data["interactions"]],
    )


def _parse_trace_width(data: dict) -> TraceWidth:
    """Parse a TraceWidth from JSON.

    Reference:
        stark-backend/src/keygen/types.rs (struct TraceWidth)
    """
    return TraceWidth(
        preprocessed=data["preprocessed"],
        cached_mains=list(data["cached_mains"]),
        common_main=data["common_main"],
        after_challenge=list(data["after_challenge"]),
    )


def _parse_stark_verifying_params(data: dict) -> StarkVerifyingParams:
    """Parse StarkVerifyingParams from JSON.

    Reference:
        stark-backend/src/keygen/types.rs (struct StarkVerifyingParams)
    """
    return StarkVerifyingParams(
        width=_parse_trace_width(data["width"]),
        num_public_values=data["num_public_values"],
        num_exposed_values_after_challenge=list(
            data["num_exposed_values_after_challenge"]
        ),
        num_challenges_to_sample=list(data["num_challenges_to_sample"]),
    )


def _parse_rap_phase_seq_kind(data: str) -> RapPhaseSeqKind:
    """Parse a RapPhaseSeqKind from JSON.

    Reference:
        stark-backend/src/interaction/mod.rs (enum RapPhaseSeqKind)
    """
    kind_map = {
        "FriLogUp": RapPhaseSeqKind.FRI_LOG_UP,
    }
    return kind_map[data]


def _parse_stark_verifying_key(data: dict) -> StarkVerifyingKey:
    """Parse a StarkVerifyingKey from JSON.

    Reference:
        stark-backend/src/keygen/types.rs (struct StarkVerifyingKey<Val, Com>)
    """
    preprocessed_data = None
    if data["preprocessed_data"] is not None:
        pd = data["preprocessed_data"]
        preprocessed_data = VerifierSinglePreprocessedData(
            commit=_parse_digest(pd["commit"]),
        )

    return StarkVerifyingKey(
        preprocessed_data=preprocessed_data,
        params=_parse_stark_verifying_params(data["params"]),
        symbolic_constraints=_parse_symbolic_constraints_dag(
            data["symbolic_constraints"]
        ),
        quotient_degree=data["quotient_degree"],
        rap_phase_seq_kind=_parse_rap_phase_seq_kind(data["rap_phase_seq_kind"]),
    )


def _parse_linear_constraint(data: dict) -> LinearConstraint:
    """Parse a LinearConstraint from JSON.

    Reference:
        stark-backend/src/keygen/types.rs (struct LinearConstraint)
    """
    return LinearConstraint(
        coefficients=list(data["coefficients"]),
        threshold=data["threshold"],
    )


def parse_vk_json(data: dict) -> MultiStarkVerifyingKey:
    """Parse a MultiStarkVerifyingKey from a JSON dict (serde_json format).

    Reference:
        stark-backend/src/keygen/types.rs (struct MultiStarkVerifyingKey<SC>)
    """
    inner_data = data["inner"]
    inner = MultiStarkVerifyingKey0(
        per_air=[_parse_stark_verifying_key(ak) for ak in inner_data["per_air"]],
        trace_height_constraints=[
            _parse_linear_constraint(lc)
            for lc in inner_data["trace_height_constraints"]
        ],
        log_up_pow_bits=inner_data["log_up_pow_bits"],
        deep_pow_bits=inner_data.get("deep_pow_bits", 0),
    )
    return MultiStarkVerifyingKey(
        inner=inner,
        pre_hash=_parse_digest(data["pre_hash"]),
    )


# ---------------------------------------------------------------------------
# E2E test vector parsing
# ---------------------------------------------------------------------------


def parse_fri_params(data: dict) -> FriParameters:
    """Parse FRI parameters from a test vector JSON dict.

    Reference:
        crates/test-vectors/src/lib.rs (struct FriParamsMeta)
    """
    return FriParameters(
        log_blowup=data["log_blowup"],
        log_final_poly_len=data["log_final_poly_len"],
        num_queries=data["num_queries"],
        query_proof_of_work_bits=data["query_proof_of_work_bits"],
        commit_proof_of_work_bits=data["commit_proof_of_work_bits"],
    )


@dataclass
class AirInputData:
    """Raw input trace matrices for one AIR, used by the prover.

    Reference:
        crates/test-vectors/src/lib.rs (struct AirInputVectors)
    """
    air_id: int
    common_main: list[list[Fe]] | None  # [rows][cols]
    cached_mains: list[list[list[Fe]]]  # list of [rows][cols]
    preprocessed: list[list[Fe]] | None  # [rows][cols]


@dataclass
class ProverInputs:
    """All input trace data needed by the prover.

    Reference:
        crates/test-vectors/src/lib.rs (struct ProverInputVectors)
    """
    per_air: list[AirInputData]


def parse_prover_inputs(data: dict) -> ProverInputs:
    """Parse prover input vectors from JSON.

    Field values are canonical u32, no Montgomery conversion needed
    (traces are raw field elements, not serialized proof data).

    Reference:
        crates/test-vectors/src/lib.rs (struct ProverInputVectors)
    """
    per_air = []
    for air_data in data["per_air"]:
        common_main = air_data["common_main"]
        cached_mains = air_data["cached_mains"]
        preprocessed = air_data["preprocessed"]
        per_air.append(AirInputData(
            air_id=air_data["air_id"],
            common_main=common_main,
            cached_mains=cached_mains,
            preprocessed=preprocessed,
        ))
    return ProverInputs(per_air=per_air)


def parse_e2e_vectors(vectors: dict) -> tuple[Proof, FriParameters, dict]:
    """Parse complete E2E test vectors.

    Returns:
        (proof, fri_params, commitments_meta) where:
        - proof: parsed Proof dataclass
        - fri_params: parsed FriParameters
        - commitments_meta: dict with main_trace_commitments, after_challenge_commitments,
          quotient_commitment from the test vector (pre-extracted canonical values)

    Reference:
        crates/test-vectors/src/lib.rs (struct E2eProofVectors)
    """
    import json

    # Decode the hex-encoded serde_json proof
    proof_bytes = bytes.fromhex(vectors["proof_bytes_hex"])
    proof_json = json.loads(proof_bytes)
    proof = parse_proof_json(proof_json)

    fri_params = parse_fri_params(vectors["fri_params"])

    commitments_meta = {
        "main_trace_commitments": vectors["main_trace_commitments"],
        "after_challenge_commitments": vectors["after_challenge_commitments"],
        "quotient_commitment": vectors["quotient_commitment"],
        "num_airs": vectors["num_airs"],
        "per_air": vectors["per_air"],
    }

    return proof, fri_params, commitments_meta


# ---------------------------------------------------------------------------
# Proof serialization (Python canonical -> Rust serde JSON)
# ---------------------------------------------------------------------------


def _ser_fe(x: Fe) -> int:
    """Serialize a canonical field element to Montgomery form."""
    return to_monty(x)


def _ser_digest(d: Digest) -> dict:
    """Serialize a Poseidon2 digest to serde JSON format."""
    return {"value": [to_monty(v) for v in d], "_marker": None}


def _ser_ef4(c: EF4Coeffs) -> dict:
    """Serialize an extension field element to serde JSON format."""
    return {"value": [to_monty(v) for v in c], "_phantom": None}


def _ser_merkle_proof(path: MerklePath) -> list:
    """Serialize a Merkle path to serde JSON format.

    Merkle proof siblings are [BabyBear; 8] (plain arrays), NOT Hash<...>.
    """
    return [[to_monty(v) for v in d] for d in path]


def _ser_adjacent_opened_values(av: AdjacentOpenedValues) -> dict:
    """Serialize AdjacentOpenedValues to serde JSON."""
    return {
        "local": [_ser_ef4(v) for v in av.local],
        "next": [_ser_ef4(v) for v in av.next],
    }


def _ser_batch_opening(bo: BatchOpening) -> dict:
    """Serialize a BatchOpening to serde JSON."""
    return {
        "opened_values": [[to_monty(v) for v in row] for row in bo.opened_values],
        "opening_proof": _ser_merkle_proof(bo.opening_proof),
    }


def _ser_commit_phase_proof_step(step: CommitPhaseProofStep) -> dict:
    """Serialize a CommitPhaseProofStep to serde JSON."""
    return {
        "sibling_value": _ser_ef4(step.sibling_value),
        "opening_proof": _ser_merkle_proof(step.opening_proof),
    }


def _ser_query_proof(qp: QueryProof) -> dict:
    """Serialize a QueryProof to serde JSON."""
    return {
        "input_proof": [_ser_batch_opening(bo) for bo in qp.input_proof],
        "commit_phase_openings": [
            _ser_commit_phase_proof_step(s) for s in qp.commit_phase_openings
        ],
    }


def _ser_fri_proof(fp: FriProof) -> dict:
    """Serialize a FriProof to serde JSON."""
    return {
        "commit_phase_commits": [_ser_digest(c) for c in fp.commit_phase_commits],
        "commit_pow_witnesses": [_ser_fe(w) for w in fp.commit_pow_witnesses],
        "query_proofs": [_ser_query_proof(qp) for qp in fp.query_proofs],
        "final_poly": [_ser_ef4(c) for c in fp.final_poly],
        "query_pow_witness": _ser_fe(fp.query_pow_witness),
    }


def _ser_opened_values(ov: OpenedValues) -> dict:
    """Serialize OpenedValues to serde JSON."""
    return {
        "preprocessed": [_ser_adjacent_opened_values(av) for av in ov.preprocessed],
        "main": [
            [_ser_adjacent_opened_values(av) for av in commit_mats]
            for commit_mats in ov.main
        ],
        "after_challenge": [
            [_ser_adjacent_opened_values(av) for av in commit_mats]
            for commit_mats in ov.after_challenge
        ],
        "quotient": [
            [
                [_ser_ef4(v) for v in chunk]
                for chunk in air_chunks
            ]
            for air_chunks in ov.quotient
        ],
    }


def _ser_opening_proof(op: OpeningProof) -> dict:
    """Serialize OpeningProof to serde JSON."""
    return {
        "proof": _ser_fri_proof(op.proof),
        "values": _ser_opened_values(op.values),
        "deep_pow_witness": _ser_fe(op.deep_pow_witness),
    }


def _ser_air_proof_data(apd: AirProofData) -> dict:
    """Serialize AirProofData to serde JSON."""
    return {
        "air_id": apd.air_id,
        "degree": apd.degree,
        "exposed_values_after_challenge": [
            [_ser_ef4(v) for v in phase_values]
            for phase_values in apd.exposed_values_after_challenge
        ],
        "public_values": [_ser_fe(v) for v in apd.public_values],
    }


def serialize_proof_json(proof: Proof) -> dict:
    """Serialize a Proof to serde-compatible JSON dict.

    Converts canonical field elements back to Montgomery form and
    uses the exact JSON structure produced by Rust's serde_json.
    """
    return {
        "commitments": {
            "main_trace": [_ser_digest(c) for c in proof.commitments.main_trace],
            "after_challenge": [_ser_digest(c) for c in proof.commitments.after_challenge],
            "quotient": _ser_digest(proof.commitments.quotient),
        },
        "opening": _ser_opening_proof(proof.opening),
        "per_air": [_ser_air_proof_data(a) for a in proof.per_air],
        "rap_phase_seq_proof": None if proof.rap_phase_seq_proof is None else {
            "logup_pow_witness": _ser_fe(proof.rap_phase_seq_proof.logup_pow_witness),
        },
    }
