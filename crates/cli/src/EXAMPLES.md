# OpenVM CLI Examples

## Basic Workflow

### 1. Initialize a New Project
```bash
# Create a new OpenVM project
cargo openvm init my-project
cd my-project
```

### 2. Build Guest Program
```bash
# Build ELF from guest program
cargo openvm build --elf ./target/riscv32im-openvm-guest/release/my-program

# Build with custom output path
cargo openvm build --elf ./guest/target/program.elf --output ./proofs/
```

### 3. Run Program
```bash
# Execute program in OpenVM
cargo openvm run --elf ./target/program.elf

# Run with input data
cargo openvm run --elf ./program.elf --input ./input.json
```

### 4. Generate Proof
```bash
# Generate proof for program execution
cargo openvm prove --elf ./program.elf

# Generate proof with custom config
cargo openvm prove --elf ./program.elf --config ./openvm.toml
```

### 5. Verify Proof
```bash
# Verify generated proof
cargo openvm verify --proof ./proof.json --vk ./verifying_key.vk

# Verify with program metadata
cargo openvm verify --proof ./proof.json --elf ./program.elf
```

## Advanced Usage

### Key Generation
```bash
# Generate proving and verifying keys
cargo openvm keygen --elf ./program.elf --output ./keys/

# Generate keys for specific config
cargo openvm keygen --config ./custom.toml --output ./keys/
```

### EVM Integration (with evm-verify feature)
```bash
# Setup EVM verification parameters
cargo openvm setup --elf ./program.elf --evm-output ./evm/

# Generate EVM-compatible proof
cargo openvm prove --elf ./program.elf --evm
```

### Commit Operations
```bash
# Commit to program execution
cargo openvm commit --elf ./program.elf --input ./data.json
```

## Configuration Examples

### Custom OpenVM Configuration (`openvm.toml`)
```toml
[system]
memory_size = "64MB"
stack_size = "8MB"

[proving]
backend = "stark"
security_level = 100

[features]
enable_profiling = true
parallel_execution = true
```

### Input Data Format (`input.json`)
```json
{
  "program_args": ["arg1", "arg2"],
  "stdin": "input data",
  "environment": {
    "VAR1": "value1",
    "VAR2": "value2"
  }
}
```

## Development Workflow

### Testing a Guest Program
```bash
# 1. Build the guest program
cargo openvm build --elf ./guest-program

# 2. Test execution locally
cargo openvm run --elf ./guest-program

# 3. Generate and verify proof
cargo openvm prove --elf ./guest-program
cargo openvm verify --proof ./proof.json --elf ./guest-program
```

### Debugging
```bash
# Run with debug output
RUST_LOG=debug cargo openvm run --elf ./program.elf

# Profile guest program execution
cargo openvm run --elf ./program.elf --profile
```

### Performance Optimization
```bash
# Build with fast profile for development
cargo build --profile=fast

# Generate proof with parallel processing
cargo openvm prove --elf ./program.elf --parallel

# Use different memory allocator
cargo openvm prove --elf ./program.elf --features jemalloc
```

## Error Handling Examples

### Common Error Scenarios
```bash
# Handle missing ELF file
cargo openvm build --elf ./nonexistent.elf
# Error: ELF file not found at path ./nonexistent.elf

# Handle unsupported target
cargo openvm build --target unsupported-target
# Error: Unsupported target platform

# Handle invalid proof
cargo openvm verify --proof ./invalid_proof.json
# Error: Proof verification failed
```

### Troubleshooting Commands
```bash
# Check OpenVM version and build info
cargo openvm --version

# Validate configuration
cargo openvm build --dry-run --config ./openvm.toml

# Test target platform support
cargo openvm build --check-target
```