# STARK Recursion Component - Detailed Documentation

## Table of Contents
1. [Overview](#overview)
2. [Core Components](#core-components)
3. [Verification Flow](#verification-flow)
4. [Implementation Details](#implementation-details)
5. [Key Algorithms](#key-algorithms)
6. [Error Handling & Validation](#error-handling--validation)

## Overview

The STARK recursion component enables recursive verification of STARK proofs within the OpenVM zkVM. It implements a complete STARK verifier that can run as a program inside the VM, allowing for proof composition and aggregation.

## Core Components

### VerifierProgram

The `VerifierProgram` struct is responsible for building verification programs that can be executed in the OpenVM VM.

```rust
pub struct VerifierProgram<C: Config> {
    _phantom: PhantomData<C>,
}
```

Key methods:
- `build()`: Creates a verification program with default compiler options
- `build_with_options()`: Creates a verification program with custom compiler options

The program flow:
1. Reads proof from input using witness generation
2. Initializes FRI PCS configuration
3. Calls the main verification logic
4. Compiles to ISA instructions

### StarkVerifier

The main verification logic implementation:

```rust
pub struct StarkVerifier<C: Config> {
    _phantom: PhantomData<C>,
}
```

Primary verification methods:
- `verify()`: Main entry point that manages memory allocation
- `verify_raps()`: Verifies Randomized AIRs with Preprocessing
- `verify_single_rap_constraints()`: Verifies constraints for a single AIR

### Outer Verifier Module

For static verification with BN254 field:
- `build_circuit_verify_operations()`: Generates Halo2 circuit operations
- Used for final EVM/on-chain verification

## Verification Flow

### 1. Proof Structure Validation

The verifier first validates the proof structure:
- AIR IDs must be a valid subsequence
- Number of challenges and phases must match expectations
- Commitment shapes must be correct

### 2. Challenge Generation

Using Fiat-Shamir transformation:
1. Observe pre-computed hash
2. Observe AIR IDs
3. Observe public values
4. Sample challenges for each phase
5. Handle proof-of-work if required

### 3. Domain Construction

For each AIR:
- Build evaluation domains based on trace height
- Create quotient domains with appropriate coset shifts
- Split quotient domains into chunks

### 4. Opening Rounds Organization

Organizes polynomial openings into rounds:
1. Preprocessed trace openings
2. Main trace openings (cached and common)
3. After-challenge trace openings
4. Quotient polynomial openings

### 5. PCS Verification

Verifies all polynomial commitments using FRI-based PCS:
- Validates opening proofs
- Checks consistency across rounds
- Verifies permutations

### 6. Constraint Verification

For each AIR:
- Evaluates symbolic constraints
- Verifies quotient polynomial reconstruction
- Checks zerofier relationships

## Implementation Details

### Multi-Trace Handling

The verifier supports multiple AIR traces with different heights:
- Traces are sorted by height (descending)
- Permutation arrays track the sorting
- Common main traces are batched together

### Memory Management

Two modes of operation:
1. **Static mode**: Direct verification
2. **Dynamic mode**: Uses sub-builders for memory recycling

### Challenge Phases

Currently supports 0 or 1 challenge phase:
- Phase 0: Main trace commitments
- Phase 1: After proof-of-work challenges

### Height Constraints

Implements a constraint system for trace heights:
```
a_i1 * h_1 + ... + a_ik * h_k < b_i
```
Where `h_i` is the height of trace `i`.

## Key Algorithms

### Quotient Reconstruction

The quotient polynomial is reconstructed from chunks:

```rust
fn recompute_quotient(
    quotient_chunks: &[Vec<Ext<F, EF>>],
    qc_domains: Vec<TwoAdicMultiplicativeCosetVariable<C>>,
    zeta: Ext<F, EF>,
) -> Ext<F, EF>
```

Uses Lagrange interpolation with zerofier polynomials.

### Constraint Evaluation

Constraints are evaluated using a folding approach:

```rust
fn eval_constraints(
    constraints: &SymbolicExpressionDag<F>,
    // ... trace values
    alpha: Ext<F, EF>,
    // ... other parameters
) -> Ext<F, EF>
```

### Cumulative Sum Verification

For LogUp arguments, verifies that cumulative sums across all AIRs sum to zero.

## Error Handling & Validation

### Shape Validations

The verifier performs extensive shape validations marked with tags (T01a, T02a, etc.):
- T01a: AIR IDs validation
- T02a: Permutation validation
- T03a: Public values shape
- T04a: Commitment shapes
- T05a-c: Opening values shapes

### Assertions

Key assertions throughout:
- Trace heights within bounds
- Correct number of commitments
- Valid permutations
- Matching quotient degrees

### Panic Conditions

The verifier panics on:
- More than 1 challenge phase (current limitation)
- Invalid proof structures
- Constraint evaluation failures

## Performance Considerations

### Cycle Tracking

The implementation includes cycle tracking for profiling:
- Reading proof from input
- PCS initialization
- Building rounds
- PCS verification
- Constraint verification

### Optimization Strategies

1. **Batched Operations**: Common traces are committed together
2. **Memory Reuse**: Sub-builders recycle heap space
3. **Static Specialization**: Compile-time optimizations for known parameters

## Security Properties

### Soundness

The verifier ensures computational soundness through:
- Proper Fiat-Shamir challenge generation
- Complete constraint verification
- FRI soundness parameters

### Completeness

Valid proofs are accepted through:
- Correct domain evaluations
- Proper quotient reconstruction
- Accurate constraint folding

## Integration Examples

### Basic Verification

```rust
let constants = MultiStarkVerificationAdvice { /* ... */ };
let fri_params = FriParameters { /* ... */ };
let program = VerifierProgram::build(constants, &fri_params);
```

### With Custom Options

```rust
let options = CompilerOptions {
    enable_cycle_tracker: true,
    // ... other options
};
let program = VerifierProgram::build_with_options(
    constants, 
    &fri_params, 
    options
);
```

### Static Verification

```rust
let operations = build_circuit_verify_operations(
    advice,
    &fri_params,
    &proof
);
```

## Future Enhancements

Potential improvements marked in code:
- Support for multiple challenge phases
- Dynamic AIR selection
- Optimized constraint evaluation
- Enhanced memory management