# OpenVM SDK Component Documentation

## Overview

The OpenVM SDK is the high-level interface for interacting with the OpenVM zkVM framework. It provides a unified API for building, executing, proving, and verifying zero-knowledge virtual machine programs.

## Core Architecture

### Main Components

- **GenericSdk<E>** - Main SDK interface parameterized by STARK engine
- **AppProver** - Generates application-specific proofs  
- **StarkProver** - Handles STARK proof generation and aggregation
- **EvmHalo2Prover** - Generates EVM-compatible Halo2 proofs (with `evm-prove` feature)

### Key Types

- **Sdk** - Type alias for `GenericSdk<BabyBearPoseidon2Engine>`
- **NonRootCommittedExe** - Committed executable for non-root applications
- **VerifiedContinuationVmPayload** - Verified execution payload with public values
- **AppExecutionCommit** - Commitment to application execution

## API Reference

### SDK Construction

```rust
// Create default SDK instance
let sdk = Sdk::new();

// Create SDK with custom aggregation tree config
let sdk = Sdk::new().with_agg_tree_config(config);
```

### Build Pipeline

#### Building Guest Programs

```rust
pub fn build<P: AsRef<Path>>(
    &self,
    guest_opts: GuestOptions,
    vm_config: &SdkVmConfig,
    pkg_dir: P,
    target_filter: &Option<TargetFilter>,
    init_file_name: Option<&str>,
) -> Result<Elf>
```

Builds a guest program from a Rust package directory.

#### Transpilation

```rust
pub fn transpile(
    &self,
    elf: Elf,
    transpiler: Transpiler<F>,
) -> Result<VmExe<F>, TranspilerError>
```

Transpiles ELF binary to VM executable format.

#### Execution

```rust
pub fn execute<VC: VmConfig<F>>(
    &self,
    exe: VmExe<F>,
    vm_config: VC,
    inputs: StdIn,
) -> Result<Vec<F>, ExecutionError>
```

Executes VM program and returns public values.

### Proof Generation

#### Application Proofs

```rust
// Generate proving key
pub fn app_keygen<VC: VmConfig<F>>(&self, config: AppConfig<VC>) -> Result<AppProvingKey<VC>>

// Commit executable
pub fn commit_app_exe(&self, app_fri_params: FriParameters, exe: VmExe<F>) -> Result<Arc<NonRootCommittedExe>>

// Generate proof
pub fn generate_app_proof<VC: VmConfig<F>>(
    &self,
    app_pk: Arc<AppProvingKey<VC>>,
    app_committed_exe: Arc<NonRootCommittedExe>,
    inputs: StdIn,
) -> Result<ContinuationVmProof<SC>>
```

#### STARK Proofs

```rust
// Generate aggregation proving key
pub fn agg_stark_keygen(&self, config: AggStarkConfig) -> Result<AggStarkProvingKey>

// Generate end-to-end STARK proof
pub fn generate_e2e_stark_proof<VC: VmConfig<F>>(
    &self,
    app_pk: Arc<AppProvingKey<VC>>,
    app_exe: Arc<NonRootCommittedExe>,
    agg_stark_pk: AggStarkProvingKey,
    inputs: StdIn,
) -> Result<VmStarkProof<SC>>
```

#### EVM Proofs (with `evm-prove` feature)

```rust
// Generate Halo2 proving key
pub fn agg_keygen(
    &self,
    config: AggConfig,
    reader: &impl Halo2ParamsReader,
    pv_handler: &impl StaticVerifierPvHandler,
) -> Result<AggProvingKey>

// Generate EVM-compatible proof
pub fn generate_evm_proof<VC: VmConfig<F>>(
    &self,
    reader: &impl Halo2ParamsReader,
    app_pk: Arc<AppProvingKey<VC>>,
    app_exe: Arc<NonRootCommittedExe>,
    agg_pk: AggProvingKey,
    inputs: StdIn,
) -> Result<EvmProof>
```

### Verification

#### Application Proof Verification

```rust
pub fn verify_app_proof(
    &self,
    app_vk: &AppVerifyingKey,  
    proof: &ContinuationVmProof<SC>,
) -> Result<VerifiedContinuationVmPayload>
```

Verifies application proofs and returns verified payload with public values.

#### STARK Proof Verification  

```rust
pub fn verify_e2e_stark_proof(
    &self,
    agg_stark_pk: &AggStarkProvingKey,
    proof: &VmStarkProof<SC>,
    expected_exe_commit: &Bn254Fr,
    expected_vm_commit: &Bn254Fr,
) -> Result<AppExecutionCommit>
```

#### EVM Verification (with `evm-verify` feature)

```rust
// Generate Solidity verifier contract
pub fn generate_halo2_verifier_solidity(
    &self,
    reader: &impl Halo2ParamsReader,
    agg_pk: &AggProvingKey,
) -> Result<types::EvmHalo2Verifier>

// Verify proof on EVM
pub fn verify_evm_halo2_proof(
    &self,
    openvm_verifier: &types::EvmHalo2Verifier,
    evm_proof: EvmProof,
) -> Result<u64>
```

## Key Modules

### config/
- **AppConfig** - Application-specific VM configuration
- **AggConfig** - Aggregation circuit configuration  
- **AggStarkConfig** - STARK aggregation configuration
- **SdkVmConfig** - High-level VM configuration interface

### keygen/
- **AppProvingKey** - Application proving keys
- **AggProvingKey** - Aggregation proving keys (Halo2)
- **AggStarkProvingKey** - STARK aggregation proving keys
- **AppVerifyingKey** - Application verifying keys

### prover/
- **AppProver** - Application-level proof generation
- **StarkProver** - STARK proof generation and aggregation
- **EvmHalo2Prover** - EVM-compatible Halo2 proof generation

### Types and Utilities

- **commit.rs** - Executable commitment utilities
- **codec.rs** - Encoding/decoding utilities
- **fs.rs** - File system utilities for EVM verifier generation
- **stdin.rs** - Standard input handling for VM execution
- **types.rs** - Core type definitions

## Feature Flags

- **default** - `["parallel", "jemalloc"]`
- **evm-prove** - Enables EVM-compatible proof generation
- **evm-verify** - Enables EVM verification and Solidity contract generation
- **bench-metrics** - Enables benchmarking metrics
- **profiling** - Enables guest program profiling
- **test-utils** - Testing utilities
- **parallel** - Parallel execution support
- **mimalloc/jemalloc** - Memory allocator options

## Error Handling

The SDK uses `eyre::Result` for error handling throughout the API. Common error types include:

- **ExecutionError** - VM execution failures
- **TranspilerError** - ELF to VM transpilation errors  
- **Proof verification failures** - Invalid proofs or mismatched commitments
- **Build errors** - Guest program compilation failures

## Thread Safety

The SDK is designed to be thread-safe with `Arc` references used for shared proving keys and executables. Multiple proofs can be generated concurrently using the same proving materials.

## Memory Management

The SDK supports multiple memory allocators via feature flags:
- **jemalloc** (default) - Better performance for proof generation
- **mimalloc** - Alternative high-performance allocator
- **jemalloc-prof** - jemalloc with profiling support

## Version Compatibility

Current SDK version: **1.3.0**

The SDK maintains compatibility with OpenVM core components and follows semantic versioning for API stability.