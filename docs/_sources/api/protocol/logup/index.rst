protocol.logup
==============

.. py:module:: protocol.logup

.. autoapi-nested-parse::

   FriLogUp auxiliary column computation.

   Computes the after-challenge trace for the FRI LogUp protocol, which implements
   the log-derivative argument for bus interactions in multi-AIR STARKs.

   Given interaction definitions from the VK, main/preprocessed trace values, and
   random challenges (alpha, beta), this module computes:
   1. Reciprocal columns for interaction denominators
   2. Chunk sums (bundled reciprocals weighted by counts)
   3. Running sum (phi) column for cumulative sum verification

   Reference:
       stark-backend/src/interaction/fri_log_up.rs  — generate_after_challenge_trace,
                                                       find_interaction_chunks
       stark-backend/src/interaction/trace.rs       — Evaluator (row-level DAG evaluator)
       stark-backend/src/interaction/utils.rs       — generate_betas



Attributes
----------

.. autoapisummary::

   protocol.logup.p


Functions
---------

.. autoapisummary::

   protocol.logup.node_degree
   protocol.logup.eval_dag_at_row
   protocol.logup.generate_betas
   protocol.logup.find_interaction_chunks
   protocol.logup.compute_max_constraint_degree
   protocol.logup.compute_after_challenge_trace


Module Contents
---------------

.. py:data:: p
   :value: 2013265921


.. py:function:: node_degree(node: protocol.proof.SymbolicExpressionNode) -> int

   Return the degree_multiple of a DAG node.

   Reference:
       stark-backend symbolic_expression.rs SymbolicExpression::degree_multiple


.. py:function:: eval_dag_at_row(dag: protocol.proof.SymbolicExpressionDag, partitioned_main: list[list[list[primitives.field.Fe]]], preprocessed: list[list[primitives.field.Fe]] | None, public_values: list[primitives.field.Fe], height: int, row_idx: int) -> list[primitives.field.Fe]

   Evaluate all DAG nodes at a specific trace row in the base field.

   Unlike the OOD evaluator (constraints.py) which works in FF4, this evaluator
   works in the base field since we're evaluating at actual trace domain points.

   IsFirstRow/IsLastRow/IsTransition are set to 0 since interaction expressions
   never reference them (the Rust Evaluator marks them as unreachable).

   Args:
       dag: The symbolic expression DAG.
       partitioned_main: [part_index][rows][cols] — cached mains first, then common main.
       preprocessed: [rows][cols] or None.
       public_values: Public input values.
       height: Trace height.
       row_idx: Current row index.

   Returns:
       Evaluated base field values for every node in the DAG.

   Reference:
       stark-backend/src/interaction/trace.rs Evaluator::eval_var


.. py:function:: generate_betas(beta: primitives.field.FF4, interactions: list[protocol.proof.Interaction]) -> list[primitives.field.FF4]

   Generate [beta^0, beta^1, ..., beta^{max_msg_len}].

   Reference:
       stark-backend/src/interaction/utils.rs generate_betas


.. py:function:: find_interaction_chunks(interactions: list[protocol.proof.Interaction], dag: protocol.proof.SymbolicExpressionDag, max_constraint_degree: int) -> list[list[int]]

   Partition interactions into chunks respecting max constraint degree.

   Returns list of lists of interaction indices.
   Width of after_challenge trace = len(partitions) + 1 (extra phi column).

   Algorithm:
   1. Sort interaction indices by ascending (max_field_degree, count_degree)
   2. Greedily pack: add to current chunk if degree constraint allows
   3. Seal chunk and start new one when degree would be exceeded

   Reference:
       stark-backend/src/interaction/fri_log_up.rs find_interaction_chunks (lines 573-643)


.. py:function:: compute_max_constraint_degree(per_air_vks: list[protocol.proof.StarkVerifyingKey]) -> int

   Compute max constraint degree across all AIRs from their DAGs.

   This is the maximum degree_multiple of any constraint output node across all AIRs.

   Reference:
       stark-backend/src/prover/coordinator.rs max_constraint_degree computation


.. py:function:: compute_after_challenge_trace(interactions: list[protocol.proof.Interaction], interaction_partitions: list[list[int]], dag: protocol.proof.SymbolicExpressionDag, partitioned_main: list[list[list[primitives.field.Fe]]], preprocessed: list[list[primitives.field.Fe]] | None, public_values: list[primitives.field.Fe], alpha: primitives.field.FF4, beta: primitives.field.FF4, height: int) -> tuple[list[list[list[int]]], list[int]]

   Compute FriLogUp after-challenge trace and cumulative sum.

   Processes ALL rows simultaneously using vectorized field arrays.

   Args:
       interactions: List of Interaction (message/count are DAG node indices).
       interaction_partitions: Chunking of interaction indices.
       dag: Symbolic expression DAG.
       partitioned_main: [part_index][rows][cols].
       preprocessed: [rows][cols] or None.
       public_values: Public input values.
       alpha: First interaction challenge (FF4).
       beta: Second interaction challenge (FF4).
       height: Trace height.

   Returns:
       (perm_trace, cumulative_sum) where:
       - perm_trace: [height][perm_width] of FF4 values (as list[int])
       - perm_width = len(interaction_partitions) + 1
       - cumulative_sum: FF4 value (as list[int])

   Reference:
       stark-backend/src/interaction/fri_log_up.rs
       generate_after_challenge_trace (lines 299-437)


