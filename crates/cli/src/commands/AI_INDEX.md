# OpenVM CLI Commands Component - AI Index

## Component Overview
The OpenVM CLI commands component provides the core command-line interface functionality for interacting with OpenVM programs. It implements all major user-facing commands for building, proving, verifying, and managing OpenVM applications.

## Key Files
- `mod.rs` - Module exports and command organization
- `build.rs` - Compile OpenVM programs to VM executables  
- `prove.rs` - Generate cryptographic proofs (app, STARK, EVM)
- `run.rs` - Execute OpenVM programs locally
- `verify.rs` - Verify generated proofs
- `keygen.rs` - Generate application proving/verifying keys
- `init.rs` - Initialize new OpenVM projects
- `commit.rs` - View Bn254 commitments of executables
- `setup.rs` - Set up EVM proving infrastructure

## Architecture Patterns
- **Command Pattern**: Each command is a separate struct implementing a `run()` method
- **Argument Parsing**: Uses clap for CLI parsing with structured argument types
- **Build Integration**: Commands integrate with Cargo build system for seamless workflow
- **Proof Hierarchy**: Supports app proofs → STARK proofs → EVM proofs pipeline

## Key Concepts
- **Multi-stage Proving**: Application proofs can be aggregated to STARK then EVM proofs
- **Executable Format**: Programs compile to `.vmexe` format for VM execution
- **Cargo Integration**: Commands wrap and extend Cargo functionality for guest programs
- **Key Management**: Proving/verifying keys are generated and cached in target directory

## Dependencies
- SDK components for proof generation and verification
- Build system for compilation
- Transpiler for ELF to VM executable conversion
- Circuit architecture for VM configuration

## Common Patterns
1. **Build-First Pattern**: Most commands can optionally build before executing
2. **Target Resolution**: Smart detection of binaries/examples to run
3. **Config Loading**: Reads `openvm.toml` for VM configuration  
4. **Artifact Management**: Outputs stored in `target/openvm/{profile}/`