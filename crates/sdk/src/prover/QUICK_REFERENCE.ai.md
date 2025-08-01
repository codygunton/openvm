# OpenVM SDK Prover Quick Reference

## Common Imports

```rust
use openvm_sdk::prover::{
    AppProver, StarkProver, AggStarkProver, RootProver,
    vm::{ContinuationVmProver, SingleSegmentVmProver},
};
use openvm_sdk::{StdIn, F, SC, RootSC};

// With evm-prove feature
use openvm_sdk::prover::{Halo2Prover, EvmHalo2Prover};
```

## Basic Prover Creation

### App Prover
```rust
let app_prover = AppProver::<VC, E>::new(
    Arc::new(app_vm_pk),
    Arc::new(app_committed_exe),
)
.with_program_name("my_program");
```

### STARK Prover
```rust
let stark_prover = StarkProver::<VC, E>::new(
    Arc::new(app_pk),
    Arc::new(app_committed_exe),
    agg_stark_pk,
    agg_tree_config,
);
```

### EVM Prover (with feature)
```rust
let evm_prover = EvmHalo2Prover::<VC, E>::new(
    &params_reader,
    Arc::new(app_pk),
    Arc::new(app_committed_exe),
    agg_pk,
    agg_tree_config,
);
```

## Proof Generation

### App Proof (with continuations)
```rust
let app_proof: ContinuationVmProof<SC> = app_prover.generate_app_proof(input);
```

### App Proof (single segment)
```rust
let proof: Proof<SC> = app_prover.generate_app_proof_without_continuations(input);
```

### Root Proof
```rust
let root_proof: Proof<RootSC> = stark_prover.generate_proof_for_outer_recursion(input);
```

### EVM Proof
```rust
let evm_proof: EvmProof = evm_prover.generate_proof_for_evm(input);
```

### E2E STARK Proof
```rust
let stark_proof: VmStarkProof<SC> = stark_prover.generate_e2e_stark_proof(input);
```

## Key Methods

### AppProver
| Method | Description | Returns |
|--------|-------------|---------|
| `new(app_vm_pk, app_committed_exe)` | Create new app prover | `Self` |
| `set_program_name(&mut self, name)` | Set program name for metrics | `&mut Self` |
| `generate_app_proof(input)` | Generate proof with continuations | `ContinuationVmProof<SC>` |
| `generate_app_proof_without_continuations(input)` | Generate single segment proof | `Proof<SC>` |
| `vm_config()` | Get VM configuration | `&VC` |

### StarkProver
| Method | Description | Returns |
|--------|-------------|---------|
| `new(app_pk, exe, agg_pk, config)` | Create new STARK prover | `Self` |
| `generate_proof_for_outer_recursion(input)` | Generate root proof | `Proof<RootSC>` |
| `generate_root_verifier_input(input)` | Generate verifier input | `RootVmVerifierInput<SC>` |
| `generate_e2e_stark_proof(input)` | Generate complete STARK proof | `VmStarkProof<SC>` |

### EvmHalo2Prover
| Method | Description | Returns |
|--------|-------------|---------|
| `new(reader, app_pk, exe, agg_pk, config)` | Create new EVM prover | `Self` |
| `generate_proof_for_evm(input)` | Generate EVM-compatible proof | `EvmProof` |

## VM Prover Traits

### ContinuationVmProver
```rust
trait ContinuationVmProver<SC: StarkGenericConfig> {
    fn prove(&self, input: impl Into<Streams<Val<SC>>>) -> ContinuationVmProof<SC>;
}
```

### SingleSegmentVmProver
```rust
trait SingleSegmentVmProver<SC: StarkGenericConfig> {
    fn prove(&self, input: impl Into<Streams<Val<SC>>>) -> Proof<SC>;
}
```

### Async Variants
```rust
#[async_trait]
trait AsyncContinuationVmProver<SC: StarkGenericConfig> {
    async fn prove(&self, input: impl Into<Streams<Val<SC>>> + Send + Sync) 
        -> ContinuationVmProof<SC>;
}
```

## Type Parameters

- `VC`: VM configuration type implementing `VmConfig<F>`
- `E`: STARK engine implementing `StarkFriEngine<SC>`
- `SC`: STARK configuration (usually `BabyBearPoseidon2`)
- `RootSC`: Root STARK configuration (for outer recursion)
- `F`: Field type (usually `BabyBear`)

## Common Patterns

### Full Proving Pipeline
```rust
// 1. Setup
let app_pk = keygen_app_pk(&app_config);
let exe = build_exe_from_elf(&elf);
let committed_exe = commit_app_exe(app_fri_params, exe);

// 2. Create prover
let prover = StarkProver::new(
    Arc::new(app_pk),
    Arc::new(committed_exe),
    agg_stark_pk,
    agg_config,
);

// 3. Generate proof
let input = StdIn::from_bytes(&input_bytes);
let proof = prover.generate_proof_for_outer_recursion(input);
```

### Conditional Proving
```rust
let proof = if vm_config.system().continuation_enabled {
    app_prover.generate_app_proof(input)
} else {
    let single_proof = app_prover.generate_app_proof_without_continuations(input);
    // Convert to continuation format if needed
};
```

## Feature Flags

| Flag | Enables | Use Case |
|------|---------|----------|
| `evm-prove` | Halo2/EVM proving | On-chain verification |
| `bench-metrics` | Performance metrics | Profiling/optimization |

## Error Handling

Common assertions to check:
```rust
// Continuation mode check
assert!(vm_config.system().continuation_enabled);

// Parameter compatibility
assert_eq!(app_pk.leaf_fri_params, agg_pk.leaf_vm_pk.fri_params);

// Public values count
assert_eq!(
    app_vm_config.system().num_public_values,
    agg_stark_pk.num_user_public_values()
);
```