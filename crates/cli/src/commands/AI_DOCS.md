# OpenVM CLI Commands - Comprehensive Documentation

## Overview
The OpenVM CLI commands component implements the core command-line interface for the OpenVM zkVM framework. It provides commands for building, running, proving, and verifying OpenVM programs, integrating tightly with Cargo workflows.

## Command Architecture

### Command Structure
Each command follows a consistent pattern:
```rust
#[derive(Parser)]
#[command(name = "command_name", about = "Description")]
pub struct CommandNameCmd {
    // Command-specific arguments
    #[clap(flatten)]
    cargo_args: CargoArgs,
}

impl CommandNameCmd {
    pub fn run(&self) -> Result<()> {
        // Command implementation
    }
}
```

### Argument Organization
Arguments are organized into logical groups:
- **OpenVM Options**: VM-specific settings (config, output dirs, etc.)
- **Cargo Options**: Standard Cargo arguments (package, target, features)
- **Command Options**: Command-specific parameters

## Core Commands

### Build Command (`build.rs`)
Compiles Rust programs to OpenVM executables.

**Key Features:**
- Transpiles ELF binaries to VM executable format (`.vmexe`)
- Integrates with Cargo build system
- Supports workspace and package builds
- Generates init files for VM configuration

**Process Flow:**
1. Parse Cargo.toml and determine targets
2. Build using `cargo build` with OpenVM target
3. Read `openvm.toml` configuration
4. Transpile ELF to VM executable
5. Write output to `target/openvm/{profile}/`

**Important Types:**
- `BuildArgs`: OpenVM-specific build options
- `BuildCargoArgs`: Cargo build arguments

### Run Command (`run.rs`)
Executes OpenVM programs locally.

**Key Features:**
- Can build and run in one step
- Accepts input via stdin or files
- Displays execution output

**Process Flow:**
1. Build executable (if not provided)
2. Load VM configuration
3. Execute with SDK
4. Display results

### Prove Command (`prove.rs`)
Generates cryptographic proofs of program execution.

**Proof Types:**
1. **App Proof**: Basic application-level proof
2. **STARK Proof**: Aggregated STARK proof with commitments
3. **EVM Proof**: Final proof verifiable on Ethereum

**Key Features:**
- Multi-stage proving pipeline
- Proof compression and aggregation
- Commitment computation

**Process Flow:**
1. Load proving keys
2. Build and commit executable
3. Generate requested proof type
4. Write proof to file

### Verify Command (`verify.rs`)
Verifies generated proofs.

**Supported Verifications:**
- App proof verification (requires verifying key)
- STARK proof verification
- EVM proof verification (on-chain compatible)

**Process Flow:**
1. Load appropriate verifying key
2. Read proof from file
3. Verify using SDK
4. Report success/failure

### Keygen Command (`keygen.rs`)
Generates proving and verifying keys for applications.

**Key Features:**
- Reads VM configuration from `openvm.toml`
- Generates app-specific keys
- Caches keys in target directory

**Output:**
- `app.pk`: Application proving key
- `app.vk`: Application verifying key

### Init Command (`init.rs`)
Initializes new OpenVM projects.

**Key Features:**
- Wraps `cargo init` with OpenVM setup
- Adds OpenVM dependencies
- Creates template files
- Generates `openvm.toml` config

**Templates:**
- `main.rs`: Binary template with OpenVM imports
- `lib.rs`: Library template
- `openvm.toml`: Default VM configuration

### Commit Command (`commit.rs`)
Computes and displays executable commitments.

**Key Features:**
- Shows Bn254 commitments
- Required for proof verification
- Outputs JSON format

**Commitments:**
- `exe_commit`: Commitment to executable code
- `vm_commit`: Commitment to VM configuration

### Setup Command (`setup.rs`)
Sets up infrastructure for EVM proving.

**Key Features:**
- Downloads KZG parameters from S3
- Generates aggregation proving keys
- Creates Solidity verifier contracts
- Memory and compute intensive

**Modes:**
- `--stark`: Generate only STARK proving keys (default)
- `--evm`: Full EVM setup with Halo2 keys

## Configuration

### VM Configuration (`openvm.toml`)
```toml
[app_vm_config]
# VM extension configuration
```

### Build Configuration
- Profile support (release, dev, custom)
- Feature flags
- Target selection

## File Organization

### Input/Output Paths
- **Executables**: `target/openvm/{profile}/{target}.vmexe`
- **Proving Keys**: `target/openvm/app.pk`
- **Verifying Keys**: `target/openvm/app.vk`
- **Proofs**: `./{target}.{type}.proof`

### Default Paths
- Config: `{manifest_dir}/openvm.toml`
- Init file: `{manifest_dir}/init.json`

## Error Handling

### Common Errors
1. **Missing Config**: Falls back to default configuration
2. **Multiple Targets**: Requires explicit target selection
3. **Missing Keys**: Prompts to run keygen/setup
4. **Build Failures**: Reports Cargo exit codes

### Recovery Strategies
- Automatic config generation
- Helpful error messages with solutions
- Fallback to defaults where sensible

## Performance Considerations

### Build Performance
- Caches build artifacts
- Supports incremental compilation
- Profile-based optimization

### Proving Performance
- Memory requirements scale with circuit size
- EVM proving requires significant resources
- Proof generation can be parallelized

## Integration Points

### Cargo Integration
- Respects Cargo.toml settings
- Uses Cargo for dependency management
- Supports workspaces

### SDK Integration
- Uses SDK for proof generation
- Delegates execution to SDK
- Leverages SDK serialization

### File System Integration
- Creates necessary directories
- Manages artifact locations
- Handles path resolution

## Security Considerations

### Key Management
- Keys stored locally in target directory
- No automatic key distribution
- User responsible for key security

### Proof Verification
- Verifies proof integrity
- Checks commitments match
- Validates proof structure

## Best Practices

### Command Usage
1. Use `--help` for detailed options
2. Specify explicit targets in workspaces
3. Cache proving keys for performance
4. Verify proofs before submission

### Project Structure
1. Keep `openvm.toml` in project root
2. Use standard Cargo layout
3. Separate guest and host code
4. Version control config files

### Development Workflow
1. `init` → `build` → `run` → `keygen` → `prove` → `verify`
2. Test locally before proving
3. Use appropriate proof types
4. Monitor resource usage