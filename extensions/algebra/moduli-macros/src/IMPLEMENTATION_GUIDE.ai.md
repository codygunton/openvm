# OpenVM Algebra Moduli Macros - Implementation Guide

## Architecture Overview

### Macro Processing Pipeline
1. **Parse Input**: Extract modulus names and values from macro invocation
2. **Validate Moduli**: Check size limits and format
3. **Generate Code**: Create type definitions and implementations
4. **Handle Special Cases**: Prime moduli get additional Field/Sqrt traits

### Key Design Decisions

#### Static Modulus Index
- Each modulus gets a unique index via `MOD_IDX` atomic counter
- Index used to differentiate zkVM instructions
- Enables multiple moduli in single program

#### Alignment Strategy
- 32-byte moduli → 32-byte alignment (single block)
- 33-48 byte moduli → 16-byte alignment (multi-block)
- Ensures efficient memory access and instruction execution

#### Dual Implementation
- Native Rust: Uses `num_bigint` for correctness
- zkVM: Custom instructions for performance
- Single API surface for both environments

## Implementation Deep Dive

### Type Generation

#### Core Structure
```rust
#[repr(C, align(block_size))]
pub struct ModulusName([u8; limbs]);
```

Key aspects:
- `repr(C)`: Predictable memory layout
- Alignment: Optimizes zkVM instruction performance
- Byte array: Little-endian representation

#### Safety Patterns
```rust
unsafe fn add_refs_impl<const CHECK_SETUP: bool>(&self, other: &Self, dst_ptr: *mut Self) {
    // CHECK_SETUP allows skipping redundant setup checks
    // dst_ptr pattern enables in-place operations
}
```

### zkVM Integration

#### Custom Instruction Encoding
```rust
funct7 = BaseFunct7::OpType as usize + mod_idx * MAX_KINDS
```
- Encodes both operation type and modulus index
- Allows VM to dispatch to correct arithmetic chip
- Supports up to MAX_KINDS moduli per program

#### Setup Protocol
1. Store modulus in `.openvm` ELF section
2. Generate setup functions per modulus
3. Runtime reads modulus from ELF
4. Configures arithmetic chips

### Hint System Implementation

#### Square Root Hints
```rust
fn hint_sqrt_impl(&self) -> Option<(bool, Self)> {
    // 1. Request hint from host
    hint_sqrt_extern_func(self);
    
    // 2. Read hint data
    let is_square = hint_store_u32!();
    let sqrt_bytes = hint_buffer_u32!();
    
    // 3. Return parsed hint
    Some((is_square == 1, Self::from_bytes(sqrt_bytes)))
}
```

#### Verification Strategy
- Host provides (is_square, value) pair
- If is_square: verify value² = self
- If !is_square: verify value² = self × non_QR
- Invalid hints cause infinite loops (proof failure)

### Optimization Techniques

#### Once Cell Pattern
```rust
static is_setup: OnceBool = OnceBool::new();
is_setup.get_or_init(|| {
    unsafe { moduli_setup_extern_func(); }
    true
});
```
- Ensures single setup per modulus
- Thread-safe initialization
- Zero overhead after setup

#### Reference Arithmetic
```rust
impl<'a> Add<&'a T> for &T {
    fn add(self, other: &'a T) -> T {
        unsafe { self.add_ref::<true>(other) }
    }
}
```
- Avoids unnecessary clones
- Enables efficient chaining
- Maintains ergonomic API

## Advanced Patterns

### Compile-Time Modulus Validation
```rust
let modulus_biguint = BigUint::from_bytes_le(&modulus_bytes);
let modulus_is_prime = is_prime(&modulus_biguint, None);

if modulus_is_prime.probably() {
    // Generate Field trait implementation
}
```

### Multi-Block Arithmetic
For moduli > 32 bytes:
- Split across 16-byte blocks
- Chips handle carry propagation
- Transparent to user code

### Error Handling Philosophy
- Compile-time errors for invalid input
- Runtime verification for security
- Infinite loops for soundness violations

## Performance Profiling

### Benchmark Methodology
```rust
#[cfg(not(target_os = "zkvm"))]
mod bench {
    // Native benchmarks using criterion
}

#[cfg(target_os = "zkvm")]
mod bench {
    // Count cycles using performance counters
}
```

### Optimization Targets
1. **Setup**: One-time cost, optimize for size
2. **Basic Ops**: Maximize instruction throughput
3. **Field Ops**: Balance between code size and speed
4. **Hint Processing**: Minimize round trips

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_arithmetic_laws() {
    // Associativity: (a + b) + c = a + (b + c)
    // Commutativity: a + b = b + a
    // Distribution: a(b + c) = ab + ac
}
```

### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_inverse_correctness(a: Vec<u8>) {
        let elem = ModType::from_bytes(&a);
        if gcd(elem, modulus) == 1 {
            assert_eq!(elem * elem.inverse(), ModType::ONE);
        }
    }
}
```

### Cross-Environment Validation
```rust
fn verify_same_result<F: Fn(&T, &T) -> T>(op: F) {
    let native_result = run_native(op);
    let zkvm_result = run_zkvm(op);
    assert_eq!(native_result, zkvm_result);
}
```

## Common Pitfalls

### Forgetting Alignment
- Always use generated types, not raw arrays
- Alignment crucial for zkVM performance
- Misalignment causes undefined behavior

### Assuming Canonical Form
- Elements may exceed modulus
- Always call `assert_reduced()` when needed
- Host can provide non-canonical elements

### Division by Non-Coprime
- `div_unsafe` assumes gcd(divisor, modulus) = 1
- Violation causes undefined behavior
- Consider explicit coprimality check

## Extension Points

### Custom Field Operations
```rust
impl ModulusName {
    pub fn custom_operation(&self) -> Self {
        // Implement using existing primitives
    }
}
```

### Integration with Other Extensions
- Combine with elliptic curve extension
- Use in pairing computations
- Build higher-level protocols

### Performance Tuning
- Adjust block sizes for specific moduli
- Implement specialized squaring
- Add Montgomery form support