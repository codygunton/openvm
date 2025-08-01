# OpenVM Algebra Guest Component Instructions

## Overview

The OpenVM algebra guest component provides the foundational traits and operations for modular arithmetic and field operations in the zkVM. This is a critical component that other algebraic extensions depend on.

## Key Principles

1. **Trait Consistency**: All algebraic types must properly implement the core traits (IntMod, Field, etc.)
2. **Hardware Acceleration**: Leverage custom RISC-V instructions for performance
3. **Dual-Target Support**: Code must work correctly in both zkVM and host environments
4. **Safety with Performance**: Balance safety checks with zkVM performance requirements

## Code Guidelines

### When Adding New Modular Types

Always implement the full IntMod trait:
```rust
impl IntMod for NewModularType {
    type Repr = [u8; 32];  // Match your modulus size
    type SelfRef<'a> = &'a Self;
    
    const MODULUS: Self::Repr = /* your modulus */;
    const NUM_LIMBS: usize = 32;
    const ZERO: Self = Self(/* zero representation */);
    const ONE: Self = Self(/* one representation */);
    
    // Don't forget setup!
    fn set_up_once() {
        // Use OnceBool to ensure single execution
    }
}
```

### Instruction Encoding

When adding operations that use custom instructions:
```rust
// Always use the defined constants
const FUNCT7: u8 = MOD_IDX * ModArithBaseFunct7::MODULAR_ARITHMETIC_MAX_KINDS 
                 + ModArithBaseFunct7::YourOp as u8;

// Never hardcode opcodes or funct3 values
custom_insn_r!(OPCODE, MODULAR_ARITHMETIC_FUNCT3, FUNCT7, ...)
```

### Canonical Representation

Be explicit about representation requirements:
```rust
// Document when canonical form is required
/// Returns the multiplicative inverse.
/// 
/// # Panics
/// Panics if self is zero or not in canonical form.
fn invert(&self) -> Self {
    self.assert_reduced();  // Explicit check
    // ... implementation
}

// Document when non-canonical is allowed
/// Adds two field elements.
/// 
/// The inputs may be in non-canonical form.
/// The output may be in non-canonical form.
fn add(&self, other: &Self) -> Self {
    // ... implementation
}
```

### Performance Considerations

1. **Avoid unnecessary clones**:
   ```rust
   // Good: Use references
   fn compute(&self, other: &Self) -> Self
   
   // Bad: Unnecessary owned values
   fn compute(self, other: Self) -> Self
   ```

2. **Use assignment operations**:
   ```rust
   // Good: In-place operation
   x.double_assign();
   
   // Less efficient: Creates temporary
   x = x.double();
   ```

3. **Batch operations when possible**:
   ```rust
   // Setup once for multiple operations
   MyField::set_up_once();
   for item in items {
       // Process without repeated setup checks
   }
   ```

### Error Handling

Division and inversion require special care:
```rust
// Always document division safety
/// Divides self by other.
/// 
/// # Safety
/// Undefined behavior if other is not invertible modulo p.
fn div_unsafe(&self, other: &Self) -> Self

// Provide safe alternatives when reasonable
fn checked_div(&self, other: &Self) -> Option<Self> {
    if other == &Self::ZERO {
        None
    } else {
        Some(self.div_unsafe(other))
    }
}
```

### Testing Requirements

All new algebraic types must include:
1. Basic arithmetic tests (add, sub, mul, div)
2. Identity tests (0 + x = x, 1 * x = x)
3. Inverse tests (x * x^(-1) = 1)
4. Edge cases (operations with 0, operations with p-1)
5. Non-canonical representation handling

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_field_axioms() {
        MyField::set_up_once();
        
        let a = MyField::from_u32(5);
        let b = MyField::from_u32(7);
        
        // Commutativity
        assert_eq!(a + b, b + a);
        assert_eq!(a * b, b * a);
        
        // Associativity
        let c = MyField::from_u32(11);
        assert_eq!((a + b) + c, a + (b + c));
        assert_eq!((a * b) * c, a * (b * c));
        
        // Distributivity
        assert_eq!(a * (b + c), a * b + a * c);
    }
}
```

### Documentation Standards

Every public trait implementation should document:
1. The modulus being used
2. Representation format (Montgomery, standard, etc.)
3. Any special properties (is it a prime field?)
4. Performance characteristics
5. Safety requirements

Example:
```rust
/// Finite field with modulus 2^255 - 19.
/// 
/// Elements are stored in little-endian byte arrays using
/// standard (non-Montgomery) representation.
/// 
/// This is a prime field suitable for elliptic curve operations.
pub struct Fp25519([u8; 32]);
```

## Common Pitfalls to Avoid

1. **Forgetting setup**: Always call `set_up_once()` before operations
2. **Assuming canonical form**: Many operations work with non-canonical values
3. **Direct byte comparison**: Use trait equality, not byte equality
4. **Ignoring alignment**: Custom instructions may require aligned data
5. **Host-only features in zkVM path**: Keep BigUint usage behind cfg flags

## Integration Guidelines

When integrating with other components:
1. Implement standard traits (Clone, Debug, PartialEq, Eq)
2. Consider implementing Serialize/Deserialize if needed
3. Provide conversion methods to/from common formats
4. Document the instruction indices used (MOD_IDX, FP2_IDX)

## Performance Optimization Checklist

When optimizing algebraic operations:
- [ ] Use custom instructions where available
- [ ] Minimize modular reductions
- [ ] Batch inversions using Montgomery's trick
- [ ] Precompute fixed bases for exponentiation
- [ ] Use window methods for large exponents
- [ ] Profile in zkVM environment, not just host