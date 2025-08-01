# OpenVM Algebra Moduli Macros - Detailed Documentation

## Overview
This crate provides procedural macros for generating modular arithmetic types in OpenVM. It creates efficient implementations that work both in standard Rust environments and within the zkVM, utilizing custom instructions for optimal performance.

## Core Macros

### `moduli_declare!`
Generates modular arithmetic types with specified moduli.

```rust
moduli_declare! {
    Bls12381 { modulus = "0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab" },
    Bn254 { modulus = "21888242871839275222246405745257275088696311157297823662689037894645226208583" },
}
```

#### Generated Types
For each declared modulus, the macro generates:
- A struct representing the modular integer type
- Arithmetic trait implementations (`Add`, `Sub`, `Mul`, `Neg`)
- `IntMod` trait implementation for modular arithmetic operations
- `Field` and `Sqrt` traits for prime moduli
- Serialization support via `serde`

#### Type Structure
```rust
#[repr(C, align(block_size))]
pub struct ModulusName([u8; limbs]);
```
- Aligned to 16 or 32 bytes for performance
- Internal representation as fixed-size byte array
- Supports up to 48-byte (384-bit) moduli

### `moduli_init!`
Initializes modular arithmetic for zkVM execution.

```rust
moduli_init! {
    "modulus_hex_1",
    "modulus_hex_2",
}
```

Generates:
- Static variables in `.openvm` section for ELF extraction
- Extern function definitions for zkVM operations
- Setup functions for runtime modulus configuration

## Implementation Details

### Arithmetic Operations

#### Standard Rust Mode
- Uses `num_bigint` for modular arithmetic
- Operations performed as: `(a op b) % modulus`
- Modular inverse computed using extended GCD

#### zkVM Mode
- Utilizes custom RISC-V instructions for efficiency
- Operations map to specialized opcodes:
  - `AddMod`, `SubMod`, `MulMod`, `DivMod`
  - `IsEqMod` for equality checking
  - `SetupMod` for modulus configuration

### Memory Layout
- **32-byte moduli**: 32-byte alignment, single block
- **33-48 byte moduli**: 16-byte alignment, may span blocks
- Little-endian byte representation

### Field Operations (Prime Moduli Only)

#### Square Root Computation
For prime moduli, implements quadratic residue checking and square root:
1. Host provides hint for square root
2. Guest verifies: `sqrt² ≡ input (mod p)`
3. For non-residues, verifies using quadratic non-residue

#### Quadratic Non-Residue
- Generated via hint system
- Verified using Euler's criterion: `a^((p-1)/2) ≡ -1 (mod p)`
- Cached using `once_cell` for efficiency

## Safety and Security

### Canonical Form
- Elements may not be in reduced form (< modulus)
- `assert_reduced()` enforces canonical representation
- `is_reduced()` checks without asserting

### Division Safety
- `div_unsafe` operations assume denominator is coprime to modulus
- Undefined behavior if denominator shares factors with modulus
- Use with caution in cryptographic contexts

### Host Dishonesty Protection
- Square root hints verified before use
- Invalid hints trigger infinite loops (proof failure)
- Ensures soundness even with malicious hosts

## Performance Considerations

### Setup Once Pattern
- Modulus setup performed once per type
- Cached using atomic boolean flag
- No overhead after initial setup

### Instruction Batching
- Multiple operations can be chained efficiently
- In-place operations minimize allocations
- Reference implementations avoid unnecessary copies

## Integration with OpenVM

### Custom Instructions
Maps to OpenVM algebra extension opcodes:
- Opcode: `OPCODE` (algebra extension)
- Funct3: `MODULAR_ARITHMETIC_FUNCT3`
- Funct7: Operation type + modulus index offset

### ELF Integration
- Modulus data stored in `.openvm` section
- Extracted during transpilation
- Used to configure VM arithmetic chips

## Error Handling

### Compile-Time Errors
- Invalid modulus format
- Modulus too large (>48 bytes)
- Malformed macro invocation

### Runtime Errors (zkVM)
- Invalid hints cause proof failure
- Dishonest host detection
- Setup verification failures