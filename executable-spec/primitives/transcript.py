"""Fiat-Shamir transcript using Poseidon2 duplex sponge.

Matches Plonky3's DuplexChallenger<BabyBear, Poseidon2<16>, 16, 8> bit-exactly.

Reference:
    p3-challenger-0.4.1/src/duplex_challenger.rs
"""

from primitives.field import FF4, Fe
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

    # <doc-anchor id="observe">
    def observe(self, value: Fe) -> None:
        """Absorb a single field element.

        Reference:
            duplex_challenger.rs lines 111-120
        """
        self.output_buffer.clear()
        self.input_buffer.append(value)
        if len(self.input_buffer) == RATE:
            self._duplexing()

    def observe_many(self, values) -> None:
        """Absorb multiple field elements or an FF4 scalar.

        Reference:
            duplex_challenger.rs lines 142-146
        """
        if isinstance(values, FF4):
            for i in range(4):
                self.observe(int(values._d[i]))
            return
        for v in values:
            self.observe(v)

    # <doc-anchor id="sample">
    def sample(self) -> Fe:
        """Squeeze one base field element (LIFO from output buffer).

        Reference:
            duplex_challenger.rs lines 172-184
        """
        if len(self.input_buffer) > 0 or len(self.output_buffer) == 0:
            self._duplexing()
        return self.output_buffer.pop()

    def sample_ext(self) -> FF4:
        """Squeeze one extension field element.

        Reference:
            duplex_challenger.rs (CanSample<EF>)
        """
        return FF4([self.sample() for _ in range(4)])

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

    def clone(self) -> "Challenger":
        """Deep copy this challenger's state.

        Used by the prover for proof-of-work grinding.

        Reference:
            duplex_challenger.rs Clone impl
        """
        return Challenger.from_state(
            self.sponge_state, self.input_buffer, self.output_buffer
        )


# <doc-anchor id="check-witness">
def check_witness(challenger: Challenger, bits: int, witness: int) -> bool:
    """Verify a proof-of-work witness against the Fiat-Shamir transcript.

    Observes the witness, samples `bits` bits, and checks == 0.

    Reference:
        p3-challenger GrindingChallenger::check_witness
    """
    if bits == 0:
        return True
    challenger.observe(witness)
    return challenger.sample_bits(bits) == 0


def grind(challenger: Challenger, bits: int) -> int:
    """Brute-force search for a proof-of-work witness.

    Tries witness = 0, 1, 2, ... until check_witness passes.
    Then calls check_witness on the original challenger to update its state.

    Args:
        challenger: The Fiat-Shamir challenger.
        bits: Number of bits for the PoW check.

    Returns:
        The winning witness value.

    Reference:
        p3-challenger grinding_challenger.rs GrindingChallenger::grind
    """
    if bits == 0:
        return 0

    for witness in range(2**31):
        test = challenger.clone()
        test.observe(witness)
        if test.sample_bits(bits) == 0:
            # Update original challenger state
            challenger.observe(witness)
            challenger.sample_bits(bits)
            return witness

    raise RuntimeError(f"failed to find PoW witness for {bits} bits")
