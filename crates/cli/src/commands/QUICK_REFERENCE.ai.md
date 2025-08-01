# OpenVM CLI Commands - Quick Reference

## Command Overview

| Command | Purpose | Key Options |
|---------|---------|-------------|
| `build` | Compile to VM executable | `--no-transpile`, `--config` |
| `run` | Execute VM program | `--exe`, `--input` |
| `prove` | Generate proofs | `app`, `stark`, `evm` subcommands |
| `verify` | Verify proofs | `app`, `stark`, `evm` subcommands |
| `keygen` | Generate proving keys | `--config`, `--output-dir` |
| `init` | Create new project | `--bin`, `--lib` |
| `commit` | Show commitments | `--exe`, `--app-pk` |
| `setup` | Setup EVM proving | `--evm`, `--force-agg-keygen` |

## Common Usage Patterns

### Basic Workflow
```bash
# Initialize new project
cargo openvm init myproject

# Build the project
cargo openvm build

# Run the program
cargo openvm run --input input.json

# Generate keys
cargo openvm keygen

# Generate proof
cargo openvm prove app

# Verify proof
cargo openvm verify app
```

### Build Patterns
```bash
# Build specific package
cargo openvm build -p mypackage

# Build all workspace members
cargo openvm build --workspace

# Build with features
cargo openvm build -F feature1,feature2

# Build specific binary
cargo openvm build --bin mybinary

# Skip transpilation (ELF only)
cargo openvm build --no-transpile
```

### Prove Patterns
```bash
# Generate app proof
cargo openvm prove app

# Generate STARK proof
cargo openvm prove stark

# Generate EVM proof (requires setup)
cargo openvm prove evm

# Prove with custom paths
cargo openvm prove app --app-pk custom.pk --proof custom.proof
```

### Common Arguments

#### Cargo Arguments (most commands)
```bash
-p, --package <PKG>      # Target specific package
--bin <NAME>             # Target specific binary
--example <NAME>         # Target specific example
-F, --features <LIST>    # Enable features
--profile <NAME>         # Build profile (default: release)
--target-dir <DIR>       # Custom target directory
--manifest-path <PATH>   # Path to Cargo.toml
```

#### OpenVM Arguments
```bash
--config <PATH>          # Path to openvm.toml
--output-dir <DIR>       # Output directory for artifacts
--exe <PATH>             # Pre-built executable path
--input <PATH>           # Input file for execution
```

## File Locations

### Default Paths
```
project/
├── Cargo.toml
├── openvm.toml          # VM configuration
├── src/
│   └── main.rs
└── target/
    └── openvm/
        ├── app.pk       # Application proving key
        ├── app.vk       # Application verifying key
        └── release/
            └── myapp.vmexe  # VM executable
```

### Proof Outputs
```
./
├── myapp.app.proof      # Application proof
├── myapp.stark.proof    # STARK proof
└── myapp.evm.proof      # EVM proof
```

## Quick Code Snippets

### Command Implementation
```rust
#[derive(Parser)]
#[command(name = "cmd")]
pub struct CmdName {
    #[arg(long)]
    option: String,
}

impl CmdName {
    pub fn run(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

### Config Loading
```rust
let config = read_config_toml_or_default(
    args.config.unwrap_or_else(|| PathBuf::from("openvm.toml"))
)?;
```

### Build Integration
```rust
let exe_path = if let Some(exe) = &args.exe {
    exe.clone()
} else {
    let output_dir = build(&build_args, &cargo_args)?;
    output_dir.join(format!("{}.vmexe", target_name))
};
```

### Key Loading
```rust
let app_pk = Arc::new(read_app_pk_from_file(
    args.app_pk.unwrap_or_else(|| get_app_pk_path(&target_dir))
)?);
```

### Error Context
```rust
operation()
    .context("Failed to perform operation")?;
```

## Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `RUSTFLAGS` | Rust compiler flags | `-C target-cpu=native` |
| `CARGO_TARGET_DIR` | Override target directory | `/tmp/target` |

## Common Issues

### Issue: "No binary target found"
```bash
# Specify target explicitly
cargo openvm build --bin mybin
```

### Issue: "Proving key not found"
```bash
# Generate keys first
cargo openvm keygen
```

### Issue: "Multiple proof files found"
```bash
# Specify proof file
cargo openvm verify app --proof myapp.app.proof
```

### Issue: "Out of memory during proving"
```bash
# Use release profile
cargo openvm prove app --profile release
```

## Command Aliases

Create shell aliases for common operations:
```bash
alias ovmb='cargo openvm build'
alias ovmr='cargo openvm run'
alias ovmp='cargo openvm prove app'
alias ovmv='cargo openvm verify app'
```

## Advanced Patterns

### Workspace Builds
```bash
# Build all members except one
cargo openvm build --workspace --exclude slowpkg

# Build multiple specific packages
cargo openvm build -p pkg1 -p pkg2
```

### Custom Configurations
```bash
# Use alternative config
cargo openvm build --config configs/prod.toml

# Override output directory
cargo openvm build --output-dir ./artifacts
```

### Proof Pipeline
```bash
# Full proof pipeline
cargo openvm prove app && \
cargo openvm prove stark && \
cargo openvm prove evm
```

### Development Workflow
```bash
# Fast iteration
cargo openvm build --profile dev && \
cargo openvm run --input test.json

# Production build
cargo openvm build --profile release && \
cargo openvm keygen && \
cargo openvm prove app
```

## Performance Tips

1. **Build Caching**: Reuse `--target-dir` across builds
2. **Parallel Builds**: Use `--jobs` for parallel compilation
3. **Profile Selection**: Use `release` for proving
4. **Key Caching**: Generate keys once, reuse many times
5. **Memory Management**: Monitor system resources during EVM proving

## Debugging Commands

```bash
# Verbose output
cargo openvm build -v

# Quiet mode
cargo openvm build -q

# Check configuration
cargo openvm commit --exe myapp.vmexe

# Verify setup
cargo openvm verify stark --proof test.stark.proof
```