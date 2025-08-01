# Halo2 Component Documentation

## Architecture Overview

The Halo2 component bridges OpenVM's STARK-based proving system with Halo2 SNARKs, enabling efficient on-chain verification. It follows a three-stage architecture:

1. **STARK to Halo2 Conversion**: Converts STARK proofs into Halo2 circuit witnesses
2. **Halo2 Proving**: Generates SNARK proofs using KZG commitments
3. **EVM Deployment**: Optionally generates EVM-compatible verifiers

## Core Components

### Halo2Prover

The main prover class that handles circuit construction and proof generation:

```rust
pub struct Halo2Prover;

impl Halo2Prover {
    // Creates a circuit builder with specified parameters
    pub fn builder(stage: CircuitBuilderStage, k: usize) -> BaseCircuitBuilder<Fr>
    
    // Populates builder with DSL operations and witness
    pub fn populate<C>(...) -> BaseCircuitBuilder<Fr>
    
    // Mock proving for testing
    pub fn mock<C>(...) -> Vec<Vec<Fr>>
    
    // Key generation
    pub fn keygen<C>(...) -> Halo2ProvingPinning
    
    // Proof generation
    pub fn prove<C>(...) -> Snark
}
```

### Halo2ProvingPinning

Contains proving key and metadata needed for proof generation:

```rust
pub struct Halo2ProvingPinning {
    pub pk: ProvingKey<G1Affine>,
    pub metadata: Halo2ProvingMetadata,
}

pub struct Halo2ProvingMetadata {
    pub config_params: BaseCircuitParams,
    pub break_points: MultiPhaseThreadBreakPoints,
    pub num_pvs: Vec<usize>, // Number of public values per column
}
```

### Verifier Circuit

The verifier module generates Halo2 circuits that verify STARK proofs:

```rust
pub struct Halo2VerifierProvingKey {
    pub pinning: Halo2ProvingPinning,
    pub dsl_ops: DslOperations<OuterConfig>,
}
```

### Wrapper Circuit

Aggregates multiple SNARKs and prepares them for EVM verification:

```rust
pub struct Halo2WrapperProvingKey {
    pub pinning: Halo2ProvingPinning,
}
```

## Key Workflows

### 1. Generating a Verifier Circuit

```rust
// Generate proving key for STARK verifier
let verifier_pk = generate_halo2_verifier_proving_key(
    &params,
    advice,
    &fri_params,
    &proof
);

// Prove STARK verification
let snark = verifier_pk.prove(&params, witness, profiling);
```

### 2. Mock Proving (Testing)

```rust
// Run mock prover to test circuit correctness
let public_instances = Halo2Prover::mock(
    k,
    dsl_operations,
    witness
);
```

### 3. EVM Verifier Generation

```rust
// Generate wrapper circuit
let wrapper_pk = Halo2WrapperProvingKey::keygen(&params, dummy_snark);

// Generate EVM verifier
let evm_verifier = wrapper_pk.evm_verifier(&vk);
```

## Configuration

### Circuit Parameters

- **k**: Circuit size parameter (2^k rows)
- **num_advice_per_phase**: Number of advice columns per phase
- **lookup_bits**: Bits used for lookup tables (typically k-1)

### Field Configuration

- Native field: BN254 scalar field (Fr)
- Base field: BabyBear
- Extension field: BinomialExtensionField<BabyBear, 4>

## Utilities

### Parameter Management

The `utils` module provides KZG parameter management:

```rust
pub trait Halo2ParamsReader {
    fn read_params(&self, k: usize) -> Arc<Halo2Params>;
}

pub struct CacheHalo2ParamsReader {
    // Caches loaded parameters for reuse
}
```

### Testing Utilities

Located in `testing_utils.rs`, provides helpers for:
- Generating test parameters
- Creating dummy proofs
- Circuit debugging

## Performance Considerations

1. **Circuit Size**: Choose k parameter based on constraint count
2. **Caching**: Use CacheHalo2ParamsReader to avoid reloading parameters
3. **Profiling**: Enable with `profiling = true` for metrics
4. **Break Points**: Serialized for consistent proof generation

## Security Notes

- Uses trusted setup from KZG ceremony
- Verifying key commits to circuit structure
- EVM verifier must be carefully audited before deployment
- Parameter files must be from trusted sources