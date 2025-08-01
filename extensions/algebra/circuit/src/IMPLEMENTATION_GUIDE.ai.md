# Algebra Circuit Extension - Implementation Guide

## Adding a New Modular Operation

### Step 1: Define the Opcode
In `openvm-algebra-transpiler`:
```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rv32ModularArithmeticOpcode {
    // Existing opcodes...
    NEW_OP = 10,
    SETUP_NEW_OP = 11,
}
```

### Step 2: Create the Chip Implementation
```rust
// In modular_chip/new_op.rs
use openvm_circuit::arch::{VmChipWrapper, instructions::*};
use openvm_mod_circuit_builder::*;

#[derive(Debug)]
pub struct ModularNewOpCoreChip<const NUM_LIMBS: usize> {
    pub air: ModularNewOpCoreAir<NUM_LIMBS>,
}

impl<const NUM_LIMBS: usize> ModularNewOpCoreChip<NUM_LIMBS> {
    pub fn new(
        config: ExprBuilderConfig,
        offset: usize,
        range_bus: RangeCheckBus<1>,
    ) -> Self {
        let air = ModularNewOpCoreAir::new(config, offset, range_bus);
        Self { air }
    }
}

// Implement the core logic
impl<const NUM_LIMBS: usize> VmCoreChip for ModularNewOpCoreChip<NUM_LIMBS> {
    // Implementation details...
}
```

### Step 3: Register in Extension
In `modular_extension.rs`:
```rust
// Add to executor enum
#[derive(ChipUsageGetter, Chip, InstructionExecutor, AnyEnum, From)]
pub enum ModularExtensionExecutor<F: PrimeField32> {
    // Existing variants...
    ModularNewOpRv32_32(ModularNewOpChip<F, 1, 32>),
    ModularNewOpRv32_48(ModularNewOpChip<F, 3, 16>),
}

// In build() method
let new_op_opcodes = (Rv32ModularArithmeticOpcode::NEW_OP as usize)
    ..=(Rv32ModularArithmeticOpcode::SETUP_NEW_OP as usize);

// Create and register chip
let new_op_chip = ModularNewOpChip::new(
    adapter_chip_32.clone(),
    config32.clone(),
    start_offset,
    range_checker.clone(),
    offline_memory.clone(),
);
inventory.add_executor(
    ModularExtensionExecutor::ModularNewOpRv32_32(new_op_chip),
    new_op_opcodes.map(|x| VmOpcode::from_usize(x + start_offset)),
)?;
```

## Implementing Fp2 Operations

### Understanding Fp2 Structure
Fp2 = Fp[u]/(1 + u²) where u² = -1

Elements: a + bu where a, b ∈ Fp

### Example: Implementing Fp2 Inverse
```rust
// In fp2.rs
impl Fp2 {
    pub fn inverse(&mut self) -> Fp2 {
        // For a + bu, inverse is (a - bu) / (a² + b²)
        let builder = self.c0.builder.clone();
        
        // Compute norm = a² + b²
        let norm = self.c0.square() + self.c1.square();
        
        // Compute conjugate
        let conj = Fp2 {
            c0: self.c0.clone(),
            c1: self.c1.neg(),
        };
        
        // Divide conjugate by norm
        conj.scalar_div(&mut norm)
    }
}
```

### Creating Fp2 Chip
```rust
// In fp2_chip/inverse.rs
pub struct Fp2InverseCoreChip<const NUM_LIMBS: usize> {
    pub air: Fp2InverseCoreAir<NUM_LIMBS>,
}

impl<const NUM_LIMBS: usize> Fp2InverseCoreChip<NUM_LIMBS> {
    pub fn new(config: ExprBuilderConfig, offset: usize) -> Self {
        let mut builder = ExprBuilder::new(config.clone(), 2);
        let builder_ref = Rc::new(RefCell::new(builder));
        
        // Input: a + bu
        let mut a = Fp2::new(builder_ref.clone());
        
        // Compute inverse
        let mut result = a.inverse();
        result.save_output();
        
        let builder = builder_ref.borrow().clone();
        let air = Fp2InverseCoreAir { builder, offset };
        Self { air }
    }
}
```

## Working with Phantom Sub-Executors

### Purpose
Phantom sub-executors provide non-deterministic hints for complex computations.

### Example: Square Root Hint
```rust
impl<F: PrimeField32> PhantomSubExecutor<F> for SqrtHintSubEx {
    fn phantom_execute(
        &mut self,
        memory: &MemoryController<F>,
        streams: &mut Streams<F>,
        _: PhantomDiscriminant,
        a: F,  // Register containing input pointer
        _: F,
        c_upper: u16,  // Modulus index
    ) -> eyre::Result<()> {
        // 1. Read input from memory
        let rs1 = unsafe_read_rv32_register(memory, a);
        let mut x_limbs = Vec::new();
        for i in 0..num_limbs {
            let limb = memory.unsafe_read_cell(
                F::from_canonical_u32(RV32_MEMORY_AS),
                F::from_canonical_u32(rs1 + i as u32),
            );
            x_limbs.push(limb.as_canonical_u32() as u8);
        }
        let x = BigUint::from_bytes_le(&x_limbs);
        
        // 2. Compute square root
        let (success, sqrt) = match mod_sqrt(&x, &modulus, &non_qr) {
            Some(sqrt) => (true, sqrt),
            None => {
                // Try x * non_qr
                let sqrt = mod_sqrt(&(&x * &non_qr), &modulus, &non_qr)
                    .expect("Either x or x*non_qr must be square");
                (false, sqrt)
            }
        };
        
        // 3. Return hint via stream
        streams.hint_stream = once(F::from_bool(success))
            .chain(sqrt.to_bytes_le().into_iter().map(F::from_canonical_u8))
            .chain(repeat(F::ZERO))
            .take(4 + num_limbs)
            .collect();
        
        Ok(())
    }
}
```

## Performance Optimization Tips

### 1. Minimize Auto-Save Triggers
```rust
// Check if save is needed before complex operations
let constraint = /* complex expression */;
let carry_bits = constraint.constraint_carry_bits_with_pq(
    &prime, limb_bits, num_limbs, &proper_max
);
if carry_bits > self.max_carry_bits {
    self.save();
}
```

### 2. Batch Operations
```rust
// Instead of multiple individual operations
let a1 = x1.add(&y1);
let a2 = x2.add(&y2);

// Use builder directly for batch constraints
let mut builder = /* get builder */;
let constraints = vec![
    x1.expr + y1.expr - a1.expr,
    x2.expr + y2.expr - a2.expr,
];
builder.constrain_batch(constraints);
```

### 3. Reuse Adapters and Buses
```rust
// Share bitwise lookup chip
let bitwise_lu_chip = if let Some(&chip) = builder
    .find_chip::<SharedBitwiseOperationLookupChip<8>>()
    .first()
{
    chip.clone()
} else {
    // Create new only if not found
    let chip = SharedBitwiseOperationLookupChip::new(/*...*/);
    inventory.add_periphery_chip(chip.clone());
    chip
};
```

## Testing Strategies

### 1. Unit Test Template
```rust
#[test]
fn test_modular_operation() {
    let modulus = BigUint::from_str("...").unwrap();
    let config = ExprBuilderConfig {
        modulus: modulus.clone(),
        num_limbs: 32,
        limb_bits: 8,
    };
    
    // Create test inputs
    let a = random_biguint(&modulus);
    let b = random_biguint(&modulus);
    
    // Expected result
    let expected = (a + b) % modulus;
    
    // Run through chip
    let result = run_operation(a, b, config);
    
    assert_eq!(result, expected);
}
```

### 2. Integration Test with Trace
```rust
#[test]
fn test_with_trace_generation() {
    let (range_checker, builder) = setup(&prime);
    
    // Build expression
    let mut x = FieldVariable::new(builder.clone());
    let mut y = FieldVariable::new(builder.clone());
    let result = x.add(&mut y);
    
    // Generate and verify trace
    let trace = generate_trace(/*...*/);
    BabyBearBlake3Engine::run_simple_test_no_pis_fast(
        vec![air],
        vec![trace],
    ).expect("Verification failed");
}
```

## Common Pitfalls and Solutions

### 1. Modulus Index Mismatch
**Problem**: Guest code uses wrong modulus index
**Solution**: Always use generated init code
```rust
// Good: Use generated code
include!(concat!(env!("OUT_DIR"), "/init.rs"));

// Bad: Hardcode indices
const MODULUS_INDEX: usize = 0;
```

### 2. Limb Count Errors
**Problem**: Mismatch between expected and actual limb counts
**Solution**: Use modulus size to determine limb count
```rust
let num_limbs = if modulus.bits().div_ceil(8) <= 32 {
    32
} else if modulus.bits().div_ceil(8) <= 48 {
    48
} else {
    panic!("Modulus too large");
};
```

### 3. Auto-Save Infinite Loops
**Problem**: Expression keeps triggering auto-save
**Solution**: Pre-save variables before complex operations
```rust
// Save inputs first
self.save();
other.save();
// Then perform operation
let result = complex_operation(self, other);
```

## Debugging Techniques

### 1. Trace Inspection
```rust
// Add debug prints in chip implementation
println!("Input limbs: {:?}", input_limbs);
println!("Result limbs: {:?}", result_limbs);
println!("Modulus: {}", self.modulus);
```

### 2. Constraint Verification
```rust
// Manually verify constraints
let lhs = evaluate_expression(&constraint_lhs, &vars);
let rhs = evaluate_expression(&constraint_rhs, &vars);
assert_eq!(lhs % modulus, rhs % modulus);
```

### 3. Step-by-Step Execution
```rust
// Break complex operations into steps
let step1 = a.mul(&b);
println!("After mul: {:?}", step1);
let step2 = step1.add(&c);
println!("After add: {:?}", step2);
```