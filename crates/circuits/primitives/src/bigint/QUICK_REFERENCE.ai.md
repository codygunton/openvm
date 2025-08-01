# BigInt Primitives Quick Reference

## OverflowInt Construction
```rust
// From unsigned limbs (non-negative)
let overflow_int = OverflowInt::from_canonical_unsigned_limbs(limbs, limb_bits);

// From signed limbs (can be negative)
let overflow_int = OverflowInt::from_canonical_signed_limbs(limbs, limb_bits);

// From BigUint
let overflow_int = OverflowInt::from_biguint(&big_uint, limb_bits, None);

// With minimum limbs (padded)
let overflow_int = OverflowInt::from_biguint(&big_uint, limb_bits, Some(32));

// Manual construction
let overflow_int = OverflowInt::from_computed_limbs(limbs, max_abs, overflow_bits);
```

## Arithmetic Operations
```rust
// Addition
let sum = overflow_a + overflow_b;

// Subtraction (can produce negative limbs)
let diff = overflow_a - overflow_b;

// Multiplication
let product = overflow_a * overflow_b;

// Integer operations
let plus_one = overflow.int_add(1, |x| AB::Expr::from_canonical_usize(x));
let scaled = overflow.int_mul(5, |x| AB::Expr::from_canonical_usize(x));
```

## Carry Generation
```rust
// For trace generation (isize limbs)
let carries = overflow_int.calculate_carries(limb_bits);

// Manual carry calculation
let mut carry = 0;
for i in 0..limbs.len() {
    carry = (carry + limbs[i]) >> limb_bits; // arithmetic right shift
    carries.push(carry);
}
```

## SubAir Usage
```rust
// CheckCarryToZero - constrain to zero
let subair = CheckCarryToZeroSubAir::new(limb_bits, range_bus, decomp);
let cols = CheckCarryToZeroCols { carries };
subair.eval(builder, (overflow_expr, cols, is_valid));

// CheckCarryModToZero - modular constraint  
let subair = CheckCarryModToZeroSubAir::new(modulus, limb_bits, range_bus, decomp);
let cols = CheckCarryModToZeroCols { carries, quotient };
subair.eval(builder, (overflow_expr, cols, is_valid));
```

## Common Patterns
```rust
// Modular multiplication: a * b ≡ r (mod p)
let expr = overflow_a * overflow_b - overflow_r - overflow_q * overflow_p;
check_carry_to_zero(expr);

// Modular addition: a + b ≡ r (mod p)
let expr = overflow_a + overflow_b - overflow_r - overflow_q * overflow_p;
check_carry_to_zero(expr);

// Modular inverse: a * a_inv ≡ 1 (mod p)
let one = OverflowInt::from_canonical_unsigned_limbs(vec![1], limb_bits);
let expr = overflow_a * overflow_inv - one - overflow_q * overflow_p;
check_carry_to_zero(expr);
```

## Range Checking
```rust
// Range check a value
range_check(builder, range_bus, decomp, bits, value, count);

// Range check with offset (for signed values)
let offset = 1 << (bits - 1);
range_check(builder, range_bus, decomp, bits, value + offset, count);
```

## Constants and Utils
```rust
// Common primes
let secp256k1_p = secp256k1_coord_prime();
let secp256k1_n = secp256k1_scalar_prime();
let secp256r1_p = secp256r1_coord_prime();
let ed25519_p = ed25519_coord_prime();
let bn254_p = bn254_coord_prime();
let bls12_381_p = bls12_381_coord_prime();

// BigUint conversion
let limbs = big_uint_to_limbs(&big_uint, limb_bits);
let limbs_padded = big_uint_to_num_limbs(&big_uint, limb_bits, min_limbs);

// Modular operations
let inv = big_uint_mod_inverse(&a, &modulus);
let product = big_uint_mod_mult(&a, &b, &modulus);
```

## Overflow Tracking
```rust
// Get current bounds
let max_bits = overflow_int.max_overflow_bits();
let max_abs = overflow_int.limb_max_abs();
let num_limbs = overflow_int.num_limbs();

// Access limbs
let limb_i = overflow_int.limb(i);
let all_limbs = overflow_int.limbs();
```

## Typical Parameters
```rust
// For 256-bit integers
const LIMB_BITS: usize = 8;     // 32 limbs
const LIMB_BITS: usize = 10;    // 26 limbs
const LIMB_BITS: usize = 16;    // 16 limbs

// Range checker decomposition
const DECOMP: usize = 8;         // Common choice
const DECOMP: usize = 16;        // For larger ranges

// Carry bits calculation
let (carry_offset, carry_bits) = get_carry_max_abs_and_bits(overflow_bits, limb_bits);
```

## Debug Helpers
```rust
// Print BigUint as hex
println!("Value: 0x{:x}", big_uint);

// Check overflow safety
assert!(overflow_int.max_overflow_bits() < F::bits() - 1);

// Validate carries
let computed = overflow_int.calculate_carries(limb_bits);
assert_eq!(computed, expected_carries);
```

## Performance Tips
```rust
// Batch operations before carry
let expr = a * b + c * d - e * f;  // Single carry check

// Reuse carry columns across rows
// Share range checker bus across SubAirs
// Use native field ops for small values
if value < (1 << 30) {
    // Use field arithmetic directly
}
```

## Common Errors
```rust
// ❌ Wrong: regular division for carries
carry = limb / (1 << limb_bits);

// ✅ Correct: arithmetic right shift
carry = limb >> limb_bits;

// ❌ Wrong: forgetting overflow update
let result = OverflowInt { limbs: new_limbs, ..self };

// ✅ Correct: update all fields
let result = OverflowInt {
    limbs: new_limbs,
    limb_max_abs: new_max,
    max_overflow_bits: log2_ceil_usize(new_max),
};
```