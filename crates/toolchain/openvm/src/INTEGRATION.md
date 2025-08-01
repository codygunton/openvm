# OpenVM Standard Library Integration Guide

## Project Integration

### Adding OpenVM Standard Library to Your Project

#### Cargo.toml Configuration
```toml
[dependencies]
openvm = { version = "1.3.0" }

# For zkVM target builds
[target.'cfg(target_os = "zkvm")'.dependencies]
openvm = { version = "1.3.0", default-features = false }

# Optional features
[features]
default = []
std = ["openvm/std"]
heap-embedded-alloc = ["openvm/heap-embedded-alloc"]
```

#### Basic Integration Pattern
```rust
// For no-std zkVM guests
#![no_main]
#![no_std]

use openvm;

openvm::entry!(main);

fn main() {
    // Your guest program logic
}
```

```rust
// For std environments
use openvm;

fn main() {
    // Your program logic with std support
}
```

### Build Configuration

#### Cross-compilation Setup
```bash
# Add zkVM target
rustup target add riscv32im-risc0-zkvm-elf

# Build for zkVM
cargo build --target riscv32im-risc0-zkvm-elf --release
```

#### Feature Configuration
```toml
# Minimal no-std configuration
[dependencies.openvm]
version = "1.3.0"
default-features = false

# With standard library support
[dependencies.openvm]
version = "1.3.0"
features = ["std"]

# With alternative heap allocator
[dependencies.openvm]
version = "1.3.0"
features = ["heap-embedded-alloc"]
```

## API Integration Patterns

### Input/Output Integration

#### Structured Data Processing Pipeline
```rust
use openvm::io::{read, reveal_bytes32};
use serde::{Deserialize, Serialize};

// Define your data structures
#[derive(Serialize, Deserialize)]
struct ProcessingRequest {
    data: Vec<u8>,
    parameters: ProcessingParams,
}

#[derive(Serialize, Deserialize)]
struct ProcessingParams {
    algorithm: String,
    iterations: u32,
}

// Integration function
pub fn process_request() -> [u8; 32] {
    let request: ProcessingRequest = read();
    let result = execute_algorithm(&request);
    
    // Hash and reveal result
    let hash = compute_hash(&result);
    reveal_bytes32(hash);
    hash
}
```

#### Multi-stage Data Processing
```rust
use openvm::io::{read, read_u32, reveal_u32};

pub struct DataProcessor {
    stage: u32,
    accumulated: Vec<u8>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            stage: 0,
            accumulated: Vec::new(),
        }
    }
    
    pub fn process_stage(&mut self) -> bool {
        let stage_data_len = read_u32();
        if stage_data_len == 0 {
            return false; // End of processing
        }
        
        let mut stage_data = Vec::with_capacity(stage_data_len as usize);
        for _ in 0..stage_data_len {
            stage_data.push(read_u32() as u8);
        }
        
        self.accumulated.extend(stage_data);
        self.stage += 1;
        true
    }
    
    pub fn finalize(&self) -> u32 {
        let result = self.accumulated.len() as u32 + self.stage;
        reveal_u32(result, 0);
        result
    }
}
```

### Memory Management Integration

#### Custom Allocator Integration
```rust
#[cfg(feature = "heap-embedded-alloc")]
use openvm::platform::heap::embedded;

pub fn initialize_memory_system() {
    #[cfg(all(target_os = "zkvm", feature = "heap-embedded-alloc"))]
    {
        embedded::init();
        println!("Initialized embedded allocator");
    }
}

// Memory-efficient data structure
pub struct CircularBuffer<T> {
    data: Vec<T>,
    head: usize,
    tail: usize,
    capacity: usize,
}

impl<T: Default + Clone> CircularBuffer<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: vec![T::default(); capacity],
            head: 0,
            tail: 0,
            capacity,
        }
    }
    
    pub fn push(&mut self, item: T) -> bool {
        if self.is_full() {
            return false;
        }
        
        self.data[self.tail] = item;
        self.tail = (self.tail + 1) % self.capacity;
        true
    }
    
    fn is_full(&self) -> bool {
        (self.tail + 1) % self.capacity == self.head
    }
}
```

### Error Handling Integration

#### Robust Error Management
```rust
use openvm::io::{read, reveal_u32};

#[derive(Debug)]
pub enum ProcessingError {
    InvalidInput,
    ComputationOverflow,
    InsufficientMemory,
}

pub type ProcessingResult<T> = Result<T, ProcessingError>;

pub fn safe_processing_pipeline() -> u32 {
    match execute_safe_processing() {
        Ok(result) => {
            reveal_u32(result, 0);
            result
        }
        Err(error) => {
            let error_code = match error {
                ProcessingError::InvalidInput => 0xE001,
                ProcessingError::ComputationOverflow => 0xE002,
                ProcessingError::InsufficientMemory => 0xE003,
            };
            reveal_u32(error_code, 0);
            error_code
        }
    }
}

fn execute_safe_processing() -> ProcessingResult<u32> {
    let input: u32 = read();
    
    if input > 1_000_000 {
        return Err(ProcessingError::InvalidInput);
    }
    
    let result = input.checked_mul(42)
        .ok_or(ProcessingError::ComputationOverflow)?;
    
    Ok(result)
}
```

## Testing Integration

### Host-side Testing Setup
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_processing_logic() {
        // Host-side tests automatically use mock implementations
        let result = process_test_data();
        assert_eq!(result, expected_value());
    }
    
    #[cfg(not(target_os = "zkvm"))]
    fn process_test_data() -> u32 {
        // Mock implementation for testing
        42
    }
    
    fn expected_value() -> u32 {
        42
    }
}
```

### Conditional Test Compilation
```rust
#[cfg(test)]
mod integration_tests {
    use openvm::io::{read, reveal_u32};
    
    #[test]
    #[cfg(not(target_os = "zkvm"))]
    fn host_side_integration_test() {
        // Test host-side mock implementations
        test_io_operations();
    }
    
    fn test_io_operations() {
        // This will use host::read_u32() instead of zkVM hints
        let value = openvm::io::read_u32();
        openvm::io::reveal_u32(value, 0);
    }
}
```

## Advanced Integration Patterns

### Custom Hint Integration
```rust
use openvm::io::{hint_load_by_key, read_vec};

pub struct HintManager {
    cache: std::collections::HashMap<Vec<u8>, Vec<u8>>,
}

impl HintManager {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }
    
    pub fn load_data(&mut self, key: &[u8]) -> Vec<u8> {
        if let Some(cached) = self.cache.get(key) {
            return cached.clone();
        }
        
        #[cfg(target_os = "zkvm")]
        {
            hint_load_by_key(key);
            let data = read_vec();
            self.cache.insert(key.to_vec(), data.clone());
            data
        }
        
        #[cfg(not(target_os = "zkvm"))]
        {
            // Mock implementation for testing
            let mock_data = format!("mock_data_for_{:?}", key).into_bytes();
            self.cache.insert(key.to_vec(), mock_data.clone());
            mock_data
        }
    }
}
```

### Serialization Integration

#### Custom Serialization Patterns
```rust
use openvm::serde::{Deserializer, de::DeserializeOwned};
use openvm::io::read;

pub trait WordAligned {
    fn from_words(words: &[u32]) -> Self;
    fn to_words(&self) -> Vec<u32>;
}

// Efficient word-aligned data structure
#[repr(C)]
pub struct AlignedData {
    header: u32,
    payload: [u32; 8],
    checksum: u32,
}

impl WordAligned for AlignedData {
    fn from_words(words: &[u32]) -> Self {
        assert_eq!(words.len(), 10);
        Self {
            header: words[0],
            payload: words[1..9].try_into().unwrap(),
            checksum: words[9],
        }
    }
    
    fn to_words(&self) -> Vec<u32> {
        let mut words = Vec::with_capacity(10);
        words.push(self.header);
        words.extend_from_slice(&self.payload);
        words.push(self.checksum);
        words
    }
}

pub fn read_aligned_data() -> AlignedData {
    read::<AlignedData>()
}
```

### Platform Abstraction Integration

#### Multi-platform Service Layer
```rust
pub trait PlatformService {
    fn read_external_data(&self, key: &str) -> Vec<u8>;
    fn store_result(&self, data: &[u8]);
    fn get_timestamp(&self) -> u64;
}

pub struct ZkVMService;
pub struct HostService;

impl PlatformService for ZkVMService {
    #[cfg(target_os = "zkvm")]
    fn read_external_data(&self, key: &str) -> Vec<u8> {
        openvm::io::hint_load_by_key(key.as_bytes());
        openvm::io::read_vec()
    }
    
    #[cfg(target_os = "zkvm")]
    fn store_result(&self, data: &[u8]) {
        if data.len() >= 32 {
            let mut result = [0u8; 32];
            result.copy_from_slice(&data[..32]);
            openvm::io::reveal_bytes32(result);
        }
    }
    
    fn get_timestamp(&self) -> u64 {
        // zkVM doesn't have system time, use program-provided timestamp
        openvm::io::read::<u64>()
    }
}

impl PlatformService for HostService {
    #[cfg(not(target_os = "zkvm"))]
    fn read_external_data(&self, _key: &str) -> Vec<u8> {
        // Mock implementation
        vec![1, 2, 3, 4]
    }
    
    #[cfg(not(target_os = "zkvm"))]
    fn store_result(&self, data: &[u8]) {
        println!("Storing result: {:?}", data);
    }
    
    fn get_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

pub fn create_platform_service() -> Box<dyn PlatformService> {
    #[cfg(target_os = "zkvm")]
    return Box::new(ZkVMService);
    
    #[cfg(not(target_os = "zkvm"))]
    return Box::new(HostService);
}
```

## Best Practices for Integration

### 1. Always Provide Host Implementations
```rust
// Good: Provides both implementations
#[cfg(target_os = "zkvm")]
pub fn secure_operation() -> u32 {
    // zkVM implementation with actual security
    openvm::io::read_u32()
}

#[cfg(not(target_os = "zkvm"))]
pub fn secure_operation() -> u32 {
    // Host mock for testing
    42
}
```

### 2. Use Word-Aligned Data Structures
```rust
// Good: Word-aligned structure
#[repr(C)]
struct EfficientData {
    field1: u32,
    field2: u32,
    field3: [u32; 4],
}

// Avoid: Byte-aligned structures that require padding
struct IneffientData {
    field1: u8,
    field2: u16,
    field3: u64,
}
```

### 3. Implement Proper Error Handling
```rust
pub fn robust_integration() -> Result<[u8; 32], &'static str> {
    let input = openvm::io::read::<Vec<u8>>();
    
    if input.len() > MAX_INPUT_SIZE {
        return Err("Input too large");
    }
    
    let processed = process_data(&input)?;
    let result = finalize_result(processed)?;
    
    openvm::io::reveal_bytes32(result);
    Ok(result)
}
```

### 4. Use Feature Gates Appropriately
```rust
#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

pub fn platform_agnostic_function() {
    let mut map = HashMap::new();
    map.insert("key", "value");
}
```

## Troubleshooting Integration Issues

### Common Problems and Solutions

1. **Missing Host Implementations**
   - Always provide `#[cfg(not(target_os = "zkvm"))]` variants
   - Use mock implementations for testing

2. **Alignment Issues**
   - Ensure data structures are word-aligned
   - Use `#[repr(C)]` for predictable layout

3. **Feature Conflicts**
   - Check feature dependencies in Cargo.toml
   - Use appropriate conditional compilation

4. **Memory Allocation Failures**
   - Consider using `heap-embedded-alloc` feature
   - Implement memory-efficient algorithms

5. **Serialization Errors**
   - Ensure custom types implement serde traits
   - Use word-stream compatible serialization