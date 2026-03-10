"""Fiat-Shamir transcript using Poseidon2 duplex sponge.

Matches Plonky3's DuplexChallenger<BabyBear, Poseidon2<16>, 16, 8> bit-exactly.

Reference:
    p3-challenger-0.4.1/src/duplex_challenger.rs
"""

from primitives.poseidon2 import permute

WIDTH = 16
RATE = 8


class Challenger:
    """Fiat-Shamir challenger using Poseidon2 duplex sponge.

    Matches Plonky3's DuplexChallenger<BabyBear, Poseidon2<16>, 16, 8>.

    Reference:
        p3-challenger-0.4.1/src/duplex_challenger.rs
    """

    def __init__(self):
        self.sponge_state = [0] * WIDTH
        self.input_buffer = []
        self.output_buffer = []

    @classmethod
    def from_state(
        cls,
        sponge_state: list[int],
        input_buffer: list[int],
        output_buffer: list[int],
    ) -> "Challenger":
        """Create a Challenger seeded with exported internal state.

        Used to resume a transcript from a checkpoint captured in Rust.
        """
        c = cls()
        c.sponge_state = list(sponge_state)
        c.input_buffer = list(input_buffer)
        c.output_buffer = list(output_buffer)
        return c

    def observe(self, value: int) -> None:
        """Absorb a single field element.

        Clears output buffer (invalidating pending samples), pushes to input
        buffer, and triggers duplexing when the buffer is full.

        Reference:
            duplex_challenger.rs lines 111-120
        """
        self.output_buffer.clear()
        self.input_buffer.append(value)
        if len(self.input_buffer) == RATE:
            self._duplexing()

    def observe_many(self, values: list[int]) -> None:
        """Absorb multiple field elements sequentially.

        Reference:
            duplex_challenger.rs CanObserve<Hash<F,F,N>> lines 142-146
        """
        for v in values:
            self.observe(v)

    def sample(self) -> int:
        """Squeeze one base field element.

        Pops from the BACK of the output buffer (LIFO order).
        Triggers duplexing if there are pending inputs or the output buffer
        is empty.

        Reference:
            duplex_challenger.rs lines 172-184
        """
        if len(self.input_buffer) > 0 or len(self.output_buffer) == 0:
            self._duplexing()
        return self.output_buffer.pop()

    def sample_ext(self) -> list[int]:
        """Squeeze one extension field element (4 base elements).

        Samples 4 base field elements via from_basis_coefficients_fn,
        returning ascending coefficient order [c0, c1, c2, c3].

        Reference:
            duplex_challenger.rs lines 172-184 (CanSample<EF>)
        """
        return [self.sample() for _ in range(4)]

    def sample_bits(self, bits: int) -> int:
        """Sample a random index with the given number of bits.

        Samples one base field element, converts to integer, and masks
        to the requested number of bits.

        Reference:
            duplex_challenger.rs lines 201-207 (CanSampleBits)
        """
        val = self.sample()
        return val & ((1 << bits) - 1)

    def _duplexing(self) -> None:
        """Apply duplex sponge: overwrite rate portion, permute, extract output.

        IMPORTANT: Does NOT zero-pad unused rate positions. Only the first
        len(input_buffer) positions are overwritten; the rest of the sponge
        state retains its previous values.

        After permutation, the output buffer receives the first RATE elements
        of the sponge state.

        Reference:
            duplex_challenger.rs lines 81-94
        """
        # Overwrite only the positions that have input (no zero-padding)
        for i, val in enumerate(self.input_buffer):
            self.sponge_state[i] = val
        self.input_buffer.clear()

        self.sponge_state = permute(self.sponge_state)

        self.output_buffer = list(self.sponge_state[:RATE])
