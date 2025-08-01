# OpenVM Algebra Moduli Macros - Quick Reference

## Common Usage Patterns

### Declaring Modular Arithmetic Types
```rust
use openvm_algebra_moduli_macros::moduli_declare;

moduli_declare! {
    // Hex format (with 0x prefix)
    Bls12381 { modulus = "0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab" },
    
    // Decimal format
    Bn254 { modulus = "21888242871839275222246405745257275088696311157297823662689037894645226208583" },
    
    // Multiple moduli in one declaration
    Secp256k1 { modulus = "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f" },
}
```

### Using Generated Types
```rust
// Construction
let a = Bn254::from_u32(42);
let b = Bn254::from_le_bytes(&bytes).unwrap();
let c = Bn254::from_be_bytes(&bytes).unwrap();

// Arithmetic
let sum = &a + &b;        // Addition
let diff = &a - &b;       // Subtraction  
let prod = &a * &b;       // Multiplication
let quot = a.div_unsafe(b); // Division (b must be coprime to modulus)

// Field operations (prime moduli only)
let squared = a.square();
let doubled = a.double();
let cubed = a.cube();
let negated = -&a;

// Square root (prime moduli only)
match a.sqrt() {
    Some(root) => println!("Square root exists"),
    None => println!("Not a quadratic residue"),
}
```

### Initialization for zkVM
```rust
use openvm_algebra_moduli_macros::moduli_init;

// Initialize with modulus hex strings
moduli_init! {
    "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab",
    "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47",
}
```

## Key APIs

### IntMod Trait Methods
```rust
// Constants
T::ZERO           // Additive identity
T::ONE            // Multiplicative identity
T::MODULUS        // Modulus as byte array

// Conversions
T::from_u8(val)
T::from_u32(val)
T::from_u64(val)
T::from_repr(bytes)
T::as_le_bytes()
T::to_be_bytes()

// Validation
elem.is_reduced()     // Check if < modulus
elem.assert_reduced() // Assert canonical form
```

### Field Trait Methods (Prime Moduli)
```rust
// In-place operations
elem.double_assign()  // elem *= 2
elem.square_assign()  // elem *= elem
elem.neg_assign()     // elem = -elem

// Non-mutating operations
elem.double()         // 2 * elem
elem.square()         // elem * elem
elem.cube()          // elem * elem * elem
```

## Performance Tips

### Use References for Operations
```rust
// Good - no unnecessary clones
let result = &a + &b;

// Less efficient - clones a
let result = a + &b;
```

### Batch Operations
```rust
// Efficient - single setup check
let sum: T = values.iter().sum();

// Also efficient
let product: T = values.iter().product();
```

### In-Place Operations
```rust
// Most efficient for accumulation
let mut acc = T::ZERO;
for val in values {
    acc += val;  // or acc.add_assign(val)
}
```

## Common Patterns

### Modular Exponentiation
```rust
use openvm_algebra_guest::ExpBytes;

let base = Bn254::from_u32(2);
let exp_bytes = exp.to_be_bytes();
let result = base.exp_bytes(true, &exp_bytes);
```

### Batch Verification
```rust
// Verify multiple elements are reduced
for elem in elements {
    elem.assert_reduced();
}
```

### Safe Division
```rust
// Check coprimality before division
if gcd(&a, &modulus) == 1 {
    let result = a.div_unsafe(b);
} else {
    // Handle error case
}
```

## zkVM-Specific Considerations

### Setup Happens Automatically
- First operation triggers setup
- No manual initialization needed
- Setup cached for efficiency

### Hint Verification
- Square roots always verified
- Invalid hints cause proof failure
- Trust model: honest host assumed

### Memory Alignment
- Types are properly aligned
- Direct pointer operations safe
- Efficient custom instruction use

## Debugging Tips

### Check Canonical Form
```rust
if !elem.is_reduced() {
    println!("Warning: element not in canonical form");
    elem.assert_reduced(); // Force canonicalization
}
```

### Verify Modulus Size
- Max 48 bytes (384 bits)
- Common sizes: 32, 48 bytes
- Alignment: 32 or 16 bytes

### Test Both Environments
```rust
#[cfg(not(target_os = "zkvm"))]
{
    // Native testing code
}

#[cfg(target_os = "zkvm")]
{
    // zkVM-specific code
}
```