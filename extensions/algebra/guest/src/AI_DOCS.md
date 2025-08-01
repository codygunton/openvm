# OpenVM Algebra Guest Component Documentation

## Overview

The OpenVM algebra guest component (`openvm-algebra-guest`) provides algebraic traits and operations for zkVM guest programs. It defines modular arithmetic interfaces, field operations, and complex extension field traits that enable efficient cryptographic computations within the OpenVM zkVM environment.

This crate serves as the guest-side interface for algebraic operations, with implementations optimized for zero-knowledge proof generation. It provides a unified trait system for modular integers, prime fields, and extension fields.

## Architecture

### Core Components

1. **Modular Arithmetic Traits** (`lib.rs`)
   - `IntMod` trait for modular integer operations
   - `DivUnsafe` and `DivAssignUnsafe` for unchecked division
   - Custom instruction opcodes for hardware acceleration
   - Setup and reduction operations

2. **Field Traits** (`field/mod.rs`)
   - `Field` trait for prime field elements
   - `FieldExtension` trait for extension fields
   - `ComplexConjugate` for complex field operations
   - Optimized arithmetic operations

3. **Exponentiation** (`exp_bytes.rs`)
   - `ExpBytes` trait for efficient exponentiation
   - Big-endian byte-based power computation
   - Window-based optimization (2-bit windows)

4. **Hardware Integration**
   - Custom RISC-V opcodes (0x2b)
   - Modular arithmetic funct3: 0b000
   - Complex extension field funct3: 0b010
   - Up to 8 different moduli/fields per type

### Instruction Encoding

The algebra extension uses custom RISC-V instructions with the following encoding:

```
Modular Arithmetic:
- opcode: 0x2b (custom-1)
- funct3: 0b000
- funct7: mod_idx * 8 + base_funct7

Complex Extension Field:
- opcode: 0x2b (custom-1)  
- funct3: 0b010
- funct7: fp2_idx * 8 + base_funct7
```

### Key Design Principles

1. **Zero-Knowledge Optimization**
   - Operations designed for efficient constraint generation
   - Minimal branching for deterministic execution
   - Hardware-accelerated operations via custom instructions

2. **Type Safety**
   - Strong typing for different moduli/fields
   - Compile-time modulus checking where possible
   - Safe abstractions over unsafe operations

3. **Flexibility**
   - Support for multiple concurrent moduli
   - Configurable field extensions
   - Platform-agnostic trait definitions

## Traits

### IntMod Trait

The core trait for modular integers with the following requirements:

```rust
pub trait IntMod:
    Sized + Eq + Clone + Debug +
    // Arithmetic operations
    Neg<Output = Self> + Add<Output = Self> + 
    Sub<Output = Self> + Mul<Output = Self> +
    DivUnsafe<Output = Self> +
    // Assignment operations
    AddAssign + SubAssign + MulAssign + DivAssignUnsafe +
    // Iterator traits
    Sum + Product
{
    type Repr: AsRef<[u8]> + AsMut<[u8]>;
    const MODULUS: Self::Repr;
    const NUM_LIMBS: usize;
    const ZERO: Self;
    const ONE: Self;
    
    // Construction and conversion
    fn from_repr(repr: Self::Repr) -> Self;
    fn from_le_bytes(bytes: &[u8]) -> Option<Self>;
    fn from_u8(val: u8) -> Self;
    // ... more methods
}
```

### Field Trait

Simplified field trait for prime fields:

```rust
pub trait Field:
    Sized + Eq + Clone + Debug +
    // Arithmetic operations  
    Neg<Output = Self> + Add<Output = Self> +
    Sub<Output = Self> + Mul<Output = Self> +
    DivUnsafe<Output = Self> +
    // Assignment operations
    AddAssign + SubAssign + MulAssign + DivAssignUnsafe
{
    const ZERO: Self;
    const ONE: Self;
    
    fn double_assign(&mut self);
    fn square_assign(&mut self);
    fn invert(&self) -> Self;
}
```

### FieldExtension Trait

For extension fields over a base field:

```rust
pub trait FieldExtension<BaseField> {
    const D: usize; // Extension degree
    type Coeffs: Sized;
    
    fn from_coeffs(coeffs: Self::Coeffs) -> Self;
    fn to_coeffs(self) -> Self::Coeffs;
    fn embed(base_elem: BaseField) -> Self;
    fn frobenius_map(&self, power: usize) -> Self;
    fn mul_base(&self, rhs: &BaseField) -> Self;
}
```

## Features

### Core Features

- **Modular Arithmetic**: Generic trait system for modular integers
- **Field Operations**: Prime field and extension field support
- **Hardware Acceleration**: Custom RISC-V instructions for performance
- **Byte-based Exponentiation**: Efficient power computation
- **Multiple Moduli**: Support for up to 8 concurrent moduli/fields

### Optional Features

- `halo2curves`: Integration with halo2curves library (host-only)
- Host-side BigUint support for testing and development

## API Usage

### Basic Modular Arithmetic

```rust
use openvm_algebra_guest::{IntMod, Field};

// Assuming MyField implements IntMod
let a = MyField::from_u32(42);
let b = MyField::from_u32(13);

let c = a + b;        // Addition mod p
let d = a * b;        // Multiplication mod p
let e = a.div_unsafe(b); // Division (undefined if b not invertible)
```

### Field Operations

```rust
let x = MyField::from_u32(5);
let x_squared = x.square();     // x²
let x_doubled = x.double();     // 2x
let x_inv = x.invert();         // x⁻¹
let x_cubed = x.cube();        // x³
```

### Exponentiation

```rust
use openvm_algebra_guest::ExpBytes;

let base = MyField::from_u32(3);
let exp_bytes = [0x01, 0x23, 0x45]; // Big-endian
let result = base.exp_bytes(true, &exp_bytes); // 3^0x012345
```

### Extension Fields

```rust
use openvm_algebra_guest::FieldExtension;

// Fp2 over Fp
let base_elem = Fp::from_u32(7);
let ext_elem = Fp2::embed(base_elem);
let conjugate = ext_elem.conjugate();
let frob = ext_elem.frobenius_map(1);
```

## Implementation Notes

### Memory Safety

- All operations assume valid modular representations
- `div_unsafe` has undefined behavior for non-invertible denominators
- `assert_reduced` ensures canonical representation
- Bounds checking on byte conversions

### Performance Considerations

- Hardware instructions provide significant speedup
- Exponentiation uses 2-bit sliding window
- In-place operations (`*_assign`) reduce allocations
- Setup must be called before first use of each modulus

### VM-Specific Behavior

- `assert_reduced` may have different behavior in zkVM vs host
- Setup operations register moduli with VM runtime
- Custom instructions only available in zkVM target
- Host implementations use software fallbacks

## Security Considerations

1. **Timing Side Channels**: Operations are not constant-time by default
2. **Division Safety**: Always check invertibility before division
3. **Modulus Setup**: Must call `set_up_once()` before operations
4. **Representation Canonicity**: Use `assert_reduced()` when needed

## Related Components

- `openvm-algebra-circuit` - Circuit implementations for these operations
- `openvm-algebra-moduli-macros` - Macro for generating modular types
- `openvm-algebra-complex-macros` - Macro for complex extension fields
- `openvm-native-recursion` - Uses these traits for recursive verification