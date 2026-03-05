# Executable Specification Style Guide

A comprehensive guide for creating executable Python specifications of zkSNARK proving systems. This guide codifies the principles, patterns, conventions, and enforcement mechanisms needed to produce specs that are simultaneously readable as protocol descriptions and executable as test oracles.

## Table of Contents

1. [Philosophy & Audience](#chapter-1-philosophy--audience)
2. [Three-Phase Workflow](#chapter-2-three-phase-workflow)
3. [Directory Architecture](#chapter-3-directory-architecture)
4. [Code Style — The 10 Readability-First Principles](#chapter-4-code-style--the-10-readability-first-principles)
5. [Type System](#chapter-5-type-system)
6. [Protocol Purity](#chapter-6-protocol-purity)
7. [Testing Philosophy](#chapter-7-testing-philosophy)
8. [Constraint Evaluation Patterns](#chapter-8-constraint-evaluation-patterns)
9. [Source Translation Patterns](#chapter-9-source-translation-patterns)
10. [Agent Enforcement Pipeline](#chapter-10-agent-enforcement-pipeline)
- [Appendix A: Quick Reference Checklist](#appendix-a-quick-reference-checklist)
- [Appendix B: Per-Project Adaptation](#appendix-b-per-project-adaptation)

---

# Chapter 1: Philosophy & Audience

## What Is an Executable Specification?

An executable specification is Python code that **is** the specification of a zkSNARK proving system. It is not a reference implementation. It is not a prototype. It is not a teaching tool. The code itself is the normative description of the protocol.

If the spec code and a production implementation disagree, the disagreement must be investigated. One of them is wrong. There is no third option where "the spec is just approximate" or "the production code has its own interpretation." They must agree, and when they do not, someone has a bug.

## Audience

The intended reader is a cryptographer or protocol engineer who understands STARK proofs, polynomial commitment schemes, FRI, and finite field arithmetic. They know what a Reed-Solomon codeword is. They know why you fold in FRI. They know the difference between a commitment and an evaluation. They do not need to be told.

What they want is clean code they can verify against a production implementation. They want to read a function, see the algorithm it implements, and confirm it matches their understanding of the protocol. They want to do this without reverse-engineering abstractions that exist for engineering reasons they do not care about.

## The Fundamental Trade-Off

Every decision in the spec resolves the same way:

- **Readability over performance.** A slow loop that a reader can follow beats a fast one they cannot.
- **Explicit over clever.** Spelling out an intermediate variable beats folding it into an expression.
- **Verbose over dense.** Three clear lines beat one packed line.

The spec exists for human understanding, not machine efficiency.

## Success Criteria

A knowledgeable reader can read the spec top-to-bottom and understand the complete protocol without consulting the source implementation. Every function reads like a textbook explanation of the step it implements.

## Anti-Patterns

- **Code that requires reverse-engineering to understand.**
- **Educational comments explaining well-known concepts.** Do not explain what FRI is. The readers know. Comments explaining common knowledge add noise that obscures the comments that actually matter.
- **Performance-optimized code that obscures the algorithm.** Bit-twiddling, lookup tables, SIMD-style batching belong in production implementations, not here.
- **Dense one-liners that pack multiple concepts.** Unpack them. Use intermediate variables.
- **Abstraction layers that exist for engineering reasons rather than conceptual clarity.** Factory patterns, dependency injection, and plugin architectures solve software engineering problems. The spec has communication problems.

## The Meta-Goal

Convince humans they understand the proving system. If a knowledgeable but patience-limited reader cannot follow the spec comfortably, the spec has failed.

---

# Chapter 2: Three-Phase Workflow

Every executable spec is built in three phases, in order, with explicit gates between them.

```
Phase 1          Phase 2              Phase 3
TESTING    -->  TRANSLATION    -->  SIMPLIFICATION

Setup golden     Bottom-up            Refactor for
vectors &        faithful             readability,
binary equiv     translation          maintain test
framework                            equivalence

Agent gates:     Agent gates:         Agent gates:
paranoid-        faithful-spec-       crypto-spec-simplifier
skeptic          translator +         all review agents
                 paranoid-skeptic     paranoid-skeptic (final)
```

## Phase 1: Establish Testing

Before translating a single function, build the testing infrastructure.

### Golden Vector Tests

Run the source implementation on chosen inputs and capture every intermediate value that matters: committed polynomials, Merkle roots, FRI query responses, challenge values, final proof bytes. Store these as test fixtures.

### Binary Equivalence Framework

Build a harness that runs the Python spec and compares its outputs byte-for-byte against the golden vectors. Not "close enough." Not "equivalent up to serialization differences." Byte-identical. This comparison leaves no room for subtle divergence to hide.

### Test Hierarchy

**Critical tests** — must pass at all times:
- E2E prover producing a proof binary-identical to the source implementation's proof.
- Verifier accepting proofs generated by the source implementation.
- FRI folding producing golden-vector-identical intermediate states at every round.

**High-value tests** — catch bugs early and localize failures:
- Primitive unit tests: NTT, Merkle tree, field arithmetic edge cases.
- Constraint evaluation against golden values.
- Serialization round-trips.

**Low-value tests** — remove when stable:
- Syntax and import verification.
- Redundant coverage.
- Tests that confirm something runs without checking it produces correct output.

### The Cardinal Rule

**Under no circumstances do we break tests.** A broken test is not "expected" or "temporary." It is a defect that blocks progress.

### Advancing to Phase 2

Phase 1 is complete when the golden vectors exist, the binary comparison framework passes on identity cases, and the `paranoid-skeptic` agent has reviewed the testing infrastructure.

## Phase 2: Faithful Translation

Translate the source implementation into Python. The goal is correctness — not readability, not elegance.

### Strategy: Bottom-Up

1. Field arithmetic (base field, extension fields).
2. Hash functions.
3. Merkle tree construction and opening.
4. NTT / polynomial evaluation.
5. PCS (polynomial commitment scheme).
6. FRI — folding, query, verification.
7. Constraint system — AIR definitions, trace generation, quotient computation.
8. Prover — top-level orchestration.
9. Verifier — top-level checking.

Each layer is tested against golden vectors before moving to the next.

### What to Preserve

Function signatures, class boundaries, mathematical operations, loop structures, conditional logic, error handling.

### What to Adapt

Memory management (let Python handle it), type system (use Python type hints), naming conventions (`snake_case`), iterators (use Python idioms).

### Discipline

**No simplification during this phase.** The complexity may encode a subtlety you do not yet understand. Simplification is Phase 3's job.

Every public function in the source gets a corresponding function in the spec. Add type annotations to every function signature. Add a one-line docstring referencing the source location (file and line number).

### Advancing to Phase 3

Phase 2 is complete when every source function is translated, all critical and high-value tests pass, and binary equivalence is confirmed end-to-end.

## Phase 3: Simplification

Refactor for readability. Every change follows the same micro-cycle:

1. Make a single, focused change.
2. Run the full test suite.
3. Confirm binary equivalence.
4. If step 2 or 3 fails, revert. Do not debug forward.

### Simplification Targets

- **Separate protocol from implementation.** Establish the `protocol/` vs `primitives/` boundary (see [Chapter 6](#chapter-6-protocol-purity)).
- **Introduce semantic type aliases.** (See [Chapter 5](#chapter-5-type-system).)
- **Inline over-fragmented helpers.** If a helper is called once and adds no clarity, inline it.
- **Remove educational comments.** Keep comments that explain non-obvious choices.
- **Express WHAT, not HOW.** Replace procedural loops with declarative expressions where clearer.

### Agent Gate

The full review pipeline runs: `crypto-spec-simplifier`, `type-enforcer`, `human-simplicity-enforcer`, `protocol-purity-guardian`, `zksnark-python-style`. The `paranoid-skeptic` performs final verification.

### Phase 3 Is Iterative

Each simplification makes the spec clearer. The phase ends when the review agents are satisfied and further changes would risk introducing confusion.

| Transition | Gate Conditions |
|---|---|
| Phase 1 → 2 | Test infrastructure complete. Golden vectors generated. Binary comparison passing. |
| Phase 2 → 3 | All source functions translated. All tests passing. Binary equivalence confirmed. |
| Phase 3 iterations | Each micro-cycle: change, test, verify. Revert on failure. |

---

# Chapter 3: Directory Architecture

The canonical directory layout separates concerns along a single axis: **what the proof system specifies** versus **how computations are carried out**.

```
executable-spec/
├── protocol/      # WHAT the proof system does
├── primitives/    # HOW things are computed
├── constraints/   # Per-AIR constraint evaluation
├── witness/       # Per-AIR witness generation (optional)
├── tests/         # All tests and test data
│   └── test-data/ # Golden vectors, fixtures
└── pyproject.toml
```

## `protocol/` — Pure protocol specification

Contains the abstract description of the proving system. A reader should understand the complete STARK protocol by reading only this directory.

- Prover orchestration (stage sequencing, commitment rounds)
- Verifier logic (acceptance criteria, equation checks)
- FRI protocol (folding strategy, query generation, verification)
- Polynomial commitment scheme (abstract commit/open/verify)
- Proof data structures and serialization
- Configuration parsing

Contains ONLY abstract operations — no NTT, no specific hash implementations, no buffer management. Files here call into `primitives/` for any concrete computation.

## `primitives/` — Implementation details

Contains every concrete algorithm the protocol depends on. Swapping one implementation for another must require changes only here — never in `protocol/`.

- Field arithmetic (base field, extension field, type aliases)
- NTT/INTT
- Merkle tree construction and verification
- Hash function (Poseidon2 or similar)
- Fiat-Shamir transcript
- Batch operations (batch inverse, etc.)
- Polynomial operations (extend, evaluate, interpolate)

## `constraints/` — Per-AIR constraint modules

Each AIR type gets exactly one file implementing a standard constraint evaluation interface. Constraint code must work in both prover (array) and verifier (scalar) contexts.

## `tests/` — Test infrastructure

E2E tests, unit tests, `test-data/` for golden vectors, `conftest.py` for shared fixtures, `run-tests.sh` with filter support.

## Decision Rule

> "Could I implement this differently and get the same proof bytes? If yes → `primitives/`. If no → `protocol/`."

## File Naming

Use snake_case for Python files since they are importable modules. Directory names use hyphens only when they are not Python packages (e.g., `test-data/`). Every directory containing `.py` files must have an `__init__.py`.

---

# Chapter 4: Code Style — The 10 Readability-First Principles

Base requirement: Google Python Style Guide, with emphases on §2.21 (type annotations), §3.8 (docstrings), §3.16.4 (mathematical variable names).

All code must pass:
```bash
ruff check protocol/ primitives/ constraints/
mypy --strict protocol/
```

### Principle 1: Explicit Intermediate Variables

Break complex expressions into named steps. Each line represents one logical operation.

```python
# GOOD
wire_values = compute_wire_assignments(circuit, witness)
constraint_poly = build_constraint_polynomial(wire_values, selectors)
quotient = constraint_poly.divide_by_vanishing(domain)
commitment = commit_polynomial(quotient, srs)

# BAD
commitment = commit_polynomial(
    build_constraint_polynomial(
        compute_wire_assignments(circuit, witness), selectors
    ).divide_by_vanishing(domain), srs)
```

*Enforced by: zksnark-python-style, human-simplicity-enforcer*

### Principle 2: Type Aliases for Domain Concepts

Define semantic type aliases at module level. They serve as documentation-through-types. See [Chapter 5](#chapter-5-type-system) for the full type system specification.

```python
FF = galois.GF(PRIME)      # Base field element
FF3 = ...                  # Cubic extension element
FF3Poly = FF3              # Polynomial over extension field (array)
FFPoly = FF                # Polynomial over base field (array)
HashOutput = list[int]     # Hash digest
```

*Enforced by: type-enforcer, zksnark-python-style*

### Principle 3: Functions Compute One Thing

Each function corresponds to a single mathematical operation. Name functions for what they return.

```python
# GOOD
def compute_quotient_polynomial(constraint_poly: FF3Poly, vanishing: FFPoly) -> FF3Poly: ...

# BAD
def process_constraints(data): ...  # What does "process" mean?
```

*Enforced by: human-simplicity-enforcer*

### Principle 4: No Clever Python

Avoid features that sacrifice readability for conciseness:

```python
# BAD: walrus operator
if (n := len(items)) > 0: process(n)

# GOOD
n = len(items)
if n > 0:
    process(n)

# BAD: multi-clause comprehension
result = [f(x) for xs in groups for x in xs if pred(x)]

# GOOD
result = []
for xs in groups:
    for x in xs:
        if pred(x):
            result.append(f(x))

# BAD: implicit boolean coercion
if items: ...

# GOOD
if len(items) > 0: ...
```

*Enforced by: zksnark-python-style*

### Principle 5: Mathematical Notation Requires Citation

Single-letter variables are allowed ONLY when they match notation from a cited paper or standard reference.

```python
# GOOD
omega = get_root_of_unity(n)       # primitive nth root of unity [STARK paper §3.2]
xi = transcript.get_challenge()    # random evaluation point [FRI §4.1]

# BAD
x = compute_something()           # What is x?
```

*Enforced by: zksnark-python-style*

### Principle 6: Docstrings as Specifications

Every public function needs: one-line description, Args, Returns, Reference (if implementing a known algorithm).

```python
def fold_polynomial(evaluations: FF3Poly, challenge: FF3) -> FF3Poly:
    """Fold a polynomial's evaluations using a FRI challenge.

    Args:
        evaluations: Polynomial evaluated at domain points.
        challenge: Random folding challenge from Fiat-Shamir.

    Returns:
        Folded polynomial evaluations at half the domain size.

    Reference:
        FRI protocol, fold step [BBHR18 §5.2].
    """
```

*Enforced by: zksnark-python-style*

### Principle 7: Vertical Whitespace for Logical Grouping

Use blank lines to separate logical phases. Use `# --- Section ---` headers for module navigation.

```python
def generate_proof(config, trace):
    # --- Stage 1: Commit to witness ---
    witness_poly = extend_trace(trace)
    root_1 = commit(witness_poly)

    # --- Stage 2: Constraint evaluation ---
    challenges = derive_challenges(root_1)
    quotient = compute_quotient(witness_poly, challenges)

    # --- Stage 3: FRI ---
    fri_proof = fri_prove(quotient)

    return Proof(root_1, fri_proof)
```

*Enforced by: zksnark-python-style, human-simplicity-enforcer*

### Principle 8: Explicit Loops Over Functional Magic

Prefer `for` loops over `map`, `filter`, `reduce`.

Exception: simple `[f(x) for x in items]` comprehensions (single clause, no `if`).

*Enforced by: zksnark-python-style*

### Principle 9: No Abbreviations in Names (Except Math)

Spell out words: `polynomial_degree` not `poly_deg`, `verification_key` not `vk`, `commitment` not `comm`.

*Enforced by: zksnark-python-style*

### Principle 10: Constants at Module Level, Named

No magic numbers. All constants have descriptive names.

```python
FIELD_EXTENSION_DEGREE = 3
DEFAULT_NUM_QUERIES = 28
POSEIDON2_STATE_WIDTH = 16

degree = polynomial_degree * FIELD_EXTENSION_DEGREE  # not * 3
```

*Enforced by: zksnark-python-style*

### Accepted Exceptions

- **Nested if (`SIM102`)**: Allowed when control flow is clearer nested.
- **Short variable names used extensively**: Document at file level with a mapping comment block.
- **Simple comprehensions**: `[f(x) for x in items]` is acceptable (single clause, single expression).
- **`i`, `j`, `k` as loop indices**: Universally understood. No citation needed.
- **`n` for collection length**: Acceptable when assigned from `len(...)` on the preceding line.

---

# Chapter 5: Type System

## The Semantic Type Alias System

Every spec project defines type aliases in `primitives/field.py` that map Python types to mathematical domain concepts. These aliases are the project's type vocabulary.

**Core type hierarchy** (adapt names per project):

```python
FF                  # GF(p) — base field element
FF3                 # GF(p^3) — cubic extension field element
FFPoly              # Polynomial over base field
FF3Poly             # Polynomial over extension field
FFColumn            # Column of base field values
FF3Column           # Column of extension field values
HashOutput          # Hash digest (e.g., list[int] for Poseidon2)
```

The distinction between `FFPoly` and `FFColumn` matters: a polynomial has coefficients manipulated algebraically; a column is a vector of trace values. Different mathematical objects, same backing type.

## Why Type Aliases Matter

Type aliases are **documentation-as-types**:

```python
# Without aliases — what are these types?
def fold(evals: np.ndarray, challenge: np.ndarray) -> np.ndarray: ...

# With aliases — self-documenting
def fold(evals: FF3Poly, challenge: FF3) -> FF3Poly: ...
```

The second signature tells the reader: "takes a polynomial over the cubic extension and a single extension element, returns a polynomial." No docstring needed for the types.

## Where Type Aliases Live

Always defined in `primitives/field.py`, imported by `protocol/` code. Never define aliases locally in `protocol/` files.

```python
# GOOD
from primitives.field import FF, FF3, FF3Poly, FFPoly, HashOutput

# BAD
FF3Poly = np.ndarray  # Local definition — duplicates and may diverge
```

## Red Flags (Violations)

1. **`np.ndarray` for field elements in protocol code**: Should use a semantic type alias.
2. **Lowercase constructor as return type**: `-> ff3` is the constructor; the type is `-> FF3`.
3. **Missing type annotations on public functions**: All functions in `protocol/` need annotations.
4. **`Union[FF3, np.ndarray]` patterns**: Transitional debt. New code must not add more.
5. **`Any` in return types**: Defeats the purpose of the type system.

## When Raw Types Are Acceptable

- Loop indices: `i: int`, `count: int`
- Array dimensions: `n_rows: int`, `degree: int`
- Boolean flags: `is_extended: bool`
- String identifiers: `air_name: str`
- External library interfaces at serialization boundaries

Rule of thumb: mathematical objects get type aliases; program logistics get raw types.

## Per-Project Adaptation

1. Identify the base field and set `FF`.
2. Identify the extension field and degree, set `FF3` (or `FF2`, `FF4`).
3. Define the alias set in `primitives/field.py` with one-line comments.
4. Import consistently across all `protocol/` files.

Names like `FF`, `FF3` are conventions. New projects may use `Fe`, `Ext`, `BasePoly` — as long as they are consistent and documented in one place.

*Enforced by: type-enforcer agent; `mypy --strict` for annotation completeness.*

---

# Chapter 6: Protocol Purity

## The Protocol Invariant

> If you can change an implementation detail without changing the resulting proof bytes, that detail DOES NOT belong in the protocol specification.

This is the most important architectural principle in the spec.

## What Belongs in `protocol/`

- Abstract polynomial operations (evaluate, commit, fold, extend)
- Fiat-Shamir challenge derivation and commitment rounds
- Proof structure and serialization format
- Constraint system evaluation at the mathematical level
- FRI protocol logic (folding, queries, verification)
- Verification equations and acceptance criteria
- Protocol parameters (degree bounds, query count, blowup factor)

## What DOES NOT Belong in `protocol/`

- NTT/INTT implementation details
- Specific multiplication algorithms
- Memory layout or buffer management
- Batching strategies for performance
- SIMD, parallelization, cache optimization
- Montgomery representation internals
- Hash function implementations (only their abstract interface)
- Bit-manipulation for field arithmetic

## Violation Examples

```python
# VIOLATION: Direct NTT in protocol code
def multiply_polys(a, b):
    return intt(ntt(a) * ntt(b))

# CORRECTION: Protocol calls abstract operation
from primitives.polynomial import poly_mul
def multiply_polys(a, b):
    return poly_mul(a, b)
```

```python
# VIOLATION: Field infrastructure computation in protocol
pol_shift_inv = get_shift_inv()
for _ in range(n_bits_ext - prev_bits):
    pol_shift_inv = (pol_shift_inv * pol_shift_inv) % p

# CORRECTION: Protocol declares WHAT, primitives compute HOW
from primitives.field import shift_inv_pow2
pol_shift_inv = shift_inv_pow2(n_bits_ext - prev_bits)
```

## Litmus Test Questions

Ask for every line in `protocol/`:

1. **"Could I implement this differently and get the same proof?"** → If yes, it is an implementation detail.
2. **"Does the verifier need to know this to check the proof?"** → If no, it probably does not belong.
3. **"Is this WHAT to compute or HOW to compute it?"** → HOW belongs in `primitives/`.

## The Boundary Is Strict

Protocol code should read like a mathematical specification. A cryptographer should understand the proving protocol by reading only `protocol/`, without understanding NTT algorithms or hash internals. Conversely, a systems programmer should optimize everything in `primitives/` without understanding the cryptographic protocol.

*Enforced by: protocol-purity-guardian agent (insistent and uncompromising).*

---

# Chapter 7: Testing Philosophy

## The Cardinal Rule

**Never break tests.** Tests are the safety net for all translation and simplification work. Every other quality measure is secondary to correctness.

## Binary Equivalence

The Python spec must produce **byte-identical outputs** to the source implementation for all test vectors.

- Generate proofs with the source implementation, store as `.proof.bin` golden files
- Compare Python spec output byte-for-byte — not "close enough," byte-identical
- Proofs must be deterministic: multiple runs produce identical output

## Test Hierarchy

**Critical (keep always):**
- E2E prover with binary comparison against source
- Verifier accepting source-generated proofs
- FRI folding golden vectors

**High-value (keep):**
- Primitive unit tests (NTT, batch inverse, Merkle, field ops)
- Constraint evaluation against golden values
- Serialization round-trips
- Per-AIR type coverage

**Low-value (remove during Phase 3):**
- Python syntax verification tests
- Redundant coverage
- Tests without correctness criteria

## Test Vector Management

- Store in `tests/test-data/` as JSON and binary
- Provide `generate-test-vectors.sh` to regenerate from source
- Version control test vectors — they are part of the spec
- Regenerate when source implementation changes

## Test Organization

```
tests/
├── conftest.py              # Shared fixtures
├── test_stark_e2e.py        # E2E prover (MOST CRITICAL)
├── test_verifier_e2e.py     # Verifier against source proofs
├── test_fri.py              # FRI golden vectors
├── test_constraint_*.py     # Per-module unit tests
├── test_ntt.py              # Primitive tests
└── test-data/               # Golden vectors
```

Provide `run-tests.sh` with filter support:
```bash
./run-tests.sh              # all tests
./run-tests.sh e2e          # E2E only
./run-tests.sh fri          # FRI tests only
./run-tests.sh -k "pattern" # pytest -k filter
```

## Testing Rules

- **xfail over skip**: Use `pytest.mark.xfail(reason="...")`, never `pytest.skip()` inside test bodies.
- **Test after every change**: Run the full suite after every simplification step.
- **Determinism**: Run generators multiple times to verify. Non-determinism is critical.
- **Regression prevention**: Every bug fix must be accompanied by a test that would have caught it.

## What a Test Failure Means

Two possibilities: (1) the change introduced a bug (common — revert or fix), or (2) the test itself is wrong (rare — must be proven by showing it contradicts source behavior). Never delete a failing test to make the suite green.

*Enforced by: paranoid-skeptic agent.*

---

# Chapter 8: Constraint Evaluation Patterns

## The ConstraintContext Pattern

STARK specs evaluate the same constraint polynomials in two contexts:
- **Prover**: Over the full trace (arrays of field elements)
- **Verifier**: At a single challenge point (scalar field elements)

The ConstraintContext provides a **uniform interface** so the same constraint code works in both contexts.

## The Interface

```python
class ConstraintContext(ABC):
    """Prover: col('a') returns FF3Poly (array).
    Verifier: col('a') returns FF3 (scalar at challenge point xi)."""

    @abstractmethod
    def col(self, name: str, index: int = 0) -> FF3Poly | FF3: ...

    @abstractmethod
    def next_col(self, name: str, index: int = 0) -> FF3Poly | FF3:
        """Next row. Prover: circular shift. Verifier: eval at xi * omega."""

    @abstractmethod
    def challenge(self, name: str) -> FF3: ...

    @abstractmethod
    def const(self, name: str) -> FF3Poly | FF3: ...
```

## Why It Works

Field libraries like `galois` support **broadcasting**: array-scalar operations work naturally. The constraint expressions are polynomials over a ring — whether instantiated as arrays or scalars, the algebraic relationships are identical.

## Constraint Module Structure

Each AIR type gets one file in `constraints/`:

```python
class SimpleLeftConstraints(ConstraintModule):
    def constraint_polynomial(self, ctx: ConstraintContext) -> FF3Poly | FF3:
        alpha = ctx.challenge('std_alpha')
        a = ctx.col('a')
        b = ctx.col('b')
        L1 = ctx.const('__L1__')

        constraint_0 = a * b - L1
        constraint_1 = a + b - alpha

        return constraint_0 + constraint_1 * alpha
```

## When to Use This Pattern

Use when prover and verifier evaluate the same constraints and the field library supports broadcasting. For simpler specs where evaluation strategies differ significantly, separate implementations may be clearer.

---

# Chapter 9: Source Translation Patterns

## General Principles

During Phase 2 (faithful translation):

1. **Bottom-up**: Leaf dependencies first, entry points last
2. **Structure-preserving**: Class-by-class, function-by-function correspondence
3. **No simplification**: Preserve source structure. Simplification is Phase 3.
4. **Every public function translated**
5. **Add references**: Docstrings cite source file and line numbers

## Rust → Python

| Rust | Python | Notes |
|------|--------|-------|
| `struct Foo { x: u64 }` | `@dataclass class Foo: x: int` | Use dataclasses |
| `impl Foo { fn bar(&self) }` | Method on the class | |
| `trait Foo` | `class Foo(ABC)` or `Protocol` | |
| `Result<T, E>` | Raise exceptions or `Optional[T]` | |
| `Vec<T>` | `list[T]` | |
| `&[T]` / `&mut [T]` | Pass the list directly | Python mutables are by-reference |
| `Option<T>` | `T \| None` | |
| `iter().map().collect()` | `[f(x) for x in items]` or for loop | |
| `match` | `if/elif/else` | |
| Lifetimes | Ignore | Python has GC |
| `impl<T: Field>` | Concrete types | Prefer monomorphized specs |
| `pub(crate)` | Leading underscore `_foo` | |

## C++ → Python

| C++ | Python | Notes |
|-----|--------|-------|
| `.h` + `.cpp` | Single class definition | |
| `template<T>` | Concrete classes | |
| RAII / destructors | Context managers or GC | |
| `T*` pointer arithmetic | `array[i]` | |
| `std::vector<T>` | `list[T]` | |
| `const T&` | Document immutability | Python has no const |
| `#define CONST 42` | `CONST = 42` at module level | |
| `namespace` | Module/package | |

## Field Arithmetic

- Be explicit about modular arithmetic: `(a * b) % p`
- Document the field at the top of `field.py`
- Use a field library (`galois`) to eliminate manual modular arithmetic
- Translate extension field operations exactly as in source

## Handling Generics

Monomorphize: pick the concrete types used in production and translate only those.

```python
# Source (generic over field, hash, PCS)
# fn commit<F: Field, H: Hasher<F>, P: PCS<F, H>>(...) { ... }

# Spec (concrete)
def commit(trace: list[BabyBear], ...) -> MerkleCommitment:
    """Source: crates/stark-backend/src/prover.rs:142"""
```

## Traceability

Every translated function cites its origin:

```python
def fold_matrix(mat: list[list[FF3]], beta: FF3) -> list[FF3]:
    """Fold a matrix of evaluations using random linear combination.

    Source: crates/stark-backend/src/fri/prover.rs:87-112
    """
```

## What NOT to Translate

Build system, CLI, benchmarks, platform-specific code (SIMD, CUDA), logging infrastructure, CI/CD, documentation generators.

*Enforced by: faithful-spec-translator agent.*

---

# Chapter 10: Agent Enforcement Pipeline

The style guide is enforced by a system of 8 specialist agents. Each agent guards a specific dimension of spec quality. Together, they provide comprehensive coverage with deliberate overlap for critical concerns.

## Agent Roster

| Agent | Model | Role | Chapters Enforced |
|-------|-------|------|-------------------|
| **faithful-spec-translator** | opus | Phase 2 translation | 2, 9 |
| **crypto-spec-simplifier** | opus | Phase 3 simplification | 2, 4, 6 |
| **paranoid-skeptic** | opus | Regression detection | 7 |
| **protocol-purity-guardian** | sonnet | Protocol/implementation boundary | 3, 6 |
| **zksnark-python-style** | sonnet | Code style enforcement | 4 |
| **human-simplicity-enforcer** | sonnet | Human readability | 1, 4 |
| **type-enforcer** | haiku | Type alias consistency | 5 |
| **spec-sync-guardian** | sonnet | Documentation sync | All |

## Phase-Specific Agent Usage

**Phase 1 (Testing):**
- `paranoid-skeptic` reviews the test infrastructure

**Phase 2 (Translation):**
- `faithful-spec-translator` performs the translation
- `paranoid-skeptic` verifies binary equivalence after each module

**Phase 3 (Simplification):**
- `crypto-spec-simplifier` performs refactoring
- All review agents run after each significant change
- `paranoid-skeptic` provides final verification

## Post-Change Review Pipeline

After any significant change to spec code, run this pipeline:

```
1. Read and plan changes
2. Run in parallel:
   ├── type-enforcer          (type alias violations)
   ├── crypto-spec-simplifier (over-abstraction, dead code)
   ├── human-simplicity-enforcer (readability issues)
   └── protocol-purity-guardian  (implementation leaks)
3. Update plan based on feedback
4. Get user approval
5. Implement changes, run tests after each
6. Final paranoid-skeptic verification (all tests, binary equivalence)
```

Steps 2's agents run in parallel because they review independently. Step 6 is never skipped — the paranoid-skeptic is the final gate.

## Agent Invocation Rules

- **Pass context in prompts**: Tell agents what changed and why. Do not make them re-read files you have already read.
- **Request concise responses**: Bullet points, not essays. Line numbers and fix suggestions.
- **Run independent reviews in parallel**: The 4 review agents (step 2 above) have no dependencies on each other.
- **Never skip the paranoid-skeptic**: It is the final gate. Every change must pass all tests with binary equivalence.

## Agent Complementarity

Some agents have overlapping concerns. The overlap is intentional — it provides defense in depth.

**zksnark-python-style vs human-simplicity-enforcer**: The style agent enforces specific syntactic rules (no walrus operator, explicit loops, docstring format). The human-simplicity agent evaluates the subjective cognitive experience (chunk size, abstraction depth, linear readability). A function can pass style checks while still being too complex for a human to follow comfortably.

**protocol-purity-guardian vs crypto-spec-simplifier**: The guardian polices the `protocol/` vs `primitives/` boundary at an architectural level. The simplifier actively restructures code to make protocol logic clearer. The guardian says "this doesn't belong here." The simplifier says "here's how to restructure it."

**type-enforcer vs zksnark-python-style**: The type-enforcer focuses specifically on the semantic type alias system (`FF`, `FF3Poly`, etc.). The style agent enforces broader type annotation coverage (`mypy --strict`). The type-enforcer catches `np.ndarray` where `FF3` should be; the style agent catches missing annotations entirely.

**paranoid-skeptic vs everyone else**: The paranoid-skeptic does not review code quality — it reviews correctness. All other agents can approve a change on quality grounds; if the paranoid-skeptic finds a test failure, the change is rejected regardless.

## Adapting Agents for New Projects

When starting a new spec project, adapt each agent definition by updating:

1. **Field type names**: Replace `FF`, `FF3`, `Goldilocks` with project-specific types
2. **Directory paths**: Update references to `executable-spec/`, `protocol/`, `primitives/`
3. **Test counts**: Update "all 142 tests" references to actual test count
4. **Source language references**: Update C++ references to Rust (or vice versa)
5. **AIR type lists**: Replace `SimpleLeft, Lookup2_12, Permutation1_6` with project-specific AIRs
6. **Tool commands**: Update `uv run pytest` paths and `run-tests.sh` locations

The agent architectures and principles remain unchanged — they are generic to any zkSNARK spec project. Only the project-specific details need updating.

---

# Appendix A: Quick Reference Checklist

Use this checklist when reviewing spec code:

### Code Style
- [ ] Each line is one logical operation (no dense nesting)
- [ ] Type aliases used for all field types in `protocol/`
- [ ] Functions compute one thing and are named for what they return
- [ ] No walrus operator, multi-clause comprehensions, or implicit coercion
- [ ] Single-letter variables have citations
- [ ] Public functions have docstrings with Args, Returns, Reference
- [ ] Blank lines separate logical phases
- [ ] Explicit loops instead of `map`/`filter`/`reduce`
- [ ] No abbreviations (except cited math notation)
- [ ] No magic numbers — constants are named at module level

### Architecture
- [ ] Protocol code contains no NTT, hash implementations, or buffer management
- [ ] Type aliases defined in `primitives/field.py`, not locally
- [ ] Each constraint module is self-contained in `constraints/`
- [ ] No `np.ndarray` for field elements in `protocol/` code

### Testing
- [ ] All tests pass
- [ ] Binary equivalence confirmed with source implementation
- [ ] No `pytest.skip()` in test bodies (use `xfail` instead)
- [ ] Every bug fix has an accompanying regression test

### Translation (Phase 2 only)
- [ ] Every source public function has a spec counterpart
- [ ] Docstrings cite source file and line numbers
- [ ] No simplification introduced during translation

---

# Appendix B: Per-Project Adaptation

When applying this guide to a new speccing target:

### 1. Field Configuration
- Identify base field (BabyBear, Goldilocks, Mersenne31, etc.)
- Identify extension field and degree
- Define type alias set in `primitives/field.py`
- Choose a field library (`galois`, custom, etc.)

### 2. Source Language
- Determine whether source is Rust, C++, or other
- Apply the translation table from [Chapter 9](#chapter-9-source-translation-patterns)
- Set up cross-compilation or FFI for binary comparison testing

### 3. Test Infrastructure
- Build `generate-test-vectors.sh` for the source implementation
- Determine proof serialization format for binary comparison
- Identify all AIR types to cover

### 4. Agent Definitions
- Copy agent definitions from a reference project
- Update field type names, paths, test counts, and AIR lists
- Place in `.claude/agents/`
- Reference this style guide from the project's `CLAUDE.md`

### 5. Project Structure
```
new-project/
├── executable-spec/
│   ├── protocol/
│   ├── primitives/
│   ├── constraints/
│   ├── tests/
│   │   └── test-data/
│   ├── pyproject.toml
│   ├── setup.sh
│   ├── run-tests.sh
│   └── generate-test-vectors.sh
├── .claude/
│   ├── agents/          # 8 agent definitions
│   └── commands/        # simplify-spec and other workflows
├── CLAUDE.md            # References SPEC_STYLE_GUIDE.md
└── SPEC_STYLE_GUIDE.md  # This document (or symlink)
```

### 6. Workflow Initialization
1. Set up test infrastructure (Phase 1)
2. Generate golden vectors from source
3. Begin bottom-up translation (Phase 2)
4. Run paranoid-skeptic after each module
5. Begin simplification (Phase 3) only after full translation passes all tests
