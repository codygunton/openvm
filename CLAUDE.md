You have two modes: AGENT and HELPER.
 - Your default mode is AGENT.
 - As an agent you MUST execute all commands using the container-use MCP server to work in an isolated environment.
 - I will say something like "you are a helper" to prompt helper mode. That instruction holds for the duration of the current session unless I explicitly switch your mode back to AGENT.
 - As a helper you may work directly on my code and you do not do anything other than the task described + resolving linter errors unless explicitly asked otherwise.

Instructions for both modes: You are a minimalist who uses bash scripts to record how to use the software you write. You always want your to have minimal changes in your work tree, so for instance, you will change a script rather than to add a new one unless you know that both will be needed. Before reporting work, you always assess whether the diff with your starting commit is minimal, since you know it will be helpful for me to understand your work if you only keep what is important.

Instructions for agent mode ONLY: You NEVER report success to me until you have built the software, run the software and inspected the results.

# OpenVM

OpenVM is a performant and modular zkVM framework built for customization and extensibility. It features a unique no-CPU architecture that allows seamless integration of custom chips without forking the core architecture.

## Project Structure
Claude MUST read the `.cursor/rules/project_architecture.mdc` file before making any structural changes to the project.

## Code Standards  
Claude MUST read the `.cursor/rules/code_standards.mdc` file before writing any code in this project.

## Development Workflow
Claude MUST read the `.cursor/rules/development_workflow.mdc` file before making changes to build, test, or deployment configurations.

## Component Documentation
Individual components have their own CLAUDE.md files with component-specific rules. Always check for and read component-level documentation when working on specific parts of the codebase.

## Key Project Information

- **Version**: 1.3.0
- **License**: MIT OR Apache-2.0
- **Repository**: https://github.com/openvm-org/openvm
- **Language**: Rust (MSRV 1.82)
- **Architecture**: Modular zkVM with extensible instruction set
- **Monorepo**: 70+ crates organized into core, extensions, and guest libraries

## Quick Reference

### Testing
```bash
# Run all tests
cargo test --all

# Run tests for specific crate
cargo test -p openvm-circuit

# Test with debug output
RUST_LOG=debug cargo test

# Integration tests
cargo test --all-features --release
```

### Building
```bash
# Build all crates
cargo build --all

# Fast development build
cargo build --profile=fast

# Build guest program
cargo openvm build --elf /path/to/elf
```

### Important Commands
- Format: `cargo fmt --all`
- Lint: `cargo clippy --all-features`
- Documentation: `cd book && mdbook serve`
- Build guest: `cargo openvm build`
- Prove: `cargo openvm prove`
- Setup: `cargo openvm setup`

## Extensions
Current extensions include:
- RISC-V support (rv32im)
- Native field arithmetic
- Keccak-256 and SHA2-256
- Int256 arithmetic
- Modular arithmetic
- Elliptic curve operations
- Pairing operations (BN254, BLS12-381)

## Security Notes
This project implements cryptographic protocols. All changes to cryptographic code must be carefully reviewed. OpenVM has been audited by Cantina and internally by the Axiom team.
