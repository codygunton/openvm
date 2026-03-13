protocol.quotient
=================

.. py:module:: protocol.quotient

.. autoapi-nested-parse::

   Prover-side quotient polynomial computation.

   Computes Q(x) = (sum of alpha-weighted constraints at x) / Z_H(x) for each
   point x in the quotient domain.  The quotient domain is a coset disjoint from
   the trace domain with size = trace_size * quotient_degree.

   The output is split into `quotient_degree` chunks, each of trace_domain.size()
   rows, with extension field elements flattened to base field coefficients for
   Merkle commitment.

   Reference:
       stark-backend/src/prover/cpu/quotient/single.rs  — compute_single_rap_quotient_values
       stark-backend/src/prover/cpu/quotient/mod.rs      — QuotientCommitter
       stark-backend/src/prover/cpu/quotient/evaluator.rs — ProverConstraintEvaluator
       p3-commit-0.4.1/src/domain.rs                      — selectors_on_coset
       p3-dft-0.4.1/src/traits.rs                         — coset_lde_batch



Attributes
----------

.. autoapisummary::

   protocol.quotient.p


Classes
-------

.. autoapisummary::

   protocol.quotient.CosetSelectors


Functions
---------

.. autoapisummary::

   protocol.quotient.coset_lde_column
   protocol.quotient.extend_trace_to_quotient_domain
   protocol.quotient.selectors_on_coset
   protocol.quotient.eval_symbolic_expression_dag
   protocol.quotient.eval_symbolic_expression_dag_full
   protocol.quotient.accumulate_constraints_ef4
   protocol.quotient.accumulate_constraints
   protocol.quotient.compute_quotient_values
   protocol.quotient.quotient_values_to_chunks
   protocol.quotient.extend_after_challenge_to_quotient_domain
   protocol.quotient.compute_quotient_chunks


Module Contents
---------------

.. py:data:: p
   :value: 2013265921


.. py:function:: coset_lde_column(evals: list[primitives.field.Fe], trace_domain: protocol.domain.TwoAdicMultiplicativeCoset, quotient_domain: protocol.domain.TwoAdicMultiplicativeCoset) -> list[primitives.field.Fe]

   Extend polynomial from trace domain to quotient domain via coset LDE.

   Reference:
       p3-dft-0.4.1/src/traits.rs coset_lde_batch (lines 226-249)
       p3-dft-0.4.1/src/util.rs   coset_shift_cols (lines 28-36)

   Args:
       evals: Column evaluations on the trace domain (natural order).
       trace_domain: The trace domain (shift=1 subgroup).
       quotient_domain: The quotient domain (disjoint coset).

   Returns:
       Column evaluations on the quotient domain (natural order).


.. py:function:: extend_trace_to_quotient_domain(trace: list[list[primitives.field.Fe]], trace_domain: protocol.domain.TwoAdicMultiplicativeCoset, quotient_domain: protocol.domain.TwoAdicMultiplicativeCoset) -> list[list[primitives.field.Fe]]

   Extend the full trace matrix from trace domain to quotient domain.

   Input: trace as list of rows [[col0, col1, ...], ...] on the trace domain.
   Output: evaluations on the quotient domain, as list of rows.

   Each column is independently extended via coset LDE.

   Reference:
       stark-backend/src/prover/cpu/quotient/mod.rs single_rap_quotient_values
       (trace LDE is done externally but the math is the same)

   Args:
       trace: Trace matrix as list of rows, each row a list of base field elements.
       trace_domain: Trace domain (shift=1 subgroup).
       quotient_domain: Quotient domain (disjoint coset, larger).

   Returns:
       Trace evaluations on quotient domain, as list of rows.


.. py:class:: CosetSelectors

   Precomputed Lagrange selectors at every point of a coset.

   These are base-field-valued selectors for the trace domain evaluated at
   every point of the quotient domain (a coset).

   Reference:
       p3-commit-0.4.1/src/domain.rs selectors_on_coset (lines 252-292)


   .. py:attribute:: is_first_row
      :type:  list[primitives.field.Fe]


   .. py:attribute:: is_last_row
      :type:  list[primitives.field.Fe]


   .. py:attribute:: is_transition
      :type:  list[primitives.field.Fe]


   .. py:attribute:: inv_zeroifier
      :type:  list[primitives.field.Fe]


.. py:function:: selectors_on_coset(trace_domain: protocol.domain.TwoAdicMultiplicativeCoset, quotient_domain: protocol.domain.TwoAdicMultiplicativeCoset) -> CosetSelectors

   Compute Lagrange selectors of the trace domain at every quotient domain point.

   This is a batch computation that avoids per-point extension field arithmetic.
   All arithmetic is in the base field because quotient domain points are base
   field elements (the coset shift is a base field element).

   Algorithm from p3-commit selectors_on_coset:
   1. Compute Z_H(x) = (shift^n * rate_gen^i) - 1 for each i in [0, rate),
      where rate = quotient_size / trace_size. These values cycle with period
      `rate` over the quotient domain.
   2. For single_point_selector(j):
      - Compute denoms[i] = x_i - omega^j for each quotient domain point x_i
      - Batch invert the denoms
      - Multiply: selector[i] = Z_H(x_i) * denom_inv[i]
   3. is_first_row = single_point_selector(0)
      is_last_row = single_point_selector(n-1) where n = trace_size
   4. is_transition[i] = x_i - omega^{-1} (omega = trace generator)
   5. inv_zeroifier = batch_inverse(Z_H) cycled over quotient_size

   Reference:
       p3-commit-0.4.1/src/domain.rs selectors_on_coset (lines 252-292)

   Args:
       trace_domain: The trace domain (must have shift=1).
       quotient_domain: A coset disjoint from trace domain.

   Returns:
       CosetSelectors with base field values at each quotient domain point.


.. py:function:: eval_symbolic_expression_dag(dag: protocol.proof.SymbolicExpressionDag, local_row: list[primitives.field.Fe], next_row: list[primitives.field.Fe], public_values: list[primitives.field.Fe], is_first_row: primitives.field.Fe, is_last_row: primitives.field.Fe, is_transition: primitives.field.Fe) -> list[primitives.field.Fe]

   Evaluate all nodes of a symbolic expression DAG at a given row.

   Evaluates in topological order (nodes reference only earlier nodes).
   All arithmetic is in the base field.

   This is the simplified version for AIRs without preprocessed traces,
   permutation arguments, or challenges (e.g., FibonacciAir).

   Reference:
       stark-backend/src/prover/cpu/quotient/evaluator.rs
       ProverConstraintEvaluator::eval_nodes_mut (lines 175-217)

   Args:
       dag: The symbolic expression DAG.
       local_row: Trace values at the current row.
       next_row: Trace values at the next row.
       public_values: Public input values.
       is_first_row: Selector value for first row.
       is_last_row: Selector value for last row.
       is_transition: Selector value for transition.

   Returns:
       Evaluated values for every node in the DAG (base field).


.. py:function:: eval_symbolic_expression_dag_full(dag: protocol.proof.SymbolicExpressionDag, partitioned_local: list[list[primitives.field.Fe]], partitioned_next: list[list[primitives.field.Fe]], public_values: list[primitives.field.Fe], is_first_row: primitives.field.Fe, is_last_row: primitives.field.Fe, is_transition: primitives.field.Fe, preprocessed_local: list[primitives.field.Fe], preprocessed_next: list[primitives.field.Fe], after_challenge_local: list[list[int]] | None = None, after_challenge_next: list[list[int]] | None = None, challenges: list[list[list[int]]] | None = None, exposed_values: list[list[list[int]]] | None = None) -> list[primitives.field.FF4]

   Evaluate all DAG nodes for multi-AIR quotient computation, in FF4.

   Handles all entry types: PREPROCESSED, MAIN, PUBLIC, PERMUTATION,
   CHALLENGE, EXPOSED.  All arithmetic is in FF4 since PERMUTATION/CHALLENGE/
   EXPOSED variables are inherently extension field.

   For MAIN, partitioned_local[part_index][col_index] gives the base field
   value for the current row.  part_index selects the trace partition
   (cached mains first, then common main).

   For PERMUTATION, after_challenge_local[index] and after_challenge_next[index]
   are FF4 values indexed by extension field column number.

   Reference:
       stark-backend/src/prover/cpu/quotient/evaluator.rs
       ProverConstraintEvaluator (eval_var + eval_nodes_mut)


.. py:function:: accumulate_constraints_ef4(dag: protocol.proof.SymbolicExpressionDag, node_values: list[primitives.field.FF4], alpha: primitives.field.FF4) -> primitives.field.FF4

   Fold constraint values using powers of alpha (FF4 variant).

   Same as accumulate_constraints but node_values are already FF4.

   Reference:
       stark-backend/src/prover/cpu/quotient/evaluator.rs accumulate (lines 229-247)


.. py:function:: accumulate_constraints(dag: protocol.proof.SymbolicExpressionDag, node_values: list[primitives.field.Fe], alpha: primitives.field.FF4) -> primitives.field.FF4

   Fold constraint values using powers of alpha.

   Computes sum_{k} alpha^k * constraint_value[k], where the constraints
   are indexed by dag.constraint_idx in reverse order (highest power first)
   to match the Rust evaluator.

   Reference:
       stark-backend/src/prover/cpu/quotient/evaluator.rs
       ProverConstraintEvaluator::accumulate (lines 229-247)

   Args:
       dag: The symbolic expression DAG.
       node_values: Evaluated values for all DAG nodes (from eval_symbolic_expression_dag).
       alpha: The alpha challenge (extension field element).

   Returns:
       The folded constraint value (extension field element).


.. py:function:: compute_quotient_values(trace_on_quotient_domain: list[list[primitives.field.Fe]], constraints_dag: protocol.proof.SymbolicExpressionDag, public_values: list[primitives.field.Fe], alpha: primitives.field.FF4, quotient_domain: protocol.domain.TwoAdicMultiplicativeCoset, trace_domain: protocol.domain.TwoAdicMultiplicativeCoset, preprocessed_on_quot: list[list[primitives.field.Fe]] | None = None, after_challenge_on_quot: list[list[list[int]]] | None = None, challenges: list[list[list[int]]] | None = None, exposed_values: list[list[list[int]]] | None = None, partitioned_trace_on_quot: list[list[list[primitives.field.Fe]]] | None = None) -> list[list[int]]

   Compute quotient polynomial evaluations on the quotient domain.

   For each point x in the quotient domain:
   1. Look up precomputed selectors (is_first_row, is_last_row, is_transition).
   2. Get the local row and next row from the extended trace.
      The "next" row uses the trace-domain step size within the quotient domain:
      next[i] = trace[(i + quotient_size / trace_size) % quotient_size].
   3. Evaluate all symbolic constraints at (local, next, public_values, selectors).
   4. Fold constraints: accumulator = sum(alpha^k * C_k).
   5. Divide by vanishing polynomial: quotient[x] = accumulator * inv_zeroifier[x].

   When preprocessed/after_challenge/challenges/exposed_values are provided,
   uses the full FF4 evaluator to handle PERMUTATION/CHALLENGE/EXPOSED variables.
   Otherwise, uses the base field evaluator for simple AIRs.

   Reference:
       stark-backend/src/prover/cpu/quotient/single.rs
       compute_single_rap_quotient_values (lines 60-346)

   Args:
       trace_on_quotient_domain: Extended trace as list of rows on quotient domain.
       constraints_dag: Symbolic expression DAG for the AIR constraints.
       public_values: Public input values.
       alpha: Alpha challenge for constraint folding (extension field element).
       quotient_domain: The quotient domain (disjoint coset).
       trace_domain: The trace domain (shift=1 subgroup).
       preprocessed_on_quot: Preprocessed trace on quotient domain (optional).
       after_challenge_on_quot: After-challenge trace on quotient domain as
           [rows][perm_width] of FF4Coeffs (optional).
       challenges: Challenges per phase (optional).
       exposed_values: Exposed values per phase (optional).

   Returns:
       List of FF4 values, one per quotient domain point.


.. py:function:: quotient_values_to_chunks(quotient_values: list[list[int]], num_chunks: int) -> list[list[list[primitives.field.Fe]]]

   Split quotient evaluations into chunks and flatten to base field.

   The quotient evaluations on the quotient domain (a coset) are "vertically
   strided" into num_chunks sub-cosets.  Row i of the full quotient maps to
   chunk (i % num_chunks), row (i // num_chunks) of that chunk.

   Each chunk row has 4 base field elements (the FF4 coefficients), matching
   the Rust representation where extension field elements are transmuted to
   base field columns.

   Reference:
       p3-commit-0.4.1/src/domain.rs split_evals (lines 188-221)
       stark-backend/src/prover/cpu/quotient/single.rs lines 336-343

   The ordering follows the Rust pattern:
       quotient_values[chunk_idx + row_idx * quotient_degree]
   goes into chunk[chunk_idx][row_idx].

   Args:
       quotient_values: Flat list of FF4 quotient values on the quotient domain.
       num_chunks: Number of chunks (= quotient_degree, a power of 2).

   Returns:
       List of chunks. Each chunk is a list of rows, each row being
       [c0, c1, c2, c3] (the 4 base field coefficients of the FF4 element).


.. py:function:: extend_after_challenge_to_quotient_domain(after_challenge_trace: list[list[list[int]]], trace_domain: protocol.domain.TwoAdicMultiplicativeCoset, quotient_domain: protocol.domain.TwoAdicMultiplicativeCoset) -> list[list[list[int]]]

   Extend after-challenge trace (FF4 values) to the quotient domain.

   Each FF4 element has 4 base field coefficients. We LDE each coefficient
   column independently, then reconstruct FF4 values at quotient domain points.

   Args:
       after_challenge_trace: [rows][perm_width] of FF4Coeffs on trace domain.
       trace_domain: Trace domain (shift=1 subgroup).
       quotient_domain: Quotient domain (disjoint coset).

   Returns:
       [quot_rows][perm_width] of FF4Coeffs on quotient domain.


.. py:function:: compute_quotient_chunks(trace: list[list[primitives.field.Fe]], constraints_dag: protocol.proof.SymbolicExpressionDag, public_values: list[primitives.field.Fe], alpha: primitives.field.FF4, trace_domain: protocol.domain.TwoAdicMultiplicativeCoset, quotient_degree: int, preprocessed_trace: list[list[primitives.field.Fe]] | None = None, after_challenge_trace: list[list[list[int]]] | None = None, challenges: list[list[list[int]]] | None = None, exposed_values: list[list[list[int]]] | None = None, partitioned_traces: list[list[list[primitives.field.Fe]]] | None = None) -> tuple[list[list[list[primitives.field.Fe]]], list[protocol.domain.TwoAdicMultiplicativeCoset]]

   End-to-end quotient computation: trace -> quotient chunks ready for commitment.

   This combines:
   1. Create the quotient domain (disjoint from trace domain)
   2. LDE the trace to the quotient domain
   3. Compute quotient values on the quotient domain
   4. Split into chunks

   Reference:
       stark-backend/src/prover/cpu/quotient/mod.rs
       QuotientCommitter::single_rap_quotient_values (lines 84-130)

   Args:
       trace: Trace matrix as list of rows on the trace domain.
       constraints_dag: Symbolic expression DAG for the AIR constraints.
       public_values: Public input values.
       alpha: Alpha challenge (extension field element).
       trace_domain: Trace domain (shift=1 subgroup).
       quotient_degree: Factor multiplying trace degree to get quotient degree.
       preprocessed_trace: Preprocessed trace matrix [rows][cols] (optional).
       after_challenge_trace: After-challenge trace [rows][perm_width] of FF4 (optional).
       challenges: Challenges per phase (optional).
       exposed_values: Exposed values per phase (optional).

   Returns:
       Tuple of (chunks, chunk_domains) where:
       - chunks: list of num_chunks matrices, each a list of rows of 4 Fe elements.
       - chunk_domains: the sub-coset domains for each chunk.


