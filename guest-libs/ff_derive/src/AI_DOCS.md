# ff_derive Component Documentation

## Overview

The `openvm-ff-derive` crate provides a procedural macro for deriving finite field arithmetic implementations compatible with both standard Rust environments and the OpenVM zkVM. This is a fork of the original `ff_derive` crate, modified to generate dual implementations that work seamlessly in zkVM contexts.

## Core Purpose

This crate enables developers to define prime field structs that automatically receive:
- Complete finite field arithmetic implementations (`ff::Field` and `ff::PrimeField` traits)
- Montgomery form arithmetic for standard environments
- Native zkVM arithmetic operations when compiled for `target_os = "zkvm"`
- Automatic memory layout optimization (32 or 48 bytes)

## Key Features

1. **Dual Implementation**: Generates different implementations based on compilation target
   - Standard: Montgomery form arithmetic with optimized algorithms
   - zkVM: Direct integration with OpenVM's modular arithmetic instructions

2. **Memory Layout**: Automatically selects optimal field element size
   - 32-byte representation for smaller moduli
   - 48-byte representation for larger moduli

3. **Endianness Support**: Configurable big-endian or little-endian representations

4. **Optimized Operations**: 
   - Addition chain generation for fixed exponentiation
   - Tonelli-Shanks algorithm for square roots
   - Montgomery reduction for efficient modular arithmetic

## Architecture

The crate consists of:
- `lib.rs`: Main macro implementation and code generation
- `pow_fixed.rs`: Addition chain generation for optimized exponentiation

### Key Components

1. **Attribute Macro**: `#[openvm_prime_field]` replaces struct definitions
2. **Code Generation**: Produces trait implementations for both targets
3. **Constant Generation**: Computes field constants at compile time
4. **Validation**: Ensures correct struct format and modulus constraints

## Usage Pattern

```rust
#[openvm_prime_field]
#[PrimeFieldModulus = "..."]
#[PrimeFieldGenerator = "..."]
#[PrimeFieldReprEndianness = "little"]
struct FieldElement([u64; N]);
```

The macro removes the original struct and replaces it with a zkVM-compatible version while maintaining the same API.

## Integration Points

- Works with `openvm-algebra-guest` for zkVM arithmetic
- Compatible with `ff` crate traits
- Integrates with OpenVM's modular arithmetic transpiler
- Used by higher-level cryptographic components

## Testing Strategy

Tests verify:
- Field arithmetic operations (add, mul, square, invert)
- Square root computation for various moduli
- Constant generation correctness
- Cross-platform compatibility (standard vs zkVM)
- Integration with OpenVM transpiler and circuits