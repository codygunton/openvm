# OpenVM SDK Config Component - Detailed Documentation

## Module Structure

### `mod.rs` - Core Configuration Types

#### Application Configuration

**`AppConfig<VC>`**
- Generic over VM config type `VC`
- Contains:
  - `app_fri_params`: FRI parameters for app-level proofs
  - `app_vm_config`: The actual VM configuration
  - `leaf_fri_params`: FRI parameters for leaf aggregation
  - `compiler_options`: For AggVM debugging (not typical user flow)
- Constructors:
  - `new()`: Basic constructor with default leaf params
  - `new_with_leaf_fri_params()`: Specify custom leaf params

**FRI Parameter Wrappers**
- `AppFriParams`: Wrapper around `FriParameters` for app level
- `LeafFriParams`: Wrapper around `FriParameters` for leaf level
- Both implement `Default` with 100-bit security and appropriate log blowup

#### Aggregation Configuration

**`AggConfig`**
- Top-level aggregation configuration combining:
  - `agg_stark_config`: STARK aggregation parameters
  - `halo2_config`: SNARK wrapper configuration

**`AggStarkConfig`**
- STARK-specific aggregation settings:
  - `max_num_user_public_values`: Max public values (default from circuit arch)
  - `leaf_fri_params`: FRI params for leaf verifier
  - `internal_fri_params`: FRI params for internal verifier
  - `root_fri_params`: FRI params for root verifier
  - `profiling`: Enable/disable profiling for all agg VMs
  - `compiler_options`: For debugging
  - `root_max_constraint_degree`: For FRI logup chunking
- Methods:
  - `leaf_vm_config()`: Generate NativeConfig for leaf VM
  - `internal_vm_config()`: Generate NativeConfig for internal VM
  - `root_verifier_vm_config()`: Generate NativeConfig for root VM

**`Halo2Config`**
- SNARK configuration:
  - `verifier_k`: Log degree for outer recursion circuit
  - `wrapper_k`: Optional manual setting (auto-tuned if None)
  - `profiling`: Enable/disable halo2 VM profiling

**`AggregationTreeConfig`**
- Tree structure parameters:
  - `num_children_leaf`: App proofs per leaf (default: 1)
  - `num_children_internal`: Proofs per internal node (default: 3)
  - `max_internal_wrapper_layers`: Safety threshold (default: 4)
- Implements `clap::Args` for CLI integration

#### Constants
- `DEFAULT_APP_LOG_BLOWUP`: 1
- `DEFAULT_LEAF_LOG_BLOWUP`: 1
- `DEFAULT_INTERNAL_LOG_BLOWUP`: 2
- `DEFAULT_ROOT_LOG_BLOWUP`: 3
- `SBOX_SIZE`: 7 (for constraint degree calculations)

### `global.rs` - SDK VM Configuration

#### `SdkVmConfig`

**Structure**
- Builder pattern with `bon::Builder`
- Fields for each extension:
  - `system`: Required system configuration
  - Optional extensions: `rv32i`, `io`, `keccak`, `sha256`, `native`, `castf`
  - Configurable extensions: `rv32m`, `bigint`, `modular`, `fp2`, `pairing`, `ecc`

**Key Methods**
- `transpiler()`: Creates transpiler with extensions based on config
- `create_chip_complex()`: Builds VM chip complex with proper extension ordering
- `generate_init_file_contents()`: Generates initialization for algebraic extensions

**Extension Dependencies**
- `bigint` and `rv32m` share range checker sizes
- `fp2` requires `modular`
- Extension order matters for proper initialization

#### Executor and Periphery Enums

**`SdkVmConfigExecutor<F>`**
- Enum containing all possible executors
- Derives: `ChipUsageGetter`, `Chip`, `InstructionExecutor`, `From`, `AnyEnum`
- One variant per extension type

**`SdkVmConfigPeriphery<F>`**
- Enum containing all possible peripheries
- Derives: `From`, `ChipUsageGetter`, `Chip`, `AnyEnum`
- Matches executor variants

#### Supporting Types

**`SdkSystemConfig`**
- Wrapper around `SystemConfig`
- Default enables continuations
- Implements `InitFileGenerator` (no-op)

**`UnitStruct`**
- Serialization helper for extensions without configuration
- Implements `From` for simple extension types

## Design Patterns

### 1. Generic Configuration
The `AppConfig<VC>` pattern allows different VM configurations while maintaining type safety.

### 2. Builder Pattern
`SdkVmConfig` uses the bon builder for ergonomic construction with optional fields.

### 3. Extension System
- Optional fields control which extensions are included
- Automatic dependency resolution (e.g., range checker sharing)
- Order-dependent initialization for complex extensions

### 4. FRI Parameter Management
Separate wrappers for different proof levels ensure correct security parameters.

### 5. Init File Generation
Algebraic extensions generate initialization code dynamically based on configuration.

## Security Considerations

### FRI Parameters
- Default 100-bit conjectured security
- Different log blowup for each level (app: 1, leaf: 1, internal: 2, root: 3)
- Configurable for different security/performance tradeoffs

### Public Values
- `max_num_user_public_values` limits exposed data
- Default from `DEFAULT_MAX_NUM_PUBLIC_VALUES` in circuit arch

## Performance Notes

### Aggregation Tree
- Leaf nodes aggregate 1 app proof by default (configurable)
- Internal nodes aggregate 3 proofs by default
- Wrapper layers provide safety for large aggregations

### Constraint Degrees
- SBOX_SIZE (7) limits constraint degree for efficiency
- Each VM config calculates appropriate degree based on FRI params

## Extension Integration

### Adding New Extensions
1. Add optional field to `SdkVmConfig`
2. Add executor variant to `SdkVmConfigExecutor`
3. Add periphery variant to `SdkVmConfigPeriphery`
4. Update `transpiler()` method
5. Update `create_chip_complex()` method
6. Add init file generation if needed

### Range Checker Coordination
BigInt and Rv32M extensions coordinate range checker sizes to avoid conflicts.

## Common Usage Patterns

### Basic App Configuration
```rust
let app_config = AppConfig::new(
    FriParameters::standard_with_100_bits_conjectured_security(1),
    SdkVmConfig::builder().rv32i(UnitStruct {}).build()
);
```

### Custom Aggregation
```rust
let agg_config = AggConfig {
    agg_stark_config: AggStarkConfig {
        leaf_fri_params: custom_fri_params,
        ..Default::default()
    },
    halo2_config: Halo2Config {
        verifier_k: 22,
        ..Default::default()
    }
};
```

### Extension Configuration
```rust
let vm_config = SdkVmConfig::builder()
    .rv32i(UnitStruct {})
    .keccak(UnitStruct {})
    .modular(ModularExtension { ... })
    .fp2(Fp2Extension { ... })
    .build();
```