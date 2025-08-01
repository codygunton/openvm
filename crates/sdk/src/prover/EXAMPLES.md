# Prover Usage Examples

## Basic App Proof Generation

### Single Segment Proof
```rust
use openvm_sdk::prover::AppProver;
use openvm_sdk::{StdIn, NonRootCommittedExe};
use std::sync::Arc;

// Setup prover with proving key and committed executable
let app_prover = AppProver::new(
    app_vm_pk,
    Arc::new(committed_exe)
).with_program_name("my_program");

// Generate proof for single execution segment
let input = StdIn::default(); // Configure your inputs
let proof = app_prover.generate_app_proof_without_continuations(input);
```

### Continuation Proof
```rust
// For programs requiring multiple execution segments
let app_prover = AppProver::new(app_vm_pk, Arc::new(committed_exe))
    .with_program_name("large_program");

// Generates proof for all continuation segments
let continuation_proof = app_prover.generate_app_proof(input);
```

## STARK Proving Pipeline

### Basic STARK Proof
```rust
use openvm_sdk::prover::StarkProver;
use openvm_sdk::config::AggregationTreeConfig;

// Create STARK prover with app and aggregation keys
let stark_prover = StarkProver::new(
    app_pk,
    Arc::new(app_committed_exe),
    agg_stark_pk,
    agg_tree_config
);

// Set program name for metrics/debugging
stark_prover.set_program_name("fibonacci");

// Generate aggregated proof
let root_proof = stark_prover.generate_proof_for_outer_recursion(input);
```

### Proof Verification Input
```rust
// Generate proof with verification input for external verifiers
let (proof, verifier_input) = stark_prover.generate_proof_and_verifier_input(input);

// verifier_input contains public values and verification parameters
println!("Public values: {:?}", verifier_input.public_values);
```

## EVM-Compatible Proofs

### Halo2 EVM Proof Generation
```rust
use openvm_sdk::prover::EvmHalo2Prover;
use openvm_native_recursion::halo2::utils::Halo2ParamsReader;

// Setup EVM prover with both STARK and Halo2 components
let evm_prover = EvmHalo2Prover::new(
    &params_reader,
    app_pk,
    Arc::new(app_committed_exe),
    agg_pk,
    agg_tree_config
);

// Generate proof compatible with EVM verification
let evm_proof = evm_prover.generate_proof_for_evm(input);

// evm_proof can be verified on Ethereum
```

## Aggregation Examples

### Tree-Based Aggregation
```rust
use openvm_sdk::prover::AggStarkProver;

// Create aggregation prover
let agg_prover = AggStarkProver::new(
    agg_stark_pk,
    Arc::new(leaf_committed_exe),
    agg_tree_config
);

// Aggregate multiple proofs into a single root proof
let individual_proofs = vec![proof1, proof2, proof3];
let aggregated_proof = agg_prover.aggregate_proofs(individual_proofs);
```

## VM Prover Variants

### Local VM Proving
```rust
use openvm_sdk::prover::vm::VmLocalProver;

// Direct VM proving without aggregation
let vm_prover = VmLocalProver::new(vm_pk, Arc::new(committed_exe));
let vm_proof = vm_prover.prove(input);
```

### Continuation VM Proving
```rust
use openvm_sdk::prover::vm::ContinuationVmProver;

// For large programs requiring segmentation
let continuation_proof = ContinuationVmProver::prove(&vm_prover, input);
```

## Configuration Examples

### Aggregation Tree Configuration
```rust
use openvm_sdk::config::AggregationTreeConfig;

// Configure aggregation parameters
let agg_config = AggregationTreeConfig {
    max_num_user_public_values: 256,
    agg_params: agg_params,
    // ... other configuration
};
```

### Program Input Setup
```rust
use openvm_sdk::StdIn;

// Setup program inputs
let mut stdin = StdIn::default();
stdin.write(&42u32);        // Write integer
stdin.write(&"hello");      // Write string
stdin.write_vec(&vec![1, 2, 3]); // Write vector
```

## Error Handling Examples

### Configuration Validation
```rust
// Provers will panic on incompatible configurations
match std::panic::catch_unwind(|| {
    StarkProver::new(app_pk, committed_exe, agg_pk, config)
}) {
    Ok(prover) => {
        // Prover created successfully
    },
    Err(_) => {
        // Handle configuration mismatch
        eprintln!("Incompatible proving keys or configuration");
    }
}
```

### Continuation Mode Validation
```rust
// Check if continuation is enabled before choosing proving method
if app_prover.vm_config().system().continuation_enabled {
    let proof = app_prover.generate_app_proof(input);
} else {
    let proof = app_prover.generate_app_proof_without_continuations(input);
}
```

## Performance Monitoring

### Metrics Integration
```rust
// Metrics are automatically tracked when bench-metrics feature is enabled
#[cfg(feature = "bench-metrics")]
{
    // FRI blowup factor is automatically recorded
    // Use program names for grouping metrics
    let prover = app_prover.with_program_name("performance_test");
}
```

### Tracing Integration
```rust
use tracing::{info_span, instrument};

// Provers automatically create tracing spans
// Additional custom spans can be added
#[instrument(skip(prover, input))]
fn prove_with_tracing(prover: &AppProver<_, _>, input: StdIn) {
    let proof = prover.generate_app_proof(input);
}
```