protocol.constraints
====================

.. py:module:: protocol.constraints

.. autoapi-nested-parse::

   Constraint DAG evaluation and STARK constraint verification.

   Evaluates the SymbolicExpressionDag at the OOD (out-of-domain) point and checks
   that folded_constraints * inv_zeroifier == quotient for each AIR.

   Reference:
       stark-backend/src/verifier/constraints.rs   -- verify_single_rap_constraints
       stark-backend/src/verifier/folder.rs         -- GenericVerifierConstraintFolder
       stark-backend/src/air_builders/symbolic/symbolic_expression.rs -- SymbolicEvaluator::eval_nodes
       stark-backend/src/air_builders/symbolic/dag.rs -- SymbolicExpressionNode



Exceptions
----------

.. autoapisummary::

   protocol.constraints.VerificationError
   protocol.constraints.OodEvaluationMismatch


Functions
---------

.. autoapisummary::

   protocol.constraints.unflatten_ext_values
   protocol.constraints.eval_symbolic_dag
   protocol.constraints.fold_constraints
   protocol.constraints.reconstruct_quotient
   protocol.constraints.eval_dag_all_rows
   protocol.constraints.verify_single_rap_constraints


Module Contents
---------------

.. py:function:: unflatten_ext_values(flattened: list) -> list[primitives.field.FF4]

   Reconstitute extension field elements from flattened challenge values.

   In the Rust verifier, after_challenge (permutation) trace values are stored
   as "flattened" extension field elements: each FF4 element is stored as 4
   consecutive Challenge (== FF4) values where only the base field component
   is meaningful.  This function groups them back into proper FF4 elements.

   Given D=4 (extension degree), each group of 4 consecutive flattened values
   [c0, c1, c2, c3] (each an FF4Coeffs with only coeffs[0] used) is combined
   using the monomial basis:
       result = c0 * 1 + c1 * x + c2 * x^2 + c3 * x^3

   Since each c_i is already an FF4Coeffs, multiplying by x^e_i is the same
   as shifting coefficients. But in the verifier, each c_i is treated as a
   full FF4 element, and monomial(e_i) * c_i uses extension field multiplication.

   For BabyBear extension degree 4 with irreducible x^4 - 11:
       monomial(0) = [1,0,0,0]
       monomial(1) = [0,1,0,0]
       monomial(2) = [0,0,1,0]
       monomial(3) = [0,0,0,1]

   Reference:
       stark-backend/src/verifier/constraints.rs lines 65-75 (unflatten closure)


.. py:function:: eval_symbolic_dag(dag: protocol.proof.SymbolicExpressionDag, selectors: protocol.domain.DomainSelectors, preprocessed_local: list, preprocessed_next: list, partitioned_main_values: list[protocol.proof.AdjacentOpenedValues], after_challenge_values: list[protocol.proof.AdjacentOpenedValues], challenges: list[list], public_values: list[primitives.field.Fe], exposed_values_after_challenge: list[list]) -> list[primitives.field.FF4]

   Evaluate the symbolic expression DAG and return constraint values.

   Walks the DAG nodes in topological order (they are already sorted).
   For each node, computes the extension field value based on the node kind.

   Returns the values at the constraint output indices (dag.constraint_idx).

   Reference:
       stark-backend/src/air_builders/symbolic/symbolic_expression.rs
           SymbolicEvaluator::eval_nodes (lines 364-396)
       stark-backend/src/verifier/folder.rs
           GenericVerifierConstraintFolder impl of SymbolicEvaluator (lines 74-123)


.. py:function:: fold_constraints(constraint_evals: list[primitives.field.FF4], alpha: primitives.field.FF4) -> primitives.field.FF4

   Compute random linear combination of constraint evaluations.

   The Rust verifier uses Horner's method: starting with accumulator = 0,
   for each constraint C_i:
       accumulator = accumulator * alpha + C_i

   This produces: C_0 * alpha^{n-1} + C_1 * alpha^{n-2} + ... + C_{n-1}

   Reference:
       stark-backend/src/verifier/folder.rs lines 56-72
       (GenericVerifierConstraintFolder::eval_constraints + assert_zero)


.. py:function:: reconstruct_quotient(quotient_chunks: list[list], qc_domains: list[protocol.domain.TwoAdicMultiplicativeCoset], zeta: primitives.field.FF4) -> primitives.field.FF4

   Recompute the full quotient polynomial value at zeta from chunks.

   For each chunk domain D_i, compute:
       zps[i] = prod_{j != i} Z_{D_j}(zeta) / Z_{D_j}(first_point(D_i))

   Then the full quotient at zeta is:
       Q(zeta) = sum_i zps[i] * sum_e (monomial(e) * chunk[i][e])

   Each quotient_chunks[i] has D=4 elements, which are the coefficients
   in the extension field basis. So quotient_chunks[i] is treated as a
   single FF4 element by combining with the monomial basis.

   Reference:
       stark-backend/src/verifier/constraints.rs lines 38-63


.. py:function:: eval_dag_all_rows(dag: protocol.proof.SymbolicExpressionDag, partitioned_main: list[list[list[primitives.field.Fe]]], preprocessed: list[list[primitives.field.Fe]] | None, public_values: list[primitives.field.Fe], height: int) -> list[primitives.field.FF | None]

   Evaluate full DAG at ALL rows simultaneously using field column arrays.

   Each DAG node evaluates to a base-field column vector of length height.

   Args:
       dag: SymbolicExpressionDag
       partitioned_main: [part_index][rows][cols] — list of trace matrices
       preprocessed: [rows][cols] or None
       public_values: list of Fe
       height: trace height

   Returns:
       List of field column arrays, one per DAG node.


.. py:exception:: VerificationError

   Bases: :py:obj:`Exception`


   Raised when STARK constraint verification fails.

   Reference:
       stark-backend/src/verifier/error.rs (enum VerificationError)


.. py:exception:: OodEvaluationMismatch

   Bases: :py:obj:`VerificationError`


   Out-of-domain evaluation mismatch: constraints(zeta) != quotient(zeta) * Z_H(zeta).

   Reference:
       stark-backend/src/verifier/error.rs (VerificationError::OodEvaluationMismatch)


.. py:function:: verify_single_rap_constraints(constraints: protocol.proof.SymbolicExpressionDag, preprocessed_values: protocol.proof.AdjacentOpenedValues | None, partitioned_main_values: list[protocol.proof.AdjacentOpenedValues], after_challenge_values: list[protocol.proof.AdjacentOpenedValues], quotient_chunks: list[list], domain: protocol.domain.TwoAdicMultiplicativeCoset, qc_domains: list[protocol.domain.TwoAdicMultiplicativeCoset], zeta: primitives.field.FF4, alpha: primitives.field.FF4, challenges: list[list], public_values: list[primitives.field.Fe], exposed_values_after_challenge: list[list]) -> None

   Verify constraints for a single RAP (AIR with interactions).

   Steps:
   1. Compute selectors at zeta (is_first_row, is_last_row, is_transition, inv_zeroifier).
   2. Unflatten after_challenge values from base-field-flattened to FF4 elements.
   3. Evaluate constraint DAG at opened values.
   4. Fold constraints with alpha (random linear combination).
   5. Reconstruct quotient from chunks.
   6. Assert: folded_constraints * inv_zeroifier == quotient_value.

   Raises OodEvaluationMismatch if the check fails.

   Reference:
       stark-backend/src/verifier/constraints.rs lines 21-140
       (verify_single_rap_constraints)


