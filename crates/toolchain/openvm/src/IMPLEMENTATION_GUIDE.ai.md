# OpenVM Toolchain Implementation Guide

## Overview
This guide provides detailed implementation guidance for the OpenVM standard library (`openvm` crate), which serves as the foundational runtime for guest programs in the OpenVM zkVM.

## Architecture Deep Dive

### Entry Point Flow
1. **_start (Assembly)**: Initial entry from zkVM
   - Sets up global pointer (`gp`)
   - Initializes stack pointer (`sp`) to `STACK_TOP`
   - Calls `__start` Rust function

2. **__start (Rust)**: Runtime initialization
   - Initializes heap allocator (if enabled)
   - Calls user's `main` function
   - Exits via `process::exit()`

3. **entry! Macro**: Links user main to runtime
   ```rust
   openvm::entry!(main);
   fn main() { /* user code */ }
   ```

### Memory Layout
```
High Address
┌─────────────────┐
│   Stack (grows  │ ← STACK_TOP (from openvm_platform)
│   downward)     │
├─────────────────┤
│   Heap (grows   │ ← Managed by allocator
│   upward)       │
├─────────────────┤
│   .bss          │
├─────────────────┤
│   .data         │
├─────────────────┤
│   .text         │
└─────────────────┘
Low Address
```

### I/O Architecture

#### Hint System
The hint system provides external data to guest programs:

1. **Hint Stream**: Sequential stream of 32-bit words
2. **Reading Operations**:
   - `hint_input()`: Signal to read next hint
   - `hint_store_u32!`: Store word from hint to memory
   - `hint_buffer_u32!`: Bulk read words to buffer

3. **Implementation Pattern**:
   ```rust
   pub fn read_data() -> u32 {
       hint_input();  // Signal ready
       let ptr = /* allocate memory */;
       hint_store_u32!(ptr);  // Store hint to memory
       unsafe { *ptr }  // Read from memory
   }
   ```

#### Output Revelation
Output is revealed through indexed 32-bit words:

```rust
// Reveal at byte index (must be 4-byte aligned)
reveal!(byte_index, value, 0);
```

### Serialization Design

#### Word Alignment Strategy
All data is padded to 32-bit boundaries:

```
Original: [u8; 5] = [1, 2, 3, 4, 5]
Serialized: [u32; 2] = [0x04030201, 0x00000005]
                        └─LE bytes─┘ └─padding─┘
```

#### Type Serialization Rules
1. **Primitives**: 
   - u8/u16 → promoted to u32
   - u64 → split into 2 u32s
   - u128 → serialized as bytes with padding

2. **Collections**:
   - Length prefix (u32) + elements
   - Each element word-aligned

3. **Strings**:
   - Length (u32) + UTF-8 bytes (padded)

### Platform Abstraction Patterns

#### Target-Specific Implementation
```rust
// Platform-specific hint reading
#[cfg(target_os = "zkvm")]
pub fn platform_read() -> u32 {
    // Use zkVM hint instructions
    openvm_rv32im_guest::hint_store_u32!(addr);
}

#[cfg(not(target_os = "zkvm"))]
pub fn platform_read() -> u32 {
    // Use host mock
    crate::host::read_u32()
}
```

#### Assembly Integration
Assembly files are included via `global_asm!`:
```rust
#[cfg(target_os = "zkvm")]
core::arch::global_asm!(include_str!("memcpy.s"));
```

## Implementation Patterns

### Adding New I/O Operations

1. **Define zkVM Implementation**:
   ```rust
   #[cfg(target_os = "zkvm")]
   pub fn read_custom() -> CustomType {
       use openvm_rv32im_guest::*;
       hint_input();
       // Implementation using hint instructions
   }
   ```

2. **Add Host Mock**:
   ```rust
   #[cfg(not(target_os = "zkvm"))]
   pub fn read_custom() -> CustomType {
       use crate::host::*;
       // Mock implementation for testing
   }
   ```

3. **Ensure Word Alignment**:
   ```rust
   let word_count = byte_len.div_ceil(4);
   let buffer = Vec::with_capacity(word_count * 4);
   ```

### Memory Allocation Patterns

#### Stack Allocation (Preferred for Small Data)
```rust
let mut buffer = [0u32; 16];  // 64 bytes on stack
hint_buffer_u32!(buffer.as_mut_ptr(), 16);
```

#### Heap Allocation (For Dynamic Sizes)
```rust
let layout = Layout::from_size_align(size, 4).unwrap();
let ptr = unsafe { alloc::alloc::alloc(layout) };
// Use ptr...
unsafe { alloc::alloc::dealloc(ptr, layout) };
```

### Error Handling Strategy

1. **Recoverable Errors**: Return `Result<T, E>`
2. **Unrecoverable Errors**: Use panic handler
3. **Debug Output**: Use `println!` for diagnostics

```rust
pub fn safe_operation() -> Result<u32, Error> {
    if let Some(data) = try_read() {
        Ok(process(data))
    } else {
        Err(Error::InvalidInput)
    }
}
```

## Performance Optimization

### Critical Path Optimizations

1. **Minimize Allocations**:
   ```rust
   // Bad: Allocates unnecessarily
   let data = read_vec();
   let result = process(&data);
   
   // Good: Process in-place
   let mut data = read_vec();
   process_in_place(&mut data);
   ```

2. **Use Word Operations**:
   ```rust
   // Bad: Byte-by-byte
   for byte in bytes {
       output.push(byte);
   }
   
   // Good: Word-aligned bulk copy
   let words = bytes.chunks_exact(4);
   output.extend_from_slice(words);
   ```

3. **Leverage Assembly Routines**:
   - Use provided memcpy/memset
   - They're optimized for zkVM

### Memory Efficiency

1. **Reuse Buffers**:
   ```rust
   let mut buffer = Vec::with_capacity(1024);
   for item in items {
       buffer.clear();
       serialize_into(&mut buffer, item);
       process(&buffer);
   }
   ```

2. **Avoid Intermediate Allocations**:
   ```rust
   // Use iterators instead of collecting
   data.iter()
       .map(transform)
       .filter(predicate)
       .for_each(process);
   ```

## Testing Strategies

### Dual-Target Testing
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_operation() {
        // Works on both host and zkVM
        let result = super::operation();
        assert_eq!(result, expected);
    }
    
    #[cfg(target_os = "zkvm")]
    #[test]
    fn test_zkvm_specific() {
        // zkVM-only test
    }
}
```

### Mock Hint Injection
```rust
#[cfg(not(target_os = "zkvm"))]
pub mod test_utils {
    pub fn inject_hints(hints: Vec<u32>) {
        HOST_HINTS.lock().unwrap().extend(hints);
    }
}
```

## Common Implementation Challenges

### Challenge 1: Word Alignment Bugs
**Problem**: Data corruption from misaligned access
**Solution**: Always use word-aligned addresses and sizes
```rust
debug_assert!(addr % 4 == 0, "Address must be word-aligned");
debug_assert!(size % 4 == 0, "Size must be word-aligned");
```

### Challenge 2: Host-zkVM Divergence
**Problem**: Different behavior between environments
**Solution**: Comprehensive testing and consistent mocks
```rust
// Ensure mocks match zkVM behavior exactly
assert_eq!(mock_result, zkvm_result);
```

### Challenge 3: Memory Leaks with Bump Allocator
**Problem**: Default allocator never frees memory
**Solution**: Design for single-pass processing or use embedded-alloc
```rust
// Process data in streaming fashion
while let Some(chunk) = read_chunk() {
    process(chunk);  // Don't accumulate
}
```

## Security Implementation

### Input Validation
```rust
pub fn read_bounded(max_size: usize) -> Result<Vec<u8>, Error> {
    let size = read_u32() as usize;
    if size > max_size {
        return Err(Error::SizeLimitExceeded);
    }
    Ok(read_vec_by_len(size))
}
```

### Safe Output Revelation
```rust
pub fn reveal_hash(data: &[u8]) {
    let hash = sha256(data);
    reveal_bytes32(hash);
    // Don't reveal raw data
}
```

## Debugging Techniques

### Debug Output
```rust
#[cfg(feature = "debug")]
println!("Debug: value = {}", value);
```

### Assertion Helpers
```rust
debug_assert!(condition, "Invariant violated: {}", details);
```

### Trace Macros
```rust
macro_rules! trace {
    ($($arg:tt)*) => {
        #[cfg(feature = "trace")]
        println!("[TRACE] {}", format!($($arg)*));
    };
}
```

## Future-Proofing

### Extensibility Points
1. **Custom Allocators**: Via `GlobalAlloc` trait
2. **Platform Extensions**: Through `openvm_platform`
3. **I/O Protocols**: Extensible hint system

### Version Compatibility
- Maintain stable ABI for core functions
- Use feature flags for new functionality
- Document breaking changes clearly

### Performance Evolution
- Profile critical paths regularly
- Consider SIMD when available
- Optimize for common patterns