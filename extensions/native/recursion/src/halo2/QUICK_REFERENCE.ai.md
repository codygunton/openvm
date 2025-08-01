# Halo2 Component Quick Reference

## Essential Imports

```rust
use crate::halo2::{
    Halo2Prover, Halo2Params, Halo2ProvingPinning,
    DslOperations, RawEvmProof,
    generate_halo2_verifier_proving_key,
    utils::{Halo2ParamsReader, CacheHalo2ParamsReader},
};
use openvm_native_compiler::ir::Witness;
use snark_verifier_sdk::Snark;
```

## Quick Start

### 1. Basic Proving Flow

```rust
// Load parameters
let params_reader = CacheHalo2ParamsReader::new("./params");
let params = params_reader.read_params(k);

// Create DSL operations
let dsl_ops = DslOperations {
    operations: your_operations,
    num_public_values: 32,
};

// Create witness
let mut witness = Witness::default();
your_data.write(&mut witness);

// Generate proving key
let pk = Halo2Prover::keygen(&params, dsl_ops.clone(), witness.clone());

// Generate proof
let snark = Halo2Prover::prove(
    &params,
    pk.metadata.config_params.clone(),
    pk.metadata.break_points.clone(),
    &pk.pk,
    dsl_ops,
    witness,
    false, // profiling
);
```

### 2. Mock Testing

```rust
// Quick circuit test
let public_instances = Halo2Prover::mock(k, dsl_ops, witness);
```

### 3. STARK Verifier Circuit

```rust
// Generate verifier for STARK proof
let verifier_pk = generate_halo2_verifier_proving_key(
    &params,
    advice,
    &fri_params,
    &stark_proof,
);

// Prove STARK verification
let snark = verifier_pk.prove(&params, witness, false);
```

## Common Operations

### Parameter Management

```rust
// Default params directory
let reader = CacheHalo2ParamsReader::new_with_default_params_dir();

// Custom directory
let reader = CacheHalo2ParamsReader::new("/path/to/params");

// Read specific k
let params = reader.read_params(20); // k=20
```

### Circuit Builder Configuration

```rust
// Standard configuration
let builder = Halo2Prover::builder(CircuitBuilderStage::Prover, k);

// With custom parameters
let builder = BaseCircuitBuilder::from_stage(stage)
    .use_k(k)
    .use_lookup_bits(k - 1)
    .use_instance_columns(1)
    .use_params(custom_params);
```

### EVM Integration

```rust
// Generate wrapper
let wrapper_pk = Halo2WrapperProvingKey::keygen(&params, dummy_snark);

// Auto-tune k for wrapper
let wrapper_pk = Halo2WrapperProvingKey::keygen_auto_tune(&reader, dummy_snark);

// Generate EVM proof
let evm_proof = snark.to_evm_proof();
let calldata = evm_proof.verifier_calldata();
```

## Key Types Reference

| Type | Purpose |
|------|---------|
| `Halo2Prover` | Main prover implementation |
| `Halo2ProvingPinning` | Proving key + metadata |
| `DslOperations<C>` | Compiled circuit operations |
| `Witness<C>` | Circuit witness data |
| `Snark` | Generated proof |
| `RawEvmProof` | EVM-compatible proof format |
| `Halo2VerifierProvingKey` | Verifier circuit proving key |
| `Halo2WrapperProvingKey` | Wrapper circuit for aggregation |

## Configuration Parameters

| Parameter | Typical Value | Description |
|-----------|--------------|-------------|
| k | 16-23 | Circuit size (2^k rows) |
| lookup_bits | k-1 | Bits for lookup tables |
| num_advice_per_phase | [variable] | Advice columns per phase |
| num_public_values | app-specific | Public outputs count |

## Environment Variables

```bash
# Use random SRS for testing only
export RANDOM_SRS=1

# Enable bench metrics
# Requires feature flag: bench-metrics
```

## Feature Flags

| Flag | Purpose |
|------|---------|
| `evm-prove` | Enable EVM prover generation |
| `evm-verify` | Enable EVM verification |
| `bench-metrics` | Enable performance metrics |

## Common Errors and Solutions

| Error | Solution |
|-------|----------|
| "Params file does not exist" | Ensure KZG params are downloaded to params directory |
| "Failed to satisfy constraint" | Check witness generation and circuit constraints |
| "Invalid bytes for proving key" | Ensure proving key matches circuit configuration |
| "Circuit too large" | Increase k parameter or optimize circuit |

## Testing Commands

```bash
# Run all tests
cargo test -p openvm-native-recursion

# Run specific test
cargo test -p openvm-native-recursion halo2::tests::

# With debug output
RUST_LOG=debug cargo test

# Mock prover only
cargo test mock_prove
```

## Performance Tips

1. **Cache Parameters**: Use `CacheHalo2ParamsReader` to avoid reloading
2. **Reuse Proving Keys**: Generate once, serialize, and reuse
3. **Optimize k**: Use smallest k that satisfies constraints
4. **Profile First**: Use mock prover to estimate circuit size
5. **Batch Operations**: Process multiple proofs in sequence with same setup