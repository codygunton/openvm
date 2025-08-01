# ff_derive Component Index

## File Structure

```
ff_derive/
├── src/
│   ├── lib.rs              # Main macro implementation
│   └── pow_fixed.rs        # Addition chain generation
├── tests/
│   ├── lib.rs              # Integration tests
│   └── programs/           # Test programs
│       └── examples/
│           ├── batch_inversion.rs
│           ├── constants.rs
│           ├── fermat.rs
│           ├── from_u128.rs
│           ├── full_limbs.rs
│           ├── operations.rs
│           └── sqrt.rs
└── Cargo.toml
```

## Key Exports

### Procedural Macros
- `openvm_prime_field` - Attribute macro for deriving prime field implementations

## Internal Modules

### lib.rs (lines: 1689)
Main macro implementation containing:
- `ReprEndianness` - Endianness configuration for field representations
- `openvm_prime_field` - Main procedural macro entry point
- Code generation functions:
  - `openvm_struct_impl` - zkVM struct generation
  - `validate_struct` - Input validation
  - `prime_field_repr_impl` - Representation type implementation
  - `prime_field_constants_and_sqrt` - Constant computation
  - `prime_field_impl` - Main trait implementations
- Helper functions:
  - `biguint_to_u64_vec` - BigUint conversion utilities
  - `exp` - Modular exponentiation
  - `mont_impl` - Montgomery reduction
  - `sqr_impl` - Squaring implementation
  - `mul_impl` - Multiplication implementation
  - `inv_impl` - Inversion implementation

### pow_fixed.rs (lines: 56)
Addition chain generation for optimized exponentiation:
- `generate` - Main entry point for addition chain code generation
- Uses `addchain` crate for optimal chain computation

## Key Dependencies

- `proc-macro2` - Token stream manipulation
- `quote` - Rust code generation
- `syn` - Rust syntax parsing
- `num-bigint` - Arbitrary precision arithmetic
- `addchain` - Addition chain computation

## Test Coverage

### Integration Tests
- `test_full_limbs` - Tests maximum size field elements
- `test_fermat` - Small prime (65537) arithmetic
- `test_sqrt` - Square root computation
- `test_constants` - Field constant generation
- `test_from_u128` - u128 conversion
- `test_batch_inversion` - Batch inversion with std
- `test_operations` - General field operations

## Usage Context

This crate is used by:
- Guest programs requiring finite field arithmetic
- Cryptographic protocols (BLS12-381, BN254)
- OpenVM algebra extensions
- Any component needing zkVM-compatible field arithmetic

## Key Algorithms

1. **Montgomery Reduction** - Efficient modular multiplication
2. **Tonelli-Shanks** - Square root in prime fields
3. **Addition Chains** - Optimized fixed exponentiation
4. **Fermat's Little Theorem** - Field element inversion