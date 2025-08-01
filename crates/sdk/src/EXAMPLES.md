# OpenVM SDK Examples

This document provides practical examples of using the OpenVM SDK for building, executing, and proving zero-knowledge virtual machine programs.

## Basic Application Workflow

### 1. Setting Up VM Configuration

```rust
use openvm_sdk::{config::SdkVmConfig, Sdk};

// Configure VM with required extensions
let vm_config = SdkVmConfig::builder()
    .system(Default::default())
    .rv32i(Default::default())  // RISC-V base integer instructions
    .rv32m(Default::default())  // RISC-V multiplication/division
    .io(Default::default())     // I/O operations
    .build();

let sdk = Sdk::new();
```

### 2. Building Guest Programs

```rust
use openvm_build::GuestOptions;
use std::path::PathBuf;

// Option A: Build from package directory
let guest_opts = GuestOptions::default();
let target_path = "/path/to/guest/program";
let elf = sdk.build(
    guest_opts,
    &vm_config,
    target_path,
    &Default::default(),
    None,
)?;

// Option B: Load pre-built ELF
use std::fs;
use openvm::platform::memory::MEM_SIZE;
use openvm_transpiler::elf::Elf;

let elf_bytes = fs::read("path/to/program.elf")?;
let elf = Elf::decode(&elf_bytes, MEM_SIZE as u32)?;
```

### 3. Transpilation and Execution

```rust
use openvm_sdk::StdIn;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MyInput {
    a: u64,
    b: u64,
}

// Transpile ELF to VM executable
let exe = sdk.transpile(elf, vm_config.transpiler())?;

// Prepare input data
let input = MyInput { a: 42, b: 100 };
let mut stdin = StdIn::default();
stdin.write(&input);

// Execute program
let public_values = sdk.execute(exe.clone(), vm_config.clone(), stdin.clone())?;
println!("Program output: {:?}", public_values);
```

## Proof Generation and Verification

### Basic STARK Proofs

```rust
use std::sync::Arc;
use openvm_sdk::config::AppConfig;
use openvm_stark_sdk::config::FriParameters;

// Configure proof parameters
let app_log_blowup = 2;
let app_fri_params = FriParameters::standard_with_100_bits_conjectured_security(app_log_blowup);
let app_config = AppConfig::new(app_fri_params, vm_config);

// Commit executable and generate proving key
let app_committed_exe = sdk.commit_app_exe(app_fri_params, exe)?;
let app_pk = Arc::new(sdk.app_keygen(app_config)?);

// Generate proof
let proof = sdk.generate_app_proof(app_pk.clone(), app_committed_exe, stdin)?;

// Verify proof
let app_vk = app_pk.get_app_vk();
let verified_payload = sdk.verify_app_proof(&app_vk, &proof)?;
println!("Execution commitment: {:?}", verified_payload.exe_commit);
println!("User public values: {:?}", verified_payload.user_public_values);
```

### Custom App Prover

```rust
use openvm_sdk::prover::AppProver;
use openvm_stark_sdk::config::baby_bear_poseidon2::BabyBearPoseidon2Engine;

// Create custom app prover with metadata
let app_prover = AppProver::<_, BabyBearPoseidon2Engine>::new(
    app_pk.app_vm_pk.clone(),
    app_committed_exe.clone(),
)
.with_program_name("fibonacci_calculator")
.with_metadata([("version", "1.0"), ("author", "developer")]);

let proof = app_prover.generate_app_proof(stdin);
```

## EVM Integration (evm-prove/evm-verify features)

### Generating EVM-Compatible Proofs

```rust
use openvm_sdk::{config::AggConfig, DefaultStaticVerifierPvHandler};
use openvm_native_recursion::halo2::utils::CacheHalo2ParamsReader;

// Setup Halo2 parameters
const PARAMS_DIR: &str = concat!(env!("HOME"), "/.openvm/params/");
let halo2_params_reader = CacheHalo2ParamsReader::new(PARAMS_DIR);

// Generate aggregation proving key
let agg_config = AggConfig::default();
let agg_pk = sdk.agg_keygen(
    agg_config,
    &halo2_params_reader,
    &DefaultStaticVerifierPvHandler,
)?;

// Generate EVM proof
let evm_proof = sdk.generate_evm_proof(
    &halo2_params_reader,
    app_pk,
    app_committed_exe,
    agg_pk,
    stdin,
)?;
```

### Generating Solidity Verifier Contract

```rust
// Generate Solidity verifier contract
let verifier_contract = sdk.generate_halo2_verifier_solidity(&halo2_params_reader, &agg_pk)?;

// Access contract components
println!("Interface: {}", verifier_contract.openvm_verifier_interface);
println!("Implementation: {}", verifier_contract.openvm_verifier_code);
println!("Bytecode length: {}", verifier_contract.artifact.bytecode.len());

// Verify proof on EVM
let gas_cost = sdk.verify_evm_halo2_proof(&verifier_contract, evm_proof)?;
println!("Verification gas cost: {}", gas_cost);
```

## Advanced Features

### STARK Aggregation

```rust
use openvm_sdk::config::AggStarkConfig;

// Generate STARK aggregation proving key
let agg_stark_config = AggStarkConfig::default();
let agg_stark_pk = sdk.agg_stark_keygen(agg_stark_config)?;

// Generate end-to-end STARK proof
let stark_proof = sdk.generate_e2e_stark_proof(
    app_pk,
    app_committed_exe,
    agg_stark_pk.clone(),
    stdin,
)?;

// Verify STARK proof with expected commitments
use openvm_stark_sdk::p3_bn254_fr::Bn254Fr;

let expected_exe_commit = Bn254Fr::from(12345u64); // Your expected commitment
let expected_vm_commit = Bn254Fr::from(67890u64);  // Your expected VM commitment

let app_execution_commit = sdk.verify_e2e_stark_proof(
    &agg_stark_pk,
    &stark_proof,
    &expected_exe_commit,
    &expected_vm_commit,
)?;
```

### Root Verifier Assembly Generation

```rust
// Generate assembly for root verifier circuit
let verifier_asm = sdk.generate_root_verifier_asm(&agg_stark_pk);
println!("Root verifier assembly:\n{}", verifier_asm);

// Generate root verifier input
let root_input = sdk.generate_root_verifier_input(
    app_pk,
    app_committed_exe,
    agg_stark_pk,
    stdin,
)?;
```

### Custom VM Configuration

```rust
use openvm_sdk::config::SdkVmConfig;

// Configure VM with cryptographic extensions
let vm_config = SdkVmConfig::builder()
    .system(Default::default())
    .rv32i(Default::default())
    .rv32m(Default::default())
    .io(Default::default())
    .keccak(Default::default())     // Keccak-256 hashing
    .sha256(Default::default())     // SHA-256 hashing
    .bigint(Default::default())     // Big integer arithmetic
    .ecc(Default::default())        // Elliptic curve operations
    .pairing(Default::default())    // Pairing operations
    .build();
```

## Error Handling

### Common Error Patterns

```rust
use eyre::Result;

fn example_with_error_handling() -> Result<()> {
    let exe = match sdk.transpile(elf, vm_config.transpiler()) {
        Ok(exe) => exe,
        Err(e) => {
            eprintln!("Transpilation failed: {}", e);
            return Err(e.into());
        }
    };

    let proof = sdk.generate_app_proof(app_pk, app_committed_exe, stdin)
        .map_err(|e| {
            eprintln!("Proof generation failed: {}", e);
            e
        })?;

    Ok(())
}
```

### Validation Examples

```rust
// Validate execution before proof generation
if let Err(execution_error) = sdk.execute(exe.clone(), vm_config.clone(), stdin.clone()) {
    eprintln!("Program execution failed: {}", execution_error);
    return Err(execution_error.into());
}

// Validate proof before verification
match sdk.verify_app_proof(&app_vk, &proof) {
    Ok(payload) => println!("Proof verified successfully"),
    Err(e) => eprintln!("Proof verification failed: {}", e),
}
```

## Testing Examples

### Unit Testing with SDK

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_execution() {
        let sdk = Sdk::new();
        let vm_config = SdkVmConfig::builder().build();
        
        // Test execution
        let result = sdk.execute(exe, vm_config, StdIn::default());
        assert!(result.is_ok());
    }

    #[test] 
    fn test_proof_generation() {
        // Setup test environment
        let sdk = Sdk::new();
        // ... test proof generation workflow
        
        assert!(proof_result.is_ok());
    }
}
```

### Integration Testing

```rust
#[test]
fn test_e2e_workflow() -> Result<()> {
    let sdk = Sdk::new();
    
    // 1. Build and transpile
    let exe = /* ... */;
    
    // 2. Execute and validate
    let output = sdk.execute(exe.clone(), vm_config.clone(), stdin.clone())?;
    assert!(!output.is_empty());
    
    // 3. Generate and verify proof
    let proof = sdk.generate_app_proof(app_pk.clone(), app_committed_exe, stdin)?;
    let verified = sdk.verify_app_proof(&app_vk, &proof)?;
    
    assert_eq!(verified.user_public_values, output);
    Ok(())
}
```

## Performance Optimization

### Parallel Proof Generation

```rust
use std::thread;

// Generate multiple proofs concurrently
let handles: Vec<_> = inputs.into_iter().map(|input| {
    let sdk = sdk.clone();
    let app_pk = app_pk.clone();
    let app_committed_exe = app_committed_exe.clone();
    
    thread::spawn(move || {
        let mut stdin = StdIn::default();
        stdin.write(&input);
        sdk.generate_app_proof(app_pk, app_committed_exe, stdin)
    })
}).collect();

let proofs: Result<Vec<_>> = handles.into_iter()
    .map(|handle| handle.join().unwrap())
    .collect();
```

### Memory Optimization

```rust
// Use jemalloc for better memory performance (enabled by default)
// or configure in Cargo.toml:
// [features]
// default = ["parallel", "jemalloc"]

// For profiling memory usage:
// [features] 
// default = ["parallel", "jemalloc-prof"]
```

These examples demonstrate the core functionality of the OpenVM SDK, from basic program execution to advanced proof generation and EVM integration.