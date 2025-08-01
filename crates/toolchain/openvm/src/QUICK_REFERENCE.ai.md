# OpenVM Toolchain Quick Reference

## Essential Imports
```rust
use openvm::{entry, io::{read, reveal_bytes32}, println};
use openvm::serde::{to_vec, from_slice};
```

## Entry Point Setup
```rust
#![no_main]
#![no_std]

openvm::entry!(main);

fn main() {
    // Your code here
}
```

## Reading Input

### Read Typed Data
```rust
let value: u32 = read();
let data: Vec<String> = read();
let complex: MyStruct = read();
```

### Read Raw Bytes
```rust
let bytes: Vec<u8> = read_vec();
let word: u32 = read_u32();
```

### Load Hints by Key
```rust
hint_load_by_key(b"my_key");
let data: MyType = read();
```

## Writing Output

### Reveal 32-byte Hash (Recommended)
```rust
let hash = sha256(data);
reveal_bytes32(hash);
```

### Reveal Individual Words
```rust
reveal_u32(value, index);  // index = byte_index / 4
```

### Debug Output
```rust
println!("Debug: {}", value);
print!("No newline");
```

## Serialization

### Serialize to Words
```rust
let words: Vec<u32> = to_vec(&my_data)?;
let words_with_cap = to_vec_with_capacity(&my_data, 1024)?;
```

### Deserialize from Words
```rust
let data: MyType = from_slice(&words)?;
```

## Memory Operations

### Memory Barrier
```rust
memory_barrier(&value);  // Prevent reordering
```

### Native Memory Store
```rust
store_u32_to_native(native_addr, value);
```

## Process Control

### Exit Successfully
```rust
process::exit();  // Exit code 0
```

### Panic/Abort
```rust
process::panic();  // Exit code 1
panic!("Error message");  // With message
```

## Platform Access

### Platform Constants
```rust
use openvm::platform::memory::STACK_TOP;
use openvm::platform::WORD_SIZE;
```

### Custom Instructions (zkVM only)
```rust
#[cfg(target_os = "zkvm")]
use openvm_rv32im_guest::*;
```

## Feature Flags

### Cargo.toml
```toml
[dependencies]
openvm = { version = "0.1", features = ["std"] }

# Features:
# default = ["getrandom-unsupported"]
# std - Enable std support
# heap-embedded-alloc - Use linked-list allocator
```

## Common Patterns

### Basic Guest Program
```rust
#![no_main]
#![no_std]

use openvm::{entry, io::{read, reveal_bytes32}};

entry!(main);

fn main() {
    let input: u32 = read();
    let result = input * 2;
    
    let mut output = [0u8; 32];
    output[0..4].copy_from_slice(&result.to_le_bytes());
    reveal_bytes32(output);
}
```

### With Error Handling
```rust
use openvm::{entry, io::read};

entry!(main);

fn main() {
    match process_input() {
        Ok(result) => reveal_result(result),
        Err(e) => panic!("Error: {:?}", e),
    }
}

fn process_input() -> Result<u32, Error> {
    let data: InputData = read();
    validate(&data)?;
    Ok(compute(data))
}
```

### Streaming Processing
```rust
entry!(main);

fn main() {
    let count: u32 = read();
    let mut accumulator = 0u32;
    
    for _ in 0..count {
        let value: u32 = read();
        accumulator = accumulator.wrapping_add(value);
    }
    
    reveal_u32(accumulator, 0);
}
```

## Target-Specific Code

### Conditional Compilation
```rust
#[cfg(target_os = "zkvm")]
fn zkvm_only() {
    // Code that only runs in zkVM
}

#[cfg(not(target_os = "zkvm"))]
fn host_only() {
    // Code for testing/development
}
```

### Using Host Utilities
```rust
#[cfg(not(target_os = "zkvm"))]
use openvm::host::{read_n_bytes, read_u32};
```

## Word Alignment Rules

### Reading Aligned Data
```rust
// Bytes are padded to word boundaries
let len = bytes.len();
let word_count = len.div_ceil(4);
let padded_len = word_count * 4;
```

### Serialization Alignment
```rust
// All types serialize to word boundaries
// u8, u16 → u32
// u64 → 2 × u32
// bytes → padded to 4-byte boundary
```

## Performance Tips

1. **Minimize Allocations**
   ```rust
   let mut buffer = Vec::with_capacity(1024);
   buffer.clear();  // Reuse instead of reallocating
   ```

2. **Use Word Operations**
   ```rust
   // Process 4 bytes at once
   for chunk in bytes.chunks_exact(4) {
       let word = u32::from_le_bytes(chunk.try_into().unwrap());
   }
   ```

3. **Avoid Collect**
   ```rust
   // Don't: data.iter().map(f).collect()
   // Do: data.iter().map(f).for_each(process)
   ```

## Common Errors

### Word Alignment Error
```rust
// Error: Unaligned address
hint_store_u32!(addr);  // addr must be divisible by 4
```

### Feature Mismatch
```rust
// Error: std feature not enabled
use std::collections::HashMap;  // Need features = ["std"]
```

### Missing Entry Point
```rust
// Error: undefined reference to `main`
// Fix: Add openvm::entry!(main);
```

## Testing

### Mock Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logic() {
        // Test pure logic without I/O
        let result = process_data(42);
        assert_eq!(result, 84);
    }
}
```

### Integration Testing
```rust
// In tests/ directory
#[test]
fn test_with_mock_io() {
    // Inject test hints
    // Run program logic
    // Verify outputs
}
```