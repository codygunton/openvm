# OpenVM Platform Component Integration Guide

## Overview

The OpenVM platform component serves as the foundational layer for the entire OpenVM ecosystem. This guide covers how to integrate with, extend, and build upon the platform component.

## Integration Principles

### 1. Minimal Direct Usage
Most crates should depend on higher-level crates like `openvm` rather than `openvm-platform` directly:

```toml
# Preferred approach
[dependencies]
openvm = "1.3.0"

# Direct usage only when necessary
[dependencies]
openvm-platform = "1.3.0"
```

### 2. Feature-Based Integration
Use specific features to avoid unnecessary dependencies:

```toml
[dependencies]
openvm-platform = { 
    version = "1.3.0", 
    features = ["rust-runtime"], 
    default-features = false 
}
```

### 3. Dual-Target Compatibility
Always ensure your integration works on both zkVM and host targets.

## Common Integration Patterns

### 1. Memory Management Integration

#### Custom Allocator Integration
```rust
use openvm_platform::memory::{GUEST_MAX_MEM, is_guest_memory};

pub struct CustomMemoryManager {
    base_addr: usize,
    size: usize,
}

impl CustomMemoryManager {
    pub fn new(size: usize) -> Result<Self, &'static str> {
        if size > GUEST_MAX_MEM {
            return Err("Size exceeds guest memory limit");
        }
        
        Ok(Self {
            base_addr: 0,
            size,
        })
    }
    
    pub fn allocate(&mut self, size: usize) -> Option<*mut u8> {
        // Use platform allocator
        let ptr = unsafe { 
            openvm_platform::memory::sys_alloc_aligned(size, 8) 
        };
        
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
}
```

#### Memory Layout Integration
```rust
use openvm_platform::memory::{TEXT_START, STACK_TOP};
use openvm_platform::{align_up, WORD_SIZE};

pub fn setup_custom_memory_region(size: usize) -> usize {
    // Align size to word boundary
    let aligned_size = align_up(size, WORD_SIZE);
    
    // Ensure we don't conflict with stack or text regions
    let safe_start = TEXT_START + 0x100000; // 1MB after text start
    
    if safe_start as usize + aligned_size > STACK_TOP as usize {
        panic!("Custom memory region conflicts with stack");
    }
    
    aligned_size
}
```

### 2. Runtime Integration

#### Custom Entry Point Integration
```rust
#[cfg(feature = "entrypoint")]
use openvm_platform::rust_rt;

#[cfg(feature = "entrypoint")]
#[no_mangle]
pub extern "C" fn main() {
    // Initialize your component
    my_component_init();
    
    // Run main logic
    match run_main_logic() {
        Ok(_) => rust_rt::terminate::<0>(),
        Err(code) => rust_rt::terminate::<1>(),
    }
}

fn my_component_init() {
    // Component-specific initialization
    println!("Initializing custom component");
}

fn run_main_logic() -> Result<(), i32> {
    // Your main program logic
    Ok(())
}
```

#### Panic Handler Integration
```rust
use openvm_platform::print::println;

#[cfg(all(feature = "panic-handler", target_os = "zkvm"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Custom panic handling
    println("Custom panic handler activated");
    println(&format!("Panic: {}", info));
    
    // Perform cleanup if needed
    cleanup_resources();
    
    // Terminate
    openvm_platform::rust_rt::terminate::<1>();
}

fn cleanup_resources() {
    // Component-specific cleanup
}
```

### 3. Math Integration

#### Using Platform Math Functions
```rust
#[cfg(feature = "export-libm")]
use openvm_platform::libm_extern;

pub struct MathProcessor;

impl MathProcessor {
    #[cfg(feature = "export-libm")]
    pub fn compute_complex_math(&self, x: f64, y: f64) -> f64 {
        unsafe {
            let sin_x = libm_extern::sin(x);
            let cos_y = libm_extern::cos(y);
            let result = libm_extern::sqrt(sin_x * sin_x + cos_y * cos_y);
            result
        }
    }
    
    #[cfg(not(feature = "export-libm"))]
    pub fn compute_complex_math(&self, x: f64, y: f64) -> f64 {
        // Fallback implementation or use std
        (x.sin().powi(2) + y.cos().powi(2)).sqrt()
    }
}
```

## Advanced Integration Patterns

### 1. Custom Instruction Integration

```rust
#[cfg(all(feature = "rust-runtime", target_os = "zkvm"))]
use openvm_platform::{custom_insn_i, custom_insn_r};

pub struct CustomInstructionProcessor;

impl CustomInstructionProcessor {
    #[cfg(all(feature = "rust-runtime", target_os = "zkvm"))]
    pub fn execute_custom_operation(&self, opcode: u32, data: u32) -> u32 {
        unsafe {
            custom_insn_r!(opcode, data, 0, 0)
        }
    }
    
    #[cfg(not(all(feature = "rust-runtime", target_os = "zkvm")))]
    pub fn execute_custom_operation(&self, opcode: u32, data: u32) -> u32 {
        // Host implementation for testing
        match opcode {
            1 => data * 2,
            2 => data + 1,
            _ => data,
        }
    }
}
```

### 2. File Descriptor Integration

```rust
use openvm_platform::fileno::{STDOUT, STDERR, JOURNAL};

pub struct CustomLogger {
    output_fd: u32,
}

impl CustomLogger {
    pub fn new_stdout() -> Self {
        Self { output_fd: STDOUT }
    }
    
    pub fn new_stderr() -> Self {
        Self { output_fd: STDERR }
    }
    
    pub fn new_journal() -> Self {
        Self { output_fd: JOURNAL }
    }
    
    pub fn log(&self, message: &str) {
        // Use file descriptor for output
        match self.output_fd {
            STDOUT => println!("[STDOUT] {}", message),
            STDERR => eprintln!("[STDERR] {}", message),
            JOURNAL => {
                // Write to journal for proof inclusion
                self.write_to_journal(message.as_bytes());
            }
            _ => {}
        }
    }
    
    fn write_to_journal(&self, data: &[u8]) {
        // Implementation would use sys_write to JOURNAL fd
        // sys_write(JOURNAL, data.as_ptr(), data.len());
    }
}
```

## Feature Integration Guidelines

### 1. Feature Dependency Management

```toml
# Your crate's Cargo.toml
[dependencies]
openvm-platform = { version = "1.3.0", default-features = false }

[features]
default = ["platform-runtime"]
platform-runtime = ["openvm-platform/rust-runtime"]
platform-math = ["openvm-platform/export-libm"]
platform-embedded-alloc = ["openvm-platform/heap-embedded-alloc"]
full-platform = [
    "platform-runtime",
    "platform-math", 
    "openvm-platform/panic-handler"
]
```

### 2. Conditional Feature Usage

```rust
// Feature-gated functionality
#[cfg(feature = "platform-runtime")]
pub mod runtime_support {
    use openvm_platform::rust_rt;
    
    pub fn safe_terminate() {
        rust_rt::terminate::<0>();
    }
}

#[cfg(feature = "platform-math")]
pub mod math_support {
    use openvm_platform::libm_extern;
    
    pub fn advanced_math(x: f64) -> f64 {
        unsafe { libm_extern::exp(libm_extern::log(x)) }
    }
}

#[cfg(not(feature = "platform-math"))]
pub mod math_support {
    pub fn advanced_math(x: f64) -> f64 {
        x // Simplified fallback
    }
}
```

## Testing Integration

### 1. Dual-Target Test Setup

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use openvm_platform::{align_up, WORD_SIZE};
    
    #[test]
    fn test_platform_integration() {
        // Test that works on both zkVM and host
        let size = 100;
        let aligned = align_up(size, WORD_SIZE);
        assert!(aligned >= size);
        assert_eq!(aligned % WORD_SIZE, 0);
    }
    
    #[cfg(target_os = "zkvm")]
    #[test]
    fn test_zkvm_specific() {
        // zkVM-specific test
        use openvm_platform::memory::is_guest_memory;
        assert!(is_guest_memory(0x1000));
    }
    
    #[cfg(not(target_os = "zkvm"))]
    #[test]
    fn test_host_specific() {
        // Host-specific test
        println!("Running on host");
    }
}
```

### 2. Integration Test Patterns

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_memory_allocation_integration() {
        // Test allocation through platform
        let data = vec![0u8; 1024];
        assert_eq!(data.len(), 1024);
        
        // Test with custom alignment
        let aligned_data = create_aligned_buffer(1000);
        assert!(aligned_data.len() >= 1000);
    }
    
    fn create_aligned_buffer(size: usize) -> Vec<u8> {
        let aligned_size = align_up(size, 64);
        vec![0u8; aligned_size]
    }
    
    #[test]
    fn test_cross_platform_compatibility() {
        // Test that should work everywhere
        let processor = CustomInstructionProcessor;
        let result = processor.execute_custom_operation(1, 42);
        assert_eq!(result, 84); // 42 * 2
    }
}
```

## Best Practices

### 1. Dependency Management
- Use minimal feature sets to reduce compilation time
- Avoid direct platform dependency when possible
- Use feature flags for optional functionality

### 2. Memory Safety
- Always check memory bounds when working with raw pointers
- Use alignment functions for efficient memory access
- Prefer Rust's safe abstractions over raw allocation

### 3. Cross-Platform Compatibility
- Test on both zkVM and host targets
- Provide fallback implementations for host testing
- Use conditional compilation appropriately

### 4. Performance Optimization
- Align data structures to word boundaries
- Use bump allocator characteristics (no deallocation needed)
- Minimize memory fragmentation

### 5. Error Handling
- Use platform panic handlers appropriately
- Provide meaningful error messages
- Handle allocation failures gracefully

## Common Integration Pitfalls

### 1. Direct Platform Usage
```rust
// Avoid this
use openvm_platform::memory::sys_alloc_aligned;

// Prefer this
use std::alloc::{alloc, Layout};
// or
use alloc::vec::Vec;
```

### 2. Feature Conflicts
```rust
// Can cause conflicts
features = ["heap-embedded-alloc", "custom-allocator"]

// Be explicit about allocator choice
features = ["heap-embedded-alloc"]
```

### 3. Memory Assumptions
```rust
// Don't assume specific memory layout
let ptr = some_allocation();
assert_eq!(ptr as usize, expected_address); // Bad

// Use platform validation instead
assert!(is_guest_memory(ptr as u32)); // Good
```

## Migration Guidelines

### From Other zkVM Platforms
1. Replace custom memory management with platform allocator
2. Use platform constants instead of hardcoded values
3. Adopt platform's dual-target testing approach
4. Migrate to platform's feature system

### Version Updates
1. Check changelog for breaking changes
2. Update feature flags if needed
3. Test with new memory layout constants
4. Verify allocator behavior changes

This integration guide provides the foundation for successfully building upon the OpenVM platform component while maintaining compatibility and performance.