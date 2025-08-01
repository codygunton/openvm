# OpenVM SDK Config Examples

## Basic Application Configuration

### Simple App Config
```rust
use openvm_sdk::config::{AppConfig, AppFriParams};
use openvm_stark_sdk::config::FriParameters;

// Create basic app configuration
let fri_params = FriParameters::standard_with_100_bits_conjectured_security(1);
let vm_config = SdkVmConfig::builder()
    .system(SdkSystemConfig::default())
    .rv32i(Some(UnitStruct {}))
    .io(Some(UnitStruct {}))
    .build();

let app_config = AppConfig::new(fri_params, vm_config);
```

### App Config with Custom Leaf Parameters
```rust
use openvm_sdk::config::{AppConfig, AppFriParams, LeafFriParams};

let app_fri = FriParameters::standard_with_100_bits_conjectured_security(1);
let leaf_fri = FriParameters::standard_with_100_bits_conjectured_security(2);
let vm_config = SdkVmConfig::default();

let app_config = AppConfig::new_with_leaf_fri_params(
    app_fri,
    vm_config,
    leaf_fri
);
```

## VM Configuration Examples

### RISC-V VM Configuration
```rust
use openvm_sdk::config::{SdkVmConfig, SdkSystemConfig, UnitStruct};
use openvm_rv32im_circuit::Rv32M;

// Basic RISC-V configuration
let vm_config = SdkVmConfig::builder()
    .system(SdkSystemConfig::default())
    .rv32i(Some(UnitStruct {}))
    .rv32m(Some(Rv32M::default()))
    .io(Some(UnitStruct {}))
    .build();
```

### Cryptographic Extensions
```rust
use openvm_sdk::config::{SdkVmConfig, UnitStruct};

// VM with crypto extensions
let vm_config = SdkVmConfig::builder()
    .system(SdkSystemConfig::default())
    .rv32i(Some(UnitStruct {}))
    .keccak(Some(UnitStruct {}))
    .sha256(Some(UnitStruct {}))
    .build();
```

### Advanced Math Extensions
```rust
use openvm_sdk::config::SdkVmConfig;
use openvm_bigint_circuit::Int256;
use openvm_algebra_circuit::{ModularExtension, Fp2Extension};
use openvm_ecc_circuit::WeierstrassExtension;

// VM with advanced math capabilities
let vm_config = SdkVmConfig::builder()
    .system(SdkSystemConfig::default())
    .rv32i(Some(UnitStruct {}))
    .bigint(Some(Int256::default()))
    .modular(Some(ModularExtension::default()))
    .fp2(Some(Fp2Extension::default()))
    .ecc(Some(WeierstrassExtension::default()))
    .build();
```

## Aggregation Configuration

### Default Aggregation Config
```rust
use openvm_sdk::config::{AggConfig, AggStarkConfig, Halo2Config};

// Use default aggregation settings
let agg_config = AggConfig::default();

// Or customize the configuration
let agg_config = AggConfig {
    agg_stark_config: AggStarkConfig {
        max_num_user_public_values: 100,
        profiling: true,
        ..Default::default()
    },
    halo2_config: Halo2Config {
        verifier_k: 22,
        wrapper_k: Some(20),
        profiling: true,
    },
};
```

### Custom FRI Parameters
```rust
use openvm_sdk::config::AggStarkConfig;
use openvm_stark_sdk::config::FriParameters;

let agg_config = AggStarkConfig {
    leaf_fri_params: FriParameters::standard_with_100_bits_conjectured_security(2),
    internal_fri_params: FriParameters::standard_with_100_bits_conjectured_security(3),
    root_fri_params: FriParameters::standard_with_100_bits_conjectured_security(4),
    ..Default::default()
};
```

## Aggregation Tree Configuration

### Custom Tree Structure
```rust
use openvm_sdk::config::AggregationTreeConfig;

// Custom aggregation tree with more parallelism
let tree_config = AggregationTreeConfig {
    num_children_leaf: 2,        // Each leaf aggregates 2 app proofs
    num_children_internal: 4,    // Each internal node aggregates 4 proofs
    max_internal_wrapper_layers: 2,
};
```

### Command-Line Tree Configuration
```rust
use clap::Parser;
use openvm_sdk::config::AggregationTreeConfig;

#[derive(Parser)]
struct Args {
    #[command(flatten)]
    tree_config: AggregationTreeConfig,
}

// Usage: program --num-children-leaf 3 --num-children-internal 5
```

## Transpiler Integration

### Getting Transpiler from Config
```rust
use openvm_sdk::config::SdkVmConfig;

let vm_config = SdkVmConfig::builder()
    .rv32i(Some(UnitStruct {}))
    .keccak(Some(UnitStruct {}))
    .build();

// Get configured transpiler
let transpiler = vm_config.transpiler();
```

## VM Complex Creation

### Creating Chip Complex
```rust
use openvm_sdk::config::SdkVmConfig;
use openvm_circuit::arch::VmConfig;

let vm_config = SdkVmConfig::default();

// Create chip complex for circuit execution
let chip_complex = vm_config.create_chip_complex()
    .expect("Failed to create chip complex");
```

## Serialization Examples

### JSON Configuration
```rust
use openvm_sdk::config::{SdkVmConfig, AppConfig};
use serde_json;

// Serialize to JSON
let vm_config = SdkVmConfig::default();
let json_str = serde_json::to_string_pretty(&vm_config)
    .expect("Failed to serialize config");

// Deserialize from JSON
let loaded_config: SdkVmConfig = serde_json::from_str(&json_str)
    .expect("Failed to deserialize config");
```

### Configuration File Loading
```rust
use std::fs;
use openvm_sdk::config::AppConfig;

// Load from file
let config_contents = fs::read_to_string("app_config.json")?;
let app_config: AppConfig<SdkVmConfig> = serde_json::from_str(&config_contents)?;
```

## Native VM Configurations

### Leaf Verifier Config
```rust
use openvm_sdk::config::AggStarkConfig;

let agg_config = AggStarkConfig::default();
let leaf_vm_config = agg_config.leaf_vm_config();
```

### Internal Verifier Config
```rust
let internal_vm_config = agg_config.internal_vm_config();
```

### Root Verifier Config
```rust
let root_vm_config = agg_config.root_verifier_vm_config();
```