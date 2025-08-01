# OpenVM SDK Config Component - Quick Reference

## Import Paths
```rust
use openvm_sdk::config::{
    // Main types
    AppConfig, AggConfig, SdkVmConfig, 
    // FRI wrappers
    AppFriParams, LeafFriParams,
    // Aggregation
    AggStarkConfig, Halo2Config, AggregationTreeConfig,
    // Helpers
    UnitStruct, SdkSystemConfig,
    // Constants
    DEFAULT_APP_LOG_BLOWUP, DEFAULT_LEAF_LOG_BLOWUP,
    DEFAULT_INTERNAL_LOG_BLOWUP, DEFAULT_ROOT_LOG_BLOWUP,
};
```

## Basic Configuration Examples

### Minimal App Config
```rust
let vm_config = SdkVmConfig::builder().rv32i(UnitStruct {}).build();
let app_config = AppConfig::new(FriParameters::default(), vm_config);
```

### Common Extensions
```rust
let vm_config = SdkVmConfig::builder()
    .rv32i(UnitStruct {})      // RISC-V base
    .rv32m(Rv32M::default())   // RISC-V multiply/divide
    .keccak(UnitStruct {})     // Keccak-256
    .sha256(UnitStruct {})     // SHA-256
    .native(UnitStruct {})     // Native field ops
    .build();
```

### Algebraic Extensions
```rust
let vm_config = SdkVmConfig::builder()
    .rv32i(UnitStruct {})
    .modular(modular_config)   // Required for fp2
    .fp2(fp2_config)          // Fp2 extension
    .ecc(ecc_config)          // Elliptic curves
    .pairing(pairing_config)  // Pairing ops
    .build();
```

## Default Values

### FRI Parameters
- App: log_blowup = 1, 100-bit security
- Leaf: log_blowup = 1, 100-bit security
- Internal: log_blowup = 2, 100-bit security
- Root: log_blowup = 3, 100-bit security

### Aggregation Tree
- `num_children_leaf`: 1
- `num_children_internal`: 3
- `max_internal_wrapper_layers`: 4

### Halo2
- `verifier_k`: 24
- `wrapper_k`: None (auto-tuned)
- `profiling`: false

## Key Methods

### AppConfig
```rust
// Constructors
AppConfig::new(fri_params, vm_config)
AppConfig::new_with_leaf_fri_params(app_fri, vm_config, leaf_fri)
```

### SdkVmConfig
```rust
// Builder pattern
SdkVmConfig::builder()
    .rv32i(UnitStruct {})
    .build()

// Methods
vm_config.transpiler()              // Get transpiler
vm_config.create_chip_complex()     // Create VM
vm_config.generate_init_file_contents() // Init files
```

### AggStarkConfig
```rust
// VM config generators
agg_stark_config.leaf_vm_config()
agg_stark_config.internal_vm_config()
agg_stark_config.root_verifier_vm_config()
```

## Extension Compatibility

### Dependencies
- `fp2` requires `modular`
- `ecc` works with `modular` and `fp2`
- `pairing` works with field extensions

### Shared Resources
- `rv32m` and `bigint` share range checkers
- Sizes are automatically coordinated (takes max)

## CLI Integration

### AggregationTreeConfig Args
```bash
--num-children-leaf 2
--num-children-internal 4
--max-internal-wrapper-layers 3
```

## Common Patterns

### Development Config
```rust
// Fast proving, minimal security
let dev_config = AppConfig::new(
    FriParameters::standard_with_100_bits_conjectured_security(1),
    SdkVmConfig::builder().rv32i(UnitStruct {}).build()
);
```

### Production Config
```rust
// Higher security, all extensions
let prod_config = AppConfig::new(
    FriParameters::standard_with_100_bits_conjectured_security(3),
    SdkVmConfig::builder()
        .rv32i(UnitStruct {})
        .rv32m(Rv32M::default())
        .keccak(UnitStruct {})
        .sha256(UnitStruct {})
        .build()
);
```

### Custom Aggregation
```rust
let agg_config = AggConfig {
    agg_stark_config: AggStarkConfig {
        profiling: true,
        ..Default::default()
    },
    halo2_config: Halo2Config {
        verifier_k: 22,
        ..Default::default()
    }
};
```

## Serialization

All config types implement `Serialize`/`Deserialize`:

```rust
// Save config
let json = serde_json::to_string(&app_config)?;

// Load config
let app_config: AppConfig<SdkVmConfig> = serde_json::from_str(&json)?;
```

## Type Reference

### Core Types
- `AppConfig<VC>`: Generic app configuration
- `SdkVmConfig`: Standard VM configuration
- `AggConfig`: Aggregation configuration

### FRI Wrappers
- `AppFriParams`: App-level FRI params
- `LeafFriParams`: Leaf-level FRI params

### Aggregation Types
- `AggStarkConfig`: STARK aggregation config
- `Halo2Config`: SNARK wrapper config
- `AggregationTreeConfig`: Tree structure

### Enums
- `SdkVmConfigExecutor<F>`: All executors
- `SdkVmConfigPeriphery<F>`: All peripheries

### Helper Types
- `UnitStruct`: For simple extensions
- `SdkSystemConfig`: System config wrapper

## Constants
```rust
DEFAULT_APP_LOG_BLOWUP = 1
DEFAULT_LEAF_LOG_BLOWUP = 1
DEFAULT_INTERNAL_LOG_BLOWUP = 2
DEFAULT_ROOT_LOG_BLOWUP = 3
DEFAULT_NUM_CHILDREN_LEAF = 1
DEFAULT_NUM_CHILDREN_INTERNAL = 3
DEFAULT_MAX_INTERNAL_WRAPPER_LAYERS = 4
SBOX_SIZE = 7
```