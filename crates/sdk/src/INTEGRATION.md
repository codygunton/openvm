# OpenVM SDK Integration Guide

This guide covers how to integrate the OpenVM SDK into your Rust applications and development workflows.

## Project Setup

### Adding SDK Dependencies

Add the OpenVM SDK to your `Cargo.toml`:

```toml
[dependencies]
openvm-sdk = { version = "1.3.0" }
openvm-build = { version = "1.3.0" }  # For building guest programs
openvm = { version = "1.3.0" }        # For platform definitions

# Optional: Enable EVM features
openvm-sdk = { version = "1.3.0", features = ["evm-prove", "evm-verify"] }

# For STARK engine
openvm-stark-sdk = { version = "1.3.0" }

# For ELF handling
openvm-transpiler = { version = "1.3.0" }
```

### Feature Selection

Choose appropriate features based on your use case:

```toml
[dependencies.openvm-sdk]
version = "1.3.0"
features = [
    "parallel",      # Enable parallel processing (recommended)
    "jemalloc",      # Better memory performance (default)
    "evm-prove",     # Generate EVM-compatible proofs
    "evm-verify",    # Verify proofs on EVM and generate Solidity contracts
    "bench-metrics", # Enable benchmarking (development only)
    "profiling",     # Enable guest program profiling (development only)
]
```

## Application Architecture

### Basic Integration Pattern

```rust
use openvm_sdk::{Sdk, StdIn, config::SdkVmConfig};
use std::sync::Arc;
use eyre::Result;

pub struct ZkVmService {
    sdk: Sdk,
    vm_config: SdkVmConfig,
    app_pk: Option<Arc<openvm_sdk::keygen::AppProvingKey<SdkVmConfig>>>,
}

impl ZkVmService {
    pub fn new() -> Self {
        let vm_config = SdkVmConfig::builder()
            .system(Default::default())
            .rv32i(Default::default())
            .rv32m(Default::default())
            .io(Default::default())
            .build();

        Self {
            sdk: Sdk::new(),
            vm_config,
            app_pk: None,
        }
    }

    pub fn setup_proving_key(&mut self, exe_path: &str) -> Result<()> {
        // Load and commit executable
        let elf = self.load_elf(exe_path)?;
        let exe = self.sdk.transpile(elf, self.vm_config.transpiler())?;
        
        // Generate proving key
        let app_fri_params = openvm_stark_sdk::config::FriParameters::standard_with_100_bits_conjectured_security(2);
        let app_committed_exe = self.sdk.commit_app_exe(app_fri_params, exe)?;
        let app_config = openvm_sdk::config::AppConfig::new(app_fri_params, self.vm_config.clone());
        
        self.app_pk = Some(Arc::new(self.sdk.app_keygen(app_config)?));
        Ok(())
    }

    pub fn prove_execution<T: serde::Serialize>(&self, input: &T) -> Result<Vec<u8>> {
        let app_pk = self.app_pk.as_ref().ok_or_else(|| eyre::eyre!("Proving key not initialized"))?;
        
        let mut stdin = StdIn::default();
        stdin.write(input);
        
        // Generate proof (simplified - you'd want to handle the committed exe properly)
        // let proof = self.sdk.generate_app_proof(app_pk.clone(), app_committed_exe, stdin)?;
        // Ok(serialize_proof(proof))
        
        todo!("Implement proof serialization")
    }
}
```

### Service-Oriented Architecture

```rust
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait ZkProofService {
    async fn generate_proof(&self, program_id: &str, input: Vec<u8>) -> Result<Vec<u8>>;
    async fn verify_proof(&self, program_id: &str, proof: Vec<u8>) -> Result<bool>;
}

pub struct OpenVmProofService {
    sdk: Sdk,
    proving_keys: HashMap<String, Arc<openvm_sdk::keygen::AppProvingKey<SdkVmConfig>>>,
    verifying_keys: HashMap<String, openvm_sdk::keygen::AppVerifyingKey>,
}

#[async_trait]
impl ZkProofService for OpenVmProofService {
    async fn generate_proof(&self, program_id: &str, input: Vec<u8>) -> Result<Vec<u8>> {
        let app_pk = self.proving_keys.get(program_id)
            .ok_or_else(|| eyre::eyre!("Program not found: {}", program_id))?;
            
        // Deserialize input, generate proof, serialize result
        todo!("Implement async proof generation")
    }

    async fn verify_proof(&self, program_id: &str, proof: Vec<u8>) -> Result<bool> {
        let app_vk = self.verifying_keys.get(program_id)
            .ok_or_else(|| eyre::eyre!("Program not found: {}", program_id))?;
            
        // Deserialize proof, verify, return result
        todo!("Implement async proof verification")
    }
}
```

## Build System Integration

### Cargo Integration

Create a build script `build.rs` to handle guest program compilation:

```rust
use std::env;
use std::path::PathBuf;
use openvm_build::{build_guest_package, get_package, GuestOptions};

fn main() {
    println!("cargo:rerun-if-changed=guest/");
    
    let guest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("guest");
    let pkg = get_package(&guest_dir);
    
    let guest_opts = GuestOptions::default();
    if let Ok(target_dir) = build_guest_package(&pkg, &guest_opts, None, &None) {
        println!("cargo:rustc-env=GUEST_TARGET_DIR={}", target_dir.display());
    }
}
```

### Workspace Configuration

For multi-crate workspaces with guest programs:

```toml
# Workspace Cargo.toml
[workspace]
members = ["host", "guest"]

# Host Cargo.toml
[dependencies]
openvm-sdk = "1.3.0"

[build-dependencies]
openvm-build = "1.3.0"

# Guest Cargo.toml  
[dependencies]
openvm = "1.3.0"

[profile.release]
debug = false
panic = "abort"
```

## Configuration Management

### Environment-Based Configuration

```rust
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct SdkConfig {
    pub fri_log_blowup: usize,
    pub parallel_execution: bool,
    pub evm_integration: bool,
    pub params_dir: String,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            fri_log_blowup: env::var("OPENVM_FRI_LOG_BLOWUP")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
            parallel_execution: env::var("OPENVM_PARALLEL")
                .map(|s| s == "true")
                .unwrap_or(true),
            evm_integration: env::var("OPENVM_EVM_ENABLED")
                .map(|s| s == "true")
                .unwrap_or(false),
            params_dir: env::var("OPENVM_PARAMS_DIR")
                .unwrap_or_else(|_| format!("{}/.openvm/params/", env::var("HOME").unwrap())),
        }
    }
}

pub fn create_sdk_from_config(config: &SdkConfig) -> Result<Sdk> {
    let mut vm_config_builder = SdkVmConfig::builder()
        .system(Default::default())
        .rv32i(Default::default())
        .rv32m(Default::default())
        .io(Default::default());

    if config.evm_integration {
        // Add EVM-specific extensions
        vm_config_builder = vm_config_builder
            .keccak(Default::default())
            .sha256(Default::default());
    }

    let vm_config = vm_config_builder.build();
    Ok(Sdk::new())
}
```

## Testing Integration

### Test Utilities

```rust
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use tempfile::TempDir;

    pub struct TestSdkSetup {
        pub sdk: Sdk,
        pub vm_config: SdkVmConfig,
        pub temp_dir: TempDir,
    }

    impl TestSdkSetup {
        pub fn new() -> Self {
            let vm_config = SdkVmConfig::builder()
                .system(Default::default())
                .rv32i(Default::default())
                .rv32m(Default::default())
                .io(Default::default())
                .build();

            Self {
                sdk: Sdk::new(),
                vm_config,
                temp_dir: TempDir::new().unwrap(),
            }
        }

        pub fn build_test_program(&self, source: &str) -> Result<openvm_transpiler::elf::Elf> {
            // Write test program to temp directory and build
            todo!("Implement test program building")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let setup = test_utils::TestSdkSetup::new();
        
        // Test basic SDK functionality
        let elf = setup.build_test_program("
            fn main() {
                println!(\"Hello, OpenVM!\");
            }
        ").unwrap();
        
        let exe = setup.sdk.transpile(elf, setup.vm_config.transpiler()).unwrap();
        let result = setup.sdk.execute(exe, setup.vm_config.clone(), StdIn::default()).unwrap();
        
        assert!(!result.is_empty());
    }
}
```

### Benchmark Integration

```rust
#[cfg(feature = "bench-metrics")]
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(feature = "bench-metrics")]
fn bench_proof_generation(c: &mut Criterion) {
    let setup = test_utils::TestSdkSetup::new();
    // Setup proving key and executable
    
    c.bench_function("proof_generation", |b| {
        b.iter(|| {
            // Generate proof with black_box to prevent optimization
            black_box(setup.sdk.generate_app_proof(
                app_pk.clone(),
                app_committed_exe.clone(),
                stdin.clone()
            ))
        })
    });
}

#[cfg(feature = "bench-metrics")]
criterion_group!(benches, bench_proof_generation);
#[cfg(feature = "bench-metrics")]
criterion_main!(benches);
```

## Production Deployment

### Error Handling Strategy

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkIntegrationError {
    #[error("SDK initialization failed: {0}")]
    InitializationError(#[from] eyre::Error),
    
    #[error("Program compilation failed: {0}")]
    CompilationError(String),
    
    #[error("Proof generation failed: {0}")]
    ProofGenerationError(String),
    
    #[error("Proof verification failed: {0}")]
    VerificationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type SdkResult<T> = Result<T, SdkIntegrationError>;
```

### Logging and Monitoring

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn generate_proof_with_monitoring(&self, input: &[u8]) -> SdkResult<Vec<u8>> {
    let start = std::time::Instant::now();
    
    info!("Starting proof generation, input size: {}", input.len());
    
    match self.internal_generate_proof(input).await {
        Ok(proof) => {
            let duration = start.elapsed();
            info!("Proof generation completed in {:?}, proof size: {}", duration, proof.len());
            
            // Record metrics
            metrics::histogram!("proof_generation_duration", duration);
            metrics::counter!("proof_generation_success", 1);
            
            Ok(proof)
        }
        Err(e) => {
            let duration = start.elapsed();
            error!("Proof generation failed after {:?}: {}", duration, e);
            
            metrics::counter!("proof_generation_failure", 1);
            
            Err(e)
        }
    }
}
```

### Resource Management

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct ManagedSdkService {
    sdk: Sdk,
    // Limit concurrent proof generations
    proof_semaphore: Arc<Semaphore>,
    // Limit memory usage
    memory_limit: usize,
}

impl ManagedSdkService {
    pub fn new(max_concurrent_proofs: usize, memory_limit_mb: usize) -> Self {
        Self {
            sdk: Sdk::new(),
            proof_semaphore: Arc::new(Semaphore::new(max_concurrent_proofs)),
            memory_limit: memory_limit_mb * 1024 * 1024,
        }
    }

    pub async fn generate_proof_managed(&self, input: Vec<u8>) -> SdkResult<Vec<u8>> {
        // Acquire permit for resource management
        let _permit = self.proof_semaphore.acquire().await
            .map_err(|_| SdkIntegrationError::ConfigError("Semaphore closed".to_string()))?;

        // Check memory usage
        if input.len() > self.memory_limit {
            return Err(SdkIntegrationError::ConfigError("Input too large".to_string()));
        }

        // Generate proof
        self.generate_proof(&input).await
    }
}
```

## Best Practices

### 1. Initialization Strategy

- Initialize SDK and proving keys at application startup
- Cache proving keys for reuse across multiple proof generations
- Use lazy initialization for expensive resources

### 2. Resource Management

- Limit concurrent proof generations based on available memory
- Monitor memory usage during proof generation
- Implement proper cleanup for temporary files

### 3. Error Handling

- Use structured error types for different failure modes
- Provide detailed error context for debugging
- Implement retry logic for transient failures

### 4. Testing

- Use deterministic inputs for reproducible tests
- Test both success and failure paths
- Include integration tests with real guest programs

### 5. Monitoring

- Track proof generation time and success rates
- Monitor memory usage and resource consumption
- Set up alerts for failure rate thresholds

### 6. Security

- Validate all inputs before processing
- Implement proper access controls for proving keys
- Regular security audits of integration code

This integration guide provides the foundation for successfully incorporating the OpenVM SDK into production applications while maintaining reliability, performance, and security.