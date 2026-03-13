primitives.transcript
=====================

.. py:module:: primitives.transcript

.. autoapi-nested-parse::

   Fiat-Shamir transcript using Poseidon2 duplex sponge.

   Matches Plonky3's DuplexChallenger<BabyBear, Poseidon2<16>, 16, 8> bit-exactly.

   Reference:
       p3-challenger-0.4.1/src/duplex_challenger.rs



Attributes
----------

.. autoapisummary::

   primitives.transcript.WIDTH
   primitives.transcript.RATE


Classes
-------

.. autoapisummary::

   primitives.transcript.Challenger


Functions
---------

.. autoapisummary::

   primitives.transcript.check_witness
   primitives.transcript.grind


Module Contents
---------------

.. py:data:: WIDTH
   :value: 16


.. py:data:: RATE
   :value: 8


.. py:class:: Challenger

   Fiat-Shamir challenger using Poseidon2 duplex sponge.

   Reference:
       p3-challenger-0.4.1/src/duplex_challenger.rs


   .. py:attribute:: sponge_state
      :type:  list[primitives.field.Fe]
      :value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]



   .. py:attribute:: input_buffer
      :type:  list[primitives.field.Fe]
      :value: []



   .. py:attribute:: output_buffer
      :type:  list[primitives.field.Fe]
      :value: []



   .. py:method:: from_state(sponge_state: list[primitives.field.Fe], input_buffer: list[primitives.field.Fe], output_buffer: list[primitives.field.Fe]) -> Challenger
      :classmethod:


      Create Challenger from exported internal state (for transcript replay).



   .. py:method:: observe(value: primitives.field.Fe) -> None

      Absorb a single field element.

      Reference:
          duplex_challenger.rs lines 111-120



   .. py:method:: observe_many(values) -> None

      Absorb multiple field elements or an FF4 scalar.

      Reference:
          duplex_challenger.rs lines 142-146



   .. py:method:: sample() -> primitives.field.Fe

      Squeeze one base field element (LIFO from output buffer).

      Reference:
          duplex_challenger.rs lines 172-184



   .. py:method:: sample_ext() -> primitives.field.FF4

      Squeeze one extension field element.

      Reference:
          duplex_challenger.rs (CanSample<EF>)



   .. py:method:: sample_bits(bits: int) -> int

      Sample a random index with the given number of bits.

      Reference:
          duplex_challenger.rs lines 201-207 (CanSampleBits)



   .. py:method:: clone() -> Challenger

      Deep copy this challenger's state.

      Used by the prover for proof-of-work grinding.

      Reference:
          duplex_challenger.rs Clone impl



.. py:function:: check_witness(challenger: Challenger, bits: int, witness: int) -> bool

   Verify a proof-of-work witness against the Fiat-Shamir transcript.

   Observes the witness, samples `bits` bits, and checks == 0.

   Reference:
       p3-challenger GrindingChallenger::check_witness


.. py:function:: grind(challenger: Challenger, bits: int) -> int

   Brute-force search for a proof-of-work witness.

   Tries witness = 0, 1, 2, ... until check_witness passes.
   Then calls check_witness on the original challenger to update its state.

   Args:
       challenger: The Fiat-Shamir challenger.
       bits: Number of bits for the PoW check.

   Returns:
       The winning witness value.

   Reference:
       p3-challenger grinding_challenger.rs GrindingChallenger::grind


