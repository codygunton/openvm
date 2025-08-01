# OpenVM SDK Prover Component Index

## File Structure

```
prover/
├── mod.rs           # Module exports and EVM prover coordination
├── app.rs           # Application-level VM proving
├── stark.rs         # STARK proof generation and coordination
├── agg.rs           # Proof aggregation logic
├── root.rs          # Root proof generation
├── halo2.rs         # Halo2/EVM proof generation (feature-gated)
└── vm/              # VM-specific proving implementations
    ├── mod.rs       # Trait definitions for VM provers
    ├── local.rs     # Local VM prover implementation
    └── types.rs     # VM proving key and related types
```

## Key Types

### Provers
- `AppProver<VC, E>` - Application VM prover
- `StarkProver<VC, E>` - STARK proof coordinator
- `AggStarkProver<E>` - Aggregation prover
- `RootProver<E>` - Root proof generator
- `Halo2Prover` - EVM-compatible prover
- `EvmHalo2Prover<VC, E>` - Combined STARK+Halo2 prover

### VM Traits
- `ContinuationVmProver<SC>` - Synchronous continuation prover
- `AsyncContinuationVmProver<SC>` - Async continuation prover
- `SingleSegmentVmProver<SC>` - Single segment prover
- `AsyncSingleSegmentVmProver<SC>` - Async single segment prover

### Key Data Structures
- `VmProvingKey<SC, VC>` - VM proving parameters
- `ContinuationVmProof<SC>` - Multi-segment proof
- `VmStarkProof<SC>` - Complete STARK proof
- `RootVmVerifierInput<SC>` - Root verification data

## Module Exports

### Always Available
- All types from `agg.rs`
- All types from `app.rs`
- All types from `root.rs`
- All types from `stark.rs`
- Public `vm` module

### Feature-Gated (`evm-prove`)
- All types from `halo2.rs`
- `EvmHalo2Prover` from internal `evm` module

## Dependencies Graph

```
stark.rs
├── app.rs
│   └── vm/local.rs
│       └── vm/types.rs
└── agg.rs
    └── (aggregation logic)

halo2.rs (when evm-prove)
└── (Halo2 proving)

mod.rs (when evm-prove)
└── EvmHalo2Prover
    ├── stark.rs
    └── halo2.rs
```

## Integration Points

1. **SDK Integration**: Via `crate::prover` module
2. **VM Config**: Through `VmConfig<F>` trait
3. **Proving Keys**: Via `keygen` module types
4. **Input/Output**: `StdIn` input, various proof types output
5. **Execution**: Through `NonRootCommittedExe`

## Usage Patterns

### Basic App Proof
```rust
AppProver::new(app_vm_pk, app_committed_exe)
    .generate_app_proof(input)
```

### Full STARK Proof
```rust
StarkProver::new(app_pk, app_committed_exe, agg_stark_pk, agg_tree_config)
    .generate_proof_for_outer_recursion(input)
```

### EVM-Compatible Proof
```rust
EvmHalo2Prover::new(reader, app_pk, app_committed_exe, agg_pk, agg_tree_config)
    .generate_proof_for_evm(input)
```