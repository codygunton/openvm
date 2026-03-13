protocol.proof
==============

.. py:module:: protocol.proof

.. autoapi-nested-parse::

   STARK proof and verifying key data structures.

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



Classes
-------

.. autoapisummary::

   protocol.proof.Commitments
   protocol.proof.AdjacentOpenedValues
   protocol.proof.OpenedValues
   protocol.proof.CommitPhaseProofStep
   protocol.proof.BatchOpening
   protocol.proof.QueryProof
   protocol.proof.FriProof
   protocol.proof.OpeningProof
   protocol.proof.AirProofData
   protocol.proof.FriLogUpPartialProof
   protocol.proof.Proof
   protocol.proof.EntryType
   protocol.proof.Entry
   protocol.proof.SymbolicVariable
   protocol.proof.SymbolicNodeKind
   protocol.proof.SymbolicExpressionNode
   protocol.proof.SymbolicExpressionDag
   protocol.proof.Interaction
   protocol.proof.SymbolicConstraintsDag
   protocol.proof.TraceWidth
   protocol.proof.StarkVerifyingParams
   protocol.proof.RapPhaseSeqKind
   protocol.proof.VerifierSinglePreprocessedData
   protocol.proof.StarkVerifyingKey
   protocol.proof.LinearConstraint
   protocol.proof.MultiStarkVerifyingKey0
   protocol.proof.MultiStarkVerifyingKey
   protocol.proof.FriParameters
   protocol.proof.AirInputData
   protocol.proof.ProverInputs


Functions
---------

.. autoapisummary::

   protocol.proof.parse_proof_json
   protocol.proof.parse_vk_json
   protocol.proof.parse_fri_params
   protocol.proof.parse_prover_inputs
   protocol.proof.parse_e2e_vectors
   protocol.proof.serialize_proof_json


Module Contents
---------------

.. py:class:: Commitments

   All commitments to a multi-matrix STARK (not preprocessed).

   Reference:
       stark-backend/src/proof.rs (struct Commitments<Com>)


   .. py:attribute:: main_trace
      :type:  list[primitives.field.Digest]


   .. py:attribute:: after_challenge
      :type:  list[primitives.field.Digest]


   .. py:attribute:: quotient
      :type:  primitives.field.Digest


.. py:class:: AdjacentOpenedValues

   Opened values at zeta and zeta * g for one trace matrix.

   Reference:
       stark-backend/src/proof.rs (struct AdjacentOpenedValues<Challenge>)


   .. py:attribute:: local
      :type:  list[primitives.field.FF4Coeffs]


   .. py:attribute:: next
      :type:  list[primitives.field.FF4Coeffs]


.. py:class:: OpenedValues

   All opened values across preprocessed, main, after-challenge, and quotient.

   Reference:
       stark-backend/src/proof.rs (struct OpenedValues<Challenge>)


   .. py:attribute:: preprocessed
      :type:  list[AdjacentOpenedValues]


   .. py:attribute:: main
      :type:  list[list[AdjacentOpenedValues]]


   .. py:attribute:: after_challenge
      :type:  list[list[AdjacentOpenedValues]]


   .. py:attribute:: quotient
      :type:  list[list[list[primitives.field.FF4Coeffs]]]


.. py:class:: CommitPhaseProofStep

   One round of a FRI query: sibling value + Merkle proof.

   Reference:
       p3-fri/src/proof.rs (struct CommitPhaseProofStep<F, M>)


   .. py:attribute:: sibling_value
      :type:  primitives.field.FF4Coeffs


   .. py:attribute:: opening_proof
      :type:  primitives.field.MerklePath


.. py:class:: BatchOpening

   Opened values from a single MMCS batch at a query index.

   Reference:
       p3-fri/src/two_adic_pcs.rs (struct BatchOpening<Val, InputMmcs>)


   .. py:attribute:: opened_values
      :type:  list[list[primitives.field.Fe]]


   .. py:attribute:: opening_proof
      :type:  primitives.field.MerklePath


.. py:class:: QueryProof

   FRI query proof for a single query index.

   Reference:
       p3-fri/src/proof.rs (struct QueryProof<F, M, InputProof>)


   .. py:attribute:: input_proof
      :type:  list[BatchOpening]


   .. py:attribute:: commit_phase_openings
      :type:  list[CommitPhaseProofStep]


.. py:class:: FriProof

   Complete FRI proof.

   Reference:
       p3-fri/src/proof.rs (struct FriProof<F, M, Witness, InputProof>)


   .. py:attribute:: commit_phase_commits
      :type:  list[primitives.field.Digest]


   .. py:attribute:: query_proofs
      :type:  list[QueryProof]


   .. py:attribute:: final_poly
      :type:  list[primitives.field.FF4Coeffs]


   .. py:attribute:: commit_pow_witnesses
      :type:  list[int]


   .. py:attribute:: query_pow_witness
      :type:  int


.. py:class:: OpeningProof

   PCS opening proof with opened values.

   Reference:
       stark-backend/src/proof.rs (struct OpeningProof<PcsProof, Challenge>)


   .. py:attribute:: proof
      :type:  FriProof


   .. py:attribute:: values
      :type:  OpenedValues


   .. py:attribute:: deep_pow_witness
      :type:  int


.. py:class:: AirProofData

   Proof data for a single AIR.

   Reference:
       stark-backend/src/proof.rs (struct AirProofData<Val, Challenge>)


   .. py:attribute:: air_id
      :type:  int


   .. py:attribute:: degree
      :type:  int


   .. py:attribute:: exposed_values_after_challenge
      :type:  list[list[primitives.field.FF4Coeffs]]


   .. py:attribute:: public_values
      :type:  list[primitives.field.Fe]


.. py:class:: FriLogUpPartialProof

   Partial proof for the FRI LogUp challenge phase.

   Reference:
       stark-backend/src/interaction/fri_log_up.rs (struct FriLogUpPartialProof<Witness>)


   .. py:attribute:: logup_pow_witness
      :type:  primitives.field.Fe


.. py:class:: Proof

   Full multi-AIR STARK proof.

   Reference:
       stark-backend/src/proof.rs (struct Proof<SC>)


   .. py:attribute:: commitments
      :type:  Commitments


   .. py:attribute:: opening
      :type:  OpeningProof


   .. py:attribute:: per_air
      :type:  list[AirProofData]


   .. py:attribute:: rap_phase_seq_proof
      :type:  Optional[FriLogUpPartialProof]
      :value: None



.. py:class:: EntryType

   Bases: :py:obj:`enum.Enum`


   Kind of symbolic variable entry.

   Reference:
       stark-backend/src/air_builders/symbolic/symbolic_variable.rs (enum Entry)


   .. py:attribute:: PREPROCESSED


   .. py:attribute:: MAIN


   .. py:attribute:: PERMUTATION


   .. py:attribute:: PUBLIC


   .. py:attribute:: CHALLENGE


   .. py:attribute:: EXPOSED


.. py:class:: Entry

   Symbolic variable entry with kind and offset/part_index.

   Reference:
       stark-backend/src/air_builders/symbolic/symbolic_variable.rs (enum Entry)


   .. py:attribute:: kind
      :type:  EntryType


   .. py:attribute:: offset
      :type:  Optional[int]
      :value: None



   .. py:attribute:: part_index
      :type:  Optional[int]
      :value: None



.. py:class:: SymbolicVariable

   A variable within the evaluation window (column reference).

   Reference:
       stark-backend/src/air_builders/symbolic/symbolic_variable.rs
       (struct SymbolicVariable<F>)


   .. py:attribute:: entry
      :type:  Entry


   .. py:attribute:: index
      :type:  int


.. py:class:: SymbolicNodeKind

   Bases: :py:obj:`enum.Enum`


   Kind of symbolic expression node.

   Reference:
       stark-backend/src/air_builders/symbolic/dag.rs (enum SymbolicExpressionNode)


   .. py:attribute:: VARIABLE


   .. py:attribute:: IS_FIRST_ROW


   .. py:attribute:: IS_LAST_ROW


   .. py:attribute:: IS_TRANSITION


   .. py:attribute:: CONSTANT


   .. py:attribute:: ADD


   .. py:attribute:: SUB


   .. py:attribute:: NEG


   .. py:attribute:: MUL


.. py:class:: SymbolicExpressionNode

   A node in the symbolic expression DAG.

   Reference:
       stark-backend/src/air_builders/symbolic/dag.rs
       (enum SymbolicExpressionNode<F>)

   Fields vary by kind:
   - VARIABLE: variable is set
   - CONSTANT: constant_value is set
   - ADD/SUB/MUL: left_idx, right_idx, degree_multiple are set
   - NEG: idx, degree_multiple are set
   - IS_FIRST_ROW/IS_LAST_ROW/IS_TRANSITION: no extra fields


   .. py:attribute:: kind
      :type:  SymbolicNodeKind


   .. py:attribute:: variable
      :type:  Optional[SymbolicVariable]
      :value: None



   .. py:attribute:: constant_value
      :type:  Optional[primitives.field.Fe]
      :value: None



   .. py:attribute:: left_idx
      :type:  Optional[int]
      :value: None



   .. py:attribute:: right_idx
      :type:  Optional[int]
      :value: None



   .. py:attribute:: idx
      :type:  Optional[int]
      :value: None



   .. py:attribute:: degree_multiple
      :type:  Optional[int]
      :value: None



.. py:class:: SymbolicExpressionDag

   DAG of symbolic expressions in topological order.

   Reference:
       stark-backend/src/air_builders/symbolic/dag.rs
       (struct SymbolicExpressionDag<F>)


   .. py:attribute:: nodes
      :type:  list[SymbolicExpressionNode]


   .. py:attribute:: constraint_idx
      :type:  list[int]


.. py:class:: Interaction

   A bus interaction.

   Reference:
       stark-backend/src/interaction/mod.rs (struct Interaction<Expr>)

   In the DAG form, message and count are node indices (int).


   .. py:attribute:: message
      :type:  list[int]


   .. py:attribute:: count
      :type:  int


   .. py:attribute:: bus_index
      :type:  int


   .. py:attribute:: count_weight
      :type:  int


.. py:class:: SymbolicConstraintsDag

   Complete symbolic constraints for a single AIR.

   Reference:
       stark-backend/src/air_builders/symbolic/dag.rs
       (struct SymbolicConstraintsDag<F>)


   .. py:attribute:: constraints
      :type:  SymbolicExpressionDag


   .. py:attribute:: interactions
      :type:  list[Interaction]


.. py:class:: TraceWidth

   Widths of different parts of a trace matrix.

   Reference:
       stark-backend/src/keygen/types.rs (struct TraceWidth)


   .. py:attribute:: preprocessed
      :type:  Optional[int]


   .. py:attribute:: cached_mains
      :type:  list[int]


   .. py:attribute:: common_main
      :type:  int


   .. py:attribute:: after_challenge
      :type:  list[int]


.. py:class:: StarkVerifyingParams

   Verification parameters for a single STARK.

   Reference:
       stark-backend/src/keygen/types.rs (struct StarkVerifyingParams)


   .. py:attribute:: width
      :type:  TraceWidth


   .. py:attribute:: num_public_values
      :type:  int


   .. py:attribute:: num_exposed_values_after_challenge
      :type:  list[int]


   .. py:attribute:: num_challenges_to_sample
      :type:  list[int]


.. py:class:: RapPhaseSeqKind

   Bases: :py:obj:`enum.Enum`


   Supported challenge phase protocols.

   Reference:
       stark-backend/src/interaction/mod.rs (enum RapPhaseSeqKind)


   .. py:attribute:: FRI_LOG_UP


.. py:class:: VerifierSinglePreprocessedData

   Verifier data for preprocessed trace for a single AIR.

   Reference:
       stark-backend/src/keygen/types.rs (struct VerifierSinglePreprocessedData<Com>)


   .. py:attribute:: commit
      :type:  primitives.field.Digest


.. py:class:: StarkVerifyingKey

   Verifying key for a single STARK (single AIR).

   Reference:
       stark-backend/src/keygen/types.rs (struct StarkVerifyingKey<Val, Com>)


   .. py:attribute:: preprocessed_data
      :type:  Optional[VerifierSinglePreprocessedData]


   .. py:attribute:: params
      :type:  StarkVerifyingParams


   .. py:attribute:: symbolic_constraints
      :type:  SymbolicConstraintsDag


   .. py:attribute:: quotient_degree
      :type:  int


   .. py:attribute:: rap_phase_seq_kind
      :type:  RapPhaseSeqKind


.. py:class:: LinearConstraint

   Linear constraint on trace heights.

   Reference:
       stark-backend/src/keygen/types.rs (struct LinearConstraint)


   .. py:attribute:: coefficients
      :type:  list[int]


   .. py:attribute:: threshold
      :type:  int


.. py:class:: MultiStarkVerifyingKey0

   Inner verifying key data (without pre_hash).

   Reference:
       stark-backend/src/keygen/types.rs (struct MultiStarkVerifyingKey0<SC>)


   .. py:attribute:: per_air
      :type:  list[StarkVerifyingKey]


   .. py:attribute:: trace_height_constraints
      :type:  list[LinearConstraint]


   .. py:attribute:: log_up_pow_bits
      :type:  int


   .. py:attribute:: deep_pow_bits
      :type:  int


.. py:class:: MultiStarkVerifyingKey

   Complete multi-AIR verifying key.

   Reference:
       stark-backend/src/keygen/types.rs (struct MultiStarkVerifyingKey<SC>)


   .. py:attribute:: inner
      :type:  MultiStarkVerifyingKey0


   .. py:attribute:: pre_hash
      :type:  primitives.field.Digest


.. py:class:: FriParameters

   FRI protocol parameters.

   Reference:
       stark-sdk/src/config/mod.rs (struct FriParameters)


   .. py:attribute:: log_blowup
      :type:  int


   .. py:attribute:: log_final_poly_len
      :type:  int


   .. py:attribute:: num_queries
      :type:  int


   .. py:attribute:: query_proof_of_work_bits
      :type:  int


   .. py:attribute:: commit_proof_of_work_bits
      :type:  int


.. py:function:: parse_proof_json(data: dict) -> Proof

   Parse a Proof from a JSON dict (serde_json format).

   Handles the JSON produced by serde_json::to_vec(&proof) for
   Proof<BabyBearPoseidon2Config>.

   Reference:
       stark-backend/src/proof.rs (struct Proof<SC>)


.. py:function:: parse_vk_json(data: dict) -> MultiStarkVerifyingKey

   Parse a MultiStarkVerifyingKey from a JSON dict (serde_json format).

   Reference:
       stark-backend/src/keygen/types.rs (struct MultiStarkVerifyingKey<SC>)


.. py:function:: parse_fri_params(data: dict) -> FriParameters

   Parse FRI parameters from a test vector JSON dict.

   Reference:
       crates/test-vectors/src/lib.rs (struct FriParamsMeta)


.. py:class:: AirInputData

   Raw input trace matrices for one AIR, used by the prover.

   Reference:
       crates/test-vectors/src/lib.rs (struct AirInputVectors)


   .. py:attribute:: air_id
      :type:  int


   .. py:attribute:: common_main
      :type:  list[list[primitives.field.Fe]] | None


   .. py:attribute:: cached_mains
      :type:  list[list[list[primitives.field.Fe]]]


   .. py:attribute:: preprocessed
      :type:  list[list[primitives.field.Fe]] | None


.. py:class:: ProverInputs

   All input trace data needed by the prover.

   Reference:
       crates/test-vectors/src/lib.rs (struct ProverInputVectors)


   .. py:attribute:: per_air
      :type:  list[AirInputData]


.. py:function:: parse_prover_inputs(data: dict) -> ProverInputs

   Parse prover input vectors from JSON.

   Field values are canonical u32, no Montgomery conversion needed
   (traces are raw field elements, not serialized proof data).

   Reference:
       crates/test-vectors/src/lib.rs (struct ProverInputVectors)


.. py:function:: parse_e2e_vectors(vectors: dict) -> tuple[Proof, FriParameters, dict]

   Parse complete E2E test vectors.

   Returns:
       (proof, fri_params, commitments_meta) where:
       - proof: parsed Proof dataclass
       - fri_params: parsed FriParameters
       - commitments_meta: dict with main_trace_commitments, after_challenge_commitments,
         quotient_commitment from the test vector (pre-extracted canonical values)

   Reference:
       crates/test-vectors/src/lib.rs (struct E2eProofVectors)


.. py:function:: serialize_proof_json(proof: Proof) -> dict

   Serialize a Proof to serde-compatible JSON dict.

   Converts canonical field elements back to Montgomery form and
   uses the exact JSON structure produced by Rust's serde_json.


