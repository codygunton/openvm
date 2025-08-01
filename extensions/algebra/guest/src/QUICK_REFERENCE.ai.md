# OpenVM Algebra Guest Quick Reference

## Imports

```rust
use openvm_algebra_guest::{
    IntMod, Field, FieldExtension, ComplexConjugate,
    DivUnsafe, DivAssignUnsafe, Reduce, Sqrt, ExpBytes,
    ModArithBaseFunct7, ComplexExtFieldBaseFunct7,
    OPCODE, MODULAR_ARITHMETIC_FUNCT3, COMPLEX_EXT_FIELD_FUNCT3
};
```

## Basic Operations

```rust
// Construction
let a = MyField::from_u32(42);
let b = MyField::from_le_bytes(&[1, 2, 3, 4]).unwrap();
let c = MyField::from_repr(bytes_array);

// Arithmetic
let sum = a + b;           // Addition mod p
let diff = a - b;          // Subtraction mod p  
let prod = a * b;          // Multiplication mod p
let quot = a.div_unsafe(b); // Division (undefined if b = 0)
let neg = -a;              // Negation mod p

// Special operations
let doubled = a.double();   // 2 * a
let squared = a.square();   // a²
let cubed = a.cube();      // a³
let inv = a.invert();      // a⁻¹ (panics if a = 0)

// In-place operations
let mut x = a;
x += b;              // x = x + b
x *= b;              // x = x * b
x.double_assign();   // x = 2 * x
x.square_assign();   // x = x²
```

## Exponentiation

```rust
// Exponentiation by bytes (big-endian)
let base = MyField::from_u32(3);
let exp = [0x01, 0x23, 0x45];  // Big-endian bytes

// Positive exponent
let result = base.exp_bytes(true, &exp);  // 3^0x012345

// Negative exponent (computes base^(-exp))
let inv_result = base.exp_bytes(false, &exp);  // 3^(-0x012345)
```

## Field Extensions

```rust
// Quadratic extension Fp2
let a = Fp::from_u32(3);
let b = Fp::from_u32(4);

// Create Fp2 element
let z = Fp2::from_coeffs([a, b]);  // 3 + 4i

// Operations
let w = z.conjugate();              // 3 - 4i
let f = z.frobenius_map(1);        // Frobenius endomorphism
let scaled = z.mul_base(&a);       // (3 + 4i) * 3

// Convert back
let [c0, c1] = z.to_coeffs();     // c0 = 3, c1 = 4
```

## Modular Reduction

```rust
use openvm_algebra_guest::Reduce;

// Reduce arbitrary bytes modulo p
let bytes = [0xFF; 64];  // 512-bit value
let reduced = MyField::reduce_le_bytes(&bytes);

// Big-endian version
let reduced_be = MyField::reduce_be_bytes(&bytes);
```

## Square Roots

```rust
use openvm_algebra_guest::Sqrt;

let a = MyField::from_u32(4);
match a.sqrt() {
    Some(root) => {
        assert_eq!(root.square(), a);
    }
    None => {
        // a is not a quadratic residue
    }
}
```

## Canonical Representation

```rust
// Check if reduced
let a = MyField::from_u32(5);
assert!(a.is_reduced());  // true

// Force canonical form
let b = a + MyField::MODULUS;  // b ≡ 5 (mod p) but not reduced
b.assert_reduced();  // Panics!

// Safe checking
if !b.is_reduced() {
    // Handle non-canonical representation
}
```

## Setup and Initialization

```rust
// Call once before using a modular type
MyField::set_up_once();  // Idempotent

// Setup is automatic in most operations
let a = MyField::ONE;  // Calls setup internally if needed
```

## Constants

```rust
// Every IntMod type has these
let zero = MyField::ZERO;       // Additive identity
let one = MyField::ONE;         // Multiplicative identity  
let modulus = MyField::MODULUS; // As byte array
let limbs = MyField::NUM_LIMBS; // Size of representation
```

## Instruction Constants

```rust
// Custom RISC-V opcode
const ALGEBRA_OPCODE: u8 = 0x2b;

// Function codes
const MOD_ARITH_FUNCT3: u8 = 0b000;
const COMPLEX_FIELD_FUNCT3: u8 = 0b010;

// Operations
ModArithBaseFunct7::AddMod    // 0
ModArithBaseFunct7::SubMod    // 1
ModArithBaseFunct7::MulMod    // 2
ModArithBaseFunct7::DivMod    // 3
ModArithBaseFunct7::IsEqMod   // 4
ModArithBaseFunct7::SetupMod  // 5
ModArithBaseFunct7::HintNonQr // 6
ModArithBaseFunct7::HintSqrt  // 7

ComplexExtFieldBaseFunct7::Add   // 0
ComplexExtFieldBaseFunct7::Sub   // 1
ComplexExtFieldBaseFunct7::Mul   // 2
ComplexExtFieldBaseFunct7::Div   // 3
ComplexExtFieldBaseFunct7::Setup // 4
```

## Common Patterns

```rust
// Sum of field elements
let values = vec![a, b, c, d];
let sum: MyField = values.iter().sum();

// Product of field elements  
let product: MyField = values.iter().product();

// Conditional operations
let result = if condition {
    a + b
} else {
    a - b
};

// Multi-exponentiation
let bases = vec![g1, g2, g3];
let exps = vec![e1, e2, e3];
let result: MyField = bases.iter()
    .zip(&exps)
    .map(|(base, exp)| base.exp_bytes(true, exp))
    .product();
```

## Type Conversions

```rust
// From bytes
let a = MyField::from_le_bytes(&bytes).expect("valid field element");
let b = MyField::from_be_bytes(&bytes).expect("valid field element");

// To bytes
let le_bytes = a.as_le_bytes();  // Reference to internal bytes
let be_bytes = a.to_be_bytes();  // Owned byte array

// From integers
let c = MyField::from_u8(255);
let d = MyField::from_u32(0x12345678);  
let e = MyField::from_u64(0x123456789ABCDEF0);

// BigUint (host only)
#[cfg(not(target_os = "zkvm"))]
{
    let big = a.as_biguint();
    let f = MyField::from_biguint(big);
}
```

## Error Handling

```rust
// Safe division
fn safe_div(a: MyField, b: MyField) -> Option<MyField> {
    if b == MyField::ZERO {
        None
    } else {
        Some(a.div_unsafe(b))
    }
}

// Panicking operations
let inv = a.invert();  // Panics if a = 0
a.assert_reduced();    // Panics if not canonical

// Non-panicking checks
if a.is_reduced() && b != MyField::ZERO {
    let result = a.div_unsafe(b);
}
```

## Performance Tips

1. **Use assignment operations** to avoid allocations:
   ```rust
   x *= y;  // Better than x = x * y
   ```

2. **Batch similar operations** for better cache usage:
   ```rust
   let sums: Vec<_> = pairs.iter()
       .map(|(a, b)| a + b)
       .collect();
   ```

3. **Avoid unnecessary reductions**:
   ```rust
   let temp = a + b;  // May not be reduced
   let result = temp * c;  // Reduction happens here
   ```

4. **Precompute when possible**:
   ```rust
   let powers: Vec<_> = (0..10)
       .map(|i| base.exp_bytes(true, &[i]))
       .collect();
   ```