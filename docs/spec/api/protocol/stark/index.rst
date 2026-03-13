protocol.stark
==============

.. py:module:: protocol.stark

.. autoapi-nested-parse::

   STARK verifier orchestration.

   Top-level verifier that coordinates transcript operations, PCS verification,
   and constraint checking for a multi-AIR STARK proof.

   The Fiat-Shamir transcript operations must match the Rust verifier EXACTLY in
   order: any deviation produces wrong challenges and breaks verification.

   Reference:
       stark-backend/src/verifier/mod.rs  -- MultiTraceStarkVerifier::verify_raps
       stark-backend/src/verifier/error.rs -- VerificationError enum
       stark-backend/src/interaction/fri_log_up.rs -- FriLogUpPhase::partially_verify



Attributes
----------

.. autoapisummary::

   protocol.stark.p
   protocol.stark.STARK_LU_NUM_CHALLENGES
   protocol.stark.STARK_LU_NUM_EXPOSED_VALUES
   protocol.stark.EXT_DEGREE


Exceptions
----------

.. autoapisummary::

   protocol.stark.InvalidProofShape
   protocol.stark.InvalidOpeningArgument
   protocol.stark.ChallengePhaseError
   protocol.stark.InvalidDeepPowWitness


Functions
---------

.. autoapisummary::

   protocol.stark.verify_stark
   protocol.stark.prove_stark


Module Contents
---------------

.. py:data:: p
   :value: 2013265921


.. py:data:: STARK_LU_NUM_CHALLENGES
   :value: 2


.. py:data:: STARK_LU_NUM_EXPOSED_VALUES
   :value: 1


.. py:data:: EXT_DEGREE
   :value: 4


.. py:exception:: InvalidProofShape(message: str = '')

   Bases: :py:obj:`protocol.constraints.VerificationError`


   Proof structure does not match the verifying key.

   Reference:
       stark-backend/src/verifier/error.rs (VerificationError::InvalidProofShape)


.. py:exception:: InvalidOpeningArgument(message: str = '')

   Bases: :py:obj:`protocol.constraints.VerificationError`


   PCS opening verification failed.

   Reference:
       stark-backend/src/verifier/error.rs (VerificationError::InvalidOpeningArgument)


.. py:exception:: ChallengePhaseError(message: str = '')

   Bases: :py:obj:`protocol.constraints.VerificationError`


   RAP challenge phase verification failed (e.g. non-zero cumulative sum).

   Reference:
       stark-backend/src/verifier/error.rs (VerificationError::ChallengePhaseError)


.. py:exception:: InvalidDeepPowWitness(message: str = '')

   Bases: :py:obj:`protocol.constraints.VerificationError`


   DEEP proof-of-work witness is invalid.

   Reference:
       stark-backend/src/verifier/error.rs (VerificationError::InvalidDeepPowWitness)


.. py:function:: verify_stark(vk: protocol.proof.MultiStarkVerifyingKey, proof: protocol.proof.Proof, fri_params: protocol.proof.FriParameters) -> None

   Verify a multi-AIR STARK proof.

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


.. py:function:: prove_stark(vk: protocol.proof.MultiStarkVerifyingKey, traces: list[list[list[primitives.field.Fe]]], public_values_per_air: list[list[primitives.field.Fe]], fri_params: protocol.proof.FriParameters, preprocessed_traces: list[list[list[primitives.field.Fe]] | None] | None = None, cached_main_traces: list[list[list[list[primitives.field.Fe]]]] | None = None, air_ids: list[int] | None = None) -> protocol.proof.Proof

   Generate a multi-AIR STARK proof.

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


