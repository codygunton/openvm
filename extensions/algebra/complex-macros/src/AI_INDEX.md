# Complex Macros - Structure Index

## File Structure
```
extensions/algebra/complex-macros/
├── Cargo.toml                 # Package manifest
└── src/
    └── lib.rs                 # All macro implementations
```

## Macro Organization in lib.rs

### Main Macros
1. **`complex_declare!` (lines 18-530)**
   - Entry point for complex type declaration
   - Parses input using `MacroArgs` from `openvm-macros-common`
   - Generates complete type implementation

2. **`complex_init!` (lines 544-655)**
   - Runtime initialization macro
   - Links complex types to modulus indices
   - Generates FFI functions for zkVM

3. **`complex_impl_field!` (lines 679-723)**
   - Implements `Field` trait for complex types
   - Adds `double_assign` and `square_assign` methods

### Helper Structures
- **`ComplexSimpleItem` (lines 657-677)**
  - Parser for `complex_impl_field!` input
  - Converts expression list to path list

### Generated Code Sections

#### Type Definition (lines 73-103)
- Struct with `c0` and `c1` fields
- Constants: `ZERO`, `ONE`
- Constructor: `new()`
- Basic method: `neg_assign()`

#### Arithmetic Implementations
1. **Addition (lines 106-124, 199-219, 318-356)**
   - `add_assign_impl()`: Core implementation
   - `add_refs_impl()`: Reference-based addition
   - Trait impls: `AddAssign`, `Add`

2. **Subtraction (lines 128-146, 223-243, 358-396)**
   - `sub_assign_impl()`: Core implementation
   - `sub_refs_impl()`: Reference-based subtraction
   - Trait impls: `SubAssign`, `Sub`

3. **Multiplication (lines 148-170, 249-269, 398-440)**
   - `mul_assign_impl()`: Core implementation
   - `mul_refs_impl()`: Unsafe pointer-based multiplication
   - Formula: `(a+bi)(c+di) = (ac-bd) + (ad+bc)i`

4. **Division (lines 173-196, 273-293, 442-480)**
   - `div_assign_unsafe_impl()`: Core implementation
   - `div_unsafe_refs_impl()`: Reference-based division
   - Uses conjugate multiplication and norm division

#### Complex Conjugation (lines 305-316)
- Implements `ComplexConjugate` trait
- `conjugate()`: Returns conjugate
- `conjugate_assign()`: In-place conjugation

#### Iterator Support (lines 482-504)
- `Sum` trait for addition reduction
- `Product` trait for multiplication reduction
- Works with both owned and borrowed iterators

#### Negation (lines 506-518)
- Implements `Neg` trait
- For both owned and borrowed values
- Uses subtraction from zero

#### Debug Formatting (lines 520-524)
- Formats as `"c0 + c1 * u"`

#### Setup Helper (lines 296-302)
- `set_up_once()`: Thread-safe initialization
- Uses `OnceBool` for one-time execution

### FFI Function Generation

#### Operation Functions (lines 583-604)
- Generated for each operation type (add, sub, mul, div)
- Uses custom RISC-V instruction format
- Includes proper opcode/funct encoding

#### Setup Function (lines 607-645)
- Special handling for zkVM target
- Uses `rs2` register to distinguish setup types:
  - `x0`: Setup for add/sub
  - `x1`: Setup for mul/div
- Manages modulus byte arrays

## Key Design Patterns

### 1. Conditional Compilation
```rust
#[cfg(not(target_os = "zkvm"))]
// Native implementation

#[cfg(target_os = "zkvm")]
// zkVM external calls
```

### 2. Memory Optimization
- `MaybeUninit` for uninitialized results
- Direct pointer manipulation in zkVM mode
- In-place operations where possible

### 3. Trait Implementation Strategy
- Core `*_impl()` methods contain logic
- Public traits delegate to core methods
- Separate paths for owned vs borrowed operands

### 4. Macro Hygiene
- Uses `proc_macro::Span::call_site()`
- Proper namespacing with `quote_spanned!`
- Unique function names prevent collisions

## Import Dependencies

### External Crates
```rust
use proc_macro::TokenStream;
use syn::{parse_macro_input, Expr, ExprPath, Path, Token};
use quote::quote_spanned;
use openvm_macros_common::MacroArgs;
```

### Generated Code Dependencies
- `openvm_algebra_guest::IntMod`
- `openvm_algebra_guest::field::ComplexConjugate`
- `openvm_algebra_guest::DivUnsafe`
- `openvm::platform::custom_insn_r!`
- `serde::{Serialize, Deserialize}`

## Macro Parameter Reference

### complex_declare!
- **mod_type**: The base modular field type (required)

### complex_init!
- **mod_idx**: Index in the moduli list (required, usize)

### complex_impl_field!
- Takes a comma-separated list of type paths

## Generated External Functions

Pattern: `{operation}_{type_name}`

Examples:
- `complex_add_extern_func_Fq2`
- `complex_sub_extern_func_Fq2`
- `complex_mul_extern_func_Fq2`
- `complex_div_extern_func_Fq2`
- `complex_setup_extern_func_Fq2`

## Error Handling

### Compile-Time Errors
- Missing required parameters
- Invalid parameter types
- Unknown parameter names

### Runtime Panics
- "mod_type parameter is required"
- "mod_idx is required"
- "Unknown parameter {name}"

## Code Generation Flow

1. **Parse Input** → `MacroArgs` structure
2. **Extract Parameters** → Validate and store
3. **Generate Type Definition** → Struct and constants
4. **Generate Operations** → Native and zkVM implementations
5. **Generate Traits** → Arithmetic and utility traits
6. **Generate FFI** → External function declarations
7. **Combine and Return** → Final `TokenStream`

This modular approach allows easy extension and maintenance of the macro system.