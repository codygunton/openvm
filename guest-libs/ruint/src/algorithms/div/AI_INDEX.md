# Ruint Division Algorithms - AI Index

## Component Overview
High-performance division algorithms for arbitrary-precision unsigned integers in the ruint library. Implements Knuth's classical algorithm and modern reciprocal-based methods from MG10.

## Key Files
- `mod.rs`: Main division entry point and dispatch logic
- `small.rs`: Small-case division (1-3 limb divisors)
- `reciprocal.rs`: Reciprocal computation for division by multiplication
- `knuth.rs`: Knuth's Algorithm D for general n×m division

## Core Algorithms
1. **div_2x1**: 128-bit by 64-bit division using reciprocals
2. **div_3x2**: 192-bit by 128-bit division using reciprocals
3. **div_nx1**: Multi-limb by single limb division
4. **div_nx2**: Multi-limb by double limb division
5. **div_nxm**: General Knuth division for arbitrary sizes

## Technical Highlights
- Reciprocal-based division for performance (MG10 algorithms)
- Normalization handling for optimal computation
- In-place operations to minimize allocations
- Careful overflow/underflow handling
- Comprehensive test coverage with proptest

## Performance Characteristics
- O(n²) complexity for general division
- Optimized paths for small divisors
- ~2.7ns for 2x1 division vs 18ns for naive approach
- Branch prediction hints for hot paths

## Dependencies
- Core algorithms only (no external dependencies)
- Uses intrinsic u128 operations
- Proptest for property-based testing

## Usage Context
Used throughout ruint for:
- Modular arithmetic operations
- Base conversion and string formatting
- General arithmetic operations
- Cryptographic computations