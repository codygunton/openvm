protocol.fri
============

.. py:module:: protocol.fri

.. autoapi-nested-parse::

   FRI (Fast Reed-Solomon IOP of Proximity) protocol.

   Implements FRI folding, verification, and proving matching Plonky3's FRI.

   Reference:
       p3-fri-0.4.1/src/ (prover.rs, verifier.rs, two_adic_pcs.rs)



Attributes
----------

.. autoapisummary::

   protocol.fri.p


Classes
-------

.. autoapisummary::

   protocol.fri.CommitPhaseResult
   protocol.fri.FriQueryStep
   protocol.fri.FriQueryResult
   protocol.fri.CommitPhaseOutput


Functions
---------

.. autoapisummary::

   protocol.fri.fri_fold
   protocol.fri.fold_row
   protocol.fri.hash_fri_leaf
   protocol.fri.ef4_pairs_to_leaves
   protocol.fri.fri_verify_query
   protocol.fri.verify_fri
   protocol.fri.fold_matrix
   protocol.fri.commit_phase
   protocol.fri.answer_query
   protocol.fri.prove_fri


Module Contents
---------------

.. py:data:: p
   :value: 2013265921


.. py:class:: CommitPhaseResult

   Output of FRI commit phase.


   .. py:attribute:: commits
      :type:  list[primitives.field.Digest]


   .. py:attribute:: betas
      :type:  list[list[int]]


   .. py:attribute:: final_poly
      :type:  list[list[int]]


   .. py:attribute:: trees
      :type:  list[list[list[primitives.field.Digest]]]


   .. py:attribute:: folded_per_round
      :type:  list[list[list[int]]]


   .. py:attribute:: all_round_evals
      :type:  list[list[list[int]]]


   .. py:attribute:: commit_pow_witnesses
      :type:  list[int]


.. py:class:: FriQueryStep

   One round of a FRI query opening.


   .. py:attribute:: sibling_value
      :type:  list[int]


   .. py:attribute:: opening_proof
      :type:  primitives.field.MerklePath


.. py:class:: FriQueryResult

   FRI query proof for a single query index.


   .. py:attribute:: index
      :type:  int


   .. py:attribute:: commit_phase_openings
      :type:  list[FriQueryStep]


.. py:class:: CommitPhaseOutput

   Complete FRI proof.


   .. py:attribute:: commit_phase_commits
      :type:  list[primitives.field.Digest]


   .. py:attribute:: final_poly
      :type:  list[list[int]]


   .. py:attribute:: query_proofs
      :type:  list[FriQueryResult]


   .. py:attribute:: betas
      :type:  list[list[int]]


   .. py:attribute:: folded_per_round
      :type:  list[list[list[int]]]


.. py:function:: fri_fold(evals: list[list[int]], challenge: list[int], log_domain_size: int, coset_shift: primitives.field.Fe) -> list[list[int]]

   Fold evaluations on coset: f_even(y) + beta * f_odd(y).

   Reference:
       p3-fri two_adic_pcs.rs (TwoAdicFriFolder::fold_row)


.. py:function:: fold_row(index: int, log_height: int, beta: primitives.field.FF4, e0: primitives.field.FF4, e1: primitives.field.FF4) -> primitives.field.FF4

   Lagrange interpolation fold at challenge beta.

   Reference:
       p3-fri two_adic_pcs.rs (TwoAdicFriFolding::fold_row)


.. py:function:: hash_fri_leaf(e0: primitives.field.FF4, e1: primitives.field.FF4) -> primitives.field.Digest

   Hash pair of extension field evaluations as FRI Merkle leaf.

   Reference:
       p3-merkle-tree mmcs.rs (verify_batch leaf hashing)


.. py:function:: ef4_pairs_to_leaves(evals: list[list[int]]) -> list[list[int]]

   Pair consecutive FF4 elements into 8-element Merkle leaves.


.. py:function:: fri_verify_query(commit_phase_commits: list[primitives.field.Digest], betas: list[list[int]], query_index: int, query_proof: dict, reduced_opening: list[int], final_poly: list[list[int]], log_max_height: int, log_final_poly_len: int) -> list[int]

   Verify single FRI query: fold chain + Merkle proofs + final poly check.

   Reference:
       p3-fri verifier.rs (verify_query)


.. py:function:: verify_fri(commit_phase_commits: list[primitives.field.Digest], final_poly: list[list[int]], query_proofs: list[dict], log_blowup: int, log_final_poly_len: int, num_queries: int) -> bool

   Verify FRI proof: transcript replay and structural consistency.

   Note: Full fold-chain verification requires reduced_openings from PCS.
   This function verifies transcript replay, query index derivation, and
   proof structure (lengths, digest sizes).

   Reference:
       p3-fri verifier.rs (verify_fri)


.. py:function:: fold_matrix(evals_bit_reversed: list[list[int]], beta: primitives.field.FF4, log_height: int) -> list[list[int]]

   Fold bit-reversed evaluations: adjacent pairs are conjugates.

   Reference:
       p3-fri two_adic_pcs.rs (TwoAdicFriFolding::fold_matrix)


.. py:function:: commit_phase(evals_bit_reversed: list[list[int]], log_blowup: int, log_final_poly_len: int, challenger: primitives.transcript.Challenger, commit_pow_bits: int = 0, reduced_openings_by_height: dict[int, list[list[int]]] | None = None) -> CommitPhaseResult

   FRI commit phase: iterative folding with Merkle commitments.

   For multi-height FRI, reduced_openings_by_height maps log_height to
   bit-reversed reduced evaluations at that height.  After folding to a
   given height, the corresponding reduced opening is rolled in using
   beta^2 as the combination factor (matching the verifier).

   Reference:
       p3-fri prover.rs (commit_phase)


.. py:function:: answer_query(trees: list[list[list[primitives.field.Digest]]], all_round_evals: list[list[list[int]]], start_index: int, num_rounds: int) -> list[FriQueryStep]

   Generate FRI query opening for a single query index.

   Reference:
       p3-fri prover.rs (answer_query)


.. py:function:: prove_fri(evals_bit_reversed: list[list[int]], log_blowup: int, log_final_poly_len: int, num_queries: int, challenger: primitives.transcript.Challenger) -> CommitPhaseOutput

   Full FRI proof generation.

   Reference:
       p3-fri prover.rs (prove)


