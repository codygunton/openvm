# OpenVM CLI Integration Guide

## Integration with OpenVM Ecosystem

### Core SDK Integration
The CLI integrates deeply with the OpenVM SDK stack:

```rust
// Key dependencies from lib.rs and Cargo.toml
use openvm_build::*;           // Build system integration
use openvm_transpiler::*;      // Code transpilation
use openvm_sdk::*;             // Core SDK functionality
use openvm_stark_sdk::*;       // STARK proving system
use openvm_circuit::*;         // Circuit definitions
```

### Command Architecture Integration
Each command integrates with specific SDK components:

- **Build Command**: `openvm-build` + `openvm-transpiler`
- **Prove Command**: `openvm-stark-sdk` + `openvm-circuit`
- **Verify Command**: `openvm-stark-backend`
- **Run Command**: `openvm-sdk` execution engine

## External Tool Integration

### Rust Toolchain Integration
```rust
// From lib.rs
pub const RUSTUP_TOOLCHAIN_NAME: &str = "nightly-2025-02-14";

// CommandExecutor trait for unified command execution
impl CommandExecutor for Command {
    fn run(&mut self) -> Result<()> {
        self.stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .output()
            .with_context(|| format!("while executing `{:?}`", &self))
            .map(|_| ())
    }
}
```

### AWS S3 Integration
For proof storage and distribution:
```toml
# From Cargo.toml
aws-sdk-s3 = "1.78"
aws-config = "1.5"
```

### Platform Detection Integration
```rust
// Target platform validation
pub fn is_supported_target() -> bool {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    return true;
    
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    return true;
    
    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    return true;
    
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    return true;
    
    false
}
```

## CI/CD Integration

### Cargo Integration
The CLI is designed as a Cargo subcommand:
```toml
[package]
name = "cargo-openvm"  # Enables `cargo openvm` usage
```

### Build System Integration
```bash
# Integration with Cargo build profiles
cargo build --profile=fast        # Development builds
cargo build --release            # Production builds
cargo openvm build --elf <target>  # OpenVM-specific builds
```

### Testing Integration
```bash
# Integrate with standard Rust testing
cargo test --all                 # Run all tests
cargo test -p cargo-openvm       # Test CLI specifically
RUST_LOG=debug cargo test        # Debug testing
```

## Configuration Integration

### TOML Configuration
```rust
// From Cargo.toml dependencies
toml = { workspace = true }
toml_edit = "0.22"
```

Integration with project-level `openvm.toml`:
```toml
[system]
memory_size = "64MB"
stack_size = "8MB"

[proving]
backend = "stark"
security_level = 100
```

### Environment Variable Integration
```rust
// From Cargo.toml
clap = { version = "4.5.9", features = ["derive", "env"] }
```

Supports environment-based configuration:
```bash
OPENVM_CONFIG_PATH=/path/to/config.toml cargo openvm build
RUST_LOG=debug cargo openvm prove
```

## Feature Flag Integration

### Conditional Compilation
```toml
# From Cargo.toml
[features]
default = ["parallel", "jemalloc", "evm-verify", "bench-metrics"]
evm-prove = ["openvm-sdk/evm-prove"]
evm-verify = ["evm-prove", "openvm-sdk/evm-verify"]
parallel = ["openvm-sdk/parallel"]
```

### Runtime Feature Detection
```rust
// In commands/mod.rs
#[cfg(feature = "evm-verify")]
mod setup;
#[cfg(feature = "evm-verify")]
pub use setup::*;
```

## Development Tool Integration

### IDE Integration
- Rust-analyzer support through standard Cargo project structure
- Debug symbols for step-through debugging
- Integration with Rust toolchain ecosystem

### Profiling Integration
```toml
# From Cargo.toml
profiling = ["openvm-sdk/profiling"]
```

### Memory Management Integration
```toml
# Multiple allocator options
mimalloc = ["openvm-sdk/mimalloc"]
jemalloc = ["openvm-sdk/jemalloc"]
jemalloc-prof = ["openvm-sdk/jemalloc-prof"]
```

## API Integration Patterns

### Error Handling Integration
```rust
use eyre::{Context, Result};

// Unified error handling across all commands
.with_context(|| format!("while executing `{:?}`", &self))
```

### Async Runtime Integration
```toml
tokio = { version = "1.43.1", features = ["rt", "rt-multi-thread", "macros"] }
```

### Serialization Integration
```toml
serde.workspace = true
serde_json.workspace = true
```

## Deployment Integration

### Binary Distribution
- Single binary deployment: `cargo-openvm`
- Cross-platform support via target detection
- Embedded resources via `include_dir`

### Docker Integration
The CLI can be containerized with:
```dockerfile
FROM rust:1.82 as builder
COPY . .
RUN cargo build --release --bin cargo-openvm

FROM debian:bookworm-slim
COPY --from=builder /target/release/cargo-openvm /usr/local/bin/
```

### Package Manager Integration
- Cargo registry publication
- Platform-specific package managers
- Git-based installation support