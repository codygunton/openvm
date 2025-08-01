# OpenVM CLI Commands - Examples

This document provides practical examples of using OpenVM CLI commands for common development workflows.

## Basic Development Workflow

### 1. Project Initialization
```bash
# Create a new OpenVM project
mkdir my-openvm-project
cd my-openvm-project
openvm init

# Initialize with custom template
openvm init --template fibonacci

# Initialize in existing Rust project
openvm init --no-git
```

### 2. Building Programs
```bash
# Basic build
openvm build

# Build with release optimization
openvm build --release

# Build without transpilation (ELF only)
openvm build --no-transpile

# Build with custom config
openvm build --config custom-openvm.toml

# Build to specific output directory
openvm build --output-dir ./dist

# Build specific package in workspace
openvm build -p my-package

# Build with verbose output
openvm build --verbose
```

### 3. Running and Testing
```bash
# Execute program with trace
openvm run

# Run with debug output
openvm run --debug

# Run with custom input
openvm run --input input.json

# Run with trace output to file
openvm run --trace-output trace.json

# Run specific package
openvm run -p fibonacci-example
```

## Key Management Examples

### 4. Key Generation
```bash
# Generate proving and verification keys
openvm keygen

# Generate keys with custom config
openvm keygen --config openvm.toml

# Generate keys to specific directory
openvm keygen --output-dir ./keys

# Generate keys for specific VM configuration
openvm keygen --vm-config vm-config.json

# Generate keys with custom parameters
openvm keygen --setup-params setup.json
```

## Proof Generation and Verification

### 5. Proof Generation
```bash
# Generate proof using default keys
openvm prove

# Generate proof with custom proving key
openvm prove --app-pk ./keys/custom.pk

# Generate proof with input data
openvm prove --input program-input.json

# Generate proof to specific output
openvm prove --output proof.bin

# Generate proof with verbose logging
openvm prove --verbose

# Batch proof generation
for exe in *.vmexe; do
    openvm prove --exe "$exe" --output "${exe%.vmexe}.proof"
done
```

### 6. Proof Verification
```bash
# Verify proof using default keys
openvm verify

# Verify with custom verification key
openvm verify --app-vk ./keys/custom.vk

# Verify specific proof file
openvm verify --proof ./proofs/program.proof

# Verify with verbose output
openvm verify --verbose

# Batch verification
find ./proofs -name "*.proof" -exec openvm verify --proof {} \;
```

## Advanced Configuration Examples

### 7. Custom VM Configuration
```bash
# Create openvm.toml with extensions
cat > openvm.toml << EOF
[extensions]
rv32im = {}
native = {}
keccak = {}
sha256 = {}
bigint = {}

[memory]
max_size = 8589934592  # 8GB

[prover]
system = "STARK"
security_level = 128
EOF

# Build with custom extensions
openvm build --config openvm.toml
```

### 8. EVM Integration Setup
```bash
# Setup for EVM verification (requires evm-verify feature)
openvm setup

# Setup with custom parameters
openvm setup --config evm-config.toml

# Setup for specific chain
openvm setup --chain-id 1

# Generate EVM verifier contract
openvm setup --output-contract Verifier.sol
```

## Workspace and Multi-Package Examples

### 9. Workspace Operations
```bash
# Build all packages in workspace
openvm build --workspace

# Build specific packages
openvm build -p package1 -p package2

# Generate keys for workspace
openvm keygen --workspace

# Prove all packages
for pkg in $(cargo metadata --format-version 1 | jq -r '.packages[].name'); do
    openvm prove -p "$pkg"
done
```

### 10. Cross-Compilation Examples
```bash
# Build for different target
openvm build --target x86_64-unknown-linux-gnu

# Build with specific Rust flags
RUSTFLAGS="-C opt-level=3" openvm build

# Build with custom target directory
openvm build --target-dir ./custom-target
```

## Debugging and Profiling

### 11. Debug Builds and Tracing
```bash
# Debug build with symbols
openvm build --profile dev

# Run with detailed trace
RUST_LOG=debug openvm run --trace

# Profile memory usage
openvm run --profile-memory

# Generate execution statistics
openvm run --stats-output stats.json
```

### 12. Error Diagnosis
```bash
# Verbose error output
openvm build --verbose 2>&1 | tee build.log

# Check configuration
openvm build --dry-run

# Validate keys
openvm keygen --verify-only

# Test proof system
openvm prove --test-mode
```

## CI/CD Integration Examples

### 13. Continuous Integration
```bash
#!/bin/bash
# CI script example

set -e

echo "Building OpenVM program..."
openvm build --release

echo "Generating keys..."
openvm keygen --output-dir ./keys

echo "Running tests..."
openvm run --test

echo "Generating proof..."
openvm prove --output proof.bin

echo "Verifying proof..."
openvm verify --proof proof.bin

echo "CI pipeline completed successfully!"
```

### 14. Docker Integration
```dockerfile
FROM rust:1.82 as builder

WORKDIR /app
COPY . .

# Install OpenVM CLI
RUN cargo install --path crates/cli

# Build program
RUN openvm build --release

# Generate keys
RUN openvm keygen

FROM ubuntu:22.04
COPY --from=builder /app/target/openvm/release/ /app/
COPY --from=builder /app/keys/ /app/keys/

WORKDIR /app
CMD ["openvm", "prove"]
```

## Performance Optimization Examples

### 15. Memory Optimization
```bash
# Reduce memory usage for large programs
export OPENVM_MEMORY_LIMIT=4G
openvm prove --memory-efficient

# Use streaming for large proofs
openvm prove --streaming --chunk-size 1MB

# Parallel proof generation
openvm prove --parallel-workers 4
```

### 16. Build Optimization
```bash
# Fast development builds
openvm build --profile fast

# Link-time optimization
openvm build --release --lto

# Incremental compilation
export CARGO_INCREMENTAL=1
openvm build
```

## Testing and Validation Examples

### 17. Unit Testing Integration
```rust
// In your Rust test file
#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_openvm_build() {
        let output = Command::new("openvm")
            .args(&["build", "--no-transpile"])
            .output()
            .expect("Failed to execute openvm build");
        
        assert!(output.status.success());
    }
}
```

### 18. Integration Testing
```bash
#!/bin/bash
# Integration test script

test_fibonacci() {
    echo "Testing Fibonacci example..."
    cd examples/fibonacci
    openvm build || return 1
    openvm keygen || return 1
    openvm prove || return 1
    openvm verify || return 1
    cd ../..
}

test_sha256() {
    echo "Testing SHA256 example..."
    cd examples/sha256
    openvm build --config sha256-config.toml || return 1
    openvm run --input test-input.txt || return 1
    cd ../..
}

# Run all tests
test_fibonacci
test_sha256
echo "All integration tests passed!"
```

## Advanced Use Cases

### 19. Custom Extension Development
```bash
# Build with custom extension
cat > custom-extension.toml << EOF
[extensions]
my_custom_extension = { path = "./extensions/my_extension" }
rv32im = {}
EOF

openvm build --config custom-extension.toml
```

### 20. Proof Aggregation
```bash
# Generate multiple proofs
openvm prove --output proof1.bin --input input1.json
openvm prove --output proof2.bin --input input2.json

# Aggregate proofs (if supported)
openvm aggregate --proofs proof1.bin,proof2.bin --output aggregated.proof
```

## Environment Configuration

### 21. Environment Variables
```bash
# Set default configuration
export OPENVM_CONFIG_PATH=./config/default.toml

# Set memory limits
export OPENVM_MAX_MEMORY=8G

# Enable debug output
export RUST_LOG=openvm=debug

# Custom target directory
export CARGO_TARGET_DIR=./build

# Run with environment
openvm build
```

### 22. Shell Aliases and Functions
```bash
# Add to ~/.bashrc or ~/.zshrc

# Quick build and prove
alias ovbp='openvm build && openvm prove'

# Build, prove, and verify
alias ovbpv='openvm build && openvm prove && openvm verify'

# Development build with run
ovdev() {
    openvm build --profile dev && openvm run "$@"
}

# Production proof generation
ovprod() {
    openvm build --release
    openvm keygen --output-dir keys/
    openvm prove --output production.proof
    openvm verify --proof production.proof
}
```

These examples cover the most common use cases and patterns for OpenVM CLI commands. Each example includes practical scenarios and can be adapted to specific project requirements.