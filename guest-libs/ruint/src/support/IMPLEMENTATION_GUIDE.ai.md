# Ruint Support Module Implementation Guide

This guide provides detailed patterns and examples for implementing support for external crates in the ruint library.

## Table of Contents
1. [Adding New External Crate Support](#adding-new-external-crate-support)
2. [Serialization Implementation Patterns](#serialization-implementation-patterns)
3. [Database Type Mapping](#database-type-mapping)
4. [zkVM Optimization Techniques](#zkvm-optimization-techniques)
5. [Testing Strategies](#testing-strategies)
6. [Common Pitfalls and Solutions](#common-pitfalls-and-solutions)

## Adding New External Crate Support

### Step 1: Create Module Structure

Create a new file in `src/support/` named after your crate:

```rust
//! Support for the [`my-crate`](https://crates.io/crates/my-crate) crate.
#![cfg(feature = "my-crate")]
#![cfg_attr(docsrs, doc(cfg(feature = "my-crate")))]

use crate::{Uint, nbytes};
use my_crate::{Trait1, Trait2};

// Implementation goes here
```

### Step 2: Update Cargo.toml

Add the dependency with optional flag:
```toml
[dependencies]
my-crate = { version = "1.0", optional = true, default-features = false }

[features]
my-crate = ["dep:my-crate", "alloc"]  # Include other required features
```

### Step 3: Add Module Export

In `src/support/mod.rs`:
```rust
#[cfg(feature = "my-crate")]
mod my_crate;
```

### Step 4: Implement Required Traits

Example trait implementation pattern:
```rust
impl<const BITS: usize, const LIMBS: usize> MyTrait for Uint<BITS, LIMBS> {
    type Error = ConversionError;
    
    fn my_method(&self) -> Result<SomeType, Self::Error> {
        // Handle edge cases first
        if BITS == 0 {
            return Ok(SomeType::default());
        }
        
        // Implement conversion logic
        // ...
    }
}
```

## Serialization Implementation Patterns

### Human-Readable Format (JSON, YAML)

```rust
impl<const BITS: usize, const LIMBS: usize> Serialize for Uint<BITS, LIMBS> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            // Serialize as hex string
            if self.is_zero() {
                serializer.serialize_str("0x0")
            } else {
                serializer.serialize_str(&format!("{self:#x}"))
            }
        } else {
            // Serialize as bytes
            serializer.serialize_bytes(&self.to_be_bytes_vec())
        }
    }
}
```

### Binary Format Implementation

```rust
impl<const BITS: usize, const LIMBS: usize> Encode for Uint<BITS, LIMBS> {
    fn encode(&self) -> Vec<u8> {
        // Minimal encoding - strip leading zeros
        let bytes = self.to_be_bytes_vec();
        let first_nonzero = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
        bytes[first_nonzero..].to_vec()
    }
}

impl<const BITS: usize, const LIMBS: usize> Decode for Uint<BITS, LIMBS> {
    fn decode(data: &[u8]) -> Result<Self, DecodeError> {
        Self::try_from_be_slice(data)
            .ok_or_else(|| DecodeError::Overflow)
    }
}
```

### Visitor Pattern for Deserialization

```rust
struct UintVisitor<const BITS: usize, const LIMBS: usize>;

impl<'de, const BITS: usize, const LIMBS: usize> Visitor<'de> for UintVisitor<BITS, LIMBS> {
    type Value = Uint<BITS, LIMBS>;
    
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a {}-bit unsigned integer", BITS)
    }
    
    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        value.parse()
            .map_err(|_| Error::custom(format!("invalid uint string: {}", value)))
    }
    
    fn visit_bytes<E: Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        Uint::try_from_be_slice(value)
            .ok_or_else(|| Error::custom("bytes too large for uint"))
    }
}
```

## Database Type Mapping

### PostgreSQL Implementation

```rust
#[cfg(feature = "postgres")]
impl<'a, const BITS: usize, const LIMBS: usize> FromSql<'a> for Uint<BITS, LIMBS> {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        // PostgreSQL NUMERIC format parsing
        let numeric = Numeric::from_sql(ty, raw)?;
        
        // Convert to Uint
        let bytes = numeric_to_be_bytes(&numeric)?;
        Self::try_from_be_slice(&bytes)
            .ok_or_else(|| "value too large for Uint".into())
    }
    
    fn accepts(ty: &Type) -> bool {
        matches!(*ty, Type::NUMERIC)
    }
}

impl<const BITS: usize, const LIMBS: usize> ToSql for Uint<BITS, LIMBS> {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        // Convert to PostgreSQL NUMERIC format
        let numeric = uint_to_numeric(self)?;
        numeric.to_sql(ty, out)
    }
    
    fn accepts(ty: &Type) -> bool {
        matches!(*ty, Type::NUMERIC)
    }
}
```

### Diesel ORM Integration

```rust
#[cfg(feature = "diesel")]
impl<const BITS: usize, const LIMBS: usize> FromSql<Numeric, Pg> for Uint<BITS, LIMBS> {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        // Use PostgreSQL implementation
        <Self as postgres_types::FromSql>::from_sql(&Type::NUMERIC, bytes.as_bytes())
            .map_err(|e| e.to_string().into())
    }
}

impl<const BITS: usize, const LIMBS: usize> ToSql<Numeric, Pg> for Uint<BITS, LIMBS> {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let mut bytes = BytesMut::new();
        <Self as postgres_types::ToSql>::to_sql(self, &Type::NUMERIC, &mut bytes)?;
        out.write_all(&bytes)?;
        Ok(IsNull::No)
    }
}
```

## zkVM Optimization Techniques

### Conditional Native Operations

```rust
impl<const BITS: usize, const LIMBS: usize> Add for Uint<BITS, LIMBS> {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        #[cfg(all(target_os = "zkvm", BITS == 256))]
        {
            let mut result = MaybeUninit::<Self>::uninit();
            unsafe {
                zkvm_u256_wrapping_add_impl(
                    result.as_mut_ptr() as *mut u8,
                    self.as_ptr() as *const u8,
                    rhs.as_ptr() as *const u8,
                );
                result.assume_init()
            }
        }
        
        #[cfg(not(all(target_os = "zkvm", BITS == 256)))]
        {
            // Fallback software implementation
            self.wrapping_add(rhs)
        }
    }
}
```

### Batch Operations for zkVM

```rust
#[cfg(target_os = "zkvm")]
pub fn batch_add_256(pairs: &[(U256, U256)]) -> Vec<U256> {
    pairs.iter().map(|(a, b)| {
        let mut result = MaybeUninit::<U256>::uninit();
        unsafe {
            zkvm_u256_wrapping_add_impl(
                result.as_mut_ptr() as *mut u8,
                a.as_ptr() as *const u8,
                b.as_ptr() as *const u8,
            );
            result.assume_init()
        }
    }).collect()
}
```

## Testing Strategies

### Property-Based Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_serialization_roundtrip(value: U256) {
            let serialized = value.to_my_format();
            let deserialized = U256::from_my_format(&serialized).unwrap();
            prop_assert_eq!(value, deserialized);
        }
        
        #[test]
        fn test_conversion_bounds(value: U256) {
            if let Ok(converted) = MyType::try_from(value) {
                let back: U256 = converted.into();
                prop_assert_eq!(value, back);
            }
        }
    }
}
```

### Edge Case Testing

```rust
#[test]
fn test_edge_cases() {
    // Test zero
    assert_eq!(U256::ZERO.to_my_format(), expected_zero_format());
    
    // Test max value
    assert_eq!(U256::MAX.to_my_format(), expected_max_format());
    
    // Test one
    assert_eq!(U256::ONE.to_my_format(), expected_one_format());
    
    // Test powers of two
    for i in 0..256 {
        let value = U256::ONE << i;
        let formatted = value.to_my_format();
        assert!(is_valid_format(&formatted));
    }
}
```

### Database Integration Testing

```rust
#[cfg(all(test, feature = "postgres"))]
mod postgres_tests {
    use postgres::{Client, NoTls};
    
    #[test]
    fn test_postgres_roundtrip() {
        let mut client = Client::connect("postgresql://localhost/test", NoTls).unwrap();
        
        // Create test table
        client.execute(
            "CREATE TEMP TABLE test (value NUMERIC)",
            &[]
        ).unwrap();
        
        // Test roundtrip
        let original = U256::from(12345u64);
        client.execute(
            "INSERT INTO test (value) VALUES ($1)",
            &[&original]
        ).unwrap();
        
        let row = client.query_one("SELECT value FROM test", &[]).unwrap();
        let retrieved: U256 = row.get(0);
        
        assert_eq!(original, retrieved);
    }
}
```

## Common Pitfalls and Solutions

### Pitfall 1: Endianness Confusion

**Problem**: Mixing big-endian and little-endian representations
```rust
// WRONG
let bytes = self.to_le_bytes_vec();
OtherType::from_be_bytes(&bytes) // Mixing endianness!
```

**Solution**: Be consistent with endianness
```rust
// CORRECT
let bytes = self.to_be_bytes_vec();
OtherType::from_be_bytes(&bytes)
```

### Pitfall 2: Overflow Handling

**Problem**: Not handling overflow in conversions
```rust
// WRONG
impl From<Uint<256, 4>> for u64 {
    fn from(value: Uint<256, 4>) -> u64 {
        value.limbs[0] // Ignores upper limbs!
    }
}
```

**Solution**: Use TryFrom with proper error handling
```rust
// CORRECT
impl TryFrom<Uint<256, 4>> for u64 {
    type Error = ConversionError;
    
    fn try_from(value: Uint<256, 4>) -> Result<u64, Self::Error> {
        if value > Uint::from(u64::MAX) {
            return Err(ConversionError::Overflow);
        }
        Ok(value.limbs[0])
    }
}
```

### Pitfall 3: Feature Flag Dependencies

**Problem**: Missing transitive dependencies
```rust
// WRONG in Cargo.toml
my-feature = ["dep:my-crate"]  // Missing required features
```

**Solution**: Include all required features
```rust
// CORRECT in Cargo.toml
my-feature = ["dep:my-crate", "alloc", "std"]  // Include all dependencies
```

### Pitfall 4: Zero-Width Type Handling

**Problem**: Not handling `Uint<0, 0>` edge case
```rust
// WRONG
fn serialize(&self) -> String {
    format!("{:x}", self) // Panics for Uint<0, 0>
}
```

**Solution**: Special case zero-width types
```rust
// CORRECT
fn serialize(&self) -> String {
    if BITS == 0 {
        return "0x0".to_string();
    }
    format!("{:#x}", self)
}
```

### Pitfall 5: Unconditional Recursion

**Problem**: Accidentally calling the same trait method
```rust
// WRONG
impl Add for Uint<BITS, LIMBS> {
    fn add(self, rhs: Self) -> Self {
        self.add(rhs) // Infinite recursion!
    }
}
```

**Solution**: Call the inherent method
```rust
// CORRECT
impl Add for Uint<BITS, LIMBS> {
    fn add(self, rhs: Self) -> Self {
        Uint::wrapping_add(self, rhs) // Call inherent method
    }
}
```

## Performance Optimization Tips

1. **Use `const` generics**: Leverage compile-time optimization
2. **Minimize allocations**: Prefer stack-allocated buffers
3. **Batch operations**: Group similar operations together
4. **Feature-gate heavy dependencies**: Keep base library lightweight
5. **Profile zkVM operations**: Ensure native ops are actually faster

## Adding Documentation

Always include:
1. Module-level documentation with crate link
2. Feature flag documentation
3. Examples in doc comments
4. Links to external crate documentation
5. Performance characteristics