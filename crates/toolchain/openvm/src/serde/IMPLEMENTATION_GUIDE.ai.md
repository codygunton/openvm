# OpenVM Serde Implementation Guide

## Overview

This guide provides detailed information for developers working with or extending the OpenVM serde component. The implementation focuses on efficient word-aligned serialization optimized for zkVM environments.

## Core Design Principles

### 1. Word Alignment
All data is aligned to 32-bit word boundaries:
- Simplifies zkVM memory operations
- Enables efficient proof generation
- Reduces complexity in guest programs

### 2. Zero-Copy When Possible
- Direct deserialization from aligned memory
- Minimal allocations during processing
- Efficient use of zkVM cycles

### 3. Serde Compatibility
- Full compatibility with serde derives
- Standard trait implementations
- Familiar API for Rust developers

## Implementation Architecture

### Serializer Structure

```rust
pub struct Serializer<W: WordWrite> {
    stream: W,
}
```

The serializer wraps any `WordWrite` implementation and handles:
- Type conversion to word format
- Padding for sub-word data
- Length prefix encoding
- Recursive serialization

### Deserializer Structure

```rust
pub struct Deserializer<'de, R: WordRead + 'de> {
    reader: R,
    phantom: core::marker::PhantomData<&'de ()>,
}
```

The deserializer manages:
- Sequential word reading
- Padding removal
- Type reconstruction
- Lifetime management

## Extending the Implementation

### Adding Custom WordWrite

```rust
struct CustomWriter {
    buffer: Vec<u32>,
    capacity: usize,
}

impl WordWrite for CustomWriter {
    fn write_words(&mut self, words: &[u32]) -> Result<()> {
        if self.buffer.len() + words.len() > self.capacity {
            return Err(Error::SerializeBufferFull);
        }
        self.buffer.extend_from_slice(words);
        Ok(())
    }
    
    fn write_padded_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        let chunks = bytes.chunks_exact(WORD_SIZE);
        let remainder = chunks.remainder();
        
        // Write complete words
        for chunk in chunks {
            let word = u32::from_le_bytes(chunk.try_into().unwrap());
            self.write_words(&[word])?;
        }
        
        // Handle remainder with padding
        if !remainder.is_empty() {
            let mut last_word = [0u8; WORD_SIZE];
            last_word[..remainder.len()].copy_from_slice(remainder);
            self.write_words(&[u32::from_le_bytes(last_word)])?;
        }
        
        Ok(())
    }
}
```

### Adding Custom WordRead

```rust
struct CustomReader {
    data: Vec<u32>,
    position: usize,
}

impl WordRead for CustomReader {
    fn read_words(&mut self, out: &mut [u32]) -> Result<()> {
        let available = self.data.len() - self.position;
        if out.len() > available {
            return Err(Error::DeserializeUnexpectedEnd);
        }
        
        let end = self.position + out.len();
        out.copy_from_slice(&self.data[self.position..end]);
        self.position = end;
        Ok(())
    }
    
    fn read_padded_bytes(&mut self, out: &mut [u8]) -> Result<()> {
        let word_count = align_up(out.len(), WORD_SIZE) / WORD_SIZE;
        let mut words = vec![0u32; word_count];
        self.read_words(&mut words)?;
        
        let bytes: &[u8] = bytemuck::cast_slice(&words);
        out.copy_from_slice(&bytes[..out.len()]);
        Ok(())
    }
}
```

## Serialization Format Details

### Primitive Types

| Type | Encoding |
|------|----------|
| bool | 0u32 or 1u32 |
| i8/u8 | Zero-extended to u32 |
| i16/u16 | Zero-extended to u32 |
| i32/u32 | Direct u32 |
| i64/u64 | Two u32 (low, high) |
| i128/u128 | Four u32 (little-endian) |
| f32 | IEEE 754 bits as u32 |
| f64 | IEEE 754 bits as two u32 |
| char | Unicode scalar as u32 |

### Variable-Length Types

**Strings**:
```
[length: u32][utf8_bytes_padded_to_word_boundary]
```

**Vectors**:
```
[length: u32][element_0][element_1]...[element_n-1]
```

**Options**:
```
None: [0u32]
Some: [1u32][value]
```

**Enums**:
```
[variant_index: u32][variant_data]
```

## Performance Optimization

### Memory Pre-allocation
```rust
pub fn to_vec_with_capacity<T>(value: &T, cap: usize) -> Result<Vec<u32>>
where
    T: serde::Serialize + ?Sized,
{
    let mut vec: Vec<u32> = Vec::with_capacity(cap);
    let mut serializer = Serializer::new(&mut vec);
    value.serialize(&mut serializer)?;
    Ok(vec)
}
```

### Batch Operations
When serializing multiple values:
```rust
let mut buffer = Vec::with_capacity(total_size);
let mut serializer = Serializer::new(&mut buffer);

for item in items {
    item.serialize(&mut serializer)?;
}
```

### Zero-Copy Patterns
```rust
// Direct slice casting for aligned data
let words: &[u32] = bytemuck::cast_slice(&aligned_bytes);
let result: T = from_slice(words)?;
```

## Error Handling Best Practices

### Custom Errors
```rust
impl MySerializer {
    fn serialize_special(&mut self, value: &Special) -> Result<()> {
        if !value.is_valid() {
            return Err(Error::Custom("Invalid special value".into()));
        }
        // ... serialization logic
    }
}
```

### Error Recovery
```rust
fn safe_deserialize<T: DeserializeOwned>(data: &[u32]) -> Option<T> {
    match from_slice(data) {
        Ok(value) => Some(value),
        Err(Error::DeserializeUnexpectedEnd) => {
            // Try with partial data
            None
        }
        Err(_) => None,
    }
}
```

## Testing Strategies

### Property-Based Testing
```rust
#[test]
fn prop_roundtrip<T: Serialize + DeserializeOwned + PartialEq>(value: T) {
    let serialized = to_vec(&value).unwrap();
    let deserialized: T = from_slice(&serialized).unwrap();
    assert_eq!(value, deserialized);
}
```

### Boundary Testing
```rust
#[test]
fn test_max_string_length() {
    let s = "x".repeat(u32::MAX as usize - 1);
    let result = to_vec(&s);
    assert!(result.is_ok());
}
```

### Compatibility Testing
```rust
#[test]
fn test_std_collections() {
    use std::collections::{HashMap, BTreeMap, HashSet};
    
    let mut map = HashMap::new();
    map.insert("key", "value");
    
    let words = to_vec(&map).unwrap();
    let restored: HashMap<&str, &str> = from_slice(&words).unwrap();
    assert_eq!(map, restored);
}
```

## Common Pitfalls

### 1. Forgetting Padding
Always ensure byte data is properly padded to word boundaries.

### 2. Incorrect Length Handling
Verify that length prefixes match actual data length.

### 3. Endianness Issues
The format uses little-endian throughout - ensure consistency.

### 4. Lifetime Complications
Be careful with borrowed data in custom deserializers.

## Integration Examples

### With OpenVM Guest Programs
```rust
#[no_mangle]
pub extern "C" fn main() {
    let input: Input = openvm::io::read();
    let result = process(input);
    let output = to_vec(&result).unwrap();
    openvm::io::commit(&output);
}
```

### With Proof Generation
```rust
let public_values = to_vec(&computation_result)?;
let proof = prover.prove(program, &public_values)?;
```

### With State Persistence
```rust
impl StateStore {
    fn save<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
        let words = to_vec(value)?;
        self.backend.write(key, &words)?;
        Ok(())
    }
    
    fn load<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let words = self.backend.read(key)?;
        from_slice(&words)
    }
}
```