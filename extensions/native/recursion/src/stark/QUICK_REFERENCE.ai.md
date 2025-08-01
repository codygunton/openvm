# STARK Recursion Quick Reference

## Core Types

### VerifierProgram
```rust
VerifierProgram<C: Config>
```
- Builds verification programs for STARK proofs
- Main entry point for creating verifiers

### StarkVerifier
```rust
StarkVerifier<C: Config>
```
- Core verification logic
- Handles constraint checking and proof validation

## Key Functions

### Building a Verifier
```rust
// Basic build
let program = VerifierProgram::<InnerConfig>::build(advice, &fri_params);

// With options
let program = VerifierProgram::build_with_options(advice, &fri_params, options);
```

### Static Verification (EVM)
```rust
#[cfg(feature = "static-verifier")]
let ops = build_circuit_verify_operations(advice, &fri_params, &proof);
```

## Configuration Structures

### MultiStarkVerificationAdvice
```rust
MultiStarkVerificationAdvice<C> {
    per_air: Vec<StarkVerificationAdvice<C>>,
    pre_hash: CryptographicDigest,
    log_up_pow_bits: usize,
    num_challenges_to_sample: Vec<usize>,
    trace_height_constraint_system: TraceHeightConstraintSystem<C>,
}
```

### StarkVerificationAdvice
```rust
StarkVerificationAdvice<C> {
    preprocessed_data: Option<Commitment>,
    width: StarkTraceWidth,
    log_quotient_degree: usize,
    num_public_values: usize,
    num_challenges_to_sample: Vec<usize>,
    num_exposed_values_after_challenge: Vec<usize>,
    symbolic_constraints: SymbolicExpressionDag<C::F>,
}
```

### StarkTraceWidth
```rust
StarkTraceWidth {
    preprocessed: Option<usize>,
    cached_mains: Vec<usize>,
    common_main: usize,
    after_challenge: Vec<usize>,
}
```

## Common Patterns

### Single AIR Verification
```rust
let advice = MultiStarkVerificationAdvice {
    per_air: vec![single_air_advice],
    // ... other fields
};
let program = VerifierProgram::build(advice, &fri_params);
```

### Multi-Trace Verification
```rust
let advice = MultiStarkVerificationAdvice {
    per_air: vec![air1_advice, air2_advice, air3_advice],
    // Traces automatically sorted by height
    // ... other fields
};
```

### With Preprocessing
```rust
StarkVerificationAdvice {
    preprocessed_data: Some(preprocessed_commitment),
    width: StarkTraceWidth {
        preprocessed: Some(8),  // 8 preprocessed columns
        // ...
    },
    // ...
}
```

### With After-Challenge Phase
```rust
StarkVerificationAdvice {
    width: StarkTraceWidth {
        after_challenge: vec![4],  // 4 columns after challenge
    },
    num_challenges_to_sample: vec![2],  // 2 challenges
    num_exposed_values_after_challenge: vec![1],  // 1 exposed value
    // ...
}
```

## Compiler Options

```rust
CompilerOptions {
    enable_cycle_tracker: true,      // Performance profiling
    enable_debug_trace: false,       // Debug output
    optimization_level: Default,     // Optimization level
}
```

## FRI Parameters

```rust
FriParameters {
    log_blowup: 3,              // Blowup factor (2^3 = 8x)
    num_queries: 100,           // Number of query rounds
    proof_of_work_bits: 8,      // PoW difficulty
}
```

## Validation Tags

The verifier uses tags to track validations:
- **T01a**: AIR IDs subsequence validation
- **T01b**: Challenge phase count validation
- **T01c**: Exposed values shape validation
- **T02a**: Permutation validation
- **T02b**: Trace height bounds validation
- **T02c**: Height ordering validation
- **T03a**: Public values shape validation
- **T04a**: Main trace commits shape validation
- **T05a-c**: Opening values shape validations

## Error Conditions

Common panic scenarios:
- More than 1 challenge phase (not supported)
- Invalid AIR permutations
- Trace height exceeds maximum (2^24 with standard blowup)
- Mismatched commitment counts
- Failed constraint evaluation

## Performance Tips

1. **Enable cycle tracking** for profiling
2. **Use static mode** when parameters are compile-time known
3. **Batch similar AIRs** in multi-trace proofs
4. **Optimize FRI parameters** based on security needs

## Feature Flags

- `static-verifier`: Enables Halo2 circuit generation
- `evm-prove`: EVM proof generation
- `evm-verify`: EVM proof verification with REVM
- `test-utils`: Testing utilities
- `bench-metrics`: Performance metrics

## Quick Examples

### Minimal Verifier
```rust
let advice = create_minimal_advice();
let fri_params = default_fri_params();
let program = VerifierProgram::build(advice, &fri_params);
```

### Production Verifier
```rust
let options = CompilerOptions {
    enable_cycle_tracker: false,
    optimization_level: OptimizationLevel::Aggressive,
};
let program = VerifierProgram::build_with_options(
    production_advice,
    &production_fri_params,
    options
);
```

### EVM Verifier
```rust
#[cfg(feature = "static-verifier")]
{
    let outer_advice = convert_to_outer_config(advice);
    let circuit_ops = build_circuit_verify_operations(
        outer_advice,
        &fri_params,
        &bn254_proof
    );
}
```

## Debugging Commands

```rust
// Enable all debug features
let options = CompilerOptions {
    enable_cycle_tracker: true,
    enable_debug_trace: true,
    // ...
};

// Check constraint evaluation
// Look for folded_constraints calculation in verify_single_rap_constraints

// Monitor memory usage
// Verifier uses sub-builders for memory recycling in dynamic mode
```

## Common Imports

```rust
use openvm_native_recursion::{
    stark::{VerifierProgram, StarkVerifier},
    types::{MultiStarkVerificationAdvice, StarkVerificationAdvice},
    config::InnerConfig,
};
use openvm_stark_sdk::config::{
    baby_bear_poseidon2::BabyBearPoseidon2Config,
    FriParameters,
};
use openvm_native_compiler::prelude::*;
```