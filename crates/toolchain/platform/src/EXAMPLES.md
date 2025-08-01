# OpenVM Platform Component Examples

## Basic Platform Usage

### Memory Constants and Validation

```rust
use openvm_platform::{WORD_SIZE, PAGE_SIZE};
use openvm_platform::memory::{GUEST_MIN_MEM, GUEST_MAX_MEM, is_guest_memory};

fn validate_memory_address(addr: u32) -> bool {
    is_guest_memory(addr)
}

fn get_platform_info() {
    println!("Word size: {} bytes", WORD_SIZE);
    println!("Page size: {} bytes", PAGE_SIZE);
    println!("Guest memory range: 0x{:08x} - 0x{:08x}", 
             GUEST_MIN_MEM, GUEST_MAX_MEM);
}
```

### Alignment Utilities

```rust
use openvm_platform::align_up;

fn align_buffer_size() {
    let size = 1000;
    let aligned_size = align_up(size, 64); // Align to 64-byte boundary
    println!("Original size: {}, Aligned size: {}", size, aligned_size);
    
    // Common alignment scenarios
    let word_aligned = align_up(size, WORD_SIZE);     // 4-byte alignment
    let page_aligned = align_up(size, PAGE_SIZE);     // 1024-byte alignment
    let cache_aligned = align_up(size, 64);           // Cache line alignment
}
```

## File Descriptor Usage

```rust
use openvm_platform::fileno::{STDIN, STDOUT, STDERR, JOURNAL};

fn use_file_descriptors() {
    // These constants can be used with system calls
    // sys_read(STDIN, buffer, len);
    // sys_write(STDOUT, data, len);
    // sys_write(STDERR, error_msg, len);
    // sys_write(JOURNAL, proof_data, len);
    
    println!("Standard file descriptors:");
    println!("STDIN: {}", STDIN);
    println!("STDOUT: {}", STDOUT);
    println!("STDERR: {}", STDERR);
    println!("JOURNAL: {}", JOURNAL);
}
```

## Memory Allocation Examples

### Basic Allocation (Using Global Allocator)

```rust
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

fn basic_allocation() {
    // Vector allocation (uses bump allocator by default)
    let mut data = Vec::new();
    data.push(42u32);
    data.extend_from_slice(&[1, 2, 3, 4, 5]);
    
    // String allocation
    let message = String::from("Hello from zkVM!");
    
    // Box allocation
    let boxed_value = Box::new(100u64);
    
    println!("Vector: {:?}", data);
    println!("String: {}", message);
    println!("Boxed: {}", boxed_value);
}
```

### Large Buffer Allocation

```rust
use alloc::vec;

fn allocate_large_buffer(size_mb: usize) -> Option<Vec<u8>> {
    let size_bytes = size_mb * 1024 * 1024;
    
    // Check if allocation would exceed memory limits
    if size_bytes > (GUEST_MAX_MEM - GUEST_MIN_MEM) {
        println!("Allocation too large: {} MB", size_mb);
        return None;
    }
    
    // Allocate buffer (will panic if out of memory)
    let buffer = vec![0u8; size_bytes];
    println!("Allocated {} MB buffer", size_mb);
    Some(buffer)
}
```

## Math Functions (with export-libm feature)

```rust
#[cfg(feature = "export-libm")]
fn math_operations() {
    use core::f64::consts::PI;
    
    // Trigonometric functions
    let angle = PI / 4.0;
    let sin_val = unsafe { libm::sin(angle) };
    let cos_val = unsafe { libm::cos(angle) };
    let tan_val = unsafe { libm::tan(angle) };
    
    println!("sin(π/4) = {}", sin_val);
    println!("cos(π/4) = {}", cos_val);
    println!("tan(π/4) = {}", tan_val);
    
    // Exponential and logarithmic functions
    let x = 2.0;
    let exp_val = unsafe { libm::exp(x) };
    let log_val = unsafe { libm::log(x) };
    let sqrt_val = unsafe { libm::sqrt(x) };
    
    println!("exp(2) = {}", exp_val);
    println!("log(2) = {}", log_val);
    println!("sqrt(2) = {}", sqrt_val);
}
```

## Platform-Specific Code

### Conditional Compilation

```rust
#[cfg(target_os = "zkvm")]
fn zkvm_specific_function() {
    // This code only runs on zkVM
    println!("Running on zkVM");
    
    // Access to zkVM-specific features
    use openvm_platform::{custom_insn_i, custom_insn_r};
    // Custom instruction usage would go here
}

#[cfg(not(target_os = "zkvm"))]
fn zkvm_specific_function() {
    // This code runs on host for testing
    println!("Running on host (testing mode)");
    
    // Mock implementation for testing
}

fn cross_platform_function() {
    // This function works on both zkVM and host
    let aligned_addr = align_up(0x1000, PAGE_SIZE);
    println!("Aligned address: 0x{:x}", aligned_addr);
    zkvm_specific_function();
}
```

## Runtime Examples

### Custom Entry Point

```rust
#[cfg(feature = "entrypoint")]
#[no_mangle]
pub extern "C" fn main() {
    // Custom program entry point
    println!("Starting OpenVM program");
    
    // Your program logic here
    run_program();
    
    // Program will exit automatically
}

fn run_program() {
    // Main program logic
    let data = vec![1, 2, 3, 4, 5];
    let sum: i32 = data.iter().sum();
    println!("Sum: {}", sum);
}
```

### Panic Handler

```rust
#[cfg(feature = "panic-handler")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Program panicked: {}", info);
    
    // Terminate the program
    #[cfg(feature = "rust-runtime")]
    openvm_platform::rust_rt::terminate::<1>();
    
    #[cfg(not(feature = "rust-runtime"))]
    loop {}
}
```

## Advanced Memory Management

### Memory Layout Inspection

```rust
use openvm_platform::memory::{TEXT_START, STACK_TOP};

fn inspect_memory_layout() {
    println!("Memory Layout:");
    println!("Stack top: 0x{:08x}", STACK_TOP);
    println!("Text start: 0x{:08x}", TEXT_START);
    println!("Guest memory: 0x{:08x} - 0x{:08x}", GUEST_MIN_MEM, GUEST_MAX_MEM);
    
    // Calculate available heap space (approximate)
    let heap_start = TEXT_START + 0x10000; // Estimate after program
    let available_heap = GUEST_MAX_MEM - heap_start as usize;
    println!("Approximate heap space: {} MB", available_heap / (1024 * 1024));
}
```

### Custom Allocation Patterns

```rust
fn allocation_patterns() {
    // Small frequent allocations
    let mut small_vecs = Vec::new();
    for i in 0..1000 {
        let v = vec![i; 10];
        small_vecs.push(v);
    }
    
    // Large single allocation
    let large_buffer = vec![0u8; 1024 * 1024]; // 1MB
    
    // Nested allocations
    let nested: Vec<Vec<i32>> = (0..100)
        .map(|i| (0..i).collect())
        .collect();
    
    println!("Created {} small vectors", small_vecs.len());
    println!("Large buffer size: {} bytes", large_buffer.len());
    println!("Nested structure depth: {}", nested.len());
}
```

## Integration Examples

### Using with Higher-Level Crates

```rust
// Typically you would use openvm instead of openvm-platform directly
use openvm_platform::{WORD_SIZE, align_up};

fn integration_example() {
    // Platform constants used by higher-level code
    let data_size = 1000;
    let aligned_size = align_up(data_size, WORD_SIZE);
    
    // Allocate aligned buffer for efficient access
    let mut buffer = vec![0u32; aligned_size / WORD_SIZE];
    
    // Fill with data
    for (i, item) in buffer.iter_mut().enumerate() {
        *item = i as u32;
    }
    
    println!("Created aligned buffer with {} words", buffer.len());
}
```

### Feature Configuration Examples

```toml
# Cargo.toml dependency configurations

# Minimal platform (default)
[dependencies]
openvm-platform = "1.3.0"

# With runtime support
[dependencies]
openvm-platform = { version = "1.3.0", features = ["rust-runtime"] }

# With embedded allocator
[dependencies]
openvm-platform = { version = "1.3.0", features = ["heap-embedded-alloc"] }

# Full featured
[dependencies]
openvm-platform = { 
    version = "1.3.0", 
    features = ["rust-runtime", "panic-handler", "export-libm"] 
}
```

## Testing Examples

### Dual-Target Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_alignment() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
    }
    
    #[test]
    fn test_memory_bounds() {
        assert!(is_guest_memory(GUEST_MIN_MEM as u32));
        assert!(!is_guest_memory((GUEST_MAX_MEM) as u32));
        assert!(is_guest_memory(0x1000));
    }
    
    #[test]
    fn test_allocation() {
        let data = vec![42u8; 1000];
        assert_eq!(data.len(), 1000);
        assert_eq!(data[0], 42);
    }
}
```

These examples demonstrate the core functionality of the OpenVM platform component across different use cases and configurations.