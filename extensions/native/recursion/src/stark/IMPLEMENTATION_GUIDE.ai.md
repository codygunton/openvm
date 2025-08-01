# STARK Recursion Implementation Guide

## Overview

This guide provides detailed implementation guidance for working with the STARK recursion component in OpenVM. It covers common use cases, best practices, and implementation patterns.

## Prerequisites

Before implementing STARK recursion:

1. **Understand STARK proofs**: Familiarity with STARK proof systems and their components
2. **OpenVM architecture**: Knowledge of OpenVM's circuit and compiler infrastructure
3. **Field arithmetic**: Understanding of BabyBear and BN254 field operations
4. **FRI protocol**: Basic knowledge of Fast Reed-Solomon IOP

## Basic Implementation

### Step 1: Define Verification Advice

```rust
use openvm_native_recursion::types::MultiStarkVerificationAdvice;
use openvm_native_recursion::config::InnerConfig;

// Create verification advice for your STARK configuration
let advice = MultiStarkVerificationAdvice::<InnerConfig> {
    per_air: vec![
        StarkVerificationAdvice {
            preprocessed_data: None,  // Or Some(commit) if using preprocessing
            width: StarkTraceWidth {
                preprocessed: None,
                cached_mains: vec![],
                common_main: 16,  // Your main trace width
                after_challenge: vec![],
            },
            log_quotient_degree: 2,
            num_public_values: 4,
            num_challenges_to_sample: vec![],
            num_exposed_values_after_challenge: vec![],
            // ... constraint system
        },
        // ... more AIRs if using multi-trace
    ],
    pre_hash: /* commitment to verification key */,
    log_up_pow_bits: 8,  // Proof of work difficulty
    // ... other fields
};
```

### Step 2: Configure FRI Parameters

```rust
use openvm_stark_sdk::config::FriParameters;

let fri_params = FriParameters {
    log_blowup: 3,
    num_queries: 100,
    proof_of_work_bits: 8,
    // ... other FRI parameters
};
```

### Step 3: Build Verification Program

```rust
use openvm_native_recursion::stark::VerifierProgram;

// Build the verification program
let program = VerifierProgram::<InnerConfig>::build(advice, &fri_params);

// Or with custom compiler options
let options = CompilerOptions {
    enable_cycle_tracker: true,
    enable_debug_trace: false,
    // ... other options
};
let program = VerifierProgram::build_with_options(advice, &fri_params, options);
```

### Step 4: Execute Verification

```rust
// The program can now be executed in the OpenVM VM
// It will read the proof from input and verify it
```

## Advanced Implementation

### Multi-Trace Verification

When verifying multiple AIR traces:

```rust
let multi_trace_advice = MultiStarkVerificationAdvice {
    per_air: vec![
        // AIR 0: High degree trace
        StarkVerificationAdvice {
            width: StarkTraceWidth {
                common_main: 32,
                // ...
            },
            log_quotient_degree: 3,
            // ...
        },
        // AIR 1: Lower degree trace
        StarkVerificationAdvice {
            width: StarkTraceWidth {
                common_main: 16,
                // ...
            },
            log_quotient_degree: 2,
            // ...
        },
    ],
    // ...
};
```

### Custom Challenge Phases

For proofs with after-challenge values:

```rust
StarkVerificationAdvice {
    width: StarkTraceWidth {
        common_main: 16,
        after_challenge: vec![8],  // 8 columns after challenge
    },
    num_challenges_to_sample: vec![2],  // Sample 2 challenges
    num_exposed_values_after_challenge: vec![1],  // Expose 1 value
    // ...
}
```

### Height Constraints

Implement trace height constraints:

```rust
use openvm_native_recursion::vars::TraceHeightConstraintSystem;

let constraint_system = TraceHeightConstraintSystem {
    height_constraints: vec![
        TraceHeightConstraint {
            coefficients: /* per-AIR coefficients */,
            threshold: /* max combined height */,
            is_threshold_at_p: false,
        },
    ],
    height_maxes: vec![
        Some(TraceHeightBound { value: 1 << 20 }),  // AIR 0 max height
        None,  // AIR 1 no specific limit
    ],
};
```

### Static Verification (EVM)

For generating EVM-verifiable proofs:

```rust
#[cfg(feature = "static-verifier")]
use openvm_native_recursion::stark::outer::build_circuit_verify_operations;

let outer_advice = /* convert to OuterConfig */;
let proof = /* your BN254 proof */;

let circuit_ops = build_circuit_verify_operations(
    outer_advice,
    &fri_params,
    &proof
);
```

## Implementation Patterns

### Pattern 1: Proof Aggregation

```rust
// Aggregate multiple proofs into one
pub fn aggregate_proofs(proofs: Vec<Proof>) -> Program<BabyBear> {
    let advice = MultiStarkVerificationAdvice {
        per_air: proofs.iter().map(|p| {
            // Extract verification parameters from each proof
            StarkVerificationAdvice { /* ... */ }
        }).collect(),
        // ...
    };
    
    VerifierProgram::build(advice, &fri_params)
}
```

### Pattern 2: Custom Constraint Systems

```rust
impl StarkVerificationAdvice {
    fn with_custom_constraints(mut self, constraints: SymbolicExpressionDag<F>) -> Self {
        self.symbolic_constraints = constraints;
        self
    }
}
```

### Pattern 3: Optimized Verification

```rust
// Use cycle tracking to optimize
let options = CompilerOptions {
    enable_cycle_tracker: true,
    // ...
};

// Build with optimization hints
let program = VerifierProgram::build_with_options(
    advice,
    &fri_params,
    options
);
```

## Best Practices

### 1. Validate Input Parameters

Always validate verification parameters:

```rust
fn validate_advice(advice: &MultiStarkVerificationAdvice<C>) {
    assert!(!advice.per_air.is_empty(), "No AIRs to verify");
    assert!(advice.num_challenges_to_sample.len() <= 1, "Max 1 challenge phase");
    
    for air in &advice.per_air {
        assert!(air.log_quotient_degree <= MAX_QUOTIENT_LOG_DEGREE);
        assert!(air.width.total_width() > 0);
    }
}
```

### 2. Memory Management

Use sub-builders for large verifications:

```rust
// The verifier automatically uses sub-builders in dynamic mode
// This recycles memory after verification
```

### 3. Error Handling

Handle verification failures gracefully:

```rust
// The verifier will panic on invalid proofs
// Wrap in error handling if needed
match std::panic::catch_unwind(|| {
    VerifierProgram::build(advice, &fri_params)
}) {
    Ok(program) => program,
    Err(_) => {
        // Handle verification failure
    }
}
```

### 4. Performance Optimization

- Enable cycle tracking for profiling
- Use static mode when parameters are known at compile time
- Batch similar AIRs together in multi-trace proofs

## Common Pitfalls

### 1. Incorrect Trace Heights

```rust
// WRONG: Trace height exceeds maximum
log_degree: 25,  // 2^25 is too large

// CORRECT: Within bounds
log_degree: 20,  // 2^20 is acceptable
```

### 2. Mismatched Challenge Counts

```rust
// WRONG: Mismatch between challenges and exposed values
num_challenges_to_sample: vec![2],
num_exposed_values_after_challenge: vec![3],  // Should match phase

// CORRECT: Aligned counts
num_challenges_to_sample: vec![2],
num_exposed_values_after_challenge: vec![1],  // One cumulative sum
```

### 3. Invalid Permutations

```rust
// The verifier expects air_perm_by_height to be a valid permutation
// sorted by decreasing trace height
```

## Testing Your Implementation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_verification() {
        let advice = create_test_advice();
        let program = VerifierProgram::build(advice, &default_fri_params());
        
        // Execute program with test proof
        // ...
    }
}
```

### Integration Tests

```rust
#[test]
fn test_multi_air_verification() {
    // Create complex multi-trace proof
    let proof = generate_multi_trace_proof();
    
    // Build verifier
    let advice = extract_advice_from_proof(&proof);
    let program = VerifierProgram::build(advice, &fri_params);
    
    // Verify execution succeeds
}
```

## Debugging Tips

### 1. Enable Cycle Tracking

```rust
let options = CompilerOptions {
    enable_cycle_tracker: true,
    // ...
};
```

### 2. Check Constraint Evaluation

Monitor constraint evaluation by examining the folded results.

### 3. Verify Domain Construction

Ensure domains are correctly constructed for each AIR's trace height.

## Performance Tuning

### 1. FRI Parameters

```rust
// Balance security and performance
let fri_params = FriParameters {
    log_blowup: 3,      // Lower = faster, higher = more secure
    num_queries: 100,   // Lower = faster, higher = more secure
    // ...
};
```

### 2. Trace Organization

- Group similar-height traces together
- Minimize the number of commitment rounds
- Use cached mains for frequently accessed columns

### 3. Compiler Optimization

```rust
let options = CompilerOptions {
    optimization_level: OptimizationLevel::Aggressive,
    // ...
};
```

## Next Steps

1. Study the example implementations in the tests
2. Understand the FRI protocol implementation
3. Learn about the Halo2 backend for EVM verification
4. Explore custom constraint systems for your use case