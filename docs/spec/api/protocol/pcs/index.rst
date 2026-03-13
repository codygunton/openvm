protocol.pcs
============

.. py:module:: protocol.pcs

.. autoapi-nested-parse::

   PCS (Polynomial Commitment Scheme) for TwoAdicFriPcs.

   Implements both prover and verifier sides of the FRI-based PCS.

   Verifier: given committed polynomial batches opened at evaluation points,
   reduces them to a single FRI verification problem.

   Prover: commits polynomials via LDE + Merkle, evaluates at opening points,
   computes reduced polynomials, and delegates to the FRI prover.

   Reference:
       p3-fri-0.4.1/src/two_adic_pcs.rs  -- TwoAdicFriPcs::commit, open, verify
       p3-fri-0.4.1/src/verifier.rs      -- verify_fri, verify_query, open_input
       p3-fri-0.4.1/src/prover.rs        -- prove (FRI prover)



Attributes
----------

.. autoapisummary::

   protocol.pcs.p


Classes
-------

.. autoapisummary::

   protocol.pcs.PcsRound
   protocol.pcs.CommittedData
   protocol.pcs.PcsOpeningRound


Functions
---------

.. autoapisummary::

   protocol.pcs.pcs_verify
   protocol.pcs.pcs_commit
   protocol.pcs.pcs_open


Module Contents
---------------

.. py:data:: p
   :value: 2013265921


.. py:class:: PcsRound

   One round of PCS verification data.

   Corresponds to a single MMCS commitment (batch of matrices) and the
   domains/openings associated with it.

   Reference:
       p3-fri/src/two_adic_pcs.rs (CommitmentWithOpeningPoints type alias)


   .. py:attribute:: commitment
      :type:  primitives.field.Digest


   .. py:attribute:: domains_and_openings
      :type:  list[tuple[protocol.domain.TwoAdicMultiplicativeCoset, list[tuple[list[int], list[list[int]]]]]]


.. py:function:: pcs_verify(rounds: list[PcsRound], fri_proof: protocol.proof.FriProof, challenger: primitives.transcript.Challenger, fri_params: protocol.proof.FriParameters) -> None

   Verify PCS openings using FRI.

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


.. py:class:: CommittedData

   Prover data for a single MMCS commitment (batch of matrices).

   Reference:
       p3-merkle-tree MerkleTreeMmcs::commit


   .. py:attribute:: root
      :type:  primitives.field.Digest


   .. py:attribute:: tree
      :type:  list[list[primitives.field.Digest]]


   .. py:attribute:: lde_rows
      :type:  list[list[list[primitives.field.Fe]]]


   .. py:attribute:: coeffs
      :type:  list[list[list[primitives.field.Fe]]]


   .. py:attribute:: domains
      :type:  list[protocol.domain.TwoAdicMultiplicativeCoset]


.. py:function:: pcs_commit(evaluations: list[tuple[protocol.domain.TwoAdicMultiplicativeCoset, list[list[primitives.field.Fe]]]], log_blowup: int) -> CommittedData

   Commit to a batch of matrices via LDE + Merkle tree.

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


.. py:class:: PcsOpeningRound

   One round of PCS opening data for the prover.

   Reference:
       p3-fri-0.4.1/src/two_adic_pcs.rs TwoAdicFriPcs::open


   .. py:attribute:: committed
      :type:  CommittedData


   .. py:attribute:: points_per_mat
      :type:  list[list[list[int]]]


.. py:function:: pcs_open(rounds: list[PcsOpeningRound], challenger: primitives.transcript.Challenger, fri_params: protocol.proof.FriParameters) -> tuple[list[list[list[list[list[int]]]]], dict, list[int]]

   Open committed polynomials at specified points.

   Prover-side PCS open: evaluates polynomials, computes reduced
   polynomials, runs FRI, and generates Merkle openings.

   Args:
       rounds: Per-commitment opening data.
       challenger: The Fiat-Shamir challenger (quotient already observed).
       fri_params: FRI protocol parameters.

   Returns:
       (all_opened_values, fri_proof_data, query_indices) where:
       - all_opened_values[round][mat][point] = list[list[int]]
       - fri_proof_data contains the FRI proof components
       - query_indices for Merkle opening generation

   Reference:
       p3-fri-0.4.1/src/two_adic_pcs.rs TwoAdicFriPcs::open lines 286-440


