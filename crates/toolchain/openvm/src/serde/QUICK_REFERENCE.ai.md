# OpenVM Serde Quick Reference

## Import
```rust
use openvm_toolchain::serde::{to_vec, from_slice, Error, Result};
```

## Basic Serialization
```rust
// Serialize any type implementing serde::Serialize
let data = MyStruct { field: 42 };
let words: Vec<u32> = to_vec(&data)?;

// With capacity hint for better performance
let words = to_vec_with_capacity(&data, 100)?;
```

## Basic Deserialization
```rust
// Deserialize from word slice
let data: MyStruct = from_slice(&words)?;

// Works with any Pod type (not just u32)
let bytes: Vec<u8> = vec![1, 2, 3, 4];
let data: MyStruct = from_slice(&bytes)?;
```

## Custom Word Writers
```rust
impl WordWrite for MyWriter {
    fn write_words(&mut self, words: &[u32]) -> Result<()> {
        // Write words to your destination
    }
    
    fn write_padded_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        // Write bytes with padding to word boundary
    }
}

// Use with Serializer
let mut writer = MyWriter::new();
let mut serializer = Serializer::new(&mut writer);
value.serialize(&mut serializer)?;
```

## Custom Word Readers
```rust
impl WordRead for MyReader {
    fn read_words(&mut self, words: &mut [u32]) -> Result<()> {
        // Read words from your source
    }
    
    fn read_padded_bytes(&mut self, bytes: &mut [u8]) -> Result<()> {
        // Read bytes, discarding padding
    }
}

// Use with Deserializer
let mut reader = MyReader::new();
let mut deserializer = Deserializer::new(&mut reader);
let value = MyType::deserialize(&mut deserializer)?;
```

## Error Handling
```rust
use openvm_toolchain::serde::Error;

match to_vec(&data) {
    Ok(words) => // Success
    Err(Error::NotSupported) => // Feature not supported
    Err(Error::SerializeBufferFull) => // Buffer full
    Err(Error::Custom(msg)) => // Custom error
    Err(e) => // Other errors
}
```

## Supported Types

### Primitives
- All integers (i8-i128, u8-u128)
- Floats (f32, f64)
- bool, char

### Collections
- Vec, String
- BTreeMap, HashMap (with known size)
- Arrays, tuples

### Complex Types
- Structs (via derive)
- Enums (unit, tuple, struct variants)
- Option<T>

## Serialization Format

### Word Alignment
- All data aligned to 32-bit boundaries
- Bytes padded with zeros to next word
- Strings: length (u32) + padded UTF-8 bytes

### Type Encoding
```rust
// Option<T>
None => [0u32]
Some(value) => [1u32, ...serialized_value]

// Enum
Variant => [variant_index_u32, ...variant_data]

// Vec/String
[length_u32, ...elements]
```

## Common Patterns

### Round-trip Testing
```rust
let original = create_test_data();
let serialized = to_vec(&original)?;
let deserialized: TestType = from_slice(&serialized)?;
assert_eq!(original, deserialized);
```

### Guest-Host Communication
```rust
// In guest
let result = compute_result();
let words = to_vec(&result)?;
env::commit(&words);

// In host
let words = read_output();
let result: ResultType = from_slice(&words)?;
```

### Efficient Serialization
```rust
// Estimate size for complex structures
let capacity = std::mem::size_of::<MyStruct>() / 4;
let words = to_vec_with_capacity(&data, capacity)?;
```

## Performance Tips
1. Use `to_vec_with_capacity` for large structures
2. Reuse `Vec<u32>` buffers as `WordWrite` implementations
3. Prefer stack-allocated types when possible
4. Batch serialization operations

## Limitations
- No support for `deserialize_any`
- Sequences must have known length
- Maps must have known size
- No borrowed string deserialization