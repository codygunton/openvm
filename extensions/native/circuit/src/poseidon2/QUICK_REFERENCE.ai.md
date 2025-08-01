# Native Poseidon2 Quick Reference

## Constants
```rust
const CHUNK: usize = 8;  // Poseidon2 state size
const AS_NATIVE: u32 = 4;  // Native address space
const VERIFY_BATCH_BUS: usize = 7;  // Internal communication bus
```

## Instruction Formats

### PERM_POS2 (Permutation)
```rust
Instruction {
    opcode: PERM_POS2,
    operands: Operands {
        a: input_ptr,    // Pointer to 16 elements
        b: output_ptr,   // Pointer to write 16 elements
        c_f: unused,
    }
}
```

### COMP_POS2 (Compression)
```rust
Instruction {
    opcode: COMP_POS2,
    operands: Operands {
        a: left_ptr,     // Pointer to 8 elements
        b: right_ptr,    // Pointer to 8 elements
        c: output_ptr,   // Pointer to write 8 elements
        d_f: unused,
    }
}
```

### VERIFY_BATCH
```rust
Instruction {
    opcode: VERIFY_BATCH,
    operands: Operands {
        a: dim_ptr,                  // Dimensions array pointer
        b: opened_values_ptr,        // Opened values array pointer
        c: opened_values_len,        // Length of opened values
        d: proof_id,                 // Hint ID for proof
        e: index_bits_ptr,           // Index bits array pointer
        f: commit_ptr,               // Commitment pointer
        g: opened_value_size_inv,    // 1 for F, 1/4 for EF
    }
}
```

## Row Type Quick Check
```rust
// Determine row type from flags
fn row_type(cols: &NativePoseidon2Cols<F>) -> RowType {
    if cols.simple == F::ONE { return RowType::Simple; }
    if cols.incorporate_row == F::ONE { return RowType::TopLevelIncorporateRow; }
    if cols.incorporate_sibling == F::ONE { return RowType::TopLevelIncorporateSibling; }
    if cols.inside_row == F::ONE { return RowType::InsideRow; }
    RowType::Disabled
}
```

## Memory Operations

### Read with Auxiliary
```rust
let (value, aux) = memory.read(timestamp, address, F::from(AS_NATIVE));
record.read_aux = aux;  // Store for constraints
state.timestamp += 1;
```

### Write with Auxiliary
```rust
let aux = memory.write(timestamp, address, value, F::from(AS_NATIVE));
record.write_aux = aux;  // Store for constraints
state.timestamp += 1;
```

### Array Dereferencing
```rust
// Read pointer -> Read length -> Read elements
let (arr_ptr, _) = memory.read(ts, ptr_addr, space);
let (arr_len, _) = memory.read(ts + 1, arr_ptr + F::ONE, space);
// Elements start at arr_ptr + F::from(2)
```

## Column Casting

### TopLevel Specific
```rust
let specific: &TopLevelSpecificCols<F> = cols.specific.borrow();
// Access: specific.sibling, specific.index_bit, etc.
```

### InsideRow Specific
```rust
let specific: &InsideRowSpecificCols<F> = cols.specific.borrow();
// Access: specific.hash_state, specific.element_counter, etc.
```

### Simple Specific
```rust
let specific: &SimplePoseidonSpecificCols<F> = cols.specific.borrow();
// Access: specific.input, specific.output, etc.
```

## Common Patterns

### Poseidon2 Permutation
```rust
// Input: [F; 16], Output: [F; 16]
let output = poseidon2_permute(input);
```

### Poseidon2 Compression
```rust
// Input: two [F; 8], Output: [F; 8]
let compressed = poseidon2_compress(left, right);
```

### Rolling Hash
```rust
// Hash arbitrary length input to [F; CHUNK]
let mut state = [F::ZERO; CHUNK];
for chunk in input.chunks(CHUNK) {
    // Mix chunk into state and permute
    state = rolling_hash_step(state, chunk);
}
```

## Execution Flow

### Simple Execution
```rust
1. Fetch instruction
2. Read operands from memory
3. Execute Poseidon2 operation
4. Write result to memory
5. Update PC and timestamp
```

### VERIFY_BATCH Execution
```rust
1. Dereference all operands
2. For each height level:
   a. Hash opened values (InsideRow)
   b. Incorporate hash (TopLevel)
   c. Incorporate sibling (TopLevel)
3. Compare final node with commitment
4. Update execution state
```

## AIR Constraint Helpers

### Row Type Mutual Exclusion
```rust
let sum = cols.simple + cols.incorporate_row + 
          cols.incorporate_sibling + cols.inside_row;
builder.assert_bool(sum);  // 0 or 1
```

### Timestamp Ordering
```rust
builder.when_transition()
    .when_not(is_new_instruction)
    .assert_eq(
        next.start_timestamp,
        current.start_timestamp + F::ONE
    );
```

### Bus Balance
```rust
// Send from InsideRow
builder.when(end_inside_row)
    .push_send(bus_id, hash_values, multiplicity);

// Receive in TopLevel
builder.when(incorporate_row)
    .push_receive(bus_id, hash_values, multiplicity);
```

## Test Utilities

### Create Test Chip
```rust
let chip = NativePoseidon2Chip::new(
    memory_controller,
    Poseidon2Config::default(),
);
```

### Generate Test Vectors
```rust
let input = (0..16).map(|i| F::from(i)).collect::<Vec<_>>();
let expected = known_poseidon2_output(&input);
```

### Verify Execution
```rust
let record = chip.execute(instruction, state, &mut ctx)?;
assert_eq!(record.to_state.pc, from_state.pc + DEFAULT_PC_STEP);
```

## Performance Tips

- Batch memory operations when possible
- Use contiguous memory layouts
- Minimize pointer dereferencing
- Leverage parallel iterators for trace generation

## Debug Checklist

- [ ] Row type flags mutually exclusive?
- [ ] Timestamps strictly increasing?
- [ ] Bus sends match receives?
- [ ] Memory reads before writes?
- [ ] Correct address space used?
- [ ] Operand count matches instruction?
- [ ] Column cast matches row type?