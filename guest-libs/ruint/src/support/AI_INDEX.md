# Ruint Support Module AI Documentation Index

This directory contains AI-focused documentation for the Ruint support module, which provides integrations between the `ruint` arbitrary precision integer library and the broader Rust ecosystem.

## Documentation Files

### [AI_DOCS.md](./AI_DOCS.md)
High-level architectural overview of the support module, including:
- Integration categories and external crate support
- zkVM optimization strategies for 256-bit operations
- Serialization patterns for various formats
- Database integration approaches
- Security and performance considerations

### [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md)
Detailed implementation patterns and code examples for:
- Adding new external crate support
- Implementing serialization formats
- Creating database type mappings
- Optimizing for zkVM targets
- Testing strategies for integrations

### [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md)
Concise reference for common operations:
- Feature flag reference
- Trait implementation patterns
- Conversion snippets
- Error handling templates
- Performance optimization tips

## Component Source Files

### Serialization Support
- [`serde.rs`](./serde.rs) - Serde serialize/deserialize (human-readable hex, binary formats)
- [`borsh.rs`](./borsh.rs) - Borsh binary serialization
- [`ssz.rs`](./ssz.rs) - SimpleSerialize for Ethereum 2.0
- [`scale.rs`](./scale.rs) - SCALE codec for Substrate/Polkadot
- [`rlp.rs`](./rlp.rs) - Recursive Length Prefix encoding
- [`alloy_rlp.rs`](./alloy_rlp.rs) - Alloy framework RLP support
- [`fastrlp_03.rs`](./fastrlp_03.rs) - FastRLP v0.3 support
- [`fastrlp_04.rs`](./fastrlp_04.rs) - FastRLP v0.4 support
- [`der.rs`](./der.rs) - Distinguished Encoding Rules

### Numeric Traits
- [`num_traits.rs`](./num_traits.rs) - Comprehensive numeric trait implementations
- [`num_bigint.rs`](./num_bigint.rs) - BigInt/BigUint conversions
- [`num_integer.rs`](./num_integer.rs) - Integer-specific operations
- [`primitive_types.rs`](./primitive_types.rs) - Parity primitive type conversions

### Cryptographic Support
- [`ark_ff.rs`](./ark_ff.rs) - Arkworks finite field v0.3
- [`ark_ff_04.rs`](./ark_ff_04.rs) - Arkworks finite field v0.4
- [`bn_rs.rs`](./bn_rs.rs) - BN curve library support
- [`subtle.rs`](./subtle.rs) - Constant-time operations
- [`zeroize.rs`](./zeroize.rs) - Secure memory clearing

### Database Integration
- [`postgres.rs`](./postgres.rs) - PostgreSQL NUMERIC type support
- [`diesel.rs`](./diesel.rs) - Diesel ORM integration
- [`sqlx.rs`](./sqlx.rs) - SQLx async database support

### Testing & Random Generation
- [`rand.rs`](./rand.rs) - Random number generation v0.8
- [`rand_09.rs`](./rand_09.rs) - Random number generation v0.9
- [`proptest.rs`](./proptest.rs) - Property-based testing
- [`quickcheck.rs`](./quickcheck.rs) - QuickCheck support
- [`arbitrary.rs`](./arbitrary.rs) - Fuzzing support

### Special Integrations
- [`zkvm.rs`](./zkvm.rs) - OpenVM zkVM optimized operations ⚡
- [`bytemuck.rs`](./bytemuck.rs) - Pod trait for safe transmutes
- [`valuable.rs`](./valuable.rs) - Structured logging support
- [`pyo3.rs`](./pyo3.rs) - Python bindings
- [`mod.rs`](./mod.rs) - Module exports and organization

## Feature Flags Reference

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `serde` | JSON/binary serialization | serde |
| `borsh` | Borsh serialization | borsh |
| `ssz` | Ethereum 2.0 SSZ | ethereum_ssz |
| `scale` | Substrate SCALE codec | parity-scale-codec |
| `rlp` | Ethereum RLP | rlp |
| `alloy-rlp` | Alloy RLP | alloy-rlp |
| `num-traits` | Numeric traits | num-traits |
| `num-bigint` | BigInt support | num-bigint |
| `postgres` | PostgreSQL | postgres-types, bytes |
| `diesel` | Diesel ORM | diesel |
| `sqlx` | Async SQL | sqlx-core |
| `rand` | Random gen v0.8 | rand-08 |
| `rand-09` | Random gen v0.9 | rand-09 |
| `proptest` | Property testing | proptest |
| `arbitrary` | Fuzzing | arbitrary |
| `subtle` | Constant-time ops | subtle |
| `zeroize` | Memory clearing | zeroize |

## zkVM Optimizations

The zkVM support provides hardware-accelerated operations for 256-bit integers:

### Optimized Operations
- Arithmetic: `add`, `sub`, `mul`
- Bitwise: `xor`, `and`, `or`, `shl`, `shr`
- Comparison: `eq`, `cmp`
- Memory: `clone`

### Performance Impact
- Up to 10x speedup for 256-bit operations
- Zero-copy implementations where possible
- Automatic detection and use when targeting zkVM

## Quick Start

For AI assistants new to this component:
1. Start with [AI_DOCS.md](./AI_DOCS.md) for architectural understanding
2. Review specific integration files for implementation details
3. Reference [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md) for common patterns
4. Use [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md) for adding new integrations

## Common Integration Patterns

### Trait Implementation
Most integrations follow a pattern of:
1. Feature-gated module
2. Trait implementations for `Uint<BITS, LIMBS>`
3. Conversion functions with error handling
4. Comprehensive tests

### Type Conversions
Standard patterns include:
- `From`/`TryFrom` for type conversions
- Custom error types for failed conversions
- Both owned and borrowed variants

### Serialization
Common approach:
- Human-readable: Hex strings with `0x` prefix
- Binary: Big-endian byte arrays
- Support for both minimal and fixed-width formats

## Related Components

The support module integrates with:
- `ruint` core library - The main `Uint` type and operations
- `ruint-macro` - Procedural macros for `uint!` literals
- External crates - All the supported third-party libraries

## Performance Notes

- Feature flags have no runtime cost when disabled
- zkVM optimizations are automatic when targeting zkVM
- Most conversions are zero-copy where possible
- Database operations may allocate for type conversions