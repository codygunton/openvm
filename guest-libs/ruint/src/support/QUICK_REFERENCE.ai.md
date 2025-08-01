# Ruint Support Module Quick Reference

## Feature Flags

```toml
# Serialization
serde = ["dep:serde", "alloc"]
borsh = ["dep:borsh"]
ssz = ["std", "dep:ethereum_ssz"]
scale = ["dep:parity-scale-codec", "alloc"]
rlp = ["dep:rlp", "alloc"]
alloy-rlp = ["dep:alloy-rlp", "alloc"]

# Numeric
num-traits = ["dep:num-traits", "alloc"]
num-bigint = ["dep:num-bigint", "alloc"]
primitive-types = ["dep:primitive-types"]

# Database
postgres = ["dep:postgres-types", "dep:bytes", "dep:thiserror", "std"]
diesel = ["dep:diesel", "std", "dep:thiserror"]
sqlx = ["dep:sqlx-core", "std", "dep:thiserror"]

# Testing/Random
rand = ["dep:rand-08"]
proptest = ["dep:proptest", "alloc"]
arbitrary = ["dep:arbitrary", "std"]

# Security
subtle = ["dep:subtle"]
zeroize = ["dep:zeroize"]
```

## Common Conversions

### From Primitives
```rust
use ruint::aliases::U256;

// From unsigned integers
let a = U256::from(42u64);
let b = U256::from(u128::MAX);

// From signed integers (with TryFrom)
let c = U256::try_from(42i64)?;
let d = U256::try_from(-1i64); // Error: negative
```

### To Primitives
```rust
// To unsigned integers
let value = U256::from(42u64);
let as_u64: u64 = value.try_into()?;
let as_u128: u128 = value.try_into()?;

// Saturating conversions
let large = U256::MAX;
let saturated: u64 = large.saturating_to::<u64>(); // u64::MAX
```

### String Parsing
```rust
// Decimal
let dec = U256::from_str("12345")?;

// Hexadecimal (with or without 0x)
let hex1 = U256::from_str("0x1234")?;
let hex2 = U256::from_str("1234")?; // Assumes hex without 0x

// Binary
let bin = U256::from_str("0b1010")?;

// Octal
let oct = U256::from_str("0o777")?;

// With radix
let custom = U256::from_str_radix("ZZZ", 36)?;
```

## Serialization Snippets

### Serde JSON
```rust
#[cfg(feature = "serde")]
{
    use serde::{Serialize, Deserialize};
    
    #[derive(Serialize, Deserialize)]
    struct MyStruct {
        amount: U256,
    }
    
    let data = MyStruct { amount: U256::from(100u64) };
    
    // Serialize to JSON (hex string)
    let json = serde_json::to_string(&data)?;
    // {"amount":"0x64"}
    
    // Deserialize from JSON
    let parsed: MyStruct = serde_json::from_str(&json)?;
}
```

### Binary Serialization
```rust
#[cfg(feature = "borsh")]
{
    use borsh::{BorshSerialize, BorshDeserialize};
    
    let value = U256::from(12345u64);
    
    // Serialize
    let bytes = value.try_to_vec()?;
    
    // Deserialize
    let restored = U256::try_from_slice(&bytes)?;
}
```

### RLP Encoding
```rust
#[cfg(feature = "rlp")]
{
    use rlp::{Encodable, Decodable};
    
    let value = U256::from(42u64);
    
    // Encode
    let encoded = rlp::encode(&value);
    
    // Decode
    let decoded: U256 = rlp::decode(&encoded)?;
}
```

## Database Operations

### PostgreSQL
```rust
#[cfg(feature = "postgres")]
{
    use postgres::{Client, NoTls};
    
    let mut client = Client::connect("postgresql://localhost/mydb", NoTls)?;
    
    // Insert
    let value = U256::from(12345u64);
    client.execute(
        "INSERT INTO balances (amount) VALUES ($1)",
        &[&value],
    )?;
    
    // Query
    let row = client.query_one("SELECT amount FROM balances", &[])?;
    let amount: U256 = row.get(0);
}
```

### Diesel ORM
```rust
#[cfg(feature = "diesel")]
{
    use diesel::prelude::*;
    
    #[derive(Queryable)]
    struct Balance {
        id: i32,
        amount: U256,
    }
    
    let results = balances
        .filter(amount.gt(U256::from(1000u64)))
        .load::<Balance>(&mut conn)?;
}
```

## Random Generation

### Basic Random
```rust
#[cfg(feature = "rand")]
{
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    
    // Random U256
    let random: U256 = rng.gen();
    
    // Random in range
    let range = U256::ZERO..U256::from(1000u64);
    let in_range: U256 = rng.gen_range(range);
}
```

### Property Testing
```rust
#[cfg(all(test, feature = "proptest"))]
{
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_addition_commutative(a: U256, b: U256) {
            assert_eq!(a + b, b + a);
        }
    }
}
```

## zkVM Optimizations

### Conditional Compilation
```rust
// Automatically uses hardware acceleration on zkVM
let sum = a + b;

// Manual optimization check
#[cfg(target_os = "zkvm")]
{
    // zkVM-specific code
}
#[cfg(not(target_os = "zkvm"))]
{
    // Fallback implementation
}
```

### Direct zkVM Calls
```rust
#[cfg(target_os = "zkvm")]
use openvm_bigint_guest::externs::*;

#[cfg(target_os = "zkvm")]
unsafe {
    let mut result = [0u8; 32];
    zkvm_u256_wrapping_add_impl(
        result.as_mut_ptr(),
        a.to_le_bytes().as_ptr(),
        b.to_le_bytes().as_ptr(),
    );
    let sum = U256::from_le_bytes(result);
}
```

## Numeric Traits

### Basic Traits
```rust
#[cfg(feature = "num-traits")]
{
    use num_traits::{Zero, One, Num};
    
    // Identity elements
    let zero = U256::zero();
    let one = U256::one();
    
    // From string with radix
    let hex = U256::from_str_radix("FF", 16)?;
}
```

### Checked Operations
```rust
#[cfg(feature = "num-traits")]
{
    use num_traits::CheckedAdd;
    
    let a = U256::MAX - U256::ONE;
    let b = U256::from(2u64);
    
    // Checked addition
    match a.checked_add(&b) {
        Some(sum) => println!("Sum: {}", sum),
        None => println!("Overflow!"),
    }
}
```

## Error Handling Patterns

### Conversion Errors
```rust
use ruint::{FromUintError, ToUintError};

// Handle too large values
match u64::try_from(U256::MAX) {
    Ok(val) => println!("Value: {}", val),
    Err(FromUintError::Overflow) => println!("Too large for u64"),
}

// Handle negative values
match U256::try_from(-1i64) {
    Ok(val) => println!("Value: {}", val),
    Err(ToUintError::ValueNegative) => println!("Cannot convert negative"),
}
```

### Parse Errors
```rust
use ruint::ParseError;

match U256::from_str("invalid") {
    Ok(val) => println!("Parsed: {}", val),
    Err(ParseError::InvalidDigit) => println!("Invalid digit"),
    Err(ParseError::Overflow) => println!("Value too large"),
    Err(e) => println!("Other error: {}", e),
}
```

## Type Aliases

```rust
use ruint::aliases::*;

// Common sizes
type U8 = Uint<8, 1>;
type U16 = Uint<16, 1>;
type U32 = Uint<32, 1>;
type U64 = Uint<64, 1>;
type U128 = Uint<128, 2>;
type U256 = Uint<256, 4>;
type U512 = Uint<512, 8>;

// Custom sizes
type U160 = Uint<160, 3>; // Ethereum address
type U2048 = Uint<2048, 32>; // RSA-2048
```

## Performance Tips

### Avoid Allocations
```rust
// GOOD: Stack allocated
let bytes: [u8; 32] = value.to_le_bytes();

// AVOID: Heap allocated
let bytes: Vec<u8> = value.to_le_bytes_vec();
```

### Batch Operations
```rust
// GOOD: Single allocation
let sums: Vec<U256> = pairs
    .iter()
    .map(|(a, b)| a + b)
    .collect();

// AVOID: Multiple allocations
let mut sums = Vec::new();
for (a, b) in pairs {
    sums.push(a + b);
}
```

### Use Appropriate Methods
```rust
// GOOD: Direct comparison
if value == U256::ZERO { }

// AVOID: Unnecessary conversion
if value == U256::from(0u64) { }

// GOOD: Bit operations
let shifted = value << 8;

// AVOID: Multiplication for shifting
let shifted = value * U256::from(256u64);
```

## Common Patterns

### Safe Arithmetic
```rust
// Wrapping (default)
let wrapped = a.wrapping_add(b);

// Checked
let checked = a.checked_add(b)?;

// Saturating
let saturated = a.saturating_add(b);

// Overflowing
let (result, overflow) = a.overflowing_add(b);
```

### Bit Manipulation
```rust
// Set bit
let set = value | (U256::ONE << 42);

// Clear bit
let cleared = value & !(U256::ONE << 42);

// Toggle bit
let toggled = value ^ (U256::ONE << 42);

// Test bit
let is_set = (value & (U256::ONE << 42)) != U256::ZERO;
```

### Endianness Conversion
```rust
// To bytes
let be_bytes = value.to_be_bytes_vec();
let le_bytes = value.to_le_bytes_vec();

// From bytes
let from_be = U256::from_be_bytes(be_bytes);
let from_le = U256::from_le_bytes(le_bytes);

// From slice
let from_slice = U256::try_from_be_slice(&bytes)?;
```