"""Fiat-Shamir transcript using Poseidon2 duplex sponge.

Matches Plonky3's DuplexChallenger<BabyBear, Poseidon2<16>, 16, 8> bit-exactly.

Reference:
    p3-challenger-0.4.1/src/duplex_challenger.rs
"""

from primitives.field import EF4Coeffs, Fe
from primitives.poseidon2 import permute

WIDTH = 16
RATE = 8


class Challenger:
    """Fiat-Shamir challenger using Poseidon2 duplex sponge.

    Reference:
        p3-challenger-0.4.1/src/duplex_challenger.rs
    """

    def __init__(self) -> None:
        self.sponge_state: list[Fe] = [0] * WIDTH
        self.input_buffer: list[Fe] = []
        self.output_buffer: list[Fe] = []

    @classmethod
    def from_state(
        cls,
        sponge_state: list[Fe],
        input_buffer: list[Fe],
        output_buffer: list[Fe],
    ) -> "Challenger":
        """Create Challenger from exported internal state (for transcript replay)."""
        c = cls()
        c.sponge_state = list(sponge_state)
        c.input_buffer = list(input_buffer)
        c.output_buffer = list(output_buffer)
        return c

    def observe(self, value: Fe) -> None:
        """Absorb a single field element.

        Reference:
            duplex_challenger.rs lines 111-120
        """
        self.output_buffer.clear()
        self.input_buffer.append(value)
        if len(self.input_buffer) == RATE:
            self._duplexing()

    def observe_many(self, values: list[Fe]) -> None:
        """Absorb multiple field elements.

        Reference:
            duplex_challenger.rs lines 142-146
        """
        for v in values:
            self.observe(v)

    def sample(self) -> Fe:
        """Squeeze one base field element (LIFO from output buffer).

        Reference:
            duplex_challenger.rs lines 172-184
        """
        if len(self.input_buffer) > 0 or len(self.output_buffer) == 0:
            self._duplexing()
        return self.output_buffer.pop()

    def sample_ext(self) -> EF4Coeffs:
        """Squeeze one extension field element [c0, c1, c2, c3].

        Reference:
            duplex_challenger.rs (CanSample<EF>)
        """
        return [self.sample() for _ in range(4)]

    def sample_bits(self, bits: int) -> int:
        """Sample a random index with the given number of bits.

        Reference:
            duplex_challenger.rs lines 201-207 (CanSampleBits)
        """
        val = self.sample()
        return val & ((1 << bits) - 1)

    def _duplexing(self) -> None:
        """Apply duplex sponge: overwrite rate portion, permute, extract output.

        IMPORTANT: Does NOT zero-pad unused rate positions.

        Reference:
            duplex_challenger.rs lines 81-94
        """
        # Overwrite only the positions that have input (no zero-padding)
        for i, val in enumerate(self.input_buffer):
            self.sponge_state[i] = val
        self.input_buffer.clear()

        self.sponge_state = permute(self.sponge_state)

        self.output_buffer = list(self.sponge_state[:RATE])
