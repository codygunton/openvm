# Phase 1: Testing Infrastructure — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Scaffold the `executable-spec/` Python test infrastructure, extract golden test vectors from OpenVM's Rust implementation at primitive and E2E levels, write Rust tests that verify the vectors, and create failing Python test stubs that load the same vectors.

**Architecture:** A Rust test-vector-generation crate exercises OpenVM's proving pipeline on small guest programs (Fibonacci, etc.) and serializes intermediate values + full proof artifacts to JSON. These JSON files become golden vectors consumed by both Rust verification tests and Python test stubs. The Python stubs fail because no implementation exists yet (Phase 2's job).

**Tech Stack:** Rust (OpenVM, stark-backend, Plonky3), Python 3.10+ (pytest, pytest-xdist, numpy), JSON for vector format, serde_json for Rust serialization.

---

### Task 1: Scaffold `executable-spec/` Directory

**Files:**
- Create: `executable-spec/__init__.py`
- Create: `executable-spec/protocol/__init__.py`
- Create: `executable-spec/primitives/__init__.py`
- Create: `executable-spec/constraints/__init__.py`
- Create: `executable-spec/witness/__init__.py`
- Create: `executable-spec/tests/__init__.py`
- Create: `executable-spec/tests/conftest.py`
- Create: `executable-spec/pyproject.toml`

**Step 1: Create directory structure**

```bash
mkdir -p executable-spec/{protocol,primitives,constraints,witness,tests/test-data/{primitives,fri,e2e}}
```

**Step 2: Create `__init__.py` files**

Every Python package directory needs an `__init__.py`:

```bash
touch executable-spec/__init__.py
touch executable-spec/protocol/__init__.py
touch executable-spec/primitives/__init__.py
touch executable-spec/constraints/__init__.py
touch executable-spec/witness/__init__.py
touch executable-spec/tests/__init__.py
```

**Step 3: Create `executable-spec/pyproject.toml`**

```toml
[project]
name = "executable-spec"
version = "0.1.0"
requires-python = ">=3.10"
dependencies = [
    "numpy>=1.24.0",
    "pytest>=7.0.0",
    "pytest-xdist>=3.0.0",
]

[tool.pytest.ini_options]
testpaths = ["tests"]
pythonpath = ["."]

[tool.ruff]
target-version = "py310"
line-length = 100
```

**Step 4: Create `executable-spec/tests/conftest.py`**

```python
"""Shared test fixtures for executable-spec tests."""
import json
import sys
from pathlib import Path

# Add executable-spec root to Python path for absolute imports.
SPEC_ROOT = Path(__file__).resolve().parent.parent
if str(SPEC_ROOT) not in sys.path:
    sys.path.insert(0, str(SPEC_ROOT))

TEST_DATA_DIR = Path(__file__).resolve().parent / "test-data"


def load_test_vectors(category: str, name: str) -> dict:
    """Load JSON test vectors from test-data/<category>/<name>.json.

    Args:
        category: Subdirectory under test-data (e.g., "primitives", "fri", "e2e").
        name: Vector file name without extension.

    Returns:
        Parsed JSON dict.

    Raises:
        FileNotFoundError: If the vector file does not exist.
    """
    path = TEST_DATA_DIR / category / f"{name}.json"
    if not path.exists():
        raise FileNotFoundError(
            f"Test vector not found: {path}\n"
            f"Run generate-test-vectors.sh to create test vectors."
        )
    with open(path) as f:
        return json.load(f)
```

**Step 5: Verify the structure**

```bash
find executable-spec -type f | sort
```

Expected output:
```
executable-spec/__init__.py
executable-spec/constraints/__init__.py
executable-spec/primitives/__init__.py
executable-spec/protocol/__init__.py
executable-spec/pyproject.toml
executable-spec/tests/__init__.py
executable-spec/tests/conftest.py
executable-spec/witness/__init__.py
```

**Step 6: Commit**

```bash
git add executable-spec/
git commit -m "scaffold: initialize executable-spec directory structure

Create the canonical directory layout per SPEC_STYLE_GUIDE Chapter 3:
protocol/, primitives/, constraints/, witness/, tests/ with conftest.py
and pyproject.toml. All empty except for test infrastructure."
```

---

### Task 2: Create `setup.sh` Script

**Files:**
- Create: `setup.sh` (repo root)

**Step 1: Write `setup.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SPEC_DIR="$SCRIPT_DIR/executable-spec"

echo "=== Setting up executable-spec ==="

# Check Python version
python3 -c "import sys; assert sys.version_info >= (3, 10), f'Python 3.10+ required, got {sys.version}'" || {
    echo "ERROR: Python 3.10+ is required"
    exit 1
}

# Install Python dependencies
echo "Installing Python dependencies..."
if command -v uv &> /dev/null; then
    cd "$SPEC_DIR" && uv sync
else
    pip install -e "$SPEC_DIR"
fi

echo "=== Setup complete ==="
```

**Step 2: Make executable**

```bash
chmod +x setup.sh
```

**Step 3: Verify it runs**

```bash
./setup.sh
```

Expected: Python deps install successfully.

**Step 4: Commit**

```bash
git add setup.sh
git commit -m "add setup.sh for executable-spec Python environment"
```

---

### Task 3: Create `run-tests.sh` Script

**Files:**
- Create: `run-tests.sh` (repo root)

**Step 1: Write `run-tests.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SPEC_DIR="$SCRIPT_DIR/executable-spec"
WORKERS="${PYTEST_WORKERS:-48}"

usage() {
    echo "Usage: $0 [filter]"
    echo ""
    echo "Filters:"
    echo "  all           Run all tests (default)"
    echo "  primitives    Field, Poseidon2, NTT, Merkle tests"
    echo "  fri           FRI protocol tests"
    echo "  e2e           End-to-end prover/verifier tests"
    echo "  verifier      Verifier-only tests"
    echo "  -k PATTERN    Pass arbitrary pytest -k filter"
    echo ""
    echo "Environment:"
    echo "  PYTEST_WORKERS=N  Number of parallel workers (default: 48, 0=sequential)"
}

FILTER="${1:-all}"

PYTEST_ARGS=("-v" "--tb=short")

if [ "$WORKERS" -gt 0 ]; then
    PYTEST_ARGS+=("-n" "$WORKERS")
fi

case "$FILTER" in
    all)
        ;;
    primitives)
        PYTEST_ARGS+=("-k" "test_field or test_poseidon2 or test_ntt or test_merkle")
        ;;
    fri)
        PYTEST_ARGS+=("-k" "test_fri")
        ;;
    e2e)
        PYTEST_ARGS+=("-k" "test_stark_e2e or test_verifier_e2e")
        ;;
    verifier)
        PYTEST_ARGS+=("-k" "test_verifier")
        ;;
    -k)
        PYTEST_ARGS+=("-k" "${2:?Missing pattern after -k}")
        ;;
    -h|--help)
        usage
        exit 0
        ;;
    *)
        echo "Unknown filter: $FILTER"
        usage
        exit 1
        ;;
esac

cd "$SPEC_DIR"
exec python3 -m pytest "${PYTEST_ARGS[@]}"
```

**Step 2: Make executable**

```bash
chmod +x run-tests.sh
```

**Step 3: Verify it runs (expect failures — no tests yet)**

```bash
./run-tests.sh 2>&1 || true
```

Expected: pytest runs, finds no tests or reports collection errors. That's OK — we'll add test files next.

**Step 4: Commit**

```bash
git add run-tests.sh
git commit -m "add run-tests.sh with filter support and parallel execution"
```

---

### Task 4: Create Failing Python Test Stubs — Primitives

**Files:**
- Create: `executable-spec/tests/test_field.py`
- Create: `executable-spec/tests/test_poseidon2.py`
- Create: `executable-spec/tests/test_ntt.py`
- Create: `executable-spec/tests/test_merkle.py`

**Step 1: Write `executable-spec/tests/test_field.py`**

```python
"""Tests for BabyBear field arithmetic.

These tests load golden vectors generated by Plonky3's BabyBear implementation
and verify that the Python field implementation produces identical results.

Vectors are in tests/test-data/primitives/babybear_field.json.
"""
import pytest
from conftest import load_test_vectors


class TestBabyBearField:
    """BabyBear prime field (p = 2^31 - 2^27 + 1) arithmetic."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("primitives", "babybear_field")

    def test_addition(self, vectors):
        """Field addition: (a + b) mod p."""
        for case in vectors["addition"]:
            a, b, expected = case["a"], case["b"], case["expected"]
            assert False, f"Not implemented: {a} + {b} should equal {expected}"

    def test_multiplication(self, vectors):
        """Field multiplication: (a * b) mod p."""
        for case in vectors["multiplication"]:
            a, b, expected = case["a"], case["b"], case["expected"]
            assert False, f"Not implemented: {a} * {b} should equal {expected}"

    def test_inverse(self, vectors):
        """Multiplicative inverse: a^(-1) mod p."""
        for case in vectors["inverse"]:
            a, expected = case["a"], case["expected"]
            assert False, f"Not implemented: inverse({a}) should equal {expected}"

    def test_subtraction(self, vectors):
        """Field subtraction: (a - b) mod p."""
        for case in vectors["subtraction"]:
            a, b, expected = case["a"], case["b"], case["expected"]
            assert False, f"Not implemented: {a} - {b} should equal {expected}"

    def test_negation(self, vectors):
        """Additive inverse: -a mod p."""
        for case in vectors["negation"]:
            a, expected = case["a"], case["expected"]
            assert False, f"Not implemented: negation({a}) should equal {expected}"


class TestBabyBearExtensionField:
    """Quartic extension field BabyBear4 (degree-4 over BabyBear)."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("primitives", "babybear_ext_field")

    def test_ext_addition(self, vectors):
        """Extension field addition."""
        for case in vectors["addition"]:
            assert False, f"Not implemented: ext field addition"

    def test_ext_multiplication(self, vectors):
        """Extension field multiplication."""
        for case in vectors["multiplication"]:
            assert False, f"Not implemented: ext field multiplication"

    def test_ext_inverse(self, vectors):
        """Extension field inverse."""
        for case in vectors["inverse"]:
            assert False, f"Not implemented: ext field inverse"
```

**Step 2: Write `executable-spec/tests/test_poseidon2.py`**

```python
"""Tests for Poseidon2 hash function.

Golden vectors generated from p3-poseidon2 (Plonky3's Poseidon2 over BabyBear).
OpenVM uses Poseidon2 with width=16 for Merkle tree hashing and Fiat-Shamir transcript.

Vectors are in tests/test-data/primitives/poseidon2.json.
"""
import pytest
from conftest import load_test_vectors


class TestPoseidon2:
    """Poseidon2 permutation and compression over BabyBear."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("primitives", "poseidon2")

    def test_permutation(self, vectors):
        """Full Poseidon2 permutation on width-16 state."""
        for case in vectors["permutation"]:
            input_state = case["input"]
            expected_output = case["expected"]
            assert False, f"Not implemented: permutation on {len(input_state)}-element state"

    def test_compress(self, vectors):
        """Poseidon2 compression: two 8-element inputs -> 8-element output."""
        for case in vectors["compress"]:
            left, right = case["left"], case["right"]
            expected = case["expected"]
            assert False, f"Not implemented: compress({len(left)} + {len(right)} elements)"
```

**Step 3: Write `executable-spec/tests/test_ntt.py`**

```python
"""Tests for Number Theoretic Transform (NTT) over BabyBear.

Golden vectors generated from Plonky3's DFT implementation (p3-dft).
The NTT is used for polynomial evaluation/interpolation in the STARK prover.

Vectors are in tests/test-data/primitives/ntt.json.
"""
import pytest
from conftest import load_test_vectors


class TestNTT:
    """NTT and inverse NTT over BabyBear."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("primitives", "ntt")

    def test_forward_ntt(self, vectors):
        """Forward NTT: coefficient form -> evaluation form."""
        for case in vectors["forward"]:
            coeffs = case["input"]
            expected_evals = case["expected"]
            assert False, f"Not implemented: NTT on {len(coeffs)}-element polynomial"

    def test_inverse_ntt(self, vectors):
        """Inverse NTT: evaluation form -> coefficient form."""
        for case in vectors["inverse"]:
            evals = case["input"]
            expected_coeffs = case["expected"]
            assert False, f"Not implemented: INTT on {len(evals)}-element evaluation"

    def test_ntt_round_trip(self, vectors):
        """NTT(INTT(x)) == x for all test cases."""
        for case in vectors["forward"]:
            coeffs = case["input"]
            assert False, f"Not implemented: NTT round-trip on {len(coeffs)} elements"
```

**Step 4: Write `executable-spec/tests/test_merkle.py`**

```python
"""Tests for Merkle tree construction using Poseidon2.

Golden vectors generated from OpenVM's Merkle tree implementation
(which uses p3-merkle-tree with Poseidon2 compression).

Vectors are in tests/test-data/primitives/merkle.json.
"""
import pytest
from conftest import load_test_vectors


class TestMerkleTree:
    """Poseidon2-based Merkle tree construction and opening verification."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("primitives", "merkle")

    def test_tree_construction(self, vectors):
        """Build Merkle tree from leaves and verify root hash."""
        for case in vectors["construction"]:
            leaves = case["leaves"]
            expected_root = case["expected_root"]
            assert False, f"Not implemented: Merkle tree from {len(leaves)} leaves"

    def test_opening_verification(self, vectors):
        """Verify Merkle opening proof for a specific leaf."""
        for case in vectors["openings"]:
            root = case["root"]
            leaf_index = case["leaf_index"]
            leaf = case["leaf"]
            proof = case["proof"]
            assert False, f"Not implemented: verify opening at index {leaf_index}"
```

**Step 5: Run tests to verify they fail**

```bash
PYTEST_WORKERS=0 ./run-tests.sh primitives
```

Expected: All tests FAIL (either with FileNotFoundError for missing vectors, or with AssertionError). No tests should be skipped.

**Step 6: Commit**

```bash
git add executable-spec/tests/test_field.py executable-spec/tests/test_poseidon2.py \
       executable-spec/tests/test_ntt.py executable-spec/tests/test_merkle.py
git commit -m "add failing Python test stubs for primitives

BabyBear field, extension field, Poseidon2 hash, NTT, and Merkle tree.
All tests fail because no implementation exists yet (Phase 2).
All tests fail rather than skip per project policy."
```

---

### Task 5: Create Failing Python Test Stubs — FRI and E2E

**Files:**
- Create: `executable-spec/tests/test_fri.py`
- Create: `executable-spec/tests/test_stark_e2e.py`
- Create: `executable-spec/tests/test_verifier_e2e.py`

**Step 1: Write `executable-spec/tests/test_fri.py`**

```python
"""Tests for FRI (Fast Reed-Solomon IOP of Proximity) protocol.

Golden vectors capture intermediate states at every FRI folding round,
generated from stark-backend's FRI implementation.

Vectors are in tests/test-data/fri/.
"""
import pytest
from conftest import load_test_vectors


class TestFRIFolding:
    """FRI folding produces golden-vector-identical intermediate states."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("fri", "fri_folding")

    def test_fold_per_round(self, vectors):
        """Each FRI folding round produces the expected polynomial."""
        for round_data in vectors["rounds"]:
            round_idx = round_data["round"]
            input_poly = round_data["input"]
            challenge = round_data["challenge"]
            expected_output = round_data["expected_output"]
            assert False, f"Not implemented: FRI fold round {round_idx}"

    def test_final_polynomial(self, vectors):
        """Final FRI polynomial matches golden value."""
        expected = vectors["final_polynomial"]
        assert False, f"Not implemented: FRI final polynomial ({len(expected)} coefficients)"


class TestFRIVerification:
    """FRI verification checks against golden query responses."""

    @pytest.fixture
    def vectors(self):
        return load_test_vectors("fri", "fri_verification")

    def test_query_verification(self, vectors):
        """Each FRI query response verifies correctly."""
        for query in vectors["queries"]:
            query_index = query["index"]
            assert False, f"Not implemented: FRI query verification at index {query_index}"
```

**Step 2: Write `executable-spec/tests/test_stark_e2e.py`**

```python
"""End-to-end STARK prover tests.

These tests exercise the complete proving pipeline: trace generation,
constraint evaluation, polynomial commitment, FRI, and proof serialization.

Vectors are in tests/test-data/e2e/.
"""
import pytest
from conftest import load_test_vectors


class TestStarkProverE2E:
    """Full STARK prover produces proofs matching the Rust implementation."""

    @pytest.fixture(params=["fibonacci"])
    def program_name(self, request):
        return request.param

    @pytest.fixture
    def vectors(self, program_name):
        return load_test_vectors("e2e", program_name)

    def test_proof_binary_equivalence(self, program_name, vectors):
        """Python prover output matches Rust prover output byte-for-byte."""
        expected_proof_hex = vectors["proof_bytes_hex"]
        assert False, (
            f"Not implemented: full STARK proof for '{program_name}' "
            f"({len(expected_proof_hex) // 2} bytes expected)"
        )

    def test_trace_commitment(self, program_name, vectors):
        """Trace commitment (Stage 1) matches golden value."""
        expected_commitment = vectors["stage1"]["trace_commitment"]
        assert False, f"Not implemented: trace commitment for '{program_name}'"

    def test_quotient_commitment(self, program_name, vectors):
        """Quotient polynomial commitment matches golden value."""
        expected = vectors["stages"]["quotient_commitment"]
        assert False, f"Not implemented: quotient commitment for '{program_name}'"
```

**Step 3: Write `executable-spec/tests/test_verifier_e2e.py`**

```python
"""End-to-end STARK verifier tests.

These tests load proofs generated by the Rust prover and verify them
with the Python verifier implementation. This is the north star test:
if the Python verifier accepts Rust-generated proofs, the spec is correct.

Vectors are in tests/test-data/e2e/.
"""
import pytest
from pathlib import Path
from conftest import load_test_vectors, TEST_DATA_DIR


class TestStarkVerifierE2E:
    """Python verifier accepts proofs generated by OpenVM Rust prover."""

    @pytest.fixture(params=["fibonacci"])
    def program_name(self, request):
        return request.param

    @pytest.fixture
    def vectors(self, program_name):
        return load_test_vectors("e2e", program_name)

    @pytest.fixture
    def proof_binary(self, program_name):
        proof_path = TEST_DATA_DIR / "e2e" / f"{program_name}.proof.bin"
        if not proof_path.exists():
            raise FileNotFoundError(
                f"Binary proof not found: {proof_path}\n"
                f"Run generate-test-vectors.sh to create proof artifacts."
            )
        return proof_path.read_bytes()

    def test_verify_rust_proof(self, program_name, vectors, proof_binary):
        """Verifier accepts a valid proof from the Rust prover."""
        assert False, (
            f"Not implemented: verify Rust proof for '{program_name}' "
            f"({len(proof_binary)} bytes)"
        )

    def test_reject_corrupted_proof(self, program_name, vectors, proof_binary):
        """Verifier rejects a proof with a corrupted commitment."""
        assert False, f"Not implemented: reject corrupted proof for '{program_name}'"
```

**Step 4: Run all tests to verify they fail**

```bash
PYTEST_WORKERS=0 ./run-tests.sh all
```

Expected: All tests FAIL. Count the total — should be ~20+ failing tests across all files.

**Step 5: Commit**

```bash
git add executable-spec/tests/test_fri.py executable-spec/tests/test_stark_e2e.py \
       executable-spec/tests/test_verifier_e2e.py
git commit -m "add failing Python test stubs for FRI and E2E

FRI folding/verification, full STARK prover E2E, and verifier E2E.
Verifier E2E is the north star: Python verifier accepts Rust proofs.
All tests fail because no implementation exists yet (Phase 2)."
```

---

### Task 6: Create Rust Test Vector Generation Crate

**Files:**
- Create: `crates/test-vectors/Cargo.toml`
- Create: `crates/test-vectors/src/lib.rs`
- Modify: `Cargo.toml` (workspace members)

**Step 1: Check workspace Cargo.toml for member patterns**

Read `/home/cody/openvm/Cargo.toml` to see how workspace members are listed, so we know where to add our crate.

**Step 2: Create `crates/test-vectors/Cargo.toml`**

```toml
[package]
name = "openvm-test-vectors"
version = "0.1.0"
edition = "2021"

[dependencies]
openvm-circuit = { path = "../vm" }
openvm-instructions = { path = "../toolchain/instructions" }
openvm-rv32im-circuit = { path = "../../extensions/rv32im/circuit" }
openvm-rv32im-transpiler = { path = "../../extensions/rv32im/transpiler" }
openvm-transpiler = { path = "../toolchain/transpiler" }
openvm-toolchain-tests = { path = "../toolchain/tests" }
openvm-stark-sdk.workspace = true
openvm-stark-backend.workspace = true
p3-baby-bear.workspace = true
p3-field.workspace = true
serde = { version = "1", features = ["derive"] }
serde_json = "1"
eyre.workspace = true
```

**Step 3: Create `crates/test-vectors/src/lib.rs`**

```rust
//! Test vector generation for the executable specification.
//!
//! This crate generates golden test vectors by exercising OpenVM's proving
//! pipeline and serializing intermediate values to JSON. The vectors are
//! consumed by both Rust verification tests and Python spec tests.

use std::path::Path;

use p3_baby_bear::BabyBear;
use p3_field::PrimeField32;
use serde::{Deserialize, Serialize};

/// A single field arithmetic test case.
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldOpCase {
    pub a: u32,
    pub b: u32,
    pub expected: u32,
}

/// A single unary field operation test case (negation, inverse).
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldUnaryCase {
    pub a: u32,
    pub expected: u32,
}

/// BabyBear field test vectors.
#[derive(Debug, Serialize, Deserialize)]
pub struct BabyBearFieldVectors {
    pub modulus: u64,
    pub addition: Vec<FieldOpCase>,
    pub subtraction: Vec<FieldOpCase>,
    pub multiplication: Vec<FieldOpCase>,
    pub negation: Vec<FieldUnaryCase>,
    pub inverse: Vec<FieldUnaryCase>,
}

/// Generate BabyBear base field test vectors.
pub fn generate_babybear_field_vectors() -> BabyBearFieldVectors {
    use p3_field::Field;

    let test_values: Vec<u32> = vec![
        0, 1, 2, 100, 1000,
        BabyBear::ORDER_U32 - 1,  // p - 1
        BabyBear::ORDER_U32 - 2,  // p - 2
        BabyBear::ORDER_U32 / 2,  // ~p/2
        1 << 27,                   // 2^27 (relevant to BabyBear structure)
        (1 << 27) - 1,
    ];

    let mut addition = Vec::new();
    let mut subtraction = Vec::new();
    let mut multiplication = Vec::new();
    let mut negation = Vec::new();
    let mut inverse = Vec::new();

    for &a_val in &test_values {
        let a = BabyBear::from_canonical_u32(a_val);

        // Negation
        let neg_a = -a;
        negation.push(FieldUnaryCase {
            a: a_val,
            expected: neg_a.as_canonical_u32(),
        });

        // Inverse (skip 0)
        if a_val != 0 {
            let inv_a = a.inverse();
            inverse.push(FieldUnaryCase {
                a: a_val,
                expected: inv_a.as_canonical_u32(),
            });
        }

        for &b_val in &test_values {
            let b = BabyBear::from_canonical_u32(b_val);

            addition.push(FieldOpCase {
                a: a_val,
                b: b_val,
                expected: (a + b).as_canonical_u32(),
            });

            subtraction.push(FieldOpCase {
                a: a_val,
                b: b_val,
                expected: (a - b).as_canonical_u32(),
            });

            multiplication.push(FieldOpCase {
                a: a_val,
                b: b_val,
                expected: (a * b).as_canonical_u32(),
            });
        }
    }

    BabyBearFieldVectors {
        modulus: BabyBear::ORDER_U32 as u64,
        addition,
        subtraction,
        multiplication,
        negation,
        inverse,
    }
}

/// Write vectors to a JSON file, creating parent directories as needed.
pub fn write_vectors_json<T: Serialize>(vectors: &T, path: &Path) -> eyre::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(vectors)?;
    std::fs::write(path, json)?;
    Ok(())
}
```

**Step 4: Add crate to workspace**

Read the workspace Cargo.toml `members` list and add `"crates/test-vectors"` to it.

**Step 5: Verify it compiles**

```bash
cargo build --profile fast -p openvm-test-vectors
```

Expected: Clean build.

**Step 6: Commit**

```bash
git add crates/test-vectors/ Cargo.toml
git commit -m "add openvm-test-vectors crate for golden vector generation

Generates BabyBear field arithmetic test vectors by exercising Plonky3.
Will be extended with Poseidon2, NTT, Merkle, and E2E proof vectors."
```

---

### Task 7: Generate Primitive Vectors — BabyBear Field

**Files:**
- Create: `crates/test-vectors/tests/generate_field_vectors.rs`

**Step 1: Write the vector generation test**

```rust
//! Generate BabyBear field arithmetic test vectors.
//! Run with: cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_field

use std::path::PathBuf;

use openvm_test_vectors::{generate_babybear_field_vectors, write_vectors_json};

fn vectors_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../executable-spec/tests/test-data/primitives")
}

#[test]
fn generate_field_vectors() {
    let vectors = generate_babybear_field_vectors();

    // Sanity check: verify a known identity
    assert_eq!(
        vectors.modulus,
        (1u64 << 31) - (1u64 << 27) + 1,
        "BabyBear modulus should be 2^31 - 2^27 + 1"
    );

    // Verify we generated a reasonable number of test cases
    assert!(vectors.addition.len() >= 100, "Expected at least 100 addition cases");
    assert!(vectors.inverse.len() >= 9, "Expected at least 9 inverse cases");

    let output = vectors_output_dir().join("babybear_field.json");
    write_vectors_json(&vectors, &output).expect("Failed to write field vectors");

    println!("Wrote field vectors to {}", output.display());
}
```

**Step 2: Run the generation test**

```bash
cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_field
```

Expected: PASS, and `executable-spec/tests/test-data/primitives/babybear_field.json` is created.

**Step 3: Verify the JSON file exists and looks reasonable**

```bash
python3 -c "import json; d=json.load(open('executable-spec/tests/test-data/primitives/babybear_field.json')); print(f'modulus={d[\"modulus\"]}, addition_cases={len(d[\"addition\"])}, inverse_cases={len(d[\"inverse\"])}')"
```

Expected: `modulus=2013265921, addition_cases=100, inverse_cases=9`

**Step 4: Run the Python field tests (expect failure with assertion errors, NOT file-not-found)**

```bash
PYTEST_WORKERS=0 ./run-tests.sh -k test_field
```

Expected: Tests FAIL with `AssertionError: Not implemented` (vectors load successfully, tests fail on assert False).

**Step 5: Commit**

```bash
git add crates/test-vectors/tests/generate_field_vectors.rs \
       executable-spec/tests/test-data/primitives/babybear_field.json
git commit -m "generate BabyBear field arithmetic golden vectors

100 addition/subtraction/multiplication cases and 9 inverse cases
covering edge values (0, 1, p-1, p-2, p/2, 2^27). Vectors verified
by Plonky3's BabyBear implementation."
```

---

### Task 8: Generate Primitive Vectors — Extension Field

**Files:**
- Modify: `crates/test-vectors/src/lib.rs` (add extension field vector generation)
- Create: `crates/test-vectors/tests/generate_ext_field_vectors.rs`

**Step 1: Add extension field types and generation to `lib.rs`**

Add structs and a generation function for BabyBear's quartic extension field (degree-4 binomial extension, as used in OpenVM/stark-backend for FRI challenges).

The extension field type is `BinomialExtensionField<BabyBear, 4>` from Plonky3.

**Step 2: Write generation test**

Similar pattern to Task 7 but exercises extension field ops. Output to `babybear_ext_field.json`.

**Step 3: Run and verify**

```bash
cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_ext_field
```

**Step 4: Commit**

```bash
git add crates/test-vectors/ executable-spec/tests/test-data/primitives/babybear_ext_field.json
git commit -m "generate BabyBear quartic extension field golden vectors"
```

---

### Task 9: Generate Primitive Vectors — Poseidon2

**Files:**
- Modify: `crates/test-vectors/src/lib.rs` (add Poseidon2 vector generation)
- Modify: `crates/test-vectors/Cargo.toml` (add p3-poseidon2 dependency)
- Create: `crates/test-vectors/tests/generate_poseidon2_vectors.rs`

**Step 1: Add Poseidon2 dependencies**

Add to Cargo.toml:
```toml
p3-poseidon2.workspace = true
p3-symmetric.workspace = true
```

**Step 2: Add Poseidon2 vector generation to `lib.rs`**

Generate vectors for:
- Full permutation (width=16 state)
- Compression (two 8-element inputs → 8-element output)

Use the same Poseidon2 configuration OpenVM uses (find it via `vm_poseidon2_hasher()` in `crates/vm/src/arch/hasher/poseidon2/`).

**Step 3: Write generation test, run, verify**

Output to `executable-spec/tests/test-data/primitives/poseidon2.json`.

**Step 4: Commit**

---

### Task 10: Generate Primitive Vectors — NTT

**Files:**
- Modify: `crates/test-vectors/src/lib.rs` (add NTT vector generation)
- Modify: `crates/test-vectors/Cargo.toml` (add p3-dft dependency)
- Create: `crates/test-vectors/tests/generate_ntt_vectors.rs`

**Step 1: Add NTT dependency**

```toml
p3-dft.workspace = true
p3-matrix.workspace = true
```

**Step 2: Add NTT vector generation**

Generate forward NTT and inverse NTT vectors for polynomial sizes: 4, 8, 16, 32.

**Step 3: Write generation test, run, verify**

Output to `executable-spec/tests/test-data/primitives/ntt.json`.

**Step 4: Commit**

---

### Task 11: Generate Primitive Vectors — Merkle Tree

**Files:**
- Modify: `crates/test-vectors/src/lib.rs` (add Merkle vector generation)
- Modify: `crates/test-vectors/Cargo.toml` (add p3-merkle-tree dependency)
- Create: `crates/test-vectors/tests/generate_merkle_vectors.rs`

**Step 1: Add Merkle tree dependency**

```toml
p3-merkle-tree.workspace = true
```

**Step 2: Add Merkle tree vector generation**

Generate:
- Tree construction (leaves → root) for 4, 8, 16 leaves
- Opening proofs (leaf + siblings → root verification)

**Step 3: Write generation test, run, verify**

Output to `executable-spec/tests/test-data/primitives/merkle.json`.

**Step 4: Commit**

---

### Task 12: Generate E2E Proof Vectors — Fibonacci

**Files:**
- Modify: `crates/test-vectors/src/lib.rs` (add E2E proof generation)
- Modify: `crates/test-vectors/Cargo.toml` (add rv32im deps)
- Create: `crates/test-vectors/tests/generate_e2e_vectors.rs`
- Create: `crates/test-vectors/programs/` (symlink or copy fibonacci)

**Step 1: Add E2E dependencies**

The crate needs the same deps as `extensions/rv32im/tests/`. Check that Cargo.toml for exact versions.

**Step 2: Write E2E vector generation**

This test will:
1. Build the Fibonacci guest program ELF
2. Transpile to VmExe
3. Run `air_test_impl` to get `Vec<VerificationDataWithFriParams>`
4. Serialize each `Proof<SC>` using the codec from `crates/sdk/src/codec.rs`
5. Also extract: verifying key, FRI parameters, per-AIR public values
6. Write JSON metadata + hex-encoded proof bytes to `e2e/fibonacci.json`
7. Write raw binary proof to `e2e/fibonacci.proof.bin`

**Step 3: Run the generation (note: this is slow — set timeout)**

```bash
cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_e2e --test-threads=1
```

**Step 4: Verify Python tests now fail with assertion errors (not file-not-found)**

```bash
PYTEST_WORKERS=0 ./run-tests.sh e2e
```

**Step 5: Commit vectors and generation code**

---

### Task 13: Create `generate-test-vectors.sh` Script

**Files:**
- Create: `generate-test-vectors.sh` (repo root)

**Step 1: Write the script**

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    echo "Usage: $0 [target]"
    echo ""
    echo "Targets:"
    echo "  all          Generate all vectors (default)"
    echo "  primitives   Field, Poseidon2, NTT, Merkle vectors only"
    echo "  e2e          E2E proof vectors only (slow)"
    echo "  clean        Remove all generated vectors"
}

TARGET="${1:-all}"

generate_primitives() {
    echo "=== Generating primitive vectors ==="
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_field
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_ext_field
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_poseidon2
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_ntt
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_merkle
    echo "=== Primitive vectors complete ==="
}

generate_e2e() {
    echo "=== Generating E2E proof vectors (this may take a while) ==="
    cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- generate_e2e --test-threads=1
    echo "=== E2E vectors complete ==="
}

case "$TARGET" in
    all)
        generate_primitives
        generate_e2e
        ;;
    primitives)
        generate_primitives
        ;;
    e2e)
        generate_e2e
        ;;
    clean)
        echo "Removing generated vectors..."
        rm -rf "$SCRIPT_DIR/executable-spec/tests/test-data/primitives/"*.json
        rm -rf "$SCRIPT_DIR/executable-spec/tests/test-data/fri/"*.json
        rm -rf "$SCRIPT_DIR/executable-spec/tests/test-data/e2e/"*.json
        rm -rf "$SCRIPT_DIR/executable-spec/tests/test-data/e2e/"*.bin
        echo "Done."
        ;;
    -h|--help)
        usage
        ;;
    *)
        echo "Unknown target: $TARGET"
        usage
        exit 1
        ;;
esac
```

**Step 2: Make executable and test**

```bash
chmod +x generate-test-vectors.sh
./generate-test-vectors.sh primitives
```

**Step 3: Commit**

```bash
git add generate-test-vectors.sh
git commit -m "add generate-test-vectors.sh for reproducible vector generation"
```

---

### Task 14: Rust Verification Tests

**Files:**
- Create: `crates/test-vectors/tests/verify_field_vectors.rs`
- Create: `crates/test-vectors/tests/verify_poseidon2_vectors.rs`

**Step 1: Write Rust test that loads and re-verifies field vectors**

These tests load the JSON vectors and re-compute each operation, confirming the vectors are consistent with the live Plonky3 code. This catches any staleness if Plonky3 is updated.

```rust
#[test]
fn verify_field_vectors_against_live_code() {
    let json = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../executable-spec/tests/test-data/primitives/babybear_field.json")
    ).expect("Field vectors must exist — run generate-test-vectors.sh");

    let vectors: BabyBearFieldVectors = serde_json::from_str(&json).unwrap();

    for case in &vectors.addition {
        let a = BabyBear::from_canonical_u32(case.a);
        let b = BabyBear::from_canonical_u32(case.b);
        let result = (a + b).as_canonical_u32();
        assert_eq!(result, case.expected, "a={}, b={}", case.a, case.b);
    }
    // ... same for subtraction, multiplication, negation, inverse
}
```

**Step 2: Write similar verification for Poseidon2, NTT, Merkle**

**Step 3: Run all verification tests**

```bash
cargo nextest run --cargo-profile=fast -p openvm-test-vectors -- verify_
```

Expected: All PASS (vectors match live code).

**Step 4: Commit**

```bash
git add crates/test-vectors/tests/verify_*.rs
git commit -m "add Rust verification tests for golden vectors

Re-compute every operation from the JSON vectors against live Plonky3
code. Catches vector staleness if dependencies are updated."
```

---

### Task 15: Final Verification and Documentation

**Step 1: Run the full Python test suite**

```bash
PYTEST_WORKERS=0 ./run-tests.sh all 2>&1 | tail -20
```

Expected: All tests FAIL (with assertion errors, not import errors or file-not-found for primitives).

**Step 2: Run the full Rust verification suite**

```bash
cargo nextest run --cargo-profile=fast -p openvm-test-vectors
```

Expected: All generation and verification tests PASS.

**Step 3: Count total test coverage**

```bash
PYTEST_WORKERS=0 ./run-tests.sh all 2>&1 | grep -E "(FAILED|PASSED|ERROR)" | tail -5
```

Record the total number of failing Python stubs.

**Step 4: Update design doc with completion status**

Update `docs/plans/2026-03-05-phase1-testing-infrastructure-design.md` to check off completed criteria.

**Step 5: Final commit**

```bash
git add -A
git commit -m "complete Phase 1 testing infrastructure

All primitive golden vectors generated and verified.
E2E proof vectors for Fibonacci generated and verified.
Python test stubs fail as expected (Phase 2 will implement).
Rust verification tests confirm vector freshness."
```

---

## Dependency Graph

```
Task 1 (scaffold) ──┬── Task 4 (Python primitive stubs)
                     ├── Task 5 (Python FRI/E2E stubs)
                     └── Task 2 (setup.sh) ── Task 3 (run-tests.sh)

Task 6 (Rust crate) ─┬── Task 7 (field vectors) ── Task 8 (ext field)
                      ├── Task 9 (poseidon2) ── Task 10 (NTT) ── Task 11 (merkle)
                      └── Task 12 (E2E vectors)

Task 13 (generate script) depends on Tasks 7-12
Task 14 (Rust verification) depends on Tasks 7-12
Task 15 (final verification) depends on all above
```

**Parallelizable pairs:**
- Tasks 1-3 (Python scaffold) and Task 6 (Rust crate) are independent
- Tasks 4-5 (Python stubs) can run in parallel after Task 1
- Tasks 7-11 (primitive vectors) can run in parallel after Task 6
