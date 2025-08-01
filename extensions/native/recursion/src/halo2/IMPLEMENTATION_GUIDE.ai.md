# Halo2 Component Implementation Guide

## Overview

This guide provides detailed implementation instructions for working with the Halo2 component in OpenVM's native recursion framework.

## Core Implementation Patterns

### 1. Creating a Halo2 Prover Instance

The Halo2Prover is stateless and uses static methods:

```rust
use crate::halo2::{Halo2Prover, DslOperations};
use openvm_native_compiler::ir::Witness;

// Create DSL operations from your circuit
let dsl_ops = DslOperations {
    operations: traced_operations,
    num_public_values: 32, // Number of public outputs
};

// Create witness
let mut witness = Witness::default();
// Populate witness with your data
my_data.write(&mut witness);

// Generate proving key
let params = read_kzg_params(k);
let proving_key = Halo2Prover::keygen(&params, dsl_ops.clone(), witness.clone());
```

### 2. Implementing a Custom Verifier

To create a verifier for your specific proof system:

```rust
use crate::halo2::{generate_halo2_verifier_proving_key, Halo2VerifierProvingKey};
use openvm_stark_backend::proof::Proof;

pub struct MyVerifierProvingKey {
    inner: Halo2VerifierProvingKey,
}

impl MyVerifierProvingKey {
    pub fn new(
        params: &Halo2Params,
        advice: MultiStarkVerificationAdvice<MyConfig>,
        fri_params: &FriParameters,
        proof: &Proof<MyConfig>,
    ) -> Self {
        let inner = generate_halo2_verifier_proving_key(
            params,
            advice,
            fri_params,
            proof,
        );
        Self { inner }
    }
    
    pub fn prove(&self, params: &Halo2Params, witness: Witness<MyConfig>) -> Snark {
        self.inner.prove(params, witness, false)
    }
}
```

### 3. Circuit Builder Configuration

Configure the circuit builder for optimal performance:

```rust
let builder = Halo2Prover::builder(CircuitBuilderStage::Prover, k)
    .use_k(k)
    .use_lookup_bits(k - 1)  // Standard: k-1 bits for lookups
    .use_instance_columns(1); // Usually 1 instance column

// For custom configurations
let builder = builder
    .use_params(custom_params)
    .use_break_points(saved_break_points);
```

### 4. Witness Management

Properly structure witness data for the circuit:

```rust
use openvm_native_compiler::ir::Witness;
use crate::witness::Witnessable;

// Implement Witnessable for your types
impl<C: Config> Witnessable<C> for MyProofData {
    fn write(&self, witness: &mut Witness<C>) {
        witness.write_valu(self.commitment);
        witness.write_array(&self.openings);
        // Write all necessary data
    }
}

// Create witness
let mut witness = Witness::default();
proof_data.write(&mut witness);
```

### 5. EVM Integration

Generate EVM-compatible proofs and verifiers:

```rust
// Generate wrapper circuit for EVM
let dummy_snark = verifier_pk.generate_dummy_snark(&params_reader);
let wrapper_pk = Halo2WrapperProvingKey::keygen(&params, dummy_snark);

// Generate actual proof
let snark = verifier_pk.prove(&params, witness, false);
let wrapped_snark = wrapper_pk.prove(&params, snark);

// Generate EVM verifier
#[cfg(feature = "evm-prove")]
let evm_verifier = wrapper_pk.evm_verifier(wrapper_pk.pinning.pk.get_vk());

// Convert to EVM proof format
let evm_proof = wrapped_snark.to_evm_proof();
```

## Advanced Patterns

### Parameter Management

Implement efficient parameter loading:

```rust
use crate::halo2::utils::{CacheHalo2ParamsReader, Halo2ParamsReader};

// Create cached reader
let params_reader = CacheHalo2ParamsReader::new("./params");

// Auto-select k for wrapper circuit
let wrapper_pk = Halo2WrapperProvingKey::keygen_auto_tune(
    &params_reader,
    dummy_snark,
);
```

### Circuit Optimization

1. **Column Usage**: Minimize advice columns by packing data efficiently
2. **Lookup Tables**: Use lookup tables for repeated operations
3. **Break Points**: Save and reuse break points for consistent proving

```rust
// Save break points after keygen
let break_points = proving_key.metadata.break_points.clone();
save_break_points(&break_points, "circuit_break_points.json");

// Load for proving
let break_points: MultiPhaseThreadBreakPoints = load_break_points("circuit_break_points.json");
```

### Error Handling

Handle common errors gracefully:

```rust
use anyhow::{Context, Result};

pub fn prove_with_retry(
    params: &Halo2Params,
    proving_key: &Halo2ProvingPinning,
    witness: Witness<OuterConfig>,
) -> Result<Snark> {
    // Retry logic for transient failures
    for attempt in 0..3 {
        match Halo2Prover::prove(
            params,
            proving_key.metadata.config_params.clone(),
            proving_key.metadata.break_points.clone(),
            &proving_key.pk,
            dsl_ops.clone(),
            witness.clone(),
            false,
        ) {
            Ok(snark) => return Ok(snark),
            Err(e) if attempt < 2 => {
                eprintln!("Proving attempt {} failed: {:?}", attempt + 1, e);
                continue;
            }
            Err(e) => return Err(e).context("Failed to generate proof after retries"),
        }
    }
    unreachable!()
}
```

## Testing Strategies

### 1. Mock Proving

Always test with mock prover first:

```rust
#[test]
fn test_circuit_correctness() {
    let k = 16; // Start with smaller k for testing
    let public_instances = Halo2Prover::mock(k, dsl_ops, witness);
    
    // Verify public instances
    assert_eq!(public_instances[0].len(), expected_public_values);
    // Add more assertions
}
```

### 2. Integration Tests

Test the full proving pipeline:

```rust
#[test]
fn test_full_proving_pipeline() {
    // 1. Generate test STARK proof
    let stark_proof = generate_test_stark_proof();
    
    // 2. Create verifier circuit
    let verifier_pk = generate_halo2_verifier_proving_key(
        &params,
        advice,
        &fri_params,
        &stark_proof,
    );
    
    // 3. Generate SNARK
    let snark = verifier_pk.prove(&params, witness, false);
    
    // 4. Verify the SNARK
    verify_snark(&dk, &snark);
}
```

## Performance Optimization

1. **Parallel Witness Generation**: Generate witness data in parallel when possible
2. **Circuit Caching**: Cache compiled circuits and proving keys
3. **Parameter Selection**: Choose minimal k that satisfies constraints
4. **Profiling**: Use `profiling = true` to identify bottlenecks

## Common Pitfalls

1. **Mismatched Field Elements**: Ensure correct conversion between BabyBear and BN254Fr
2. **Public Instance Ordering**: Maintain consistent ordering of public values
3. **Break Point Consistency**: Use same break points for keygen and proving
4. **Parameter File Corruption**: Verify parameter file integrity with checksums