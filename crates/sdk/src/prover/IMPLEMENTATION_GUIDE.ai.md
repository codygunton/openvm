# OpenVM SDK Prover Implementation Guide

## Overview

This guide provides implementation details for working with the OpenVM SDK prover component, including how to create custom provers, integrate with different proof systems, and optimize proving performance.

## Core Architecture

### Prover Hierarchy

```
EvmHalo2Prover (optional, for EVM)
    └── StarkProver (main coordinator)
        ├── AppProver (application execution)
        │   └── VmLocalProver (VM-specific)
        └── AggStarkProver (aggregation)
            ├── Leaf proving
            └── Root proving
```

### Key Design Patterns

1. **Separation of Concerns**
   - App proving handles VM execution proofs
   - Aggregation proving handles recursion
   - Optional Halo2 layer for EVM compatibility

2. **Generic Over Engines**
   - All provers are generic over `StarkFriEngine<SC>`
   - Allows different field/hash configurations

3. **Arc-based Sharing**
   - Proving keys wrapped in `Arc` for efficiency
   - Committed executables shared across provers

## Implementation Details

### Creating a Custom App Prover

```rust
use openvm_sdk::prover::AppProver;
use openvm_sdk::{F, SC};

impl<VC, E: StarkFriEngine<SC>> AppProver<VC, E> {
    pub fn new(
        app_vm_pk: Arc<VmProvingKey<SC, VC>>,
        app_committed_exe: Arc<NonRootCommittedExe>,
    ) -> Self
    where
        VC: VmConfig<F>,
    {
        Self {
            program_name: None,
            app_prover: VmLocalProver::<SC, VC, E>::new(app_vm_pk, app_committed_exe),
        }
    }
}
```

Key points:
- Requires VM proving key and committed executable
- Program name is optional for metrics/tracing
- Delegates to `VmLocalProver` for actual proving

### Implementing Continuation Support

The prover supports two modes:

1. **With Continuations** (for large programs):
```rust
pub fn generate_app_proof(&self, input: StdIn) -> ContinuationVmProof<SC> {
    assert!(self.vm_config().system().continuation_enabled);
    ContinuationVmProver::prove(&self.app_prover, input)
}
```

2. **Without Continuations** (for small programs):
```rust
pub fn generate_app_proof_without_continuations(&self, input: StdIn) -> Proof<SC> {
    assert!(!self.vm_config().system().continuation_enabled);
    SingleSegmentVmProver::prove(&self.app_prover, input)
}
```

### STARK Prover Coordination

The `StarkProver` manages the proving pipeline:

```rust
pub fn generate_proof_for_outer_recursion(&self, input: StdIn) -> Proof<RootSC> {
    // 1. Generate app proof (may have multiple segments)
    let app_proof = self.app_prover.generate_app_proof(input);
    
    // 2. Aggregate into root proof
    self.agg_prover.generate_root_proof(app_proof)
}
```

### Parameter Validation

The STARK prover validates compatibility:

```rust
assert_eq!(
    app_pk.leaf_fri_params, agg_stark_pk.leaf_vm_pk.fri_params,
    "App VM is incompatible with Agg VM because of leaf FRI parameters"
);
assert_eq!(
    app_pk.app_vm_pk.vm_config.system().num_public_values,
    agg_stark_pk.num_user_public_values(),
    "App VM is incompatible with Agg VM because of the number of public values"
);
```

### EVM Integration (Feature-Gated)

When `evm-prove` is enabled:

```rust
pub fn generate_proof_for_evm(&self, input: StdIn) -> EvmProof {
    // 1. Generate STARK proof with outer recursion
    let root_proof = self.stark_prover.generate_proof_for_outer_recursion(input);
    
    // 2. Wrap in Halo2 proof for EVM
    self.halo2_prover.prove_for_evm(&root_proof)
}
```

## Performance Optimization

### 1. FRI Parameter Tuning
- Adjust `log_blowup` for proof size vs generation time
- Monitor with `bench-metrics` feature

### 2. Continuation Segment Size
- Balance memory usage vs number of segments
- Configure in VM system parameters

### 3. Parallelization
- App segments can be proven in parallel
- Aggregation tree structure affects parallelism

### 4. Caching
- Proving keys should be loaded once and reused
- Use `Arc` to share across threads

## Error Handling

Common error scenarios:

1. **Parameter Mismatch**
   - FRI parameters between app and agg
   - Public values count mismatch

2. **Configuration Errors**
   - Using wrong proof method for continuation setting
   - Invalid aggregation tree configuration

3. **Resource Constraints**
   - Out of memory for large proofs
   - Timeout on proof generation

## Testing Provers

Example test setup:

```rust
#[test]
fn test_app_prover() {
    let app_pk = generate_app_proving_key(&vm_config);
    let exe = compile_program("test.elf");
    let committed_exe = commit_exe(exe);
    
    let prover = AppProver::new(Arc::new(app_pk), Arc::new(committed_exe));
    let input = StdIn::default();
    
    let proof = prover.generate_app_proof(input);
    verify_proof(&proof, &app_pk.vk);
}
```

## Integration with SDK

The prover integrates with other SDK components:

1. **Keygen**: Provides proving/verifying keys
2. **Commit**: Generates committed executables
3. **Config**: Supplies VM and aggregation configuration
4. **StdIn**: Provides program input

## Advanced Usage

### Custom VM Prover

Implement the trait for custom proving logic:

```rust
impl<SC: StarkGenericConfig> ContinuationVmProver<SC> for MyCustomProver {
    fn prove(&self, input: impl Into<Streams<Val<SC>>>) -> ContinuationVmProof<SC> {
        // Custom proving logic
    }
}
```

### Async Proving

Use async traits for concurrent proving:

```rust
#[async_trait]
impl<SC: StarkGenericConfig> AsyncContinuationVmProver<SC> for MyAsyncProver {
    async fn prove(
        &self,
        input: impl Into<Streams<Val<SC>>> + Send + Sync,
    ) -> ContinuationVmProof<SC> {
        // Async proving logic
    }
}
```