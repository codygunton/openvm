# OpenVM Standard Library Examples

## Basic Guest Program Setup

### No-std Guest Program
```rust
#![no_main]
#![no_std]

use openvm;

openvm::entry!(main);

fn main() {
    openvm::io::println!("Hello, OpenVM!");
}
```

### Guest Program with Standard Library
```rust
use openvm;

fn main() {
    openvm::io::println!("Hello from std environment!");
}
```

## Input/Output Operations

### Reading Typed Data
```rust
use openvm::io::{read, reveal_bytes32};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct InputData {
    value: u32,
    flag: bool,
    data: Vec<u8>,
}

fn main() {
    // Read structured data from hint stream
    let input: InputData = read();
    
    // Process the data
    let result = process_data(input);
    
    // Reveal 32-byte hash of result
    reveal_bytes32(result);
}

fn process_data(input: InputData) -> [u8; 32] {
    // Processing logic here
    [0u8; 32] // placeholder
}
```

### Reading Raw Data
```rust
use openvm::io::{read_vec, read_u32};

fn main() {
    // Read a u32 directly
    let count = read_u32();
    
    // Read a variable-length byte vector
    let data = read_vec();
    
    println!("Read {} items with {} bytes", count, data.len());
}
```

### Reading Multiple Values
```rust
use openvm::io::read;

fn main() {
    // Read multiple typed values in sequence
    let x: u32 = read();
    let y: u64 = read();
    let name: String = read();
    let values: Vec<i32> = read();
    
    println!("x={}, y={}, name={}, values={:?}", x, y, name, values);
}
```

## Output Operations

### Publishing Hash Outputs
```rust
use openvm::io::reveal_bytes32;
use sha2::{Sha256, Digest};

fn main() {
    let input_data = b"important computation result";
    
    // Create hash of the result
    let mut hasher = Sha256::new();
    hasher.update(input_data);
    let hash: [u8; 32] = hasher.finalize().into();
    
    // Reveal the hash as public output
    reveal_bytes32(hash);
}
```

### Publishing Individual Values
```rust
use openvm::io::reveal_u32;

fn main() {
    let results = [42u32, 100, 255, 1000];
    
    // Reveal each result at a specific index
    for (i, &value) in results.iter().enumerate() {
        reveal_u32(value, i);
    }
}
```

## Memory Operations

### Using Memory Barriers
```rust
use openvm::memory_barrier;

fn main() {
    let data = vec![1, 2, 3, 4, 5];
    let ptr = data.as_ptr();
    
    // Ensure memory accesses before this point aren't reordered
    memory_barrier(ptr);
    
    // Critical section that depends on previous memory state
    let sum: u32 = data.iter().sum();
    reveal_u32(sum, 0);
}
```

## Serialization Examples

### Custom Serializable Types
```rust
use openvm::io::{read, reveal_bytes32};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Serialize, Deserialize)]
struct Circle {
    center: Point,
    radius: f64,
}

fn main() {
    let circle: Circle = read();
    
    // Calculate area
    let area = std::f64::consts::PI * circle.radius * circle.radius;
    
    // Convert to bytes and hash
    let area_bytes = area.to_le_bytes();
    let mut result = [0u8; 32];
    result[..8].copy_from_slice(&area_bytes);
    
    reveal_bytes32(result);
}
```

### Working with Complex Data Structures
```rust
use openvm::io::read;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
    timestamp: u64,
}

#[derive(Serialize, Deserialize)]
struct BlockData {
    transactions: Vec<Transaction>,
    metadata: HashMap<String, String>,
}

fn main() {
    let block: BlockData = read();
    
    // Process transactions
    let total_amount: u64 = block.transactions
        .iter()
        .map(|tx| tx.amount)
        .sum();
    
    println!("Block contains {} transactions totaling {}", 
             block.transactions.len(), total_amount);
}
```

## Error Handling

### Panic Handling in No-std
```rust
#![no_main]
#![no_std]

use openvm;

openvm::entry!(main);

fn main() {
    let input: Result<u32, &str> = Ok(42);
    
    match input {
        Ok(value) => {
            println!("Success: {}", value);
        }
        Err(msg) => {
            // This will trigger the panic handler
            panic!("Error: {}", msg);
        }
    }
}
```

### Graceful Error Handling
```rust
use openvm::io::{read, reveal_u32};

fn main() {
    // Read input that might be invalid
    let input: u32 = read();
    
    match validate_input(input) {
        Ok(processed) => {
            reveal_u32(processed, 0);
        }
        Err(_) => {
            // Reveal error code
            reveal_u32(0xFFFFFFFF, 0);
        }
    }
}

fn validate_input(input: u32) -> Result<u32, &'static str> {
    if input > 1000 {
        Err("Input too large")
    } else {
        Ok(input * 2)
    }
}
```

## Advanced Usage Patterns

### Reading Hint Data by Key
```rust
use openvm::io::{hint_load_by_key, read_vec};

fn main() {
    // Load specific data by key from hint store
    let key = b"user_data";
    hint_load_by_key(key);
    
    // Now read the loaded data
    let user_data = read_vec();
    
    println!("Loaded {} bytes of user data", user_data.len());
}
```

### Conditional Compilation for Testing
```rust
use openvm::io::{read, reveal_bytes32};

fn main() {
    let input: u32 = read();
    
    #[cfg(target_os = "zkvm")]
    let processed = zkvm_specific_processing(input);
    
    #[cfg(not(target_os = "zkvm"))]
    let processed = host_mock_processing(input);
    
    reveal_bytes32(processed);
}

#[cfg(target_os = "zkvm")]
fn zkvm_specific_processing(input: u32) -> [u8; 32] {
    // Use zkVM-specific optimizations
    [input as u8; 32]
}

#[cfg(not(target_os = "zkvm"))]
fn host_mock_processing(input: u32) -> [u8; 32] {
    // Host-side implementation for testing
    [input as u8; 32]
}
```

### Initialization Macro Usage
```rust
#![no_main]
#![no_std]

use openvm;

// Include generated initialization code
openvm::init!();

openvm::entry!(main);

fn main() {
    println!("Program initialized and running");
}
```

## Performance Optimization Examples

### Efficient Batch Processing
```rust
use openvm::io::{read_u32, reveal_u32};

fn main() {
    let count = read_u32();
    
    // Process in batches for better performance
    const BATCH_SIZE: usize = 1000;
    let mut total = 0u64;
    
    for batch in 0..(count as usize).div_ceil(BATCH_SIZE) {
        let batch_size = std::cmp::min(BATCH_SIZE, 
                                      count as usize - batch * BATCH_SIZE);
        
        let mut batch_sum = 0u64;
        for _ in 0..batch_size {
            batch_sum += read_u32() as u64;
        }
        
        total += batch_sum;
    }
    
    reveal_u32(total as u32, 0);
    reveal_u32((total >> 32) as u32, 1);
}
```

### Word-Aligned Data Processing
```rust
use openvm::io::{read_vec, reveal_bytes32};

fn main() {
    let data = read_vec();
    
    // Process in 4-byte chunks for optimal performance
    let mut result = [0u8; 32];
    let mut hasher_state = 0u32;
    
    for chunk in data.chunks_exact(4) {
        let word = u32::from_le_bytes(chunk.try_into().unwrap());
        hasher_state = hasher_state.wrapping_add(word);
    }
    
    // Handle remaining bytes
    let remainder = data.chunks_exact(4).remainder();
    if !remainder.is_empty() {
        let mut padded = [0u8; 4];
        padded[..remainder.len()].copy_from_slice(remainder);
        let word = u32::from_le_bytes(padded);
        hasher_state = hasher_state.wrapping_add(word);
    }
    
    result[..4].copy_from_slice(&hasher_state.to_le_bytes());
    reveal_bytes32(result);
}
```