# OpenVM SDK Config Component - Implementation Guide

## Overview
This guide covers implementing custom configurations and extending the SDK config system.

## Common Implementation Tasks

### 1. Creating a Basic App Configuration

```rust
use openvm_sdk::config::{AppConfig, SdkVmConfig, UnitStruct};
use openvm_stark_sdk::config::FriParameters;

// Minimal configuration with RV32I
let vm_config = SdkVmConfig::builder()
    .rv32i(UnitStruct {})
    .build();

let app_config = AppConfig::new(
    FriParameters::standard_with_100_bits_conjectured_security(1),
    vm_config
);
```

### 2. Configuring Multiple Extensions

```rust
use openvm_sdk::config::{SdkVmConfig, UnitStruct};
use openvm_bigint_circuit::Int256;
use openvm_rv32im_circuit::Rv32M;

let vm_config = SdkVmConfig::builder()
    .rv32i(UnitStruct {})
    .rv32m(Rv32M {
        range_tuple_checker_sizes: [20, 22], // Must coordinate with bigint
    })
    .bigint(Int256 {
        range_tuple_checker_sizes: [20, 22], // Same as rv32m
    })
    .keccak(UnitStruct {})
    .sha256(UnitStruct {})
    .build();
```

### 3. Setting Up Algebraic Extensions

```rust
use openvm_algebra_circuit::{ModularExtension, Fp2Extension};

// Modular arithmetic configuration
let modular_config = ModularExtension {
    // Configuration for modular arithmetic
};

// Fp2 requires modular
let fp2_config = Fp2Extension {
    // Configuration for Fp2 operations
};

let vm_config = SdkVmConfig::builder()
    .rv32i(UnitStruct {})
    .modular(modular_config)
    .fp2(fp2_config)
    .build();

// This will automatically generate init files
```

### 4. Custom Aggregation Configuration

```rust
use openvm_sdk::config::{AggConfig, AggStarkConfig, Halo2Config};

let agg_config = AggConfig {
    agg_stark_config: AggStarkConfig {
        max_num_user_public_values: 1024, // Custom limit
        leaf_fri_params: FriParameters::standard_with_100_bits_conjectured_security(2),
        internal_fri_params: FriParameters::standard_with_100_bits_conjectured_security(3),
        root_fri_params: FriParameters::standard_with_100_bits_conjectured_security(4),
        profiling: true, // Enable profiling
        ..Default::default()
    },
    halo2_config: Halo2Config {
        verifier_k: 20, // Smaller circuit
        wrapper_k: Some(18), // Manual tuning
        profiling: true,
    }
};
```

### 5. Configuring Aggregation Tree

```rust
use openvm_sdk::config::AggregationTreeConfig;

let tree_config = AggregationTreeConfig {
    num_children_leaf: 2,      // 2 app proofs per leaf
    num_children_internal: 4,   // 4 proofs per internal node
    max_internal_wrapper_layers: 3, // Fewer wrapper layers
};
```

## Advanced Implementation Patterns

### Adding a New Extension

To add a new extension to the SDK config:

1. **Define Extension Config** (in extension crate):
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyExtension {
    pub config_field: u32,
}
```

2. **Add to SdkVmConfig**:
```rust
// In global.rs
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
pub struct SdkVmConfig {
    // ... existing fields ...
    pub my_extension: Option<MyExtension>,
}
```

3. **Add Executor Variant**:
```rust
#[derive(ChipUsageGetter, Chip, InstructionExecutor, From, AnyEnum)]
pub enum SdkVmConfigExecutor<F: PrimeField32> {
    // ... existing variants ...
    #[any_enum]
    MyExtension(MyExtensionExecutor<F>),
}
```

4. **Add Periphery Variant**:
```rust
#[derive(From, ChipUsageGetter, Chip, AnyEnum)]
pub enum SdkVmConfigPeriphery<F: PrimeField32> {
    // ... existing variants ...
    #[any_enum]
    MyExtension(MyExtensionPeriphery<F>),
}
```

5. **Update Transpiler Method**:
```rust
impl SdkVmConfig {
    pub fn transpiler(&self) -> Transpiler<F> {
        let mut transpiler = Transpiler::default();
        // ... existing extensions ...
        if self.my_extension.is_some() {
            transpiler = transpiler.with_extension(MyExtensionTranspiler);
        }
        transpiler
    }
}
```

6. **Update Chip Complex Creation**:
```rust
fn create_chip_complex(&self) -> Result<VmChipComplex<F, Self::Executor, Self::Periphery>, VmInventoryError> {
    // ... existing code ...
    if let Some(ref my_ext) = self.my_extension {
        complex = complex.extend(my_ext)?;
    }
    Ok(complex)
}
```

### Implementing Custom FRI Parameters

```rust
use openvm_stark_sdk::config::FriParameters;

// Custom FRI params with specific security level
fn custom_fri_params(log_blowup: usize, security_bits: usize) -> FriParameters {
    FriParameters {
        log_blowup,
        num_queries: calculate_num_queries(security_bits, log_blowup),
        proof_of_work_bits: 16,
        mmcs: Default::default(),
    }
}

// Use in config
let app_config = AppConfig::new(
    custom_fri_params(2, 128),
    vm_config
);
```

### Coordinating Shared Resources

When extensions share resources (like range checkers):

```rust
impl SdkVmConfig {
    fn create_chip_complex(&self) -> Result<...> {
        // ... setup ...
        
        // Coordinate range checker sizes
        if let Some(rv32m) = self.rv32m {
            let mut rv32m = rv32m;
            if let Some(ref bigint) = self.bigint {
                // Take max of both configurations
                rv32m.range_tuple_checker_sizes[0] = 
                    rv32m.range_tuple_checker_sizes[0].max(bigint.range_tuple_checker_sizes[0]);
                rv32m.range_tuple_checker_sizes[1] = 
                    rv32m.range_tuple_checker_sizes[1].max(bigint.range_tuple_checker_sizes[1]);
            }
            complex = complex.extend(&rv32m)?;
        }
        
        // Apply same coordination to bigint
        // ...
    }
}
```

### Generating Init Files for Extensions

```rust
impl InitFileGenerator for SdkVmConfig {
    fn generate_init_file_contents(&self) -> Option<String> {
        if self.needs_init_file() {
            let mut contents = String::new();
            contents.push_str("// Auto-generated init file\n");
            
            if let Some(ext) = &self.my_extension {
                contents.push_str(&ext.generate_init_code());
                contents.push('\n');
            }
            
            Some(contents)
        } else {
            None
        }
    }
}
```

## Error Handling

### Common Configuration Errors

1. **Missing Dependencies**:
```rust
// This will panic at runtime
let vm_config = SdkVmConfig::builder()
    .fp2(fp2_config) // Error: fp2 requires modular
    .build();
```

2. **Incompatible Range Checkers**:
```rust
// Avoid by coordinating sizes
let vm_config = SdkVmConfig::builder()
    .rv32m(Rv32M { range_tuple_checker_sizes: [20, 22] })
    .bigint(Int256 { range_tuple_checker_sizes: [18, 20] }) // Mismatch!
    .build();
```

### Validation Pattern

```rust
impl SdkVmConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Check dependencies
        if self.fp2.is_some() && self.modular.is_none() {
            return Err(ConfigError::MissingDependency("fp2 requires modular"));
        }
        
        // Check compatibility
        if let (Some(rv32m), Some(bigint)) = (&self.rv32m, &self.bigint) {
            if rv32m.range_tuple_checker_sizes != bigint.range_tuple_checker_sizes {
                return Err(ConfigError::IncompatibleRangeCheckers);
            }
        }
        
        Ok(())
    }
}
```

## Performance Optimization

### FRI Parameter Tuning

```rust
// Development: Fast proving, lower security
let dev_fri = FriParameters::standard_with_100_bits_conjectured_security(1);

// Production: Higher security, slower proving  
let prod_fri = FriParameters::standard_with_100_bits_conjectured_security(3);

// Custom: Balance security and performance
let custom_fri = FriParameters {
    log_blowup: 2,
    num_queries: 50,
    proof_of_work_bits: 20,
    mmcs: Default::default(),
};
```

### Aggregation Tree Optimization

```rust
// For many small proofs: wide tree
let many_small = AggregationTreeConfig {
    num_children_leaf: 4,
    num_children_internal: 8,
    max_internal_wrapper_layers: 2,
};

// For few large proofs: narrow tree
let few_large = AggregationTreeConfig {
    num_children_leaf: 1,
    num_children_internal: 2,
    max_internal_wrapper_layers: 5,
};
```

## Testing Configurations

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = create_test_config();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig<SdkVmConfig> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_vm_creation() {
        let config = create_test_config();
        let complex = config.app_vm_config.create_chip_complex();
        assert!(complex.is_ok());
    }
}
```