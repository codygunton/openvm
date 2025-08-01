# OpenVM Prove Benchmarks - Implementation Guide

## Core Implementation Details

### BenchmarkCli Structure

The CLI parser provides unified configuration for all benchmarks:

```rust
pub struct BenchmarkCli {
    pub app_log_blowup: Option<usize>,      // Application proof blowup
    pub leaf_log_blowup: Option<usize>,     // Leaf aggregation blowup
    pub internal_log_blowup: Option<usize>, // Internal aggregation blowup
    pub root_log_blowup: Option<usize>,     // Root aggregation blowup
    pub halo2_outer_k: Option<usize>,       // Halo2 circuit size
    pub halo2_wrapper_k: Option<usize>,     // Wrapper circuit size
    pub kzg_params_dir: Option<PathBuf>,    // KZG parameters location
    pub max_segment_length: Option<usize>,  // Continuation segments
    pub agg_tree_config: AggregationTreeConfig,
    pub profiling: bool,
}
```

### Proof Generation Pipeline

The `bench_from_exe` function implements the core proving pipeline:

1. **Key Generation Phase**
   ```rust
   let app_pk = AppProvingKey::keygen(app_config.clone());
   ```
   - Generates arithmetic gates
   - Computes fixed traces
   - Creates commitment keys

2. **Executable Commitment**
   ```rust
   let committed_exe = commit_app_exe(fri_params, exe);
   ```
   - Pre-processes program code
   - Generates static traces
   - Creates program commitments

3. **Proof Generation**
   ```rust
   let prover = AppProver::new(app_pk.app_vm_pk, committed_exe);
   let app_proof = prover.generate_app_proof(input_stream);
   ```
   - Executes program
   - Generates execution trace
   - Creates STARK proofs per segment

4. **Verification**
   ```rust
   sdk.verify_app_proof(&app_vk, &app_proof)
   ```
   - Verifies STARK proofs
   - Checks boundary conditions
   - Validates program commitment

### VM Configuration Patterns

#### Minimal Configuration (RV32IM)
```rust
let config = Rv32ImConfig::default();
let transpiler = Transpiler::<BabyBear>::default()
    .with_extension(Rv32ITranspilerExtension)
    .with_extension(Rv32MTranspilerExtension)
    .with_extension(Rv32IoTranspilerExtension);
```

#### Full Feature Configuration
```rust
let vm_config = SdkVmConfig::builder()
    .system(SystemConfig::default().with_continuations())
    .rv32i(Default::default())
    .rv32m(Default::default())
    .io(Default::default())
    .keccak(Default::default())
    .sha256(Default::default())
    .bigint(Default::default())
    .modular(ModularExtension::new(moduli))
    .fp2(Fp2Extension::new(fp2_moduli))
    .ecc(WeierstrassExtension::new(curves))
    .pairing(PairingExtension::new(pairing_curves))
    .build();
```

### Extension Integration

#### Cryptographic Extensions

**Keccak256 Integration**:
```rust
.keccak(Default::default())
// Transpiler must include:
.with_extension(Keccak256TranspilerExtension)
```

**ECC Operations**:
```rust
.ecc(WeierstrassExtension::new(vec![
    SECP256K1_CONFIG,
    P256_CONFIG,
]))
```

**Pairing Operations**:
```rust
.pairing(PairingExtension::new(vec![
    PairingCurve::Bn254,
    PairingCurve::Bls12_381,
]))
```

### Memory Management

#### Segmentation Strategy
```rust
if let Some(max_segment_length) = self.max_segment_length {
    app_vm_config.system_mut().set_segmentation_strategy(Arc::new(
        DefaultSegmentationStrategy::new_with_max_segment_len(max_segment_length)
    ));
}
```

#### Allocator Selection
- **jemalloc**: Better for multi-threaded proving
- **mimalloc**: Lower memory overhead
- **system**: Default allocator, predictable behavior

### Profiling Implementation

When profiling is enabled:
```rust
app_vm_config.system_mut().profiling = true;
// In compiler options:
compiler_options: CompilerOptions {
    enable_cycle_tracker: self.profiling,
    ..Default::default()
}
```

Profiling adds:
- Cycle counting per instruction
- Memory access tracking
- Constraint evaluation timing

### Input/Output Handling

#### StdIn Configuration
```rust
let mut stdin = StdIn::default();
stdin.write(&input_data);        // Generic data
stdin.write_slice(&byte_array);  // Byte arrays
stdin.write_field_slice(&fields); // Field elements
```

#### Program Output
Programs write output via:
- `env::write` for structured data
- `env::commit` for public commitments
- Return values in guest `main()`

### Error Handling Patterns

```rust
fn build_bench_program(&self, name: &str) -> Result<Elf> {
    let profile = if self.profiling { "profiling" } else { "release" };
    let manifest_dir = get_programs_dir().join(name);
    vm_config.write_to_init_file(&manifest_dir, None)?;
    build_elf(&manifest_dir, profile)
}
```

### Aggregation Implementation

When aggregation is enabled:
```rust
#[cfg(feature = "aggregation")]
let leaf_vm_config = self.agg_config().agg_stark_config.leaf_vm_config();
let leaf_vm_pk = leaf_keygen(app_config.leaf_fri_params.fri_params, leaf_vm_config);
let leaf_prover = VmLocalProver::new(leaf_vm_pk, app_pk.leaf_committed_exe);
let leaf_controller = LeafProvingController {
    num_children: agg_tree_config.num_children_leaf,
};
leaf_controller.generate_proof(&leaf_prover, &app_proof);
```

### Performance Optimization Techniques

1. **Parallel Proof Generation**
   - Uses rayon for parallel constraint evaluation
   - Multi-threaded witness generation
   - Concurrent polynomial commitments

2. **Memory Optimization**
   - Streaming trace generation
   - Incremental proof construction
   - Efficient field arithmetic

3. **Caching Strategies**
   - Preprocessed gates
   - Static trace caching
   - KZG parameter reuse

### Testing Infrastructure

#### Benchmark Validation
```rust
run_with_metric_collection("OUTPUT_PATH", || -> Result<()> {
    // Benchmark implementation
    args.bench_from_exe("name", config, exe, stdin)
})
```

#### Metric Collection
- Proving time per segment
- Total memory usage
- Proof size
- Verification time
- CPU utilization

### Common Implementation Patterns

#### Feature-Gated Compilation
```rust
#[cfg(not(feature = "aggregation"))]
None,
#[cfg(feature = "aggregation")]
Some(self.agg_config().agg_stark_config.leaf_vm_config())
```

#### Binary Configuration
```toml
[[bin]]
name = "kitchen_sink"
path = "src/bin/kitchen_sink.rs"
required-features = ["evm"]
```

### Integration with Guest Programs

1. **ELF Loading**
   ```rust
   let elf = std::fs::read(elf_path)?;
   let elf = Elf::decode(&elf)?;
   ```

2. **Transpilation**
   ```rust
   let exe = VmExe::from_elf(elf, transpiler)?;
   ```

3. **Execution**
   - Set up memory layout
   - Initialize program counter
   - Load input data
   - Execute until termination

### Advanced Features

#### Custom Initialization
```rust
vm_config.write_to_init_file(&manifest_dir, Some("custom_init.rs"))?;
```

#### EVM Verification
```rust
let evm_prover = EvmHalo2Prover::new(
    DefaultStaticVerifierPvHandler,
    sdk.app_verifier.clone(),
    halo2_params_reader,
);
```

### Debugging Support

1. **Trace Inspection**
   - Enable trace dumping
   - Instruction-level debugging
   - Memory state snapshots

2. **Constraint Verification**
   - Individual gate checking
   - Polynomial degree validation
   - Commitment verification

3. **Performance Analysis**
   - Flamegraph generation
   - Memory profiling
   - Bottleneck identification