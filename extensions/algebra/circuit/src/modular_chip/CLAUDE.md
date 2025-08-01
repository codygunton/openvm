# Modular Chip Component Instructions for AI Assistants

## Overview
When working with the Modular Chip component, you are dealing with cryptographically sensitive arithmetic operations. This component implements modular arithmetic for arbitrary prime moduli in a zero-knowledge proof context.

## Key Principles

### 1. Modular Arithmetic Correctness
- **Always ensure values are reduced**: All inputs and outputs must be in range [0, modulus)
- **Setup operations are mandatory**: Never perform arithmetic without proper setup
- **Check modulus validity**: Modulus must be prime for division operations
- **Handle edge cases**: Test behavior at modulus boundaries (0, modulus-1)

### 2. Constraint System Integrity
- **Never modify constraint logic** without understanding the mathematical proof
- **Preserve soundness invariants**: All values < modulus constraint is critical
- **Range checks are mandatory**: Never skip or optimize away range checks
- **Lt_marker system is delicate**: The marker-based comparison system has subtle edge cases

### 3. Common Pitfalls to Avoid

#### Incorrect Modulus Handling
```rust
// WRONG: Using non-prime modulus for division
let modulus = BigUint::from(15u32); // 15 = 3 * 5, not prime!

// CORRECT: Ensure modulus is prime
let modulus = BigUint::from_str("0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f").unwrap(); // secp256k1 prime
```

#### Missing Setup Operations
```rust
// WRONG: Direct arithmetic without setup
chip.execute(ADD, result, a, b);

// CORRECT: Setup before operations
chip.execute(SETUP_ADDSUB, modulus_ptr);
chip.execute(ADD, result, a, b);
```

#### Improper Range Checking
```rust
// WRONG: Assuming inputs are already reduced
let result = (a + b) % modulus;

// CORRECT: Verify inputs are in range
assert!(a < modulus && b < modulus);
let result = (a + b) % modulus;
```

## Testing Requirements

### Essential Test Cases
1. **Boundary conditions**: Test with 0, 1, modulus-1, modulus-2
2. **Wraparound**: Operations that produce results ≥ modulus
3. **Identity elements**: Addition with 0, multiplication by 1
4. **Inverse operations**: a - a = 0, a / a = 1
5. **Setup validation**: Ensure setup properly initializes state

### Test Pattern Template
```rust
#[test]
fn test_modular_operation_edge_case() {
    let modulus = test_modulus();
    let chip = create_test_chip(modulus.clone());
    
    // Setup phase
    chip.execute_setup(modulus.clone());
    
    // Test edge case
    let a = modulus.clone() - 1u32;
    let b = 2u32;
    let result = chip.execute_add(a, b);
    
    // Verify modular reduction occurred
    assert_eq!(result, BigUint::from(1u32));
    
    // Verify constraints were satisfied
    chip.verify_constraints();
}
```

## Performance Guidelines

### Optimization Priorities
1. **Correctness over performance**: Never compromise soundness for speed
2. **Batch range checks**: Use shared range checkers when possible
3. **Minimize limb operations**: Choose appropriate limb sizes for modulus
4. **Reuse computations**: Share expression builders across similar operations

### Anti-Patterns to Avoid
- Creating new range checkers for each operation
- Using oversized limb counts for small moduli
- Reconstructing expression builders repeatedly
- Performing redundant modular reductions

## Code Modification Guidelines

### When Modifying Arithmetic Logic
1. **Understand the mathematical proof** in README.md
2. **Maintain constraint equivalence**: New constraints must imply old ones
3. **Update tests comprehensively**: Add tests for new edge cases
4. **Document changes clearly**: Explain why changes maintain soundness

### When Adding New Operations
1. **Follow existing patterns**: Study add/sub and mul/div implementations
2. **Add corresponding setup operation**: Each operation type needs setup
3. **Update transpiler**: Add new opcodes to `openvm_algebra_transpiler`
4. **Implement proper range checking**: All intermediate values need bounds

## Security Considerations

### Critical Security Invariants
1. **Modulus immutability**: Once set, modulus cannot change without setup
2. **Input validation**: All inputs must be verified < modulus
3. **No timing leaks**: Operations should be constant-time where possible
4. **Deterministic behavior**: Same inputs must produce same outputs

### Audit Checklist
When reviewing changes:
- [ ] All new operations have setup variants
- [ ] Range checks cover all intermediate values
- [ ] Edge cases are tested (especially modulus boundaries)
- [ ] Constraints maintain mathematical soundness
- [ ] No new side channels introduced
- [ ] Documentation updated for new functionality

## Common Implementation Patterns

### Adding a New Modular Operation
```rust
// 1. Define expression builder
pub fn new_op_expr(config: ExprBuilderConfig, range_bus: VariableRangeCheckerBus) 
    -> (FieldExpr, usize) {
    // Build expression with proper constraints
}

// 2. Create chip struct
#[derive(Chip, ChipUsageGetter, InstructionExecutor)]
pub struct ModularNewOpChip<F: PrimeField32, const BLOCKS: usize, const BLOCK_SIZE: usize>(
    pub VmChipWrapper<F, AdapterType, CoreChipType>,
);

// 3. Add setup operation
impl ModularNewOpChip {
    pub fn new(...) -> Self {
        // Include SETUP_NEWOP in supported opcodes
    }
}
```

### Debugging Constraint Failures
1. **Enable trace logging**: Use `RUST_LOG=debug` for execution traces
2. **Check modulus setup**: Verify setup was called with correct modulus
3. **Validate inputs**: Ensure all inputs are properly reduced
4. **Examine lt_marker**: For IS_EQ, check marker array is properly formed
5. **Verify range checks**: Confirm all range checks are satisfied

## Final Reminders

- **Mathematical correctness is paramount**: This is cryptographic code
- **Test exhaustively**: Edge cases in modular arithmetic can be subtle
- **Document assumptions**: Make implicit requirements explicit
- **Preserve proofs**: The mathematical proofs in README.md are critical
- **When in doubt, ask**: Modular arithmetic in ZK has many subtleties