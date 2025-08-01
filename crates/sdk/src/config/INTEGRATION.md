# OpenVM SDK Config Integration Guide

## Integration Overview
The SDK config component serves as the central configuration hub for OpenVM applications. It integrates with multiple components across the OpenVM ecosystem to provide consistent configuration management.

## Key Integration Points

### 1. CLI Integration
The config component integrates directly with the OpenVM CLI (`cargo-openvm`) for build and prove operations.

#### CLI Command Integration
```rust
// In CLI commands, configurations are loaded and applied
use openvm_sdk::config::{AppConfig, SdkVmConfig};

// Build command uses app config
let app_config = AppConfig::new(fri_params, vm_config);
let transpiler = app_config.app_vm_config.transpiler();
```

#### Configuration File Support
- JSON configuration files for persistent settings
- Command-line overrides for development workflows
- Environment variable integration for CI/CD

### 2. Circuit Integration
Configuration drives circuit construction and constraint system setup.

#### VM Complex Creation
```rust
use openvm_circuit::arch::VmConfig;

// Config creates the entire chip complex
let chip_complex = vm_config.create_chip_complex()?;
```

#### Extension Integration Pattern
Each extension follows a consistent integration pattern:
1. **Configuration Declaration**: Extension config in `SdkVmConfig`
2. **Transpiler Integration**: Extension transpiler added conditionally
3. **Complex Extension**: Chip complex extended with extension circuits

### 3. Transpiler Integration
The config system manages transpiler extensions for instruction set support.

#### Dynamic Transpiler Configuration
```rust
impl SdkVmConfig {
    pub fn transpiler(&self) -> Transpiler<F> {
        let mut transpiler = Transpiler::default();
        if self.rv32i.is_some() {
            transpiler = transpiler.with_extension(Rv32ITranspilerExtension);
        }
        // ... other extensions
        transpiler
    }
}
```

#### Extension Registration
- Extensions self-register with transpiler
- Conditional compilation based on config
- Automatic instruction set discovery

### 4. Aggregation System Integration
Configuration defines the entire aggregation pipeline structure.

#### Tree Construction
```rust
use openvm_sdk::config::AggregationTreeConfig;

// Tree config drives aggregation strategy
let tree_config = AggregationTreeConfig {
    num_children_leaf: 2,
    num_children_internal: 3,
    max_internal_wrapper_layers: 4,
};
```

#### Verifier Configurations
```rust
// Different verifier levels use different configs
let leaf_config = agg_config.leaf_vm_config();
let internal_config = agg_config.internal_vm_config();
let root_config = agg_config.root_verifier_vm_config();
```

## Integration Patterns

### 1. Builder Pattern Integration
The configuration system uses the builder pattern for flexible configuration construction.

```rust
use openvm_sdk::config::SdkVmConfig;

let config = SdkVmConfig::builder()
    .system(custom_system_config)
    .rv32i(Some(UnitStruct {}))
    .keccak(Some(UnitStruct {}))
    .build();
```

### 2. Feature Flag Integration
Configuration supports conditional compilation and feature flags.

```rust
// Extensions are optional and compile conditionally
#[cfg(feature = "keccak256")]
pub keccak: Option<UnitStruct>,
```

### 3. Default Configuration Chain
Intelligent defaults propagate through the configuration hierarchy.

```rust
impl Default for SdkSystemConfig {
    fn default() -> Self {
        Self {
            config: SystemConfig::default().with_continuations(),
        }
    }
}
```

## Development Integration Workflow

### 1. Adding New Extensions
When adding a new VM extension:

1. **Update SdkVmConfig**: Add optional field for extension config
2. **Update Executors/Periphery**: Add extension to enum variants
3. **Update Transpiler**: Add conditional transpiler extension
4. **Update Complex Creation**: Add extension to chip complex
5. **Update Tests**: Add integration tests for new extension

### 2. Configuration Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_serialization() {
        let config = SdkVmConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: SdkVmConfig = serde_json::from_str(&serialized).unwrap();
        // Validate config integrity
    }
}
```

### 3. Backwards Compatibility
- Maintain serialization compatibility across versions
- Use `#[serde(default)]` for new optional fields
- Document breaking changes in configuration format

## Integration Best Practices

### 1. Configuration Validation
```rust
impl SdkVmConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate extension dependencies
        if self.fp2.is_some() && self.modular.is_none() {
            return Err(ConfigError::MissingDependency("Fp2 requires Modular"));
        }
        Ok(())
    }
}
```

### 2. Resource Optimization
```rust
// Optimize shared resources between extensions
if let Some(rv32m) = self.rv32m {
    let mut rv32m = rv32m;
    if let Some(ref bigint) = self.bigint {
        // Share range checkers between extensions
        rv32m.range_tuple_checker_sizes[0] = 
            rv32m.range_tuple_checker_sizes[0].max(bigint.range_tuple_checker_sizes[0]);
    }
}
```

### 3. Init File Generation
```rust
impl InitFileGenerator for SdkVmConfig {
    fn generate_init_file_contents(&self) -> Option<String> {
        // Generate initialization code for extensions
        if self.modular.is_some() || self.fp2.is_some() || self.ecc.is_some() {
            let mut contents = String::new();
            // Add initialization code
            Some(contents)
        } else {
            None
        }
    }
}
```

## External System Integration

### 1. Ethereum Integration
Configuration supports on-chain verification parameters.

```rust
// Halo2 config for Ethereum verification
pub struct Halo2Config {
    pub verifier_k: usize,     // Circuit size for on-chain verifier
    pub wrapper_k: Option<usize>, // Optional wrapper circuit size
    pub profiling: bool,       // Debug mode for development
}
```

### 2. CI/CD Integration
Configuration files can be versioned and managed in CI/CD pipelines.

```bash
# Environment-specific configurations
cargo openvm prove --config production.json
cargo openvm prove --config development.json
```

### 3. Monitoring Integration
Configuration supports telemetry and monitoring systems.

```rust
// Profiling mode enables detailed metrics
pub struct SystemConfig {
    pub profiling: bool,  // Enables performance monitoring
    // ...
}
```

## Troubleshooting Integration Issues

### Common Integration Problems
1. **Missing Extension Dependencies**: Ensure required extensions are enabled
2. **Incompatible FRI Parameters**: Validate parameter compatibility
3. **Resource Conflicts**: Check for shared resource conflicts between extensions
4. **Serialization Errors**: Verify configuration file format compatibility

### Debug Configuration
```rust
// Enable debug mode for troubleshooting
let mut config = SdkVmConfig::default();
config.system.config.profiling = true;
```

### Validation Tools
```rust
// Use validation methods to catch configuration errors early
config.validate().expect("Invalid configuration");
```