# GCD Algorithms Component - AI Index

## Overview
This component implements high-performance Greatest Common Divisor (GCD) algorithms for arbitrary-precision unsigned integers in the `ruint` library. It provides Lehmer's GCD algorithm with matrix optimizations for standard GCD, extended GCD, and modular inverse operations.

## Key Files
- `mod.rs` - Main module exposing GCD functions (`gcd`, `gcd_extended`, `inv_mod`)
- `matrix.rs` - Lehmer update matrix implementation for fast GCD computation
- `gcd_old.rs` - Alternative implementation with specialized U256 optimizations

## Core Algorithms

### Lehmer's GCD Algorithm
The implementation uses Lehmer's algorithm which accelerates the standard Euclidean algorithm by:
1. Working with 64-bit or 128-bit prefixes of large numbers
2. Computing update matrices that encode multiple Euclidean steps
3. Applying matrices to update values in bulk

### Matrix Operations
The `LehmerMatrix` struct encodes 2x2 transformation matrices with implicit signs:
- Efficient composition of multiple Euclidean steps
- Specialized methods for different bit sizes (u64, u128, Uint)
- Sign patterns encoded in a boolean flag for space efficiency

### Extended GCD
Computes GCD along with Bezout coefficients satisfying:
- `gcd = a * x + b * y` (when sign is true)
- `gcd = b * y - a * x` (when sign is false)

### Modular Inverse
Specialized implementation that:
- Only computes the required cofactor
- Returns `None` if numbers aren't coprime
- Handles sign corrections for modular arithmetic

## Performance Characteristics
- Optimized for large integers (256+ bits)
- Falls back to simple Euclidean steps when Lehmer steps fail
- Matrix operations reduce number of expensive division operations
- SIMD-friendly cofactor operations using SWAR techniques

## Usage Context
- Core component of the `ruint` arbitrary-precision integer library
- Used for cryptographic operations requiring modular arithmetic
- Foundation for rational number arithmetic and other advanced operations