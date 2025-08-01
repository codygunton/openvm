# MulH Component Implementation Guide

## Core Algorithm

### Mathematical Foundation

The MulH operations compute: `result = (a × b) >> 32`

For 32-bit multiplication producing 64-bit result:
- Full product: `a × b = high_32_bits << 32 + low_32_bits`
- MulH returns: `high_32_bits`

### Limb-Based Multiplication

The implementation splits 32-bit values into 8-bit limbs:
```
Value = limb[0] + limb[1]×2^8 + limb[2]×2^16 + limb[3]×2^24
```

Full multiplication with carry tracking:
```rust
// Compute low bits (a_mul) with carries
for i in 0..NUM_LIMBS {
    let sum = carry[i-1] + Σ(b[k] × c[i-k], k=0..i)
    a_mul[i] = sum % (1 << LIMB_BITS)
    carry[i] = sum / (1 << LIMB_BITS)
}

// Compute high bits (a) with sign extension
for j in 0..NUM_LIMBS {
    let sum = carry[NUM_LIMBS-1+j] + 
              Σ(b[k] × c[NUM_LIMBS+j-k], k=j+1..NUM_LIMBS) +
              Σ(b[k] × c_ext + c[k] × b_ext, k=0..j)
    a[j] = sum % (1 << LIMB_BITS)
    carry[NUM_LIMBS+j] = sum / (1 << LIMB_BITS)
}
```

## Constraint System Details

### Column Assignment

```rust
pub struct MulHCoreCols<T, const NUM_LIMBS: usize, const LIMB_BITS: usize> {
    pub a: [T; NUM_LIMBS],          // High 32 bits of result
    pub b: [T; NUM_LIMBS],          // First operand
    pub c: [T; NUM_LIMBS],          // Second operand
    pub a_mul: [T; NUM_LIMBS],      // Low 32 bits of result
    pub b_ext: T,                   // Sign extension of b
    pub c_ext: T,                   // Sign extension of c
    pub opcode_mulh_flag: T,        // Is MULH instruction
    pub opcode_mulhsu_flag: T,      // Is MULHSU instruction
    pub opcode_mulhu_flag: T,       // Is MULHU instruction
}
```

### Key Constraints

1. **Opcode Validity**
   ```rust
   // Exactly one opcode flag must be set
   is_valid = mulh_flag + mulhsu_flag + mulhu_flag
   assert_bool(is_valid)
   ```

2. **Carry Propagation**
   ```rust
   // For each limb position, verify carry computation
   expected_limb = carry[i-1] + multiplication_terms
   carry[i] = (expected_limb - actual_limb[i]) / (1 << LIMB_BITS)
   ```

3. **Range Checks**
   ```rust
   // Each (limb, carry) pair must be in valid range
   range_tuple_checker.send([limb, carry])
   ```

4. **Sign Extension**
   ```rust
   // b_ext = 0 or (2^LIMB_BITS - 1) based on sign bit
   // c_ext follows similar pattern based on opcode
   bitwise_lookup.send_range(b_msb, c_msb)
   ```

## Sign Extension Logic

### MULH (Signed × Signed)
- `b_ext = (b[3] >> 7) ? 0xFF : 0x00`
- `c_ext = (c[3] >> 7) ? 0xFF : 0x00`

### MULHSU (Signed × Unsigned)
- `b_ext = (b[3] >> 7) ? 0xFF : 0x00`
- `c_ext = 0x00` (always zero)

### MULHU (Unsigned × Unsigned)
- `b_ext = 0x00` (always zero)
- `c_ext = 0x00` (always zero)

## Implementation Patterns

### Creating the AIR

```rust
impl<F: Field, const NUM_LIMBS: usize, const LIMB_BITS: usize> 
    BaseAir<F> for MulHCoreAir<NUM_LIMBS, LIMB_BITS> 
{
    fn width(&self) -> usize {
        MulHCoreCols::<F, NUM_LIMBS, LIMB_BITS>::width()
    }
}
```

### Execution Flow

```rust
fn execute_instruction(&self, instruction: &Instruction<F>, _from_pc: u32, reads: I::Reads) 
    -> Result<(AdapterRuntimeContext<F, I>, Self::Record)> 
{
    // 1. Decode opcode
    let opcode = MulHOpcode::from_usize(instruction.opcode.local_opcode_idx());
    
    // 2. Extract operands
    let [[b_limbs], [c_limbs]] = reads.into();
    
    // 3. Run multiplication algorithm
    let (a, a_mul, carry, b_ext, c_ext) = run_mulh(opcode, &b_limbs, &c_limbs);
    
    // 4. Update lookup tables
    self.range_tuple_chip.add_count(&[a_mul[i], carry[i]]);
    self.bitwise_lookup_chip.request_range(...);
    
    // 5. Return result and execution record
    Ok((AdapterRuntimeContext::without_pc([a]), record))
}
```

### Trace Generation

```rust
fn generate_trace_row(&self, row_slice: &mut [F], record: Self::Record) {
    let cols: &mut MulHCoreCols<_, NUM_LIMBS, LIMB_BITS> = row_slice.borrow_mut();
    
    // Copy values from record
    cols.a = record.a;
    cols.b = record.b;
    cols.c = record.c;
    cols.a_mul = record.a_mul;
    cols.b_ext = record.b_ext;
    cols.c_ext = record.c_ext;
    
    // Set opcode flags
    cols.opcode_mulh_flag = F::from_bool(record.opcode == MulHOpcode::MULH);
    cols.opcode_mulhsu_flag = F::from_bool(record.opcode == MulHOpcode::MULHSU);
    cols.opcode_mulhu_flag = F::from_bool(record.opcode == MulHOpcode::MULHU);
}
```

## Testing Strategies

### Positive Testing
```rust
// Generate random inputs and verify constraints pass
for _ in 0..num_tests {
    let b = generate_long_number(&mut rng);
    let c = generate_long_number(&mut rng);
    execute_and_verify(opcode, b, c);
}
```

### Negative Testing
```rust
// Modify trace to violate constraints
fn run_negative_test(modify_trace: impl Fn(&mut DenseMatrix<F>)) {
    // Execute normally
    chip.execute_instruction(&instruction);
    
    // Corrupt the trace
    modify_trace(&mut trace);
    
    // Verify constraints fail
    assert!(verify().is_err());
}
```

## Optimization Considerations

### Lookup Table Sharing
- Bitwise operations shared across arithmetic chips
- Range checker used by multiple components
- Minimize total lookup entries

### Constraint Efficiency
- Single pass evaluation of all constraints
- Batch range checks for all limbs
- Conditional constraints based on opcode

### Memory Access Pattern
- Sequential reads of operands
- Single write of result
- Aligned with RV32 register layout

## Common Implementation Pitfalls

1. **Incorrect Carry Computation**
   - Ensure carries account for all partial products
   - Don't forget carry from previous limb

2. **Sign Extension Errors**
   - MULHSU only sign-extends first operand
   - MULHU doesn't sign-extend at all

3. **Range Check Bounds**
   - Carries can be larger than single limb
   - Maximum carry ≈ NUM_LIMBS × (2^LIMB_BITS)

4. **Endianness Issues**
   - Limbs stored in little-endian order
   - MSB is in limb[NUM_LIMBS-1]