# Complex Macros - AI Documentation

## Overview

The `openvm-algebra-complex-macros` crate provides procedural macros for declaring and implementing quadratic field extensions (complex fields) in OpenVM. These macros generate type definitions and arithmetic operations for complex numbers over arbitrary modular fields, optimized for both native execution and zkVM execution.

## Core Functionality

### Complex Field Structure
Complex fields are quadratic extensions of the form Fp[u]/(u² + 1), where:
- Elements are represented as `c0 + c1*u` where `c0, c1 ∈ Fp`
- The irreducible polynomial is `X² + 1`, meaning `u² = -1`
- This construction requires the base field to have p ≡ 3 (mod 4) to ensure -1 is not a quadratic residue

### Key Macros

#### 1. `complex_declare!`
- **Purpose**: Declares complex field types with their base modular field
- **Syntax**: 
  ```rust
  complex_declare! {
      ComplexType1 { mod_type = ModularType1 },
      ComplexType2 { mod_type = ModularType2 },
  }
  ```
- **Generated Code**:
  - Struct definition with `c0` and `c1` fields
  - Arithmetic trait implementations (Add, Sub, Mul, Div)
  - Field constants (ZERO, ONE)
  - Complex conjugation support
  - Serialization/deserialization support

#### 2. `complex_init!`
- **Purpose**: Initializes complex fields at runtime, linking them to moduli indices
- **Syntax**:
  ```rust
  complex_init!(
      ComplexType1 { mod_idx = 0 },
      ComplexType2 { mod_idx = 1 }
  );
  ```
- **Requirements**: Must be called after `moduli_init!` from the moduli-macros crate
- **Generated Code**: FFI functions for zkVM arithmetic operations

#### 3. `complex_impl_field!`
- **Purpose**: Implements the `Field` trait for complex types
- **Syntax**: `complex_impl_field!(ComplexType1, ComplexType2);`
- **Generated Methods**: `double_assign()` and `square_assign()`

## Implementation Details

### zkVM vs Native Execution
The macros generate dual-mode code:

1. **Native Mode** (`#[cfg(not(target_os = "zkvm"))]`):
   - Direct arithmetic using the underlying modular field operations
   - Complex multiplication: `(a+bi)(c+di) = (ac-bd) + (ad+bc)i`
   - Complex division: multiply by conjugate and divide by norm

2. **zkVM Mode** (`#[cfg(target_os = "zkvm"))]`):
   - External function calls to specialized VM instructions
   - Lazy initialization via `set_up_once()` using thread-safe `OnceBool`
   - Memory-efficient operations using `MaybeUninit` for uninitialized results

### Generated Extern Functions
For each complex type, the following extern functions are generated:
- `complex_add_extern_func_TypeName`: Addition operation
- `complex_sub_extern_func_TypeName`: Subtraction operation
- `complex_mul_extern_func_TypeName`: Multiplication operation
- `complex_div_extern_func_TypeName`: Division operation
- `complex_setup_extern_func_TypeName`: One-time setup

### Memory Layout
Complex numbers are stored as C-compatible structs:
```rust
#[repr(C)]
pub struct ComplexType {
    pub c0: ModularType,  // Real part
    pub c1: ModularType,  // Imaginary part
}
```

## Architecture Integration

### Opcode Assignment
- Each complex field type gets a unique index (0, 1, 2, ...)
- Operations are mapped to opcodes using:
  - Base operation type (ADD, SUB, MUL, DIV, SETUP)
  - Complex field index offset
  - Maximum kinds constant for spacing

### Instruction Format
zkVM instructions use the RISC-V custom instruction format:
- `opcode`: `openvm_algebra_guest::OPCODE`
- `funct3`: `openvm_algebra_guest::COMPLEX_EXT_FIELD_FUNCT3`
- `funct7`: Operation type + complex field offset
- `rd`, `rs1`, `rs2`: Register/memory operands

### Setup Instructions
Setup uses a special encoding where `rs2` distinguishes between:
- `x0` (0): Setup for addition/subtraction operations
- `x1` (1): Setup for multiplication/division operations

## Performance Optimizations

### 1. Lazy Initialization
- Setup functions called only once per complex field type
- Thread-safe initialization using `OnceBool`
- No overhead after first use

### 2. Memory Efficiency
- `MaybeUninit` avoids unnecessary initialization for results
- Direct pointer operations for zkVM external calls
- In-place operations to minimize allocations

### 3. Specialized Implementations
- Reference-based operations (`&ComplexType`) avoid clones
- Separate implementations for owned vs borrowed operands
- Unsafe pointer-based multiplication for performance

## Usage Patterns

### Basic Declaration and Usage
```rust
// Declare modular types
moduli_declare! {
    Fq { modulus = "0x30644e72e131a029..." },
}

// Declare complex types
complex_declare! {
    Fq2 { mod_type = Fq },
}

// Initialize in main
moduli_init! { "0x30644e72e131a029..." }
complex_init! { Fq2 { mod_idx = 0 } }

// Use complex arithmetic
let a = Fq2::new(Fq::from_u32(1), Fq::from_u32(2));
let b = Fq2::new(Fq::from_u32(3), Fq::from_u32(4));
let c = &a + &b;  // (4, 6)
let d = &a * &b;  // (-5, 10)
```

### Complex Conjugation
```rust
impl ComplexConjugate for Fq2 {
    fn conjugate(self) -> Self {
        Self { c0: self.c0, c1: -self.c1 }
    }
    
    fn conjugate_assign(&mut self) {
        self.c1.neg_assign();
    }
}
```

## Integration with OpenVM

### 1. Guest Code Integration
- Macros work with `openvm::init!` for automatic initialization
- Generated code compatible with `openvm::entry!` programs
- Supports `no_std` environments

### 2. Circuit Integration
- Complex operations map to circuit chips in `openvm-algebra-circuit`
- Each operation becomes a constraint in the zkVM proof system
- Automatic opcode routing based on complex field index

### 3. Transpiler Integration
- Instructions recognized by `openvm-algebra-transpiler`
- Setup instructions transformed based on `rs2` register value
- Proper handling of modulus byte arrays

## Error Handling

### Common Issues
1. **Modulus Index Mismatch**: Order in `complex_init!` must match `moduli_init!`
2. **Missing Setup**: Forgetting to call init macros causes runtime errors
3. **Invalid Modulus**: Base field must support quadratic non-residue (-1)

### Debugging Support
- Generated code includes clear function names for stack traces
- Panic messages indicate which parameter is missing/invalid
- Print statements during macro expansion show field assignments

## Security Considerations

1. **No Direct Memory Access**: All operations go through safe abstractions
2. **Bounds Checking**: Modulus indices validated at compile time
3. **Type Safety**: Strong typing prevents mixing different complex fields
4. **Deterministic Operations**: No randomness in arithmetic operations

## Extension Points

### Adding New Operations
1. Add operation to `ComplexExtFieldBaseFunct7` enum
2. Generate corresponding extern function in macro
3. Implement trait with new operation
4. Update circuit implementation to handle new opcode

### Supporting Different Polynomials
Currently hardcoded to `X² + 1`, but could be extended:
1. Add polynomial parameter to `complex_declare!`
2. Adjust arithmetic formulas based on polynomial
3. Update constraint generation in circuits

## Dependencies and Requirements

- **syn**: Parsing macro input with full feature set
- **quote**: Generating Rust code with proper spans
- **openvm-macros-common**: Shared macro utilities
- **openvm-algebra-guest**: Runtime support and traits
- **serde**: Serialization support for complex types

The macros are designed to work seamlessly with the broader OpenVM ecosystem while providing efficient complex field arithmetic for cryptographic applications.