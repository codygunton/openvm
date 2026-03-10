# Milestone 1: Standalone FRI Prover/Verifier — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make all primitive tests (field, poseidon2, ntt, merkle) and FRI tests pass against golden vectors.

**Architecture:** Bottom-up translation: BabyBear field → Poseidon2 hash → Merkle tree → NTT → Fiat-Shamir transcript → FRI verify. Primitives reuse pil2-proofman patterns (adapted for BabyBear). FRI protocol is freshly translated from p3-fri v0.4.1.

**Tech Stack:** Python 3.10+, galois library (field arithmetic + NTT), numpy, pytest

---

## Reference Files

### Translation Sources (Rust)
- **p3-fri v0.4.1**: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/p3-fri-0.4.1/src/`
- **Plonky3 local**: `/home/cody/plonky3/` (for Poseidon2 constants, BabyBear params)
- **p3-baby-bear**: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/p3-baby-bear-0.4.1/`

### Pattern Reference (Python)
- **pil2-proofman**: `/home/cody/pil2-proofman/executable-spec/primitives/` (field.py, ntt.py, transcript.py, merkle_tree.py)

### Test Vectors
- `/home/cody/openvm/executable-spec/tests/test-data/primitives/*.json`
- `/home/cody/openvm/executable-spec/tests/test-data/fri/*.json`

### Style Guide
- `/home/cody/openvm/SPEC_STYLE_GUIDE.md`

---

## Key Constants

```
BabyBear prime:           p = 2013265921 = 2^31 - 2^27 + 1
Two-adicity:              27 (p - 1 = 2^27 * 15)
Multiplicative generator: 31 (Val::GENERATOR in Plonky3)
Quartic extension W:      11 (irreducible poly: x^4 - 11)
Poseidon2 width:          16
Poseidon2 rate:           8
Poseidon2 full rounds:    8 (4 initial + 4 terminal)
Poseidon2 partial rounds: 13
Poseidon2 S-box:          x^7
Digest size:              8 BabyBear elements
```

---

## Implementation Tasks

### Visual Dependency Tree

```
executable-spec/
├── pyproject.toml                    (Task 1: add galois dependency)
├── primitives/
│   ├── field.py                      (Task 2: BabyBear base + quartic extension)
│   ├── poseidon2.py                  (Task 4: permutation + compress)
│   ├── merkle.py                     (Task 5: tree build + verify)
│   ├── ntt.py                        (Task 6: forward/inverse NTT)
│   └── transcript.py                 (Task 7: DuplexChallenger Fiat-Shamir)
├── protocol/
│   └── fri.py                        (Task 8: fold + verify)
└── tests/
    ├── test_field.py                 (Task 3: wire test stubs to field.py)
    ├── test_poseidon2.py             (Task 4: wire test stubs)
    ├── test_merkle.py                (Task 5: wire test stubs)
    ├── test_ntt.py                   (Task 6: wire test stubs)
    └── test_fri.py                   (Task 8: wire test stubs)
```

### Execution Plan

#### Group A: Foundation (parallel)

- [ ] **Task 1**: Add galois dependency to pyproject.toml
- [ ] **Task 2**: Implement BabyBear base field + quartic extension

#### Group B: Field Tests (after Group A)

- [ ] **Task 3**: Wire test_field.py to field.py and verify passing

#### Group C: Hash Primitive (after Group A)

- [ ] **Task 4**: Implement Poseidon2 + wire test_poseidon2.py

#### Group D: Tree + NTT (after Group C for merkle, after Group A for NTT — parallel with each other)

- [ ] **Task 5**: Implement Merkle tree + wire test_merkle.py
- [ ] **Task 6**: Implement NTT + wire test_ntt.py

#### Group E: Transcript (after Group C)

- [ ] **Task 7**: Implement Fiat-Shamir transcript (DuplexChallenger)

#### Group F: FRI Protocol (after all above)

- [ ] **Task 8**: Implement FRI fold + verify + wire test_fri.py

---

### Task 1: Add galois dependency

**Files:**
- Modify: `executable-spec/pyproject.toml`

**Step 1: Add galois to dependencies**

```toml
[project]
name = "executable-spec"
version = "0.1.0"
requires-python = ">=3.10"
dependencies = [
    "galois>=0.4.1",
    "numpy>=1.24.0",
    "pytest>=7.0.0",
    "pytest-xdist>=3.0.0",
]
```

**Step 2: Install**

Run: `cd /home/cody/openvm/executable-spec && pip install -e .`
Expected: galois installed successfully

**Step 3: Verify galois works with BabyBear**

Run: `python3 -c "import galois; FF = galois.GF(2013265921); print(FF(5) + FF(10))"`
Expected: `15`

**Step 4: Commit**

```bash
git add executable-spec/pyproject.toml
git commit -m "add galois dependency for field arithmetic"
```

---

### Task 2: Implement BabyBear field + quartic extension

**Files:**
- Create: `executable-spec/primitives/field.py`

**Pattern reference:** `/home/cody/pil2-proofman/executable-spec/primitives/field.py`

**Step 1: Implement field.py**

The file must provide:

1. **BabyBear base field** using `galois.GF(2013265921)`:
   - `BABYBEAR_PRIME = 2013265921`
   - `FF = galois.GF(BABYBEAR_PRIME)` — base field type
   - `FIELD_EXTENSION_DEGREE = 4`

2. **Quartic extension field** using `galois.GF(p^4)`:
   - Irreducible polynomial: `x^4 - 11` over BabyBear
   - In galois descending coefficient format: `Poly([1, 0, 0, 0, p-11], field=FF)`
   - Cache the FF4 construction to avoid slow re-initialization (see pil2-proofman pattern with pickle)

3. **Type aliases** (following SPEC_STYLE_GUIDE):
   - `FF4 = <quartic extension type>` — extension field
   - `FF4Poly = FF4` — polynomial over extension
   - `FFPoly = FF` — polynomial over base
   - `HashOutput = list[int]` — 8-element Poseidon2 digest

4. **Conversion helpers** (following pil2-proofman pattern):
   - `ff4_coeffs(elem) -> list[int]` — ascending order [a0, a1, a2, a3]
   - `ff4(coeffs: list[int]) -> FF4` — construct from ascending [a0, a1, a2, a3]
   - `ff4_array(c0, c1, c2, c3) -> FF4` — construct array from parallel coefficient lists
   - `ff4_from_base(val: int) -> FF4` — embed base field element
   - `ff4_from_json(json_arr) -> FF4` — parse `[[c0,c1,c2,c3],...]`
   - `ff4_to_json(arr) -> list` — convert to JSON format

5. **NTT support** (roots of unity for BabyBear):
   - `GENERATOR = 31` — multiplicative generator (Val::GENERATOR in Plonky3)
   - Precomputed roots: `W[n]` = primitive 2^n-th root of unity
   - `get_omega(n_bits) -> int`
   - `get_omega_inv(n_bits) -> int`
   - Note: Extract roots from Plonky3. BabyBear's TWO_ADICITY = 27.
     The 2^27-th root of unity can be computed as `GENERATOR^((p-1)/2^27) = 31^15`.
     Then `W[k]` for k < 27 is `W[27]^(2^(27-k))`.

6. **Batch inverse** (copy from pil2-proofman — Montgomery batch inversion)

**Key difference from pil2-proofman:**
- Goldilocks → BabyBear (different prime, smaller)
- Cubic (degree 3) → Quartic (degree 4): all `ff3_*` functions become `ff4_*`
- Galois descending order is `[a3, a2, a1, a0]` — our ascending is `[a0, a1, a2, a3]`
- The irreducible poly for quartic is `x^4 - 11` (not `x^3 - x - 1`)

**Finding roots of unity:**
Read `/home/cody/plonky3/baby-bear/src/baby_bear.rs` for the `TWO_ADIC_GENERATORS` table,
or compute them:
```python
# The smallest 2-adic generator (2^27-th root of unity)
# In Plonky3: BabyBear::TWO_ADIC_GENERATORS[27]
# Compute: find g such that g^(2^27) = 1 and g^(2^26) != 1
```

**Step 2: Run field tests to verify**

Run: `cd /home/cody/openvm/executable-spec && python -m pytest tests/test_field.py -v`
Expected: All 8 tests should fail (ImportError or assertion — tests still have `assert False`)

**Step 3: Commit**

```bash
git add executable-spec/primitives/field.py
git commit -m "implement BabyBear base field and quartic extension"
```

---

### Task 3: Wire test_field.py to field.py

**Files:**
- Modify: `executable-spec/tests/test_field.py`

**Step 1: Update test_field.py to import and use field.py**

Replace `assert False` stubs with actual assertions. The tests load vectors from
`test-data/primitives/babybear_field.json` and `babybear_ext_field.json`.

**Vector formats:**
```json
// babybear_field.json
{"addition": [{"a": 5, "b": 10, "expected": 15}, ...]}
{"multiplication": [{"a": 5, "b": 10, "expected": 50}, ...]}
{"inverse": [{"a": 1, "expected": 1}, ...]}
{"subtraction": [{"a": 5, "b": 3, "expected": 2}, ...]}
{"negation": [{"a": 5, "expected": 2013265916}, ...]}

// babybear_ext_field.json
{"addition": [{"a": {"coeffs": [1,2,3,4]}, "b": {"coeffs": [5,6,7,8]}, "expected": {"coeffs": [6,8,10,12]}}, ...]}
```

**Test implementation pattern:**
```python
from primitives.field import FF, FF4, ff4

class TestBabyBearField:
    def test_addition(self, vectors):
        for case in vectors["addition"]:
            a, b, expected = FF(case["a"]), FF(case["b"]), FF(case["expected"])
            assert a + b == expected

    def test_multiplication(self, vectors):
        for case in vectors["multiplication"]:
            a, b, expected = FF(case["a"]), FF(case["b"]), FF(case["expected"])
            assert a * b == expected

    def test_inverse(self, vectors):
        for case in vectors["inverse"]:
            a, expected = FF(case["a"]), FF(case["expected"])
            assert a ** -1 == expected

    def test_subtraction(self, vectors):
        for case in vectors["subtraction"]:
            a, b, expected = FF(case["a"]), FF(case["b"]), FF(case["expected"])
            assert a - b == expected

    def test_negation(self, vectors):
        for case in vectors["negation"]:
            a, expected = FF(case["a"]), FF(case["expected"])
            assert -a == expected

class TestBabyBearExtensionField:
    def test_ext_addition(self, vectors):
        for case in vectors["addition"]:
            a = ff4(case["a"]["coeffs"])
            b = ff4(case["b"]["coeffs"])
            expected = ff4(case["expected"]["coeffs"])
            assert a + b == expected

    def test_ext_multiplication(self, vectors):
        for case in vectors["multiplication"]:
            a = ff4(case["a"]["coeffs"])
            b = ff4(case["b"]["coeffs"])
            expected = ff4(case["expected"]["coeffs"])
            assert a * b == expected

    def test_ext_inverse(self, vectors):
        for case in vectors["inverse"]:
            a = ff4(case["a"]["coeffs"])
            expected = ff4(case["expected"]["coeffs"])
            assert a ** -1 == expected
```

**Step 2: Run tests**

Run: `cd /home/cody/openvm/executable-spec && python -m pytest tests/test_field.py -v`
Expected: All 8 tests PASS

**Step 3: Commit**

```bash
git add executable-spec/tests/test_field.py
git commit -m "wire field tests to BabyBear implementation, all passing"
```

---

### Task 4: Implement Poseidon2 + wire tests

**Files:**
- Create: `executable-spec/primitives/poseidon2.py`
- Modify: `executable-spec/tests/test_poseidon2.py`

**Translation source:**
- Plonky3 Poseidon2: `/home/cody/plonky3/poseidon2/src/` (implementation)
- BabyBear-specific: `/home/cody/plonky3/baby-bear/src/` (internal linear layer constants)
- Round constants: search Plonky3 for `RC16` or `round_constants` for BabyBear width-16

**Algorithm — Poseidon2 permutation (width=16, BabyBear):**

```
Parameters:
  WIDTH = 16
  ROUNDS_F = 8 (4 initial full + 4 terminal full)
  ROUNDS_P = 13 (partial/internal)
  SBOX_DEGREE = 7
  M_4 = [[2,3,1,1],[1,2,3,1],[1,1,2,3],[3,1,1,2]]

permute(state: list[int]) -> list[int]:
    # Initial external linear layer
    state = external_linear_layer(state)

    # 4 initial full rounds
    for r in range(4):
        state = add_round_constants(state, RC16[r])
        state = [sbox(x) for x in state]  # x^7 mod p
        state = external_linear_layer(state)

    # 13 partial rounds
    for r in range(13):
        state[0] = (state[0] + RC16[4 + r][0]) % P
        state[0] = sbox(state[0])
        state = internal_linear_layer(state)

    # 4 terminal full rounds
    for r in range(4):
        state = add_round_constants(state, RC16[17 + r])
        state = [sbox(x) for x in state]
        state = external_linear_layer(state)

    return state
```

**S-box:** `sbox(x) = x^7 mod p = x * x * x * x * x * x * x mod p`
(compute as `x^2 * x^2 * x^2 * x` with intermediate reductions)

**External linear layer (MDS light permutation for width-16):**
Apply 4x4 circulant MDS matrix `M_4 = circ(2,3,1,1)` to each of 4 chunks of 4 elements,
then add element-wise sums across chunks. Reference: `mds_light_permutation` in Plonky3.

**Internal linear layer:**
Apply `(1 + diag(V))` matrix where `V` contains specific field elements.
Reference: `INTERNAL_DIAG_MONTY_16` in Plonky3 `baby-bear/src/`.
These constants need to be extracted from the Rust source — search for `INTERNAL_DIAG_MONTY` or `InternalLayerParameters`.

**Round constants:**
Must extract from Plonky3 source. The `BabyBearPoseidon2Engine::new()` in openvm-stark-sdk
constructs Poseidon2 with specific constants. Trace the code to find the exact values.
File to check: `/home/cody/plonky3/poseidon2/src/babybear.rs` or similar.

**Compression function:**
```python
def compress(left: list[int], right: list[int]) -> list[int]:
    """Compress two 8-element digests into one 8-element digest."""
    state = left + right  # Concatenate to width-16
    state = permute(state)
    return state[:8]  # Truncate to first 8
```

**Hashing (PaddingFreeSponge):**
```python
def hash_to_digest(inputs: list[int]) -> list[int]:
    """Hash variable-length input to 8-element digest."""
    state = [0] * 16
    i = 0
    while i < len(inputs):
        # Fill rate portion (first 8 elements)
        for j in range(8):
            if i < len(inputs):
                state[j] = inputs[i]
                i += 1
            else:
                break
        state = permute(state)
    return state[:8]
```

**Test wiring (test_poseidon2.py):**
```python
from primitives.poseidon2 import permute, compress

class TestPoseidon2:
    def test_permutation(self, vectors):
        for case in vectors["permutation"]:
            result = permute(case["input"])
            assert result == case["expected"]

    def test_compress(self, vectors):
        for case in vectors["compress"]:
            result = compress(case["left"], case["right"])
            assert result == case["expected"]
```

**Step: Run tests**

Run: `cd /home/cody/openvm/executable-spec && python -m pytest tests/test_poseidon2.py -v`
Expected: All 2 tests PASS

**Commit:**
```bash
git add executable-spec/primitives/poseidon2.py executable-spec/tests/test_poseidon2.py
git commit -m "implement Poseidon2 permutation and compression, tests passing"
```

---

### Task 5: Implement Merkle tree + wire tests

**Files:**
- Create: `executable-spec/primitives/merkle.py`
- Modify: `executable-spec/tests/test_merkle.py`

**Pattern reference:** `/home/cody/pil2-proofman/executable-spec/primitives/merkle_tree.py`

**Algorithm:**

Plonky3's Merkle tree uses:
- **Leaf hash**: `PaddingFreeSponge` — hash each leaf (8 u32 values) through Poseidon2 sponge → 8-element digest
- **Node compress**: `TruncatedPermutation` — `compress(left_digest, right_digest)` → 8-element digest
- **Opening proof**: List of sibling digests from leaf to root

```python
def build_merkle_tree(leaves: list[list[int]]) -> tuple[list[int], list]:
    """Build tree, return (root, tree_data)."""
    # Hash each leaf to digest
    digests = [hash_to_digest(leaf) for leaf in leaves]
    # Build tree bottom-up
    tree = [digests]
    while len(digests) > 1:
        next_level = []
        for i in range(0, len(digests), 2):
            next_level.append(compress(digests[i], digests[i+1]))
        digests = next_level
        tree.append(digests)
    root = digests[0]
    return root, tree

def get_opening_proof(tree, leaf_index: int) -> list[list[int]]:
    """Get Merkle proof (sibling digests from leaf to root)."""
    proof = []
    idx = leaf_index
    for level in tree[:-1]:
        sibling_idx = idx ^ 1
        proof.append(level[sibling_idx])
        idx >>= 1
    return proof

def verify_opening(root, leaf, leaf_index, proof) -> bool:
    """Verify a Merkle opening proof."""
    current = hash_to_digest(leaf)
    idx = leaf_index
    for sibling in proof:
        if idx % 2 == 0:
            current = compress(current, sibling)
        else:
            current = compress(sibling, current)
        idx >>= 1
    return current == root
```

**IMPORTANT:** Check exactly how Plonky3's `FieldMerkleTreeMmcs` handles leaf hashing.
The leaves in the test vectors are 8-element arrays — check whether they're hashed through
the sponge or treated as pre-hashed digests. Read the test vector generator:
`/home/cody/openvm/crates/test-vectors/src/lib.rs` for the Merkle test generation code.

**Test wiring:**
```python
from primitives.merkle import build_merkle_tree, verify_opening

class TestMerkleTree:
    def test_tree_construction(self, vectors):
        for case in vectors["construction"]:
            root, _ = build_merkle_tree(case["leaves"])
            assert root == case["expected_root"]

    def test_opening_verification(self, vectors):
        for case in vectors["openings"]:
            assert verify_opening(
                case["root"], case["leaf"], case["leaf_index"], case["proof"]
            )
```

**Run tests:**
Run: `cd /home/cody/openvm/executable-spec && python -m pytest tests/test_merkle.py -v`
Expected: All 2 tests PASS

**Commit:**
```bash
git add executable-spec/primitives/merkle.py executable-spec/tests/test_merkle.py
git commit -m "implement Merkle tree with Poseidon2, tests passing"
```

---

### Task 6: Implement NTT + wire tests

**Files:**
- Create: `executable-spec/primitives/ntt.py`
- Modify: `executable-spec/tests/test_ntt.py`

**Pattern reference:** `/home/cody/pil2-proofman/executable-spec/primitives/ntt.py`

**Algorithm:**

Use `galois.ntt()` and `galois.intt()` with BabyBear roots of unity. This is the same
approach as pil2-proofman.

```python
import galois
from primitives.field import FF, get_omega, get_omega_inv

def ntt(coeffs: list[int]) -> list[int]:
    """Forward NTT: coefficients → evaluations on subgroup."""
    n = len(coeffs)
    n_bits = (n - 1).bit_length()
    omega = get_omega(n_bits)
    ff_coeffs = FF(coeffs)
    result = galois.ntt(ff_coeffs, omega=omega)
    return [int(x) for x in result]

def intt(evals: list[int]) -> list[int]:
    """Inverse NTT: evaluations → coefficients."""
    n = len(evals)
    n_bits = (n - 1).bit_length()
    omega_inv = get_omega_inv(n_bits)
    ff_evals = FF(evals)
    result = galois.intt(ff_evals, omega=omega_inv)
    return [int(x) for x in result]
```

**IMPORTANT:** Verify that `galois.ntt` uses the same convention as Plonky3.
Plonky3's forward DFT evaluates at `[1, omega, omega^2, ..., omega^(n-1)]`.
galois.ntt should do the same. If the test vectors don't match, check if bit-reversal
is needed.

**Test wiring:**
```python
from primitives.ntt import ntt, intt

class TestNTT:
    def test_forward_ntt(self, vectors):
        for case in vectors["forward"]:
            result = ntt(case["input"])
            assert result == case["expected"]

    def test_inverse_ntt(self, vectors):
        for case in vectors["inverse"]:
            result = intt(case["input"])
            assert result == case["expected"]

    def test_ntt_round_trip(self, vectors):
        for case in vectors["forward"]:
            coeffs = case["input"]
            assert intt(ntt(coeffs)) == coeffs
```

**Run tests:**
Run: `cd /home/cody/openvm/executable-spec && python -m pytest tests/test_ntt.py -v`
Expected: All 3 tests PASS

**Commit:**
```bash
git add executable-spec/primitives/ntt.py executable-spec/tests/test_ntt.py
git commit -m "implement NTT/INTT over BabyBear, tests passing"
```

---

### Task 7: Implement Fiat-Shamir transcript

**Files:**
- Create: `executable-spec/primitives/transcript.py`

**Translation source:** Plonky3's `DuplexChallenger` from `p3-challenger` crate.
**Pattern reference:** `/home/cody/pil2-proofman/executable-spec/primitives/transcript.py`

**CRITICAL:** The transcript must be bit-exact with Plonky3's DuplexChallenger.
This is required for FRI verification (query indices are derived from the transcript).

**Algorithm — DuplexChallenger<BabyBear, Poseidon2<16>, 16, 8>:**

```python
from primitives.poseidon2 import permute
from primitives.field import BABYBEAR_PRIME

WIDTH = 16
RATE = 8

class Challenger:
    """Fiat-Shamir challenger using Poseidon2 duplex sponge.

    Matches Plonky3's DuplexChallenger<BabyBear, Poseidon2<16>, 16, 8>.
    """

    def __init__(self):
        self.sponge_state = [0] * WIDTH
        self.input_buffer = []    # absorb buffer (max RATE elements)
        self.output_buffer = []   # squeeze buffer

    def observe(self, value: int) -> None:
        """Absorb a single field element."""
        self.output_buffer.clear()
        self.input_buffer.append(value % BABYBEAR_PRIME)
        if len(self.input_buffer) == RATE:
            self._duplexing()

    def observe_many(self, values: list[int]) -> None:
        """Absorb multiple field elements."""
        for v in values:
            self.observe(v)

    def sample(self) -> int:
        """Squeeze one base field element."""
        if self.input_buffer or not self.output_buffer:
            self._duplexing()
        return self.output_buffer.pop()

    def sample_ext(self) -> list[int]:
        """Squeeze one extension field element (4 base elements)."""
        return [self.sample() for _ in range(4)]

    def sample_bits(self, bits: int) -> int:
        """Sample a random index with given number of bits."""
        val = self.sample()
        return val & ((1 << bits) - 1)

    def _duplexing(self) -> None:
        """Apply duplex: overwrite rate, permute, read rate."""
        # Overwrite rate portion with input (pad with zeros)
        for i in range(RATE):
            if i < len(self.input_buffer):
                self.sponge_state[i] = self.input_buffer[i]
            # Note: Plonky3 does NOT zero-pad — it only overwrites
            # elements that have input. Check source carefully.

        self.sponge_state = permute(self.sponge_state)
        self.output_buffer = list(self.sponge_state[:RATE])
        self.input_buffer = []
```

**IMPORTANT DETAILS TO VERIFY:**
1. Does Plonky3's DuplexChallenger zero-pad the rate portion or only overwrite
   the positions that have input? Read the `duplexing()` method in
   `p3-challenger/src/duplex_challenger.rs` carefully.
2. Does `sample()` pop from the front or back of `output_buffer`?
3. Does `observe(Hash<F,F,8>)` iterate and observe each element?
4. For `sample_algebra_element` (extension field), does it sample 4 base elements
   and construct EF, or something else?

**No dedicated test file for transcript.** It will be tested indirectly through
`test_fri.py` (verification requires correct transcript replay). If transcript is
wrong, FRI verification will fail with wrong query indices or wrong challenges.

**Commit:**
```bash
git add executable-spec/primitives/transcript.py
git commit -m "implement DuplexChallenger Fiat-Shamir transcript"
```

---

### Task 8: Implement FRI fold + verify + wire tests

**Files:**
- Create: `executable-spec/protocol/fri.py`
- Modify: `executable-spec/tests/test_fri.py`

**Translation source:** `/home/cody/.cargo/registry/src/.../p3-fri-0.4.1/src/`
- `prover.rs` lines 158-230 (commit_phase — for understanding fold)
- `verifier.rs` lines 54-321 (verify_fri, verify_query)
- `two_adic_pcs.rs` lines 96-163 (TwoAdicFriFolding — fold_row, fold_matrix)

**Part A: FRI Folding (for test_fold_per_round)**

The test vectors use evaluations on a coset in natural (not bit-reversed) order.
The folding formula:

```python
from primitives.field import FF, FF4, ff4, ff4_coeffs, BABYBEAR_PRIME, get_omega

def fri_fold(evals: list[list[int]], challenge: list[int],
             log_domain_size: int, coset_shift: int) -> list[list[int]]:
    """Fold FRI evaluations with challenge beta.

    Args:
        evals: Extension field evaluations [[c0,c1,c2,c3], ...] on coset
        challenge: Extension field challenge [c0,c1,c2,c3]
        log_domain_size: Log2 of current domain size
        coset_shift: Current coset generator (g, g^2, g^4, ...)

    Returns:
        Folded evaluations (half the size)

    Reference: p3-fri-0.4.1/src/two_adic_pcs.rs:134-162 (fold_matrix)
    """
    p = BABYBEAR_PRIME
    n = len(evals)
    half = n // 2
    beta = ff4(challenge)

    omega = FF(get_omega(log_domain_size))
    two_inv = pow(2, p - 2, p)  # 2^(-1) mod p

    folded = []
    for i in range(half):
        f_pos = ff4(evals[i])           # f(x)  where x = shift * omega^i
        f_neg = ff4(evals[i + half])    # f(-x) where -x = shift * omega^(i + half)

        x = (coset_shift * int(omega ** i)) % p
        half_inv_x = pow(2 * x % p, p - 2, p)  # 1/(2x) mod p

        even = (f_pos + f_neg) * ff4_from_base(two_inv)
        odd = (f_pos - f_neg) * ff4_from_base(half_inv_x)
        result = even + beta * odd

        folded.append(ff4_coeffs(result))

    return folded
```

**IMPORTANT:** The test vector generator (`lib.rs:903-936`) uses natural-order evaluations
on coset `g * <omega_N>`. After each fold round, the coset shift squares:
`shift_{i+1} = shift_i^2`. Verify this matches Plonky3's fold.

**Part B: FRI Verification (for test_query_verification)**

This requires the full verification pipeline:

```python
def verify_fri(
    commit_phase_commits: list[list[int]],  # Merkle roots
    final_poly: list[list[int]],             # Final polynomial coefficients
    query_proofs: list[dict],                # Per-query opening data
    # FRI params
    log_blowup: int,
    log_final_poly_len: int,
    num_queries: int,
) -> bool:
    """Verify a FRI proof.

    Reference: p3-fri-0.4.1/src/verifier.rs:54-204
    """
    challenger = Challenger()

    # Reconstruct transcript: observe commitments, sample betas
    betas = []
    for commit in commit_phase_commits:
        challenger.observe_many(commit)  # Observe Merkle root (8 elements)
        # No PoW (commit_proof_of_work_bits = 0 in test vectors)
        beta = challenger.sample_ext()   # Sample extension field challenge
        betas.append(beta)

    # Observe final polynomial
    for coeff in final_poly:
        challenger.observe_many(coeff)  # Each coeff is 4 base elements

    # No PoW (query_proof_of_work_bits = 0)

    log_max_height = len(commit_phase_commits) + log_blowup + log_final_poly_len
    log_final_height = log_blowup + log_final_poly_len

    # Verify each query
    for query_proof in query_proofs:
        index = challenger.sample_bits(log_max_height)
        # verify_query(index, betas, commit_phase_commits, query_proof, ...)
        # ... (fold with siblings, check Merkle proofs, check final poly)

    return True
```

**Part C: verify_query**

```python
def verify_query(
    index: int,
    betas: list[list[int]],
    commits: list[list[int]],
    openings: list[dict],
    log_max_height: int,
    log_final_height: int,
    final_poly: list[list[int]],
) -> bool:
    """Verify single FRI query chain.

    Reference: p3-fri-0.4.1/src/verifier.rs:236-321
    """
    # Start with the initial folded_eval (from reduced openings — not in our test)
    # For standalone FRI verification, we need the initial value.
    # Check what the test vectors provide.

    for i, ((beta, comm), opening) in enumerate(zip(zip(betas, commits), openings)):
        log_folded_height = log_max_height - 1 - i
        sibling_idx = index ^ 1

        evals = [None, None]
        evals[index % 2] = folded_eval
        evals[sibling_idx % 2] = ff4(opening["sibling_value"])

        # Verify Merkle opening proof
        verify_merkle(comm, index >> 1, evals, opening["opening_proof"])

        # Fold
        folded_eval = fold_row(index >> 1, log_folded_height, ff4(beta), evals)
        index >>= 1

    # Check against final polynomial
    x = pow(generator, reverse_bits(index, log_max_height), p)
    eval_at_x = horner_eval(final_poly, x)
    assert folded_eval == eval_at_x
```

**CRITICAL NOTE:** The verification vectors were extracted from a REAL Fibonacci STARK proof.
The `verify_query` function needs the initial `folded_eval` which comes from the reduced
openings (PCS layer). Our test vectors may or may not include this. Check `fri_verification.json`
carefully — if there's no initial value, we may need to adjust the test to only verify the
Merkle proofs and folding consistency (not the initial opening).

Alternatively, we may need to extract the initial values from the E2E test vectors.
Read the Rust verification test at:
`/home/cody/openvm/crates/test-vectors/tests/verify_fri_vectors.rs`
to understand what exactly is being verified.

**Test wiring (test_fri.py):**

```python
from protocol.fri import fri_fold, verify_fri
from primitives.field import ff4_coeffs, GENERATOR

class TestFRIFolding:
    def test_fold_per_round(self, vectors):
        shift = GENERATOR  # Initial coset shift
        for round_data in vectors["rounds"]:
            log_domain = ...  # Compute from round
            result = fri_fold(
                round_data["input"], round_data["challenge"],
                log_domain, shift
            )
            assert result == round_data["expected_output"]
            shift = (shift * shift) % BABYBEAR_PRIME

    def test_final_polynomial(self, vectors):
        # After all folds, the result should equal final_polynomial
        # Run all folds and check last output
        ...

class TestFRIVerification:
    def test_query_verification(self, vectors):
        result = verify_fri(
            vectors["commit_phase_commits"],
            vectors["final_poly"],
            vectors["queries"],
            log_blowup=1, log_final_poly_len=0, num_queries=2,
        )
        assert result
```

**Run tests:**
Run: `cd /home/cody/openvm/executable-spec && python -m pytest tests/test_fri.py -v`
Expected: All 3 tests PASS

**Commit:**
```bash
git add executable-spec/protocol/fri.py executable-spec/tests/test_fri.py
git commit -m "implement FRI fold and verification, all tests passing"
```

---

## Implementation Workflow

This plan file serves as the authoritative checklist for implementation. When implementing:

### Required Process
1. **Load Plan**: Read this entire plan file before starting
2. **Sync Tasks**: Create TodoWrite tasks matching the checkboxes above
3. **Execute & Update**: For each task:
   - Mark task as `in_progress` when starting
   - Update checkbox `[ ]` to `[x]` when completing
   - Mark task as `completed` when done
4. **Maintain Sync**: Keep this file and task list synchronized throughout

### Critical Rules
- This plan file is the source of truth for progress
- Update checkboxes in real-time as work progresses
- Never lose synchronization between plan file and task list
- Mark tasks complete only when ALL tests pass (no placeholders)
- Tasks should be run in parallel where possible using subagents

### Test Commands
```bash
# Individual test files
cd /home/cody/openvm/executable-spec
python -m pytest tests/test_field.py -v
python -m pytest tests/test_poseidon2.py -v
python -m pytest tests/test_ntt.py -v
python -m pytest tests/test_merkle.py -v
python -m pytest tests/test_fri.py -v

# All primitive tests
python -m pytest tests/test_field.py tests/test_poseidon2.py tests/test_ntt.py tests/test_merkle.py -v

# All milestone 1 tests
python -m pytest tests/test_field.py tests/test_poseidon2.py tests/test_ntt.py tests/test_merkle.py tests/test_fri.py -v

# Full test runner
cd /home/cody/openvm && ./run-tests.sh python
```

### Progress Tracking
The checkboxes above represent the authoritative status of each task. Keep them updated as you work.
