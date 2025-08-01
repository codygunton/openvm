# ruint Algorithms Component Index

## Overview
The ruint algorithms module provides low-level bignum arithmetic operations optimized for unsigned integer computations. This collection of algorithms forms the foundation for arbitrary-precision arithmetic operations in the ruint library.

## Component Purpose
- Provides efficient implementations of fundamental arithmetic operations on arrays of u64 limbs
- Supports operations like addition, multiplication, division, GCD, and modular arithmetic
- Designed for performance-critical cryptographic and mathematical computations
- Uses little-endian limb ordering for all operations

## Key Files

### Core Module
- `mod.rs` - Module root, exports public API and defines common traits/utilities
  - Defines `DoubleWord` trait for 128-bit arithmetic operations
  - Provides `cmp` function for comparing u64 slices
  - Helper functions for carry/borrow operations

### Arithmetic Operations
- `add.rs` - Addition operations with carry propagation
  - `adc_n`: Add with carry for equal-length arrays
  - `sbb_n`: Subtract with borrow for equal-length arrays

- `mul.rs` - Multiplication algorithms
  - `addmul`: Multiply-accumulate with overflow detection
  - `mul_nx1`: Multiply array by single limb
  - `addmul_nx1`: Multiply-accumulate array by single limb
  - `submul_nx1`: Multiply-subtract array by single limb
  - Specialized implementations for small fixed sizes (1-4 limbs)

- `mul_redc.rs` - Montgomery multiplication
  - `mul_redc`: Montgomery multiplication with reduction
  - `square_redc`: Montgomery squaring with reduction
  - Uses CIOS (Coarsely Integrated Operand Scanning) algorithm

- `shift.rs` - Bit shifting operations
  - `shift_left_small`: Left shift by < 64 bits
  - `shift_right_small`: Right shift by < 64 bits

- `ops.rs` - Basic arithmetic primitives
  - `adc`: Add with carry (single limb)
  - `sbb`: Subtract with borrow (single limb)

### Advanced Algorithms
- `div/` - Division algorithms subdirectory
  - `mod.rs`: Main division interface
  - `knuth.rs`: Knuth's Algorithm D implementation
  - `reciprocal.rs`: Reciprocal-based division
  - `small.rs`: Optimized small divisor operations

- `gcd/` - Greatest Common Divisor algorithms subdirectory
  - `mod.rs`: GCD interface and extended GCD
  - `matrix.rs`: Lehmer matrix operations
  - `gcd_old.rs`: Legacy GCD implementation

## Dependencies
- Core Rust only (no_std compatible)
- Uses intrinsic u128 type for double-word arithmetic
- Proptest for property-based testing (dev dependency)

## API Stability
⚠️ **Warning**: Most functions in this module are not part of the stable API and may change in future minor releases. The module is primarily intended for internal use by the ruint library.

## Performance Characteristics
- Optimized for 64-bit architectures
- Uses compiler intrinsics for carry/borrow operations
- Specialized implementations for small operand sizes
- Montgomery multiplication for efficient modular arithmetic