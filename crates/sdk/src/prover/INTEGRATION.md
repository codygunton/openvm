# Prover Integration Guidelines

## Overview
This guide covers how to integrate the prover component into your OpenVM applications, including setup, configuration, and best practices for different use cases.

## Dependencies

### Required Crates
```toml
[dependencies]
openvm-sdk = { version = "1.3.0", features = ["prove"] }
openvm-circuit = "1.3.0"
openvm-stark-sdk = "1.3.0"
openvm-stark-backend = "1.3.0"

# Optional for EVM proving
openvm-native-recursion = { version = "1.3.0", features = ["halo2"], optional = true }

# For metrics and tracing
tracing = "0.1"
metrics = { version = "0.21", optional = true }
```

### Feature Flags
- `evm-prove`: Enables Halo2-based EVM-compatible proving
- `bench-metrics`: Enables performance metrics collection
- Standard features for STARK proving are enabled by default

## Key Integration Steps

### 1. Keygen Integration
```rust
use openvm_sdk::keygen::{AppProvingKey, AggProvingKey};
use openvm_sdk::config::AggregationTreeConfig;

// Generate or load proving keys
let app_pk = Arc::new(generate_app_proving_key(vm_config));
let agg_pk = generate_agg_proving_key(agg_config);

// Keys must be compatible - validation happens in prover constructor
```

### 2. Executable Integration
```rust
use openvm_sdk::{NonRootCommittedExe, VmConfig};

// Commit your executable with the VM configuration
let committed_exe = NonRootCommittedExe::commit(
    executable_bytes,
    vm_config
);
```

### 3. Input Preparation
```rust
use openvm_sdk::StdIn;

// Prepare program inputs
let mut stdin = StdIn::default();
// Add your program inputs...

// Ensure inputs match your program's expectations
```

## Integration Patterns

### Pattern 1: Simple App Proving
**Use Case**: Single program, no aggregation needed
```rust
use openvm_sdk::prover::AppProver;

pub struct SimpleProver<VC, E> {
    app_prover: AppProver<VC, E>,
}

impl<VC, E> SimpleProver<VC, E> {
    pub fn new(app_pk: Arc<AppProvingKey<VC>>, exe: Arc<NonRootCommittedExe>) -> Self {
        Self {
            app_prover: AppProver::new(app_pk, exe),
        }
    }
    
    pub fn prove(&self, input: StdIn) -> Result<Proof<SC>, ProverError> {
        if self.app_prover.vm_config().system().continuation_enabled {
            Ok(self.app_prover.generate_app_proof(input).into())
        } else {
            Ok(self.app_prover.generate_app_proof_without_continuations(input))
        }
    }
}
```

### Pattern 2: Full STARK Pipeline
**Use Case**: Production proving with aggregation
```rust
use openvm_sdk::prover::StarkProver;

pub struct ProductionProver<VC, E> {
    stark_prover: StarkProver<VC, E>,
}

impl<VC, E> ProductionProver<VC, E> {
    pub fn new(
        app_pk: Arc<AppProvingKey<VC>>,
        app_exe: Arc<NonRootCommittedExe>,
        agg_pk: AggStarkProvingKey,
        agg_config: AggregationTreeConfig,
    ) -> Self {
        Self {
            stark_prover: StarkProver::new(app_pk, app_exe, agg_pk, agg_config),
        }
    }
    
    pub fn prove_for_verification(&self, input: StdIn) -> (VmStarkProof, RootVmVerifierInput<SC>) {
        self.stark_prover.generate_proof_and_verifier_input(input)
    }
}
```

### Pattern 3: EVM Integration
**Use Case**: Blockchain verification on Ethereum
```rust
use openvm_sdk::prover::EvmHalo2Prover;
use openvm_sdk::types::EvmProof;

pub struct EvmProver<VC, E> {
    evm_prover: EvmHalo2Prover<VC, E>,
}

impl<VC, E> EvmProver<VC, E> {
    pub fn new(
        params_reader: &impl Halo2ParamsReader,
        app_pk: Arc<AppProvingKey<VC>>,
        app_exe: Arc<NonRootCommittedExe>,
        agg_pk: AggProvingKey,
        agg_config: AggregationTreeConfig,
    ) -> Self {
        Self {
            evm_prover: EvmHalo2Prover::new(
                params_reader, app_pk, app_exe, agg_pk, agg_config
            ),
        }
    }
    
    pub fn prove_for_evm(&self, input: StdIn) -> EvmProof {
        self.evm_prover.generate_proof_for_evm(input)
    }
}
```

## Configuration Best Practices

### VM Configuration Compatibility
```rust
// Ensure VM configurations match between components
assert_eq!(
    app_vm_config.system().num_public_values,
    agg_vm_config.num_user_public_values(),
    "Public value count mismatch"
);

// Validate FRI parameters compatibility
assert_eq!(
    app_pk.leaf_fri_params,
    agg_pk.leaf_vm_pk.fri_params,
    "FRI parameter mismatch"
);
```

### Resource Management
```rust
use std::sync::Arc;

// Use Arc for shared proving keys to avoid cloning
let app_pk = Arc::new(app_proving_key);
let committed_exe = Arc::new(committed_executable);

// Multiple provers can share the same keys
let prover1 = AppProver::new(app_pk.clone(), committed_exe.clone());
let prover2 = AppProver::new(app_pk.clone(), committed_exe.clone());
```

### Error Handling Strategy
```rust
use std::panic::catch_unwind;

// Prover constructors use assertions - catch panics for graceful handling
pub fn safe_prover_creation<VC, E>(
    app_pk: Arc<AppProvingKey<VC>>,
    agg_pk: AggStarkProvingKey,
    // ... other params
) -> Result<StarkProver<VC, E>, String> {
    catch_unwind(|| {
        StarkProver::new(app_pk, committed_exe, agg_pk, agg_config)
    }).map_err(|_| "Incompatible proving key configuration".to_string())
}
```

## Performance Integration

### Metrics Collection
```rust
#[cfg(feature = "bench-metrics")]
use metrics::{counter, histogram, gauge};

pub fn prove_with_metrics<VC, E>(
    prover: &StarkProver<VC, E>,
    input: StdIn,
    program_name: &str,
) -> VmStarkProof {
    let start = std::time::Instant::now();
    
    // Set program name for automatic metrics grouping
    prover.set_program_name(program_name);
    
    let proof = prover.generate_proof_for_outer_recursion(input);
    
    #[cfg(feature = "bench-metrics")]
    {
        histogram!("proof_generation_time", start.elapsed().as_secs_f64());
        counter!("proofs_generated", 1);
    }
    
    proof
}
```

### Tracing Integration
```rust
use tracing::{info_span, instrument, info};

#[instrument(skip(prover, input))]
pub async fn prove_async<VC, E>(
    prover: &StarkProver<VC, E>,
    input: StdIn,
) -> VmStarkProof {
    let span = info_span!("async_proving");
    span.in_scope(|| {
        info!("Starting proof generation");
        let proof = prover.generate_proof_for_outer_recursion(input);
        info!("Proof generation completed");
        proof
    })
}
```

## Testing Integration

### Unit Test Setup
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use openvm_sdk::config::test_utils::*;
    
    fn setup_test_prover() -> AppProver<TestVmConfig, TestEngine> {
        let app_pk = Arc::new(generate_test_app_pk());
        let committed_exe = Arc::new(generate_test_exe());
        AppProver::new(app_pk, committed_exe)
    }
    
    #[test]
    fn test_simple_proof() {
        let prover = setup_test_prover();
        let input = StdIn::default();
        let proof = prover.generate_app_proof_without_continuations(input);
        assert!(verify_proof(&proof));
    }
}
```

### Integration Test Patterns
```rust
// Test configuration compatibility
#[test]
#[should_panic(expected = "incompatible")]
fn test_incompatible_configs() {
    let app_pk = Arc::new(generate_app_pk_with_params(params1));
    let agg_pk = generate_agg_pk_with_params(params2); // Different params
    
    // Should panic due to parameter mismatch
    StarkProver::new(app_pk, committed_exe, agg_pk, config);
}
```

## Deployment Considerations

### Key Management
- Store proving keys securely in production
- Consider key versioning for upgrades
- Validate key integrity before use

### Resource Planning
- STARK proving is CPU-intensive
- Memory requirements scale with circuit size
- Consider proof caching for repeated computations

### Monitoring
- Track proof generation times
- Monitor memory usage during proving
- Alert on proof generation failures

## Common Integration Issues

### Issue 1: Configuration Mismatches
**Problem**: Panic during prover construction
**Solution**: Validate configurations before creating provers

### Issue 2: Memory Issues
**Problem**: Out of memory during large proofs
**Solution**: Use continuation mode or increase system resources

### Issue 3: Performance Issues
**Problem**: Slow proof generation
**Solution**: Optimize VM configuration, use appropriate FRI parameters

### Issue 4: Feature Flag Issues
**Problem**: Missing EVM proving capabilities
**Solution**: Enable `evm-prove` feature and include required dependencies