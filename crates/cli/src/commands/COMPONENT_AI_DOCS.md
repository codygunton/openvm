# OpenVM CLI Commands Component

## Overview
The OpenVM CLI commands component provides a comprehensive set of command-line utilities for building, proving, and verifying zero-knowledge virtual machine programs. This component implements the core user-facing interface for the OpenVM framework.

## Architecture

### Command Structure
All commands follow a consistent pattern:
- **Command Struct**: Each command is defined as a struct ending with `Cmd` suffix
- **Argument Parsing**: Uses `clap::Parser` for structured argument handling
- **Execution**: Implements a `run()` method returning `eyre::Result<()>`
- **Error Handling**: Provides actionable error messages with context

### Core Commands

#### BuildCmd (`build.rs`)
**Purpose**: Compiles OpenVM programs from Rust source code to executable format
**Key Features**:
- ELF compilation and transpilation to OpenVM executable format
- Support for custom VM configurations via `openvm.toml`
- Integration with Cargo build system
- Optional output directory specification
- Configurable memory initialization

**Key Types**:
- `BuildCmd`: Main command struct
- `BuildArgs`: OpenVM-specific build arguments
- `BuildCargoArgs`: Cargo integration arguments

#### ProveCmd (`prove.rs`)
**Purpose**: Generates zero-knowledge proofs for OpenVM programs
**Key Features**:
- Proof generation using specified proving key
- Support for multiple proof formats
- Configurable proof parameters
- Input/output file handling

#### VerifyCmd (`verify.rs`)
**Purpose**: Verifies zero-knowledge proofs
**Key Features**:
- Proof verification using verification key
- Support for multiple proof formats
- Batch verification capabilities

#### KeygenCmd (`keygen.rs`)
**Purpose**: Generates proving and verification keys
**Key Features**:
- Key pair generation for specific VM configurations
- Support for different key formats
- Secure key storage

#### SetupCmd (`setup.rs`)
**Purpose**: Performs trusted setup for EVM verification
**Key Features**:
- EVM verification contract setup
- Circuit parameter generation
- Deployment configuration

#### RunCmd (`run.rs`)
**Purpose**: Executes OpenVM programs in trace mode
**Key Features**:
- Program execution with trace generation
- Debug output and profiling
- Input/output handling

#### CommitCmd (`commit.rs`)
**Purpose**: Commits to program execution traces
**Key Features**:
- Trace commitment generation
- Merkle tree construction
- Commitment verification

#### InitCmd (`init.rs`)
**Purpose**: Initializes new OpenVM projects
**Key Features**:
- Project template generation
- Configuration file creation
- Dependency setup

## Key Design Patterns

### Argument Grouping
Commands use `#[clap(flatten)]` to group related arguments:
```rust
#[derive(Parser)]
pub struct SomeCmd {
    #[clap(flatten)]
    build_args: BuildArgs,
    
    #[clap(flatten)]
    cargo_args: CargoArgs,
}
```

### Config Management
Commands consistently handle configuration files:
- Default location: `${manifest_dir}/openvm.toml`
- Override via `--config` flag
- Fallback to default configuration if not found

### File Path Handling
Standardized file path patterns:
- Target directory: `target/openvm/{profile}/`
- Executable output: `{target_name}.vmexe`
- Proof output: `{target_name}.proof`
- Key files: `app.pk` / `app.vk`

### Error Context
All commands provide rich error context using `eyre`:
```rust
.context("Failed to read configuration file")
.wrap_err("Build process failed")
```

## Integration Points

### Cargo Integration
- Respects Cargo workspace structure
- Forwards Cargo arguments (--release, --target-dir, etc.)
- Uses Cargo metadata for package discovery
- Integrates with Cargo build profiles

### VM Configuration
- Reads VM extensions from `openvm.toml`
- Supports custom instruction sets
- Configurable memory layout
- Extension-specific parameters

### File System
- Creates output directories as needed
- Handles relative and absolute paths
- Cleans up temporary files on error
- Respects system permissions

## Memory and Performance

### Memory Management
- Lazy loading of large data structures
- Streaming file I/O for large proofs
- Memory-efficient proof generation
- Resource cleanup on completion

### Performance Optimizations
- Parallel compilation when possible
- Caching of build artifacts
- Incremental builds support
- Optimized proof serialization

## Security Considerations

### Key Management
- Never logs private keys
- Secure key generation using system randomness
- Safe key file permissions
- Key validation before use

### Input Validation
- Path traversal prevention
- File size limits for inputs
- Configuration validation
- Sanitized error messages

### Proof Integrity
- Cryptographic proof validation
- Tamper detection
- Version compatibility checks
- Format validation

## Extension Points

### Custom Commands
New commands can be added by:
1. Creating a new module in `commands/`
2. Implementing the command pattern
3. Exporting from `mod.rs`
4. Adding to main CLI dispatcher

### VM Extensions
Commands can be extended to support new VM features:
- Custom instruction sets
- New proof systems
- Additional file formats
- Enhanced debugging features

## Testing Strategy

### Unit Tests
- Argument parsing validation
- Configuration handling
- Error condition testing
- Path manipulation logic

### Integration Tests
- End-to-end command execution
- File I/O operations
- Cross-command workflows
- Error recovery scenarios

### Performance Tests
- Large program compilation
- Memory usage profiling
- Proof generation benchmarks
- Concurrent operation handling

## Common Usage Patterns

### Development Workflow
1. `openvm init` - Initialize project
2. `openvm build` - Compile program
3. `openvm run` - Test execution
4. `openvm keygen` - Generate keys
5. `openvm prove` - Generate proof
6. `openvm verify` - Verify proof

### Production Deployment
1. `openvm build --release` - Optimized build
2. `openvm keygen --output-dir keys/` - Key generation
3. `openvm setup` - EVM setup (if needed)
4. `openvm prove --output-dir proofs/` - Batch proving

### Debugging
1. `openvm build --debug` - Debug build
2. `openvm run --trace` - Trace execution
3. `openvm prove --verbose` - Verbose proving

## Configuration Schema

### openvm.toml Structure
```toml
[extensions]
rv32im = {}
native = {}
keccak = {}

[memory]
max_size = 4294967296

[prover]
system = "STARK"
```

## Error Handling Patterns

### Common Error Types
- `ConfigError`: Configuration file issues
- `BuildError`: Compilation failures
- `ProofError`: Proof generation/verification failures
- `IOError`: File system operations
- `ValidationError`: Input validation failures

### Error Recovery
- Automatic retry for transient failures
- Graceful degradation for optional features
- Clear recovery instructions in error messages
- State cleanup on failure

## Maintenance Guidelines

### Adding New Commands
1. Follow established naming conventions
2. Implement consistent argument patterns
3. Add comprehensive error handling
4. Include unit and integration tests
5. Update CLI help documentation

### Breaking Changes
- Version command interfaces appropriately
- Provide migration guides
- Maintain backward compatibility for file formats
- Deprecate features gradually

### Performance Monitoring
- Track command execution times
- Monitor memory usage patterns
- Profile proof generation performance
- Benchmark file I/O operations