# BigInt Primitives - Examples

This document provides concrete examples of using the BigInt Primitives component for various cryptographic and arithmetic operations.

## Basic OverflowInt Operations

### Creating OverflowInt from BigUint

```rust
use num_bigint::BigUint;
use crate::bigint::OverflowInt;

// Create from a large integer
let value = BigUint::parse_bytes(b"12345678901234567890123456789012345678901234567890", 10).unwrap();
let limb_bits = 10;
let min_limbs = Some(32); // For 256-bit numbers

// Create OverflowInt representation
let overflow_int = OverflowInt::from_biguint(&value, limb_bits, min_limbs);

println!("Number of limbs: {}", overflow_int.num_limbs());
println!("Max overflow bits: {}", overflow_int.max_overflow_bits());
```

### Arithmetic Operations

```rust
// Addition
let a = OverflowInt::from_biguint(&a_value, 10, Some(32));
let b = OverflowInt::from_biguint(&b_value, 10, Some(32));
let sum = a + b;

// Subtraction
let diff = a - b;

// Multiplication (results in longer limb array)
let product = a * b;
println!("Product has {} limbs", product.num_limbs()); // ~64 limbs for 32-limb inputs

// Scalar operations
let doubled = a.int_mul(2, |x| x as isize);
let offset = a.int_add(100, |x| x as isize);
```

### Carry Generation

```rust
// After arithmetic operations, generate carries for constraint checking
let limb_bits = 10;
let carries = sum.calculate_carries(limb_bits);

println!("Carries: {:?}", carries);
// Carries represent how much each limb overflows into the next limb
```

## Modular Arithmetic Examples

### RSA Modular Multiplication

```rust
use num_bigint::BigUint;
use crate::bigint::{OverflowInt, check_carry_mod_to_zero::*};

// RSA-2048 example parameters
let n = BigUint::parse_bytes(b"25195908475657893494027183240048398571429282126204032027777137836043662020707595556264018525880784406918290641249515082189298559149176184502808489120072844992687392807287776735971418347270261896375014971824691165077613379859095700097330459748808428401797429100642458691817195118746121515172654632282216869987549182422433637259085141865462043576798423387184774447920739934236584823824281198163815010674810451660377306056201619676256133844143603833904414952634432190114657544454178424020924616515723350778707749817125772467962926386356373289912154831438167899885040445364023527381951378636564391212010397122822120720357", 10).unwrap();

let a = BigUint::parse_bytes(b"12345678901234567890123456789012345678901234567890", 10).unwrap();
let b = BigUint::parse_bytes(b"98765432109876543210987654321098765432109876543210", 10).unwrap();

// Compute a * b mod n
let product = (&a * &b) % &n;
let quotient = (&a * &b) / &n;

// Convert to OverflowInt representation
let limb_bits = 10;
let num_limbs = 256; // For 2048-bit RSA

let a_overflow = OverflowInt::from_biguint(&a, limb_bits, Some(num_limbs));
let b_overflow = OverflowInt::from_biguint(&b, limb_bits, Some(num_limbs));
let r_overflow = OverflowInt::from_biguint(&product, limb_bits, Some(num_limbs));
let q_overflow = OverflowInt::from_biguint(&quotient, limb_bits, Some(num_limbs));

// Create expression: a*b - r - q*n = 0
let n_overflow = OverflowInt::from_biguint(&n, limb_bits, Some(num_limbs));
let expr = a_overflow * b_overflow - r_overflow - q_overflow * n_overflow;

// This expression should equal zero
let carries = expr.calculate_carries(limb_bits);
println!("Final carry should be 0: {}", carries.last().unwrap());
```

### Elliptic Curve Field Arithmetic

```rust
// Example: Secp256k1 field operations
// Field modulus: p = 2^256 - 2^32 - 2^9 - 2^8 - 2^7 - 2^6 - 2^4 - 1

let p_hex = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F";
let p = BigUint::parse_bytes(p_hex.as_bytes(), 16).unwrap();

let a = BigUint::parse_bytes(b"123456789ABCDEF123456789ABCDEF123456789ABCDEF123456789ABCDEF", 16).unwrap();
let b = BigUint::parse_bytes(b"FEDCBA9876543210FEDCBA9876543210FEDCBA9876543210FEDCBA9876543210", 16).unwrap();

// Field multiplication: (a * b) mod p
let product_mod_p = (&a * &b) % &p;
let quotient = (&a * &b) / &p;

// Convert to limb representation (32 limbs for 256-bit field)
let limb_bits = 8;
let num_limbs = 32;

let a_limbs = OverflowInt::from_biguint(&a, limb_bits, Some(num_limbs));
let b_limbs = OverflowInt::from_biguint(&b, limb_bits, Some(num_limbs));
let r_limbs = OverflowInt::from_biguint(&product_mod_p, limb_bits, Some(num_limbs));
let q_limbs = OverflowInt::from_biguint(&quotient, limb_bits, Some(num_limbs));
let p_limbs = OverflowInt::from_biguint(&p, limb_bits, Some(num_limbs));

// Constraint: a*b - r - q*p = 0
let constraint_expr = a_limbs * b_limbs - r_limbs - q_limbs * p_limbs;
let carries = constraint_expr.calculate_carries(limb_bits);

// Verify the constraint is satisfied
assert_eq!(*carries.last().unwrap(), 0, "Modular multiplication constraint failed");
```

## SubAir Integration Examples

### CheckCarryToZeroSubAir Usage

```rust
use openvm_stark_backend::p3_field::AbstractField;
use crate::bigint::check_carry_to_zero::*;

// In your AIR implementation
impl<AB: InteractionBuilder> Air<AB> for YourBigIntAir {
    fn eval(&self, builder: &mut AB) {
        // Get your trace columns
        let is_valid = builder.main().row_slice(0)[self.is_valid_col];
        let limb_cols: Vec<AB::Var> = (0..self.num_limbs)
            .map(|i| builder.main().row_slice(0)[self.limb_start_col + i])
            .collect();
        let carry_cols: Vec<AB::Var> = (0..self.num_limbs)
            .map(|i| builder.main().row_slice(0)[self.carry_start_col + i])
            .collect();

        // Create symbolic OverflowInt from trace columns
        let limb_exprs: Vec<AB::Expr> = limb_cols.iter().map(|&x| x.into()).collect();
        let overflow_expr = OverflowInt::from_canonical_unsigned_limbs(
            limb_exprs, 
            self.limb_bits
        );

        // Apply SubAir constraint
        let subair = CheckCarryToZeroSubAir::new(
            self.limb_bits,
            self.range_checker_bus,
            self.decomp
        );
        
        subair.eval(
            builder,
            (overflow_expr, CheckCarryToZeroCols { carries: carry_cols }, is_valid)
        );
    }
}
```

### CheckCarryModToZeroSubAir Usage

```rust
use crate::bigint::check_carry_mod_to_zero::*;

// For modular arithmetic constraints
impl<AB: InteractionBuilder> Air<AB> for ModularMultiplyAir {
    fn eval(&self, builder: &mut AB) {
        let is_valid = builder.main().row_slice(0)[self.is_valid_col];
        
        // Input columns: a, b, result, quotient
        let a_cols = self.get_limb_columns(builder, self.a_start_col);
        let b_cols = self.get_limb_columns(builder, self.b_start_col);
        let r_cols = self.get_limb_columns(builder, self.r_start_col);
        let q_cols = self.get_limb_columns(builder, self.q_start_col);
        let carry_cols = self.get_carry_columns(builder);

        // Create symbolic expressions
        let a_expr = OverflowInt::from_canonical_unsigned_limbs(
            a_cols.iter().map(|&x| x.into()).collect(),
            self.limb_bits
        );
        let b_expr = OverflowInt::from_canonical_unsigned_limbs(
            b_cols.iter().map(|&x| x.into()).collect(),
            self.limb_bits
        );
        let r_expr = OverflowInt::from_canonical_unsigned_limbs(
            r_cols.iter().map(|&x| x.into()).collect(),
            self.limb_bits
        );

        // Expression: a*b - r (should be 0 mod p)
        let expr = a_expr * b_expr - r_expr;

        // Apply modular constraint
        let subair = CheckCarryModToZeroSubAir::new(
            self.modulus.clone(),
            self.limb_bits,
            self.range_checker_bus,
            self.decomp
        );

        subair.eval(
            builder,
            (expr, CheckCarryModToZeroCols { 
                quotient: q_cols, 
                carries: carry_cols 
            }, is_valid)
        );
    }
}
```

## Testing Examples

### Unit Test for Arithmetic Correctness

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use rand::Rng;

    #[test]
    fn test_modular_multiplication_random() {
        let mut rng = rand::thread_rng();
        
        // Generate random test cases
        for _ in 0..100 {
            let a = BigUint::from(rng.gen::<u128>());
            let b = BigUint::from(rng.gen::<u128>());
            let p = BigUint::parse_bytes(b"340282366920938463463374607431768211507", 10).unwrap(); // 2^128 - 61
            
            let expected = (&a * &b) % &p;
            let quotient = (&a * &b) / &p;
            
            // Test our constraint
            let limb_bits = 8;
            let num_limbs = 16;
            
            let a_overflow = OverflowInt::from_biguint(&a, limb_bits, Some(num_limbs));
            let b_overflow = OverflowInt::from_biguint(&b, limb_bits, Some(num_limbs));
            let r_overflow = OverflowInt::from_biguint(&expected, limb_bits, Some(num_limbs));
            let q_overflow = OverflowInt::from_biguint(&quotient, limb_bits, Some(num_limbs));
            let p_overflow = OverflowInt::from_biguint(&p, limb_bits, Some(num_limbs));
            
            let constraint = a_overflow * b_overflow - r_overflow - q_overflow * p_overflow;
            let carries = constraint.calculate_carries(limb_bits);
            
            assert_eq!(*carries.last().unwrap(), 0, "Constraint should be satisfied");
        }
    }

    #[test]
    fn test_overflow_bounds() {
        let a = OverflowInt::from_canonical_unsigned_limbs(vec![255isize; 32], 8);
        let b = OverflowInt::from_canonical_unsigned_limbs(vec![255isize; 32], 8);
        
        let sum = a + b;
        assert_eq!(sum.limb_max_abs(), 510);
        assert_eq!(sum.max_overflow_bits(), 9);
        
        let product = sum * OverflowInt::from_canonical_unsigned_limbs(vec![100isize; 32], 8);
        assert_eq!(product.limb_max_abs(), 510 * 100 * 32); // Convolution factor
    }
}
```

### Integration Test with Real Primes

```rust
#[test]
fn test_with_real_world_primes() {
    // Test with common cryptographic primes
    let test_cases = vec![
        // Mersenne prime 2^127 - 1
        ("170141183460469231731687303715884105727", 127),
        // Secp256k1 field prime
        ("115792089237316195423570985008687907853269984665640564039457584007913129639935", 256),
        // BLS12-381 field prime  
        ("4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559787", 381),
    ];
    
    for (prime_str, bit_size) in test_cases {
        let p = BigUint::parse_bytes(prime_str.as_bytes(), 10).unwrap();
        let limb_bits = 8;
        let num_limbs = (bit_size + limb_bits - 1) / limb_bits;
        
        // Test random field elements
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let a_bytes: Vec<u8> = (0..num_limbs).map(|_| rng.gen()).collect();
            let b_bytes: Vec<u8> = (0..num_limbs).map(|_| rng.gen()).collect();
            
            let a = BigUint::from_bytes_le(&a_bytes) % &p;
            let b = BigUint::from_bytes_le(&b_bytes) % &p;
            
            let result = (&a * &b) % &p;
            let quotient = (&a * &b) / &p;
            
            // Verify constraint
            let a_limbs = OverflowInt::from_biguint(&a, limb_bits, Some(num_limbs));
            let b_limbs = OverflowInt::from_biguint(&b, limb_bits, Some(num_limbs));
            let r_limbs = OverflowInt::from_biguint(&result, limb_bits, Some(num_limbs));
            let q_limbs = OverflowInt::from_biguint(&quotient, limb_bits, Some(num_limbs));
            let p_limbs = OverflowInt::from_biguint(&p, limb_bits, Some(num_limbs));
            
            let constraint = a_limbs * b_limbs - r_limbs - q_limbs * p_limbs;
            let carries = constraint.calculate_carries(limb_bits);
            
            assert_eq!(*carries.last().unwrap(), 0, 
                "Failed for prime {} with bit size {}", prime_str, bit_size);
        }
    }
}
```

## Performance Benchmarking

### Carry Generation Benchmark

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_carry_generation() {
        let sizes = vec![32, 64, 128, 256]; // Different numbers of limbs
        let limb_bits = 8;
        
        for num_limbs in sizes {
            let a = OverflowInt::from_canonical_unsigned_limbs(vec![255isize; num_limbs], limb_bits);
            let b = OverflowInt::from_canonical_unsigned_limbs(vec![200isize; num_limbs], limb_bits);
            
            let product = a * b;
            
            let start = Instant::now();
            let carries = product.calculate_carries(limb_bits);
            let duration = start.elapsed();
            
            println!("Carry generation for {} limbs: {:?}", num_limbs, duration);
            println!("Number of carries: {}", carries.len());
        }
    }
}
```

These examples demonstrate the key usage patterns for the BigInt Primitives component, from basic arithmetic operations to complex cryptographic applications and testing strategies.