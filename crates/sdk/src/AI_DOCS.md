# OpenVM SDK - AI Documentation

## Overview

The OpenVM SDK is the primary interface for developers to interact with the OpenVM zkVM ecosystem. It provides a complete toolkit for building, executing, proving, and verifying zero-knowledge virtual machine programs. The SDK abstracts complex cryptographic operations while maintaining flexibility for advanced users.

## Architecture

### Core Design Principles

1. **Modular Proof System**: Supports multiple proof generation strategies
   - Direct STARK proofs for fast iteration
   - Aggregated proofs for efficiency
   - EVM-compatible proofs for on-chain verification

2. **Generic Engine Support**: Parameterized by STARK engine type
   - Default: `BabyBearPoseidon2Engine`
   - Extensible to other field/hash combinations

3. **Progressive Enhancement**: Features can be enabled as needed
   - Basic: Build, execute, prove
   - Advanced: Aggregation, recursion
   - EVM: On-chain verification

### Component Architecture

```
SDK Layer (User Interface)
    ↓
Configuration Layer
    ├── AppConfig (VM settings)
    ├── AggConfig (Aggregation settings)
    └── FriParameters (STARK parameters)
    ↓
Key Generation Layer
    ├── App Keys (per-program)
    └── Aggregation Keys (reusable)
    ↓
Proving Layer
    ├── App Prover (direct proofs)
    ├── STARK Prover (aggregated proofs)
    └── Halo2 Prover (EVM proofs)
    ↓
Verification Layer
    ├── Native Verification
    └── EVM Verification
```

## Key Concepts

### 1. Execution Commitment
Every VM execution is bound to a commitment that uniquely identifies:
- The program code
- Initial memory state
- Starting program counter

This commitment (`AppExecutionCommit`) ensures proof integrity.

### 2. Proof Aggregation Tree
The SDK implements a hierarchical proof aggregation system:
- **Leaf Level**: Aggregates application proofs
- **Internal Level**: Recursively aggregates leaf/internal proofs
- **Root Level**: Final aggregation for EVM verification

### 3. VM Configuration Flexibility
The SDK is generic over VM configurations (`VmConfig<F>`), allowing:
- Custom instruction sets
- Specialized peripherals
- Domain-specific optimizations

## Workflow

### Standard Development Flow

1. **Build Phase**
   ```rust
   let elf = sdk.build(guest_opts, vm_config, pkg_dir, target_filter, init_file);
   ```
   Compiles Rust guest code to ELF binary.

2. **Transpilation Phase**
   ```rust
   let exe = sdk.transpile(elf, transpiler);
   ```
   Converts ELF to VM-executable format.

3. **Commitment Phase**
   ```rust
   let committed_exe = sdk.commit_app_exe(fri_params, exe);
   ```
   Creates cryptographic commitment to executable.

4. **Key Generation Phase**
   ```rust
   let app_pk = sdk.app_keygen(config);
   ```
   Generates proving/verifying keys (one-time per config).

5. **Proving Phase**
   ```rust
   let proof = sdk.generate_app_proof(app_pk, committed_exe, inputs);
   ```
   Generates zero-knowledge proof of execution.

6. **Verification Phase**
   ```rust
   let payload = sdk.verify_app_proof(&app_vk, &proof);
   ```
   Verifies proof and extracts public values.

### EVM Integration Flow

For on-chain verification, additional steps are required:

1. **Aggregation Key Generation**
   ```rust
   let agg_pk = sdk.agg_keygen(config, params_reader, pv_handler);
   ```
   One-time setup for aggregation infrastructure.

2. **EVM Proof Generation**
   ```rust
   let evm_proof = sdk.generate_evm_proof(reader, app_pk, app_exe, agg_pk, inputs);
   ```
   Converts STARK proof to EVM-verifiable format.

3. **Verifier Contract Generation**
   ```rust
   let verifier = sdk.generate_halo2_verifier_solidity(reader, &agg_pk);
   ```
   Generates Solidity verifier contract.

## Memory Model

The SDK manages several memory concepts:

1. **Guest Memory**: The VM's execution memory space
2. **Public Values**: Data exposed from private computation
3. **Merkle Memory**: Authenticated memory for large datasets

Memory dimensions and public value counts are configured per application.

## Performance Considerations

### Proof Generation Costs
- **App Proofs**: Fast, suitable for development
- **Aggregated Proofs**: Slower but more efficient to verify
- **EVM Proofs**: Highest cost, optimized for on-chain gas

### Key Generation Costs
- **App Keys**: Quick, per-program basis
- **Aggregation Keys**: Very expensive (>10GB memory, >10 minutes)
  - Should be generated once and reused
  - Can be shared across applications with same parameters

### Parallelization
The SDK leverages parallelism through:
- Multi-threaded proof generation
- Concurrent proof aggregation
- Batch verification operations

## Security Model

### Commitment Security
- All proofs are bound to execution commitments
- Commitments use Poseidon2 hash function
- Merkle trees authenticate memory state

### Proof Security
- STARK proofs provide ~100 bits of security
- FRI parameters are configurable for security/performance tradeoff
- EVM proofs inherit security from both STARK and SNARK systems

### Verification Requirements
Verifiers must check:
1. Proof validity (cryptographic soundness)
2. Execution commitment matches expected program
3. Exit code is 0 (successful execution)
4. Public values match expected output

## Extension Points

### Custom VM Configurations
Implement `VmConfig<F>` to define custom:
- Instruction sets
- Memory layouts
- Peripheral devices

### Custom Transpilers
Create domain-specific transpilers for:
- Optimized instruction selection
- Specialized memory management
- Custom calling conventions

### Custom Proof Handlers
Implement proof post-processing for:
- Alternative aggregation strategies
- Custom verification logic
- Integration with external systems

## Best Practices

### Development
1. Start with direct app proofs for rapid iteration
2. Use small FRI parameters during development
3. Enable aggregation only when needed
4. Cache proving keys between runs

### Production
1. Generate aggregation keys in advance
2. Use production FRI parameters (100+ bit security)
3. Implement proper key management
4. Monitor proof generation metrics

### Integration
1. Verify commitment compatibility
2. Handle proof generation failures gracefully
3. Implement retry logic for transient failures
4. Log proof generation statistics

## Troubleshooting

### Common Issues

1. **Out of Memory During Keygen**
   - Increase system memory
   - Use smaller aggregation parameters
   - Enable disk-based swap

2. **Proof Generation Failures**
   - Check input format compatibility
   - Verify VM configuration matches executable
   - Ensure sufficient stack/heap in guest

3. **Verification Failures**
   - Confirm commitment matches executable
   - Check public value extraction
   - Verify FRI parameters match

### Debug Features
- Enable `profiling` feature for performance metrics
- Use `RUST_LOG=debug` for detailed logging
- Implement custom metrics collection