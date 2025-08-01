# Keygen Component Documentation

## Purpose
The keygen module generates proving and verifying keys for OpenVM's multi-level proof system. It manages the complex hierarchy of verifiers needed for efficient proof aggregation and optional on-chain verification.

## Architecture Overview

### Proof System Hierarchy
```
App Proof → Leaf Verifier → Internal Verifier → Root Verifier → [Static Verifier]
```

1. **Application Layer**: User programs generate proofs using app proving keys
2. **Leaf Verifier**: Verifies individual app proofs
3. **Internal Verifier**: Aggregates multiple leaf proofs
4. **Root Verifier**: Final aggregation with constant trace heights
5. **Static Verifier** (optional): Halo2-based verifier for Ethereum

### Key Generation Flow

#### App Proving Key Generation
1. Create VM with BabyBearPoseidon2 engine
2. Generate VM proving key
3. Validate constraint degrees against FRI parameters
4. Build leaf verifier program
5. Commit leaf program for verification

#### Aggregation Key Generation
1. Generate leaf VM proving key
2. Generate internal VM proving key
3. Build internal verifier program
4. Generate dummy proof to determine trace heights
5. Create root verifier with AIR permutation
6. Optionally generate Halo2 keys for EVM

## Key Components

### AIR Permutation (`perm.rs`)
Reorders Arithmetic Intermediate Representations (AIRs) by trace height for the root verifier:
- Sorts AIRs by height (descending)
- Tracks special AIRs (program, connector, public values)
- Ensures consistent ordering for static verification

### Assembly Generation (`asm.rs`)
Converts native OpenVM instructions to RISC-V assembly:
- Handles long-form instruction encoding
- Manages PC alignment with gap indicators
- Preserves instruction operands in assembly format

### Dummy Proof Generation (`dummy.rs`)
Generates minimal proofs for key generation:
- Determines trace heights without full execution
- Rounds heights to powers of 2
- Validates verifier configurations

## Implementation Details

### Proving Key Structures

```rust
pub struct AppProvingKey<VC> {
    pub leaf_committed_exe: Arc<NonRootCommittedExe>,
    pub leaf_fri_params: FriParameters,
    pub app_vm_pk: Arc<VmProvingKey<SC, VC>>,
}
```

### FRI Parameter Validation
The system validates that:
- Max constraint degree ≤ FRI max constraint degree
- Recursive verifier size fits within field constraints
- Trace heights don't exceed 2-adicity limits

### Memory Optimization
- Uses `Arc` for shared ownership of large structures
- Lazy initialization of Halo2 parameters
- Chunked proof processing for aggregation

## Usage Examples

### Generate App Proving Key
```rust
let config = AppConfig::<Rv32ImConfig> {
    app_fri_params: standard_fri_params_with_100_bits_security(),
    app_vm_config: Rv32ImConfig::default(),
    leaf_fri_params: standard_fri_params_with_100_bits_security(),
    compiler_options: Default::default(),
};
let app_pk = AppProvingKey::keygen(config);
```

### Generate Aggregation Keys
```rust
let agg_config = AggStarkConfig::default();
let agg_pk = AggStarkProvingKey::keygen(agg_config);
```

## Performance Considerations

### Memory Requirements
- App proving keys: ~100MB-1GB
- Aggregation STARK keys: ~1-5GB
- Halo2 proving keys: >10GB

### Optimization Strategies
1. Pre-compute and cache proving keys
2. Use appropriate FRI parameters for security level
3. Optimize trace heights with dummy proofs
4. Parallelize key generation where possible

## Security Notes

### Soundness Checks
1. Recursive verifier size validation
2. Log-up soundness constraints
3. FRI query bounds checking
4. Proper commitment binding

### Common Issues
- "Recursive verifier size may be too large": Adjust FRI parameters
- "May violate log up soundness": Reduce trace complexity
- Out of memory: Use streaming key generation

## Integration Points

### With Prover Module
- Proving keys used by `AppProver` and `StarkProver`
- Shared VM configurations
- Consistent FRI parameters

### With Config Module
- Reads `AppConfig` and `AggConfig`
- Validates configuration parameters
- Propagates compiler options

### With Continuations Module
- Uses verifier configurations
- Builds verifier programs
- Handles proof chunking

## Future Improvements
1. Streaming key generation for memory efficiency
2. Parallel AIR key generation
3. Key compression and serialization
4. Dynamic FRI parameter selection