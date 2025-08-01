# OpenVM Native Recursion Extension - Quick Reference

## Common Imports

```rust
use openvm_native_recursion::{
    // Main types
    VerifierProgram, StarkVerifier,
    types::{InnerConfig, MultiStarkVerificationAdvice},
    
    // Challenger
    challenger::{DuplexChallengerVariable, ChallengerVariable},
    
    // FRI
    fri::TwoAdicFriPcsVariable,
    
    // Configuration
    config::outer::{OuterConfig, new_from_outer_multi_vk},
    
    // Variables
    vars::{StarkProofVariable, CommitmentsVariable},
};
```

## Building a Verifier Program

### Basic Verification Program

```rust
// 1. Convert verifying key to advice
let advice = new_from_inner_multi_vk(&vk);

// 2. Build the program
let program = VerifierProgram::<InnerConfig>::build(advice, &fri_params);

// 3. Execute with proof
let result = program.execute(proof_bytes);
```

### With Custom Options

```rust
let options = CompilerOptions {
    enable_cycle_tracker: true,
    ..Default::default()
};

let program = VerifierProgram::<InnerConfig>::build_with_options(
    advice, 
    &fri_params, 
    options
);
```

## Configuration Examples

### Inner Recursion (BabyBear)

```rust
use openvm_native_recursion::types::InnerConfig;

// Inner config is pre-defined for BabyBear field
type F = BabyBear;
type EF = BinomialExtensionField<BabyBear, 4>;
```

### Outer Recursion (Bn254)

```rust
use openvm_native_recursion::config::outer::OuterConfig;

// Convert VK for outer recursion
let outer_advice = new_from_outer_multi_vk(&vk);
```

## Verifier Usage

### Standalone Verification

```rust
let mut builder = Builder::<InnerConfig>::default();

// Initialize PCS
let pcs = TwoAdicFriPcsVariable {
    config: const_fri_config(&mut builder, fri_params),
};

// Create challenger
let mut challenger = DuplexChallengerVariable::new(&mut builder);

// Run verification
StarkVerifier::verify(
    &mut builder,
    &pcs,
    &advice,
    &proof,
);
```

### Multi-STARK Verification

```rust
// Advice contains multiple AIRs
let multi_advice = MultiStarkVerificationAdvice {
    per_air: vec![air1_advice, air2_advice],
    num_challenges_to_sample: vec![2],
    trace_height_constraints: constraints,
    log_up_pow_bits: 20,
    pre_hash: pre_hash_digest,
};

// Verification handles all AIRs
StarkVerifier::verify(&mut builder, &pcs, &multi_advice, &proof);
```

## Working with Proofs

### Reading Proof from Input

```rust
let proof: StarkProofVariable<C> = builder.uninit();
Proof::<SC>::witness(&proof, &mut builder);
```

### Accessing Proof Components

```rust
// Get commitments
let commitments = &proof.commitments;
let main_commits = &commitments.main_trace;
let quotient_commit = &commitments.quotient;

// Get per-AIR data
let air_proofs = &proof.per_air;
for i in 0..air_proofs.len() {
    let air_proof = builder.get(air_proofs, i);
    let public_values = air_proof.public_values;
}
```

## Challenger Patterns

### Basic Challenge Generation

```rust
let mut challenger = DuplexChallengerVariable::new(builder);

// Observe values
challenger.observe(builder, value);
challenger.observe_slice(builder, values);
challenger.observe_digest(builder, commitment);

// Sample challenges
let challenge: Felt<_> = challenger.sample(builder);
let ext_challenge: Ext<_, _> = challenger.sample_ext(builder);
```

### Proof of Work

```rust
// Check proof of work witness
challenger.check_witness(
    builder, 
    log_pow_bits,    // e.g., 20
    pow_witness      // Nonce that satisfies PoW
);
```

## FRI Patterns

### Domain Creation

```rust
let log_degree = 10; // 2^10 trace size
let domain = pcs.natural_domain_for_log_degree(builder, log_degree);

// Get points for opening
let trace_points = builder.array::<Ext<_, _>>(2);
builder.set_value(&trace_points, 0, zeta);
builder.set_value(&trace_points, 1, domain.next_point(builder, zeta));
```

### FRI Verification

```rust
// Build rounds for FRI
let rounds = /* construct rounds */;

// Verify FRI proof
pcs.verify(
    builder,
    rounds,
    fri_proof,
    log_max_height,
    &mut challenger,
);
```

## Common Builder Patterns

### Array Operations

```rust
// Create array
let arr: Array<C, Felt<_>> = builder.array(size);

// Set value
builder.set_value(&arr, index, value);

// Get value
let val = builder.get(&arr, index);

// Iterate
builder.range(0, size).for_each(|i_vec, builder| {
    let i = i_vec[0];
    let elem = builder.get(&arr, i);
    // Process elem
});
```

### Conditional Logic

```rust
builder.if_eq(a, b).then(|builder| {
    // Execute if a == b
});

builder.if_ne(a, b).then_or_else(
    |builder| {
        // Execute if a != b
    },
    |builder| {
        // Execute if a == b
    },
);
```

### Performance Tracking

```rust
builder.cycle_tracker_start("MyOperation");
// ... operation code
builder.cycle_tracker_end("MyOperation");
```

## Halo2 Integration (Feature-gated)

```rust
#[cfg(feature = "static-verifier")]
{
    use openvm_native_recursion::halo2::AggregationCircuit;
    
    let circuit = AggregationCircuit::new(
        &params,
        &stark_vk,
        &proof,
    );
    
    // Generate Halo2 proof
    let halo2_proof = circuit.create_proof(&pk);
}
```

## Error Handling

### Assertion Patterns

```rust
// Size assertions
builder.assert_usize_eq(actual_size, expected_size);

// Field element assertions
builder.assert_felt_eq(a, b);

// Extension field assertions
builder.assert_ext_eq(ext_a, ext_b);
```

### Bounds Checking

```rust
// Check array bounds
builder.assert_less_than_slow_small_rhs(index, array_len);

// Check value ranges
if value >= max_value {
    builder.error("Value out of range");
}
```

## Testing Utilities

```rust
#[cfg(test)]
mod tests {
    use openvm_native_recursion::testing_utils::*;
    
    #[test]
    fn test_verification() {
        let (vk, proof) = generate_test_proof();
        let advice = new_from_inner_multi_vk(&vk);
        
        let program = VerifierProgram::<InnerConfig>::build(
            advice,
            &fri_params,
        );
        
        assert!(program.execute(proof).is_ok());
    }
}
```

## Performance Tips

1. **Use Sub-builders**: For temporary computations to manage memory
2. **Batch Operations**: Process multiple items in single loops
3. **Precompute Constants**: Use `const_fri_config` for static configs
4. **Enable Cycle Tracking**: Only in development for performance analysis
5. **Static Mode**: Use `builder.flags.static_only` for known sizes

## Common Pitfalls

1. **Forgetting to witness proof**: Always call `Proof::witness()` before verification
2. **Incorrect array sizing**: Ensure arrays match expected sizes
3. **Challenge ordering**: Observe commitments in correct order
4. **Domain parameters**: Check `log_blowup` and `log_n` compatibility
5. **Feature flags**: Enable necessary features for Halo2/EVM support