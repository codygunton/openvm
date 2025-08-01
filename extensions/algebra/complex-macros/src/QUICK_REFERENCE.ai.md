# Complex Macros - Quick Reference

## Basic Usage

### Declaration
```rust
use openvm_algebra_complex_macros::complex_declare;
use openvm_algebra_moduli_macros::moduli_declare;

// First declare modular types
moduli_declare! {
    Fq { modulus = "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001" },
    Fr { modulus = "0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47" }
}

// Then declare complex types
complex_declare! {
    Fq2 { mod_type = Fq },
    Fr2 { mod_type = Fr }
}
```

### Initialization
```rust
// In main() or entry point
use openvm_algebra_complex_macros::complex_init;
use openvm_algebra_moduli_macros::moduli_init;

// Initialize moduli first
moduli_init! {
    "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001",
    "0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47"
}

// Then initialize complex fields with modulus indices
complex_init! {
    Fq2 { mod_idx = 0 },  // Uses first modulus (Fq)
    Fr2 { mod_idx = 1 }   // Uses second modulus (Fr)
}
```

### Field Trait Implementation
```rust
use openvm_algebra_complex_macros::complex_impl_field;

// Implement Field trait for complex types
complex_impl_field!(Fq2, Fr2);
```

## Common Operations

### Construction
```rust
// Create complex numbers
let zero = Fq2::ZERO;
let one = Fq2::ONE;
let z = Fq2::new(Fq::from_u32(3), Fq::from_u32(4));  // 3 + 4i
```

### Arithmetic
```rust
let a = Fq2::new(Fq::from_u32(1), Fq::from_u32(2));  // 1 + 2i
let b = Fq2::new(Fq::from_u32(3), Fq::from_u32(4));  // 3 + 4i

// Basic operations
let sum = &a + &b;        // 4 + 6i
let diff = &a - &b;       // -2 - 2i
let prod = &a * &b;       // -5 + 10i (since i² = -1)
let quot = &a / &b;       // 0.44 + 0.08i

// In-place operations
let mut c = a.clone();
c += &b;                  // c = 4 + 6i
c -= &b;                  // c = 1 + 2i
c *= &b;                  // c = -5 + 10i
c /= &b;                  // c = 0.44 + 0.08i
```

### Complex-Specific Operations
```rust
use openvm_algebra_guest::field::ComplexConjugate;

let z = Fq2::new(Fq::from_u32(3), Fq::from_u32(4));  // 3 + 4i

// Conjugation
let conj = z.conjugate();     // 3 - 4i
let mut z2 = z.clone();
z2.conjugate_assign();        // z2 = 3 - 4i

// Negation
let neg = -&z;                // -3 - 4i
let neg2 = -z.clone();        // -3 - 4i

// Field operations (if complex_impl_field! was used)
let mut z3 = z.clone();
z3.double_assign();           // z3 = 6 + 8i
z3.square_assign();           // z3 = (6 + 8i)² = -28 + 96i
```

### Iterator Operations
```rust
let values = vec![
    Fq2::new(Fq::from_u32(1), Fq::from_u32(0)),
    Fq2::new(Fq::from_u32(2), Fq::from_u32(1)),
    Fq2::new(Fq::from_u32(3), Fq::from_u32(2)),
];

// Sum
let sum: Fq2 = values.iter().sum();      // 6 + 3i
let sum2: Fq2 = values.into_iter().sum(); // 6 + 3i

// Product
let values2 = vec![
    Fq2::new(Fq::from_u32(2), Fq::from_u32(0)),
    Fq2::new(Fq::from_u32(3), Fq::from_u32(0)),
];
let prod: Fq2 = values2.iter().product();  // 6 + 0i
```

## Complete Example

```rust
#![cfg_attr(not(feature = "std"), no_main)]
#![cfg_attr(not(feature = "std"), no_std)]

use openvm_algebra_guest::IntMod;
use openvm_algebra_guest::field::{ComplexConjugate, Field};

// Declare types
openvm_algebra_moduli_macros::moduli_declare! {
    Fq { modulus = "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001" }
}

openvm_algebra_complex_macros::complex_declare! {
    Fq2 { mod_type = Fq }
}

openvm_algebra_complex_macros::complex_impl_field!(Fq2);

openvm::entry!(main);

pub fn main() {
    // Initialize
    openvm_algebra_moduli_macros::moduli_init! {
        "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001"
    }
    openvm_algebra_complex_macros::complex_init! {
        Fq2 { mod_idx = 0 }
    }
    
    // Use complex arithmetic
    let i = Fq2::new(Fq::ZERO, Fq::ONE);  // 0 + 1i
    let i_squared = &i * &i;               // Should be -1 + 0i
    assert_eq!(i_squared, Fq2::new(-Fq::ONE, Fq::ZERO));
    
    // Quadratic formula example
    let a = Fq2::new(Fq::from_u32(1), Fq::ZERO);
    let b = Fq2::new(Fq::from_u32(2), Fq::ZERO);
    let c = Fq2::new(Fq::from_u32(5), Fq::ZERO);
    
    // x = (-b ± √(b² - 4ac)) / 2a
    let discriminant = &b * &b - &Fq2::new(Fq::from_u32(4), Fq::ZERO) * &a * &c;
    // discriminant = 4 - 20 = -16 = 16i²
    
    // Would need square root implementation for complete example
}
```

## Macro Quick Reference

### complex_declare!
```rust
complex_declare! {
    TypeName { mod_type = BaseModularType },
    // ... more types
}
```

**Generated**:
- Struct `TypeName` with fields `c0`, `c1`
- Constants: `ZERO`, `ONE`
- Constructor: `new(c0, c1)`
- Arithmetic: `+`, `-`, `*`, `/` (and `+=`, `-=`, `*=`, `/=`)
- Traits: `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`
- `ComplexConjugate` implementation
- `Sum` and `Product` for iterators
- `Neg` for negation
- `Debug` formatting

### complex_init!
```rust
complex_init! {
    TypeName { mod_idx = 0 },
    // ... more types with their modulus indices
}
```

**Requirements**:
- Must be called after `moduli_init!`
- `mod_idx` must match position in `moduli_init!`

**Generated**:
- FFI functions for zkVM execution
- No-op in native execution

### complex_impl_field!
```rust
complex_impl_field!(Type1, Type2, ...);
```

**Generated**:
- `Field` trait implementation
- Methods: `double_assign()`, `square_assign()`

## Type Requirements

### Base Field Requirements
The base modular type must implement:
- `IntMod` trait
- `Clone`
- Standard arithmetic operations
- `Serialize`/`Deserialize` (from serde)

### Mathematical Requirements
- Base field characteristic p ≡ 3 (mod 4)
- This ensures -1 is not a quadratic residue
- Required for X² + 1 to be irreducible

## Performance Notes

### Native Execution
- Direct arithmetic on components
- No setup overhead
- Efficient reference-based operations

### zkVM Execution
- One-time setup per complex type
- External function calls for arithmetic
- Memory-efficient with `MaybeUninit`
- Lazy initialization with thread safety

## Common Patterns

### Working with References
```rust
// Prefer reference operations to avoid clones
let result = &a + &b;  // Better than a + b

// Chain operations
let result = &(&a + &b) * &c;
```

### Batch Operations
```rust
// Sum many values efficiently
let values: Vec<Fq2> = generate_values();
let sum: Fq2 = values.iter().sum();

// Product of many values
let product: Fq2 = values.iter().product();
```

### Conversion from Base Field
```rust
// Convert base field element to complex
let real = Fq::from_u32(42);
let complex = Fq2::new(real, Fq::ZERO);  // 42 + 0i
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "mod_type parameter is required" | Add `mod_type = YourModularType` to `complex_declare!` |
| "mod_idx is required" | Add `mod_idx = n` to `complex_init!` |
| Type mismatch errors | Ensure base types match between declaration and usage |
| Arithmetic incorrect | Check modulus is prime with p ≡ 3 (mod 4) |
| Performance issues | Use reference operations (`&a + &b`) |

## See Also

- `openvm-algebra-moduli-macros` - For declaring base modular types
- `openvm-algebra-guest` - Runtime traits and utilities
- `openvm-algebra-circuit` - Circuit implementation details
- `openvm-algebra-transpiler` - Instruction definitions