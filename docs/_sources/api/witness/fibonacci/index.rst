witness.fibonacci
=================

.. py:module:: witness.fibonacci

.. autoapi-nested-parse::

   Fibonacci trace generation.

   Translates the Rust trace generator from:
     stark-backend/crates/stark-sdk/src/dummy_airs/fib_air/trace.rs
     stark-backend/crates/stark-sdk/src/dummy_airs/fib_air/chip.rs

   The trace has 2 columns (left, right) and num_rows rows.
   Row 0: [a, b]
   Row i: [F(i), F(i+1)] where F is the Fibonacci recurrence.



Attributes
----------

.. autoapisummary::

   witness.fibonacci.P


Functions
---------

.. autoapisummary::

   witness.fibonacci.generate_fibonacci_trace
   witness.fibonacci.fibonacci_public_values


Module Contents
---------------

.. py:data:: P
   :value: 2013265921


.. py:function:: generate_fibonacci_trace(a: int, b: int, num_rows: int) -> list[list[primitives.field.Fe]]

   Generate Fibonacci trace matrix.

   Corresponds to fib_air/trace.rs::generate_trace_rows.

   Row 0:     [a, b]
   Row i > 0: [row_{i-1}[1], row_{i-1}[0] + row_{i-1}[1]]

   All values are reduced mod BABYBEAR_PRIME.

   Args:
       a: Initial left value (0th Fibonacci number).
       b: Initial right value (1st Fibonacci number).
       num_rows: Number of rows in the trace. Must be a power of 2.

   Returns:
       List of rows, each row is [left, right].


.. py:function:: fibonacci_public_values(a: int, b: int, num_rows: int) -> list[primitives.field.Fe]

   Return public values [a, b, F(num_rows)].

   Corresponds to FibonacciChip::generate_proving_ctx in chip.rs,
   which extracts [trace[0][0], trace[0][1], trace[n-1][1]].

   The third public value is the (num_rows)-th Fibonacci number,
   i.e., trace[num_rows - 1][1] = F(num_rows).

   Args:
       a: Initial left value (0th Fibonacci number).
       b: Initial right value (1st Fibonacci number).
       num_rows: Number of rows in the trace. Must be a power of 2.

   Returns:
       [a % P, b % P, F(num_rows) % P] where F(num_rows) is the
       last row's right column value.


