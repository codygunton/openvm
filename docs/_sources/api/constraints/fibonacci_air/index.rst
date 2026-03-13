constraints.fibonacci_air
=========================

.. py:module:: constraints.fibonacci_air

.. autoapi-nested-parse::

   Fibonacci AIR constraint definition.

   Translates the Rust AIR from:
     stark-backend/crates/stark-sdk/src/dummy_airs/fib_air/air.rs
     stark-backend/crates/stark-sdk/src/dummy_airs/fib_air/columns.rs

   The Fibonacci AIR has:
     - 2 trace columns: left (index 0), right (index 1)
     - 3 public values: [a, b, x] where a and b are initial values,
       x is the final Fibonacci result
     - 5 constraints enforced via selector multiplication

   Constraint derivation from the Rust source:
     builder.when_first_row().assert_eq(local.left, a)   =>  is_first_row * (local[0] - pv[0])
     builder.when_first_row().assert_eq(local.right, b)  =>  is_first_row * (local[1] - pv[1])
     builder.when_transition().assert_eq(local.right, next.left)
                                                           =>  is_transition * (next[0] - local[1])
     builder.when_transition().assert_eq(local.left + local.right, next.right)
                                                           =>  is_transition * (next[1] - local[0] - local[1])
     builder.when_last_row().assert_eq(local.right, x)    =>  is_last_row * (local[1] - pv[2])



Classes
-------

.. autoapisummary::

   constraints.fibonacci_air.FibonacciAir


Module Contents
---------------

.. py:class:: FibonacciAir

   Fibonacci AIR definition.

   Corresponds to FibonacciAir in fib_air/air.rs.
   Width and num_public_values match NUM_FIBONACCI_COLS and
   BaseAirWithPublicValues::num_public_values respectively.


   .. py:attribute:: width
      :type:  int
      :value: 2



   .. py:attribute:: num_public_values
      :type:  int
      :value: 3



   .. py:method:: eval_constraints(local: list, next_row: list, public_values: list, is_first_row, is_last_row, is_transition) -> list

      Evaluate all AIR constraints.

      This method is generic over the element type: it works with plain
      ints (for base-field prover evaluation on the trace domain) or with
      extension-field elements (for verifier evaluation at the OOD point).
      The caller is responsible for providing consistent types.

      Corresponds to Air::eval in fib_air/air.rs.

      Args:
          local: Current row values [left, right].
          next_row: Next row values [left, right].
          public_values: [a, b, x] where a,b are initial values, x is result.
          is_first_row: Selector that is 1 on the first row, 0 elsewhere.
          is_last_row: Selector that is 1 on the last row, 0 elsewhere.
          is_transition: Selector that is 1 on all rows except the last, 0 on last.

      Returns:
          List of 5 constraint values. All should be zero on a valid trace.



   .. py:method:: quotient_degree() -> int

      Maximum constraint degree minus 1.

      Each constraint is the product of a degree-1 selector (is_first_row,
      is_transition, is_last_row) and a degree-1 expression over trace
      columns, giving max constraint degree = 2. The quotient polynomial
      degree factor is max_constraint_degree - 1 = 1, but the log quotient
      degree used for FRI decomposition is ceil(log2(max_constraint_degree))
      which is 1 as well.

      However, per the Plonky3 convention used in the prover, the value
      returned here is max_constraint_degree - 1 = 2 when accounting for
      the trace polynomial degree (each column is degree N-1 over a domain
      of size N, so the constraint polynomial can be degree up to
      2*(N-1) + (N-1) = 3*(N-1), and the quotient after dividing by the
      vanishing polynomial of degree N has degree 2*(N-1)).

      Returns:
          2



