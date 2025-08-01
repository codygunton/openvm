# OpenVM Algebra Moduli Macros - AI Index

## Component Overview
This crate provides procedural macros for declaring and working with modular arithmetic types in OpenVM. It generates optimized implementations for prime field arithmetic that work both in standard Rust and within the zkVM environment.

## Key Features
- Procedural macros for declaring modular arithmetic types
- Automatic generation of arithmetic operations (add, sub, mul, div)
- Support for both prime and composite moduli
- Field trait implementation for prime moduli
- Square root computation with hint-based proving
- zkVM-specific optimizations using custom instructions

## File Structure

### `/src/lib.rs`
Main procedural macro implementations:
- `moduli_declare!` - Declares modular arithmetic types with specified moduli
- `moduli_init!` - Initializes modular arithmetic setup for zkVM execution
- Generates arithmetic operations, equality checks, and field operations
- Contains zkVM-specific extern function implementations
- Handles both 32-byte and 48-byte limb sizes

## Dependencies
- `syn` - Parsing procedural macro input
- `quote` - Generating Rust code
- `openvm-macros-common` - Common macro utilities
- `num-prime` - Primality testing
- `num-bigint` - Big integer operations

## Usage Context
This crate is used to generate modular arithmetic types for various cryptographic operations:
- Elliptic curve field elements (BN254, BLS12-381)
- Prime field arithmetic
- Pairing-based cryptography
- General modular arithmetic operations

## Key Concepts
1. **Modular Types**: Generated structs representing integers modulo a prime/composite
2. **zkVM Integration**: Custom instructions for efficient modular arithmetic
3. **Hint-Based Proving**: Square root existence proofs using hints
4. **Setup Instructions**: Runtime modulus configuration for zkVM