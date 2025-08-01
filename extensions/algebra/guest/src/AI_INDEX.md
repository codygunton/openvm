# OpenVM Algebra Guest Component Index

## Component Location
`extensions/algebra/guest/src/`

## Purpose
Guest-side algebraic traits and operations for modular arithmetic, field operations, and extension fields within the OpenVM zkVM environment.

## Key Files

### Core Library
- **lib.rs** - Main traits (IntMod), opcodes, instruction encoding
- **exp_bytes.rs** - Byte-based exponentiation trait and implementation

### Field Module
- **field/mod.rs** - Field and FieldExtension traits, ComplexConjugate

### Optional Integration
- **halo2curves.rs** - Host-only halo2curves integration for testing

## Public API

### Opcodes and Instructions
```rust
pub const OPCODE: u8 = 0x2b;  // custom-1
pub const MODULAR_ARITHMETIC_FUNCT3: u8 = 0b000;
pub const COMPLEX_EXT_FIELD_FUNCT3: u8 = 0b010;

// Modular arithmetic operations
pub enum ModArithBaseFunct7 {
    AddMod = 0,
    SubMod = 1,
    MulMod = 2,
    DivMod = 3,
    IsEqMod = 4,
    SetupMod = 5,
    HintNonQr = 6,
    HintSqrt = 7,
}

// Complex field operations
pub enum ComplexExtFieldBaseFunct7 {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
    Setup = 4,
}
```

### Core Traits
- `DivUnsafe<Rhs>` - Division with undefined behavior for non-invertible
- `DivAssignUnsafe<Rhs>` - In-place division assignment
- `IntMod` - Modular integer operations
- `Field` - Prime field operations
- `FieldExtension<BaseField>` - Extension field operations
- `ComplexConjugate` - Complex conjugation
- `Reduce` - Modular reduction from bytes
- `Sqrt` - Square root for fields
- `ExpBytes` - Byte-based exponentiation

### IntMod Methods
```rust
// Constants
const MODULUS: Self::Repr;
const ZERO: Self;
const ONE: Self;

// Construction
fn from_repr(repr: Self::Repr) -> Self;
fn from_le_bytes(bytes: &[u8]) -> Option<Self>;
fn from_u8(val: u8) -> Self;
fn from_u32(val: u32) -> Self;
fn from_u64(val: u64) -> Self;

// Conversion
fn as_le_bytes(&self) -> &[u8];
fn to_be_bytes(&self) -> Self::Repr;

// Operations
fn double(&self) -> Self;
fn square(&self) -> Self;
fn cube(&self) -> Self;
fn assert_reduced(&self);
fn is_reduced(&self) -> bool;
fn set_up_once();
```

### Field Methods
```rust
const ZERO: Self;
const ONE: Self;

fn double_assign(&mut self);
fn square_assign(&mut self);
fn invert(&self) -> Self;
```

### FieldExtension Methods
```rust
const D: usize;  // Extension degree

fn from_coeffs(coeffs: Self::Coeffs) -> Self;
fn to_coeffs(self) -> Self::Coeffs;
fn embed(base_elem: BaseField) -> Self;
fn frobenius_map(&self, power: usize) -> Self;
fn mul_base(&self, rhs: &BaseField) -> Self;
```

## Key Features
- Hardware-accelerated modular arithmetic
- Support for multiple concurrent moduli (up to 8)
- Prime field and extension field abstractions  
- Efficient byte-based exponentiation
- Zero-knowledge proof optimized operations
- Host and zkVM dual compilation support

## Configuration

### Features
- `halo2curves` - Enable halo2curves integration (host-only)

### Dependencies
- `alloc` - For Vec support
- `strum_macros` - For enum conversions
- `serde_big_array` - For serialization
- `once_cell` - For one-time setup
- `num_bigint` (host-only) - BigUint support
- `openvm_algebra_moduli_macros` - Modular type generation
- `openvm_algebra_complex_macros` - Complex field macros

## Instruction Format
```
Modular Arithmetic:
funct7 = mod_idx * 8 + base_funct7
- mod_idx: 0-7 (which modulus)
- base_funct7: 0-7 (operation)

Complex Extension:
funct7 = fp2_idx * 8 + base_funct7
- fp2_idx: 0-7 (which field)
- base_funct7: 0-4 (operation)
```

## Usage Context
Foundation for zkVM algebraic operations:
- Elliptic curve arithmetic
- Pairing computations  
- Recursive proof verification
- Cryptographic protocols
- Finite field arithmetic

## Safety Notes
- `div_unsafe` undefined for non-invertible denominators
- Must call `set_up_once()` before operations
- Canonical representation not always enforced
- Host fallbacks for testing without custom instructions

## Performance
- Custom RISC-V instructions for acceleration
- 2-bit window exponentiation
- In-place operations reduce allocations
- Hardware modular reduction

## Related Components
- `openvm-algebra-circuit` - Circuit implementations
- `openvm-ecc-guest` - Elliptic curve operations
- `openvm-pairing-guest` - Pairing operations
- `openvm-native-recursion` - Recursive verification