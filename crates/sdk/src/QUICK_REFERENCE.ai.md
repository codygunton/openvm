# OpenVM SDK - Quick Reference

## Core Types

### Main SDK Interface
```rust
pub type Sdk = GenericSdk<BabyBearPoseidon2Engine>;

impl<E: StarkFriEngine<SC>> GenericSdk<E> {
    pub fn new() -> Self
    pub fn with_agg_tree_config(self, config: AggregationTreeConfig) -> Self
}
```

### Configuration Types

#### AppConfig
```rust
pub struct AppConfig<VC> {
    pub app_fri_params: AppFriParams,
    pub app_vm_config: VC,
    pub leaf_fri_params: LeafFriParams,
    pub compiler_options: CompilerOptions,
}
```

#### AggConfig
```rust
pub struct AggConfig {
    pub agg_stark_config: AggStarkConfig,
    pub halo2_config: Halo2Config,
}
```

#### AggStarkConfig
```rust
pub struct AggStarkConfig {
    pub max_num_user_public_values: usize,  // default: 32768
    pub leaf_fri_params: FriParameters,
    pub internal_fri_params: FriParameters,
    pub root_fri_params: FriParameters,
    pub profiling: bool,
    pub compiler_options: CompilerOptions,
    pub root_max_constraint_degree: usize,
}
```

### Key Types

#### AppProvingKey
```rust
pub struct AppProvingKey<VC> {
    pub leaf_committed_exe: Arc<NonRootCommittedExe>,
    pub leaf_fri_params: FriParameters,
    pub app_vm_pk: Arc<VmProvingKey<SC, VC>>,
}
```

#### AppVerifyingKey
```rust
pub struct AppVerifyingKey {
    pub fri_params: FriParameters,
    pub app_vm_vk: MultiStarkVerifyingKey<SC>,
    pub memory_dimensions: MemoryDimensions,
}
```

### Commitment Types

#### AppExecutionCommit
```rust
pub struct AppExecutionCommit {
    pub app_exe_commit: CommitBytes,  // Executable commitment
    pub app_vm_commit: CommitBytes,   // VM verifier commitment
}
```

#### StdIn
```rust
pub struct StdIn {
    pub buffer: VecDeque<Vec<F>>,
    pub kv_store: HashMap<Vec<u8>, Vec<u8>>,
}

impl StdIn {
    pub fn from_bytes(data: &[u8]) -> Self
    pub fn write<T: Serialize>(&mut self, data: &T)
    pub fn write_bytes(&mut self, data: &[u8])
    pub fn write_field(&mut self, data: &[F])
}
```

## Core SDK Methods

### Build & Transpile
```rust
impl<E> GenericSdk<E> {
    pub fn build(
        &self,
        guest_opts: GuestOptions,
        vm_config: &SdkVmConfig,
        pkg_dir: impl AsRef<Path>,
        target_filter: &Option<TargetFilter>,
        init_file_name: Option<&str>,
    ) -> Result<Elf>

    pub fn transpile(
        &self,
        elf: Elf,
        transpiler: Transpiler<F>,
    ) -> Result<VmExe<F>, TranspilerError>
}
```

### Execution
```rust
impl<E> GenericSdk<E> {
    pub fn execute<VC: VmConfig<F>>(
        &self,
        exe: VmExe<F>,
        vm_config: VC,
        inputs: StdIn,
    ) -> Result<Vec<F>, ExecutionError>
}
```

### Commitment
```rust
impl<E> GenericSdk<E> {
    pub fn commit_app_exe(
        &self,
        app_fri_params: FriParameters,
        exe: VmExe<F>,
    ) -> Result<Arc<NonRootCommittedExe>>
}
```

### Key Generation
```rust
impl<E> GenericSdk<E> {
    pub fn app_keygen<VC: VmConfig<F>>(
        &self,
        config: AppConfig<VC>,
    ) -> Result<AppProvingKey<VC>>

    pub fn agg_stark_keygen(
        &self,
        config: AggStarkConfig,
    ) -> Result<AggStarkProvingKey>

    #[cfg(feature = "evm-prove")]
    pub fn agg_keygen(
        &self,
        config: AggConfig,
        reader: &impl Halo2ParamsReader,
        pv_handler: &impl StaticVerifierPvHandler,
    ) -> Result<AggProvingKey>
}
```

### Proof Generation
```rust
impl<E> GenericSdk<E> {
    // Direct app proof
    pub fn generate_app_proof<VC: VmConfig<F>>(
        &self,
        app_pk: Arc<AppProvingKey<VC>>,
        app_committed_exe: Arc<NonRootCommittedExe>,
        inputs: StdIn,
    ) -> Result<ContinuationVmProof<SC>>

    // E2E STARK proof
    pub fn generate_e2e_stark_proof<VC: VmConfig<F>>(
        &self,
        app_pk: Arc<AppProvingKey<VC>>,
        app_exe: Arc<NonRootCommittedExe>,
        agg_stark_pk: AggStarkProvingKey,
        inputs: StdIn,
    ) -> Result<VmStarkProof<SC>>

    // EVM-compatible proof
    #[cfg(feature = "evm-prove")]
    pub fn generate_evm_proof<VC: VmConfig<F>>(
        &self,
        reader: &impl Halo2ParamsReader,
        app_pk: Arc<AppProvingKey<VC>>,
        app_exe: Arc<NonRootCommittedExe>,
        agg_pk: AggProvingKey,
        inputs: StdIn,
    ) -> Result<EvmProof>
}
```

### Verification
```rust
impl<E> GenericSdk<E> {
    pub fn verify_app_proof(
        &self,
        app_vk: &AppVerifyingKey,
        proof: &ContinuationVmProof<SC>,
    ) -> Result<VerifiedContinuationVmPayload>

    pub fn verify_e2e_stark_proof(
        &self,
        agg_stark_pk: &AggStarkProvingKey,
        proof: &VmStarkProof<SC>,
        expected_exe_commit: &Bn254Fr,
        expected_vm_commit: &Bn254Fr,
    ) -> Result<AppExecutionCommit>

    #[cfg(feature = "evm-verify")]
    pub fn verify_evm_halo2_proof(
        &self,
        openvm_verifier: &types::EvmHalo2Verifier,
        evm_proof: EvmProof,
    ) -> Result<u64>  // Returns gas cost
}
```

### EVM Integration
```rust
impl<E> GenericSdk<E> {
    #[cfg(feature = "evm-verify")]
    pub fn generate_halo2_verifier_solidity(
        &self,
        reader: &impl Halo2ParamsReader,
        agg_pk: &AggProvingKey,
    ) -> Result<types::EvmHalo2Verifier>

    pub fn generate_root_verifier_asm(
        &self,
        agg_stark_pk: &AggStarkProvingKey,
    ) -> String
}
```

## Prover Types

### AppProver
```rust
pub struct AppProver<VC, E> {
    pub app_vm_pk: Arc<VmProvingKey<SC, VC>>,
    pub app_committed_exe: Arc<NonRootCommittedExe>,
}

impl<VC, E> AppProver<VC, E> {
    pub fn generate_app_proof(&self, inputs: StdIn) -> ContinuationVmProof<SC>
}
```

### StarkProver
```rust
pub struct StarkProver<VC, E> {
    pub app_prover: AppProver<VC, E>,
    pub agg_stark_pk: AggStarkProvingKey,
    pub agg_tree_config: AggregationTreeConfig,
}

impl<VC, E> StarkProver<VC, E> {
    pub fn generate_e2e_stark_proof(&self, input: StdIn) -> VmStarkProof<SC>
    pub fn generate_root_verifier_input(&self, input: StdIn) -> RootVmVerifierInput<SC>
}
```

## Common Patterns

### Basic Proof Generation
```rust
let sdk = Sdk::new();
let config = AppConfig::new(fri_params, vm_config);
let app_pk = sdk.app_keygen(config)?;
let proof = sdk.generate_app_proof(app_pk, exe, inputs)?;
```

### EVM Proof Generation
```rust
let agg_config = AggConfig::default();
let agg_pk = sdk.agg_keygen(agg_config, &reader, &handler)?;
let evm_proof = sdk.generate_evm_proof(&reader, app_pk, exe, agg_pk, inputs)?;
```

### Proof Verification
```rust
let verified = sdk.verify_app_proof(&app_vk, &proof)?;
assert_eq!(verified.exe_commit, expected_commit);
```

## Constants

### Default Parameters
```rust
pub const DEFAULT_APP_LOG_BLOWUP: usize = 1;
pub const DEFAULT_LEAF_LOG_BLOWUP: usize = 1;
pub const DEFAULT_INTERNAL_LOG_BLOWUP: usize = 2;
pub const DEFAULT_ROOT_LOG_BLOWUP: usize = 3;
pub const DEFAULT_MAX_NUM_PUBLIC_VALUES: usize = 32768;
```

### Aggregation Tree Defaults
```rust
const DEFAULT_NUM_CHILDREN_LEAF: usize = 1;
const DEFAULT_NUM_CHILDREN_INTERNAL: usize = 3;
const DEFAULT_MAX_INTERNAL_WRAPPER_LAYERS: usize = 4;
```

## Feature Flags

### `evm-prove`
Enables EVM proof generation:
- `AggProvingKey` generation
- `Halo2Prover` functionality
- Static verifier support

### `evm-verify`
Enables EVM verification:
- Solidity verifier generation
- Contract interfaces
- Gas cost estimation

### Performance Features
- `parallel` - Enable parallelization (default)
- `jemalloc` - Use jemalloc allocator (default)
- `mimalloc` - Use mimalloc allocator
- `profiling` - Enable profiling metrics
- `bench-metrics` - Enable benchmarking

## Error Types

- `ExecutionError` - VM execution failures
- `TranspilerError` - ELF transpilation errors
- `EvmProofConversionError` - EVM proof format errors
- Standard `eyre::Result` for most operations