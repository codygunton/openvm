# Load Sign Extend - Implementation Guide

## Overview
This guide explains how to implement or modify sign-extending load operations in OpenVM.

## Implementation Checklist

### 1. Understanding the Data Flow

```
Memory → Adapter → Core → Sign Extension → Result
         ↓         ↓                        ↓
      alignment  shifted_data          write_data
```

1. **Memory Read**: 32-bit aligned word from memory
2. **Adapter**: Handles alignment, provides shift amount
3. **Core**: Applies sign extension based on opcode
4. **Result**: 32-bit sign-extended value

### 2. Core Implementation Steps

#### Step 1: Define Column Structure
```rust
pub struct LoadSignExtendCoreCols<T, const NUM_CELLS: usize> {
    // Opcode flags - exactly one must be true
    pub opcode_loadb_flag0: T,  // LOADB with shift 0
    pub opcode_loadb_flag1: T,  // LOADB with shift 1
    pub opcode_loadh_flag: T,   // LOADH
    
    // Shift and sign bits
    pub shift_most_sig_bit: T,  // (shift & 2) >> 1
    pub data_most_sig_bit: T,   // The sign bit to extend
    
    // Data arrays
    pub shifted_read_data: [T; NUM_CELLS],
    pub prev_data: [T; NUM_CELLS],
}
```

#### Step 2: Implement Constraints (AIR)
```rust
impl<AB, I> VmCoreAir<AB, I> for LoadSignExtendCoreAir {
    fn eval(&self, builder: &mut AB, local_core: &[AB::Var], _from_pc: AB::Var) {
        // 1. Assert all flags are boolean
        builder.assert_bool(is_loadb0);
        builder.assert_bool(is_loadb1);
        builder.assert_bool(is_loadh);
        
        // 2. Exactly one opcode flag set
        let is_valid = is_loadb0 + is_loadb1 + is_loadh;
        builder.assert_bool(is_valid);
        
        // 3. Extract sign bit based on opcode
        let most_sig_limb = shifted_read_data[0] * is_loadb0
            + shifted_read_data[1] * is_loadb1  
            + shifted_read_data[NUM_CELLS/2-1] * is_loadh;
            
        // 4. Validate sign bit extraction
        self.range_bus.range_check(
            most_sig_limb - data_most_sig_bit * (1 << (LIMB_BITS-1)),
            LIMB_BITS - 1
        );
        
        // 5. Generate write_data with sign extension
        // ... (see full implementation)
    }
}
```

#### Step 3: Implement Execution Logic
```rust
fn execute_instruction(&self, instruction: &Instruction<F>, _from_pc: u32, reads: I::Reads) {
    let (data, shift_amount) = reads.into();
    
    // Apply sign extension
    let write_data = run_write_data_sign_extend(
        opcode, 
        data[1],  // read_data
        data[0],  // prev_data (unused)
        shift_amount
    );
    
    // Extract sign bit for range check
    let most_sig_limb = /* extract based on opcode */;
    let most_sig_bit = most_sig_limb & (1 << (LIMB_BITS - 1));
    
    // Register range check
    self.range_checker_chip.add_count(
        most_sig_limb - most_sig_bit, 
        LIMB_BITS - 1
    );
    
    // Create trace record
    LoadSignExtendCoreRecord {
        opcode,
        most_sig_bit: most_sig_bit != 0,
        shifted_read_data: /* shift by (shift_amount & 2) */,
        shift_amount,
        prev_data: data[0],
    }
}
```

### 3. Sign Extension Algorithm

#### For LOADB (8-bit to 32-bit):
```rust
match shift {
    0 => write_data = [read[0], ext, ext, ext],
    1 => write_data = [read[1], ext, ext, ext],
    2 => write_data = [read[2], ext, ext, ext],
    3 => write_data = [read[3], ext, ext, ext],
}
where ext = (read[shift] >> 7) ? 0xFF : 0x00
```

#### For LOADH (16-bit to 32-bit):
```rust
match shift {
    0 => write_data = [read[0], read[1], ext, ext],
    2 => write_data = [read[2], read[3], ext, ext],
}
where ext = (read[1+shift] >> 7) ? 0xFF : 0x00
```

### 4. Critical Implementation Details

#### Shifted Data Optimization
```rust
// Pre-shift by (shift & 2) to reduce constraint complexity
let read_shift = shift_amount & 2;
let shifted_read_data = array::from_fn(|i| {
    data[1][(i + read_shift) % NUM_CELLS]
});

// In constraints, handle remaining shift of 0 or 1
let write_data[0] = 
    (is_loadh + is_loadb0) * shifted_read_data[0] +
    is_loadb1 * shifted_read_data[1];
```

#### Sign Bit Extraction
```rust
// Sign bit position depends on data type
let sign_position = match opcode {
    LOADB => shift,                    // Byte 0-3
    LOADH => NUM_CELLS/2 - 1 + shift,  // Byte 1 or 3
};

// Extract bit 7 (most significant bit of byte)
let sign_bit = (data[sign_position] >> 7) & 1;
```

### 5. Testing Strategy

#### Unit Tests
```rust
#[test]
fn test_loadb_sign_extend() {
    // Test all shift positions
    for shift in 0..4 {
        // Test positive (bit 7 = 0)
        let data = [0x7F, 0x00, 0x00, 0x00];
        assert_eq!(result[0], 0x7F);
        assert_eq!(result[1..4], [0x00; 3]);
        
        // Test negative (bit 7 = 1)  
        let data = [0xFF, 0x00, 0x00, 0x00];
        assert_eq!(result, [0xFF; 4]);
    }
}
```

#### Integration Tests
```rust
#[test]
fn test_with_adapter() {
    let mut tester = VmChipTestBuilder::default();
    let chip = create_chip();
    
    // Test unaligned access
    let addr = 0x1001;  // Shift = 1
    tester.write_memory(0x1000, [0x00, 0xFF, 0x00, 0x00]);
    
    execute_loadb(&mut tester, &chip, addr);
    assert_eq!(read_result(), [0xFF; 4]);  // Sign extended
}
```

### 6. Common Modifications

#### Adding New Load Size (e.g., LOADW)
1. Add new opcode flag in `LoadSignExtendCoreCols`
2. Update constraint logic in `eval()`
3. Add case in `run_write_data_sign_extend()`
4. Update opcode decoding in `execute_instruction()`
5. Add comprehensive tests

#### Optimizing Constraints
1. Minimize multiplications with flags
2. Use batched operations where possible
3. Consider pre-computing common expressions
4. Profile constraint evaluation time

### 7. Debugging Tips

#### Trace Verification
```rust
// Check opcode flags match instruction
assert_eq!(cols.opcode_loadb_flag0, F::from_bool(
    opcode == LOADB && (shift & 1) == 0
));

// Verify shift handling  
assert_eq!(cols.shift_most_sig_bit, F::from_canonical_u32(
    (shift & 2) >> 1
));

// Validate sign extension
let expected = run_write_data_sign_extend(...);
assert_eq!(actual_write_data, expected);
```

#### Common Bugs
1. **Wrong Flag Set**: Double-check shift & 1 logic
2. **Sign Bit Position**: Verify limb index calculation
3. **Extension Pattern**: Ensure all upper limbs filled
4. **Shift Modulo**: Remember array wraparound with %

### 8. Performance Optimization

#### Constraint Reduction
- Pre-shift eliminates half the multiplexing
- Separate LOADB flags avoid complex conditionals
- Range check only on extracted sign bit

#### Trace Generation
- Use array::from_fn for efficient initialization
- Minimize field conversions
- Pre-compute shift masks as u32

### 9. Security Considerations

#### Soundness Requirements
1. Sign bit must match actual data bit 7
2. All shifts must be handled correctly
3. No information leakage through undefined behavior

#### Validation Points
- Range checker validates sign bit claim
- Adapter ensures aligned memory access
- Constraints enforce deterministic behavior