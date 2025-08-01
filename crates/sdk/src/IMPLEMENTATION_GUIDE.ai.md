# OpenVM SDK - Implementation Guide

## Overview

This guide provides patterns, best practices, and implementation details for working with the OpenVM SDK. It covers common use cases, performance optimization strategies, and integration patterns.

## Architecture Patterns

### 1. SDK Initialization Pattern

The SDK follows a builder pattern for configuration:

```rust
// Basic initialization
let sdk = Sdk::new();

// With custom aggregation tree config
let sdk = Sdk::new()
    .with_agg_tree_config(AggregationTreeConfig {
        num_children_leaf: 2,
        num_children_internal: 4,
        max_internal_wrapper_layers: 3,
    });
```

**Key Insight**: The SDK is stateless except for aggregation configuration. Multiple SDK instances can be used concurrently.

### 2. Resource Management Pattern

The SDK uses `Arc` extensively for shared resources:

```rust
// Keys are wrapped in Arc for efficient sharing
let app_pk: Arc<AppProvingKey<VC>> = Arc::new(sdk.app_keygen(config)?);

// Committed executables are also Arc-wrapped
let exe: Arc<NonRootCommittedExe> = sdk.commit_app_exe(fri_params, vm_exe)?;

// Share across multiple provers
let prover1 = AppProver::new(app_pk.clone(), exe.clone());
let prover2 = AppProver::new(app_pk.clone(), exe.clone());
```

**Best Practice**: Clone `Arc` pointers rather than the underlying data for memory efficiency.

### 3. Error Handling Pattern

The SDK uses `eyre::Result` throughout:

```rust
use eyre::{Context, Result};

fn prove_with_context(sdk: &Sdk, config: AppConfig<VC>) -> Result<()> {
    let pk = sdk.app_keygen(config)
        .context("Failed to generate app proving key")?;
    
    // Chain multiple operations with context
    let proof = sdk.generate_app_proof(pk, exe, inputs)
        .context("Proof generation failed")
        .and_then(|p| verify_proof(p))
        .context("Verification failed")?;
    
    Ok(())
}
```

## Implementation Patterns

### 1. Guest Program Integration

The SDK expects guest programs to follow specific patterns:

```rust
// In guest code (openvm-init.rs)
openvm::init!();

// Main guest function
#[no_mangle]
fn main() {
    // Read inputs
    let input: MyInput = openvm::io::read();
    
    // Perform computation
    let result = compute(input);
    
    // Commit public values
    openvm::io::commit(&result);
}
```

**Key Points**:
- Use `openvm::init!()` macro for initialization
- Read inputs via `openvm::io::read()`
- Commit outputs via `openvm::io::commit()`

### 2. Input Handling Pattern

The `StdIn` type provides flexible input management:

```rust
// Create inputs from various sources
let mut stdin = StdIn::default();

// From serializable types
stdin.write(&my_struct);

// From raw bytes
stdin.write_bytes(b"raw data");

// From field elements
stdin.write_field(&[F::from(42), F::from(100)]);

// Add key-value pairs for lookups
stdin.add_key_value(b"config".to_vec(), config_bytes);
```

### 3. Proof Aggregation Pattern

Implement hierarchical proof aggregation:

```rust
// Generate multiple app proofs
let proofs: Vec<ContinuationVmProof<SC>> = inputs
    .into_iter()
    .map(|input| sdk.generate_app_proof(app_pk.clone(), exe.clone(), input))
    .collect::<Result<Vec<_>>>()?;

// Aggregate at leaf level
let leaf_proof = agg_prover.generate_leaf_proof(proofs)?;

// Further aggregation if needed
let internal_proof = agg_prover.generate_internal_proof(vec![leaf_proof])?;
```

## Performance Optimization

### 1. Key Generation Caching

Keys are expensive to generate but reusable:

```rust
// Cache keys to disk
impl KeyCache {
    fn save_app_key(pk: &AppProvingKey<VC>, path: &Path) -> Result<()> {
        let bytes = postcard::to_stdvec(pk)?;
        fs::write(path, bytes)?;
        Ok(())
    }
    
    fn load_app_key(path: &Path) -> Result<AppProvingKey<VC>> {
        let bytes = fs::read(path)?;
        Ok(postcard::from_bytes(&bytes)?)
    }
}

// Use cached keys
let app_pk = match KeyCache::load_app_key(&key_path) {
    Ok(pk) => pk,
    Err(_) => {
        let pk = sdk.app_keygen(config)?;
        KeyCache::save_app_key(&pk, &key_path)?;
        pk
    }
};
```

### 2. Parallel Proof Generation

Leverage parallelism for multiple proofs:

```rust
use rayon::prelude::*;

// Parallel proof generation
let proofs: Vec<_> = inputs
    .par_iter()
    .map(|input| {
        sdk.generate_app_proof(
            app_pk.clone(), 
            exe.clone(), 
            input.clone()
        )
    })
    .collect::<Result<Vec<_>>>()?;
```

### 3. Memory Management

For large-scale proving:

```rust
// Configure memory limits
std::env::set_var("RUST_MIN_STACK", "33554432"); // 32MB stack

// Use memory mapping for large inputs
use memmap2::MmapOptions;
let file = File::open("large_input.bin")?;
let mmap = unsafe { MmapOptions::new().map(&file)? };
let stdin = StdIn::from_bytes(&mmap);
```

## Advanced Patterns

### 1. Custom VM Configuration

Implement domain-specific VMs:

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct CustomVmConfig {
    pub system: SystemConfig,
    pub custom_extension: MyExtension,
}

impl VmConfig<F> for CustomVmConfig {
    type Executor = CustomExecutor;
    type Periphery = CustomPeriphery;
    
    fn system(&self) -> &SystemConfig {
        &self.system
    }
    
    fn create_chip_complex(&self) -> Self::Executor {
        CustomExecutor::new(self.clone())
    }
}
```

### 2. Commitment Verification

Ensure execution integrity:

```rust
// Compute expected commitment
let expected_commit = AppExecutionCommit::compute(
    &vm_config,
    &app_exe,
    &leaf_vm_exe,
);

// Verify against proof
let verified = sdk.verify_app_proof(&app_vk, &proof)?;
assert_eq!(
    verified.exe_commit,
    expected_commit.app_exe_commit.to_u32_digest(),
    "Execution commitment mismatch"
);
```

### 3. EVM Integration

Generate and deploy verifiers:

```rust
// Generate verifier contract
let verifier = sdk.generate_halo2_verifier_solidity(&reader, &agg_pk)?;

// Deploy contract (using ethers-rs)
let contract = ContractFactory::new(
    verifier.artifact.bytecode.clone(),
    Default::default(),
    client.clone(),
)
.deploy(())?
.send()
.await?;

// Verify proof on-chain
let proof = sdk.generate_evm_proof(&reader, app_pk, exe, agg_pk, inputs)?;
let calldata = proof.verifier_calldata();
let tx = contract.method_raw("verify", calldata)?;
```

## Testing Patterns

### 1. Unit Testing SDK Components

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proof_generation() {
        let sdk = Sdk::new();
        let config = test_config();
        let pk = sdk.app_keygen(config).unwrap();
        
        // Test with minimal input
        let stdin = StdIn::from_bytes(b"test");
        let proof = sdk.generate_app_proof(pk, exe, stdin).unwrap();
        
        assert!(sdk.verify_app_proof(&pk.get_app_vk(), &proof).is_ok());
    }
}
```

### 2. Integration Testing

```rust
#[test]
fn test_e2e_proving_pipeline() {
    // Build guest program
    let elf = sdk.build(
        GuestOptions::default(),
        &vm_config,
        "./tests/guests/simple",
        &None,
        None,
    ).unwrap();
    
    // Full pipeline
    let exe = sdk.transpile(elf, transpiler).unwrap();
    let committed = sdk.commit_app_exe(fri_params, exe).unwrap();
    let pk = sdk.app_keygen(config).unwrap();
    let proof = sdk.generate_app_proof(pk, committed, stdin).unwrap();
    
    // Verify
    let payload = sdk.verify_app_proof(&pk.get_app_vk(), &proof).unwrap();
    assert_eq!(payload.user_public_values, expected_output);
}
```

## Common Pitfalls

### 1. Commitment Mismatch
**Problem**: Proof verification fails due to commitment mismatch.
**Solution**: Ensure VM config used for keygen matches execution.

### 2. Memory Exhaustion
**Problem**: OOM during aggregation key generation.
**Solution**: Use swap space or cloud instances with >64GB RAM.

### 3. Proof Size
**Problem**: Proofs too large for on-chain verification.
**Solution**: Use proper aggregation tree configuration.

### 4. Input Serialization
**Problem**: Guest program crashes reading inputs.
**Solution**: Ensure consistent serialization format between host and guest.

## Production Checklist

1. **Security**
   - [ ] Use production FRI parameters (100+ bits)
   - [ ] Verify all commitments
   - [ ] Implement proper key management
   - [ ] Audit guest programs

2. **Performance**
   - [ ] Cache proving keys
   - [ ] Enable parallelization
   - [ ] Monitor memory usage
   - [ ] Use appropriate allocator

3. **Reliability**
   - [ ] Implement retry logic
   - [ ] Handle proof failures gracefully
   - [ ] Log all operations
   - [ ] Monitor proof generation times

4. **Integration**
   - [ ] Test with actual VM configs
   - [ ] Verify EVM gas costs
   - [ ] Implement fallback mechanisms
   - [ ] Document commitment schemes