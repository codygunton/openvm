# OpenVM CLI Component Documentation

## Overview
The OpenVM CLI (`cargo-openvm`) is the primary command-line interface for the OpenVM zkVM framework. It provides a comprehensive set of tools for building, proving, verifying, and managing OpenVM applications.

## Architecture
- **Entry Point**: Binary in `src/bin/`
- **Commands**: Modular command structure in `src/commands/`
- **Core Library**: Main functionality in `src/lib.rs`
- **Utilities**: Helper functions in `src/util.rs` and `src/input.rs`

## Key Components

### Commands Module (`src/commands/`)
- `build.rs`: Compile guest programs to ELF format
- `prove.rs`: Generate proofs for executed programs
- `verify.rs`: Verify generated proofs
- `run.rs`: Execute programs in the OpenVM
- `keygen.rs`: Generate cryptographic keys
- `setup.rs`: Setup proving/verifying parameters (EVM feature)
- `commit.rs`: Commit to program execution
- `init.rs`: Initialize new OpenVM projects

### Core Features
- **Target Support**: x86_64 and aarch64 on Linux/macOS
- **Toolchain**: Uses nightly-2025-02-14 Rust toolchain
- **Command Execution**: Unified interface with proper stdio handling
- **Version Management**: Git-based versioning with build metadata

### Dependencies
- **OpenVM Core**: `openvm-build`, `openvm-transpiler`, `openvm-sdk`
- **Cryptographic**: `openvm-stark-sdk`, `openvm-circuit`
- **CLI Framework**: `clap` for argument parsing
- **Cloud Integration**: AWS SDK for S3 operations
- **Utilities**: `serde`, `hex`, `tempfile`, `toml`

### Features
- `parallel`: Enable parallel processing
- `evm-prove`/`evm-verify`: Ethereum integration
- `bench-metrics`: Performance benchmarking
- `profiling`: Guest program profiling
- Memory allocators: `jemalloc`, `mimalloc`

## Usage Patterns
1. **Project Initialization**: `cargo openvm init`
2. **Building**: `cargo openvm build --elf <path>`
3. **Execution**: `cargo openvm run`
4. **Proving**: `cargo openvm prove`
5. **Verification**: `cargo openvm verify`

## Command Trait
The `CommandExecutor` trait provides a unified interface for running system commands with proper stdio inheritance and error handling.

## Target Platform Detection
Built-in support for detecting and validating supported target platforms (Linux/macOS on x86_64/aarch64).