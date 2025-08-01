# OpenVM Project AI Documentation

This document provides comprehensive guidance for AI assistants working with the OpenVM codebase, a performant and modular zkVM framework built for customization and extensibility.

## Project Overview

OpenVM is a zero-knowledge virtual machine framework featuring:
- **No-CPU Architecture**: Unique design without a central processing unit
- **Modular Extensions**: Seamless integration of custom chips without forking core architecture
- **STARK-based Proving**: Built on Plonky3 for zero-knowledge proof generation
- **Rust Implementation**: MSRV 1.82, 70+ crates in monorepo structure
- **Security Audited**: Reviewed by Cantina and Axiom team

## Architecture Principles

### Core Design
1. **Modular zkVM**: Extensible instruction set via chip-based extensions
2. **No Central CPU**: Operations handled by specialized chips
3. **AIR Constraints**: Circuits implemented as Algebraic Intermediate Representation
4. **Field-Generic**: Supports both native fields and extension fields
5. **Proof Continuations**: Support for large computation splitting

### Key Technologies
- **STARK Backend**: Plonky3-based proving system
- **Proof System**: STARKs with FRI (Fast Reed-Solomon Interactive Oracle Proofs)
- **On-chain Verification**: Ethereum smart contract integration
- **Guest Programs**: no_std Rust programs compiled to OpenVM bytecode

## Project Structure

### `/crates` - Core Components
```
crates/
├── sdk/           # Main SDK for building and proving programs
├── cli/           # Command-line interface (cargo-openvm)
├── vm/            # Core virtual machine implementation
├── circuits/      # Circuit building primitives
│   ├── mod-builder/     # Modular circuit builder
│   ├── poseidon2-air/   # Poseidon2 hash AIR
│   └── primitives/      # Circuit primitives and traits
├── toolchain/     # Rust toolchain integration
│   ├── transpiler/      # ELF to OpenVM bytecode transpiler
│   ├── instructions/    # Instruction definitions
│   ├── platform/        # Platform-specific code
│   └── openvm/          # Guest program runtime
├── continuations/ # Proof continuation support
└── prof/          # Profiling tools
```

### `/extensions` - VM Extensions
Standard extensions providing additional functionality:
- `rv32im/` - RISC-V 32-bit instruction support
- `native/` - Native field arithmetic for recursion
- `algebra/` - Modular arithmetic over arbitrary fields
- `bigint/` - Int256 arithmetic operations
- `ecc/` - Elliptic curve operations
- `pairing/` - Pairing operations (BN254, BLS12-381)
- `keccak256/` - Keccak-256 hash function
- `sha256/` - SHA2-256 hash function

### `/guest-libs` - Guest Libraries
Libraries for use in guest programs:
- `k256/`, `p256/` - Elliptic curve operations
- `keccak256/`, `sha2/` - Hash functions
- `pairing/` - Pairing operations
- `ruint/` - Unsigned integer types
- `verify_stark/` - STARK proof verification

## Development Guidelines

### Code Standards
1. **Rust Conventions**: Follow standard Rust naming (snake_case, PascalCase)
2. **No-std Compatibility**: Guest code must be no_std compatible
3. **Error Handling**: Use `eyre::Result` for host code, specific error types for libraries
4. **Testing**: Unit tests in src/, integration tests in tests/, use test-case for parameterized tests
5. **Documentation**: All public APIs must have doc comments with examples

### Circuit Development
1. **AIR Implementation**: Circuits as AIR constraints using SubAir trait
2. **Trace Separation**: Separate trace generation from constraint definition
3. **Field Operations**: Use generic field traits from p3-field
4. **Modular Design**: Support both native and extension fields

### Extension Guidelines
Each extension must have:
```
extension-name/
├── circuit/     # Circuit implementation (AIR constraints)
├── transpiler/  # Instruction transpilation logic
├── guest/       # Guest library for Rust programs
└── tests/       # Integration tests
```

### Security Requirements
1. **Cryptographic Code**: All crypto implementations require security review
2. **Input Validation**: Validate all inputs from untrusted sources
3. **Memory Safety**: Use safe abstractions, avoid buffer overflows in no_std
4. **Dependency Auditing**: Regular cargo audit runs, careful dependency review
5. **Documentation**: Document security assumptions and threat models

## Key Workflow Commands

### Development
```bash
# Build all crates
cargo build --all

# Fast development build
cargo build --profile=fast

# Run all tests
cargo test --all

# Format and lint
cargo fmt --all
cargo clippy --all-features
```

### Guest Programs
```bash
# Build guest program
cargo openvm build --elf /path/to/elf

# Generate proof
cargo openvm prove --exe /path/to/exe

# Setup proving keys
cargo openvm setup --app-config /path/to/config
```

### Testing Levels
1. **Unit Tests**: Module-level tests with #[cfg(test)]
2. **Integration Tests**: Cross-crate functionality in tests/
3. **End-to-End**: Full guest program execution and proving
4. **Benchmarks**: Performance testing with criterion

## Component Integration

### VM Architecture
- **Execution**: Programs execute via chip-specific operations
- **Memory System**: Persistent, volatile, and offline memory models
- **Bus Architecture**: Inter-chip communication via bus protocols
- **Proof Generation**: Incremental STARK proof construction

### Extension Integration
1. **Circuit Registration**: Extensions register AIR constraints
2. **Instruction Mapping**: Transpiler maps operations to chip instructions
3. **Guest Library**: Rust API for guest program developers
4. **Testing**: Extension-specific test suites

### Configuration Management
- **Global Config**: VM-wide settings and extension configurations
- **App Config**: Application-specific proving parameters
- **Extension Config**: Per-extension configuration options

## Common Patterns

### AIR Implementation
```rust
impl<F: Field> BaseAir<F> for MyChip {
    fn width(&self) -> usize { /* column count */ }
}

impl<AB: AirBuilder> Air<AB> for MyChip {
    fn eval(&self, builder: &mut AB) {
        // Define constraints
    }
}
```

### Trace Generation
```rust
impl<F: Field> Chip<F> for MyChip {
    fn generate_trace(&self, input: &[F]) -> RowMajorMatrix<F> {
        // Generate execution trace
    }
}
```

### Extension Structure
```rust
pub struct MyExtension;

impl<F: Field> VmExtension<F> for MyExtension {
    fn air(&self) -> Box<dyn Air<AB>> { /* return AIR */ }
    fn transpiler(&self) -> Box<dyn InstructionTranspiler> { /* return transpiler */ }
}
```

## Testing Strategies

### Circuit Testing
1. **Constraint Satisfaction**: Verify AIR constraints on valid traces
2. **Boundary Conditions**: Test edge cases and boundary values
3. **Soundness**: Ensure invalid traces are rejected
4. **Performance**: Benchmark proving and verification times

### Integration Testing
1. **Guest Programs**: End-to-end execution and proving
2. **Extension Interaction**: Multi-extension program testing
3. **Memory Consistency**: Memory system correctness
4. **Proof Verification**: On-chain and off-chain verification

## Performance Considerations

### Optimization Guidelines
1. **Profile First**: Use cargo-flamegraph for CPU profiling
2. **Algorithmic Improvements**: Prefer algorithm optimization over micro-optimization
3. **Memory Management**: Minimize allocations in hot paths
4. **Const Generics**: Leverage compile-time optimization

### Circuit Optimization
1. **Constraint Minimization**: Reduce constraint complexity
2. **Column Efficiency**: Minimize trace column count
3. **Lookup Tables**: Use lookup arguments for complex operations
4. **Batch Operations**: Group similar operations for efficiency

## Troubleshooting Guide

### Common Issues
1. **Build Failures**: Check Rust version (MSRV 1.82), dependency conflicts
2. **Test Failures**: Enable RUST_LOG=debug for detailed logging
3. **Proof Generation**: Verify circuit constraints and trace generation
4. **Guest Program Issues**: Check no_std compatibility and dependency features

### Debug Tools
1. **Logging**: Use env_logger with RUST_LOG environment variable
2. **Profiling**: cargo-flamegraph for performance analysis
3. **Memory**: heaptrack for memory usage analysis
4. **Circuit Debug**: Custom debug utilities in vm/src/arch/testing/

## Development Best Practices

### Code Quality
1. **Modularity**: Keep components loosely coupled
2. **Documentation**: Maintain comprehensive API documentation
3. **Testing**: Achieve >80% code coverage
4. **Performance**: Regular benchmark comparisons

### Security Practices
1. **Code Review**: Security-focused review for cryptographic code
2. **Dependency Management**: Regular security audits
3. **Input Validation**: Comprehensive input sanitization
4. **Documentation**: Clear security assumption documentation

## Future Considerations

### Extensibility
- Design for future instruction set extensions
- Maintain backward compatibility in core APIs
- Support for new cryptographic primitives
- Scalable proof system improvements

### Performance Roadmap
- Proof system optimizations
- Circuit compilation improvements
- Parallel proving support
- Memory system enhancements

This documentation should be updated as the project evolves, with particular attention to new extensions, API changes, and security considerations.