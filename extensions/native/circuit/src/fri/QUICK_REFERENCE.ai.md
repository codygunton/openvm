# FRI Component Quick Reference

## Constants

```rust
pub const EXT_DEG: usize = 4;              // Field extension degree
pub const OVERALL_WIDTH: usize = 27;       // Maximum trace width
const INSTRUCTION_READS: usize = 5;        // Memory reads during setup
const DEFAULT_PC_STEP: u32 = 1;           // PC increment
```

## Instruction Format

### FRI_REDUCED_OPENING Opcode
```rust
Instruction {
    a: a_ptr_ptr,      // Pointer to a-values pointer
    b: b_ptr_ptr,      // Pointer to b-values pointer  
    c: length_ptr,     // Pointer to array length
    d: alpha_ptr,      // Pointer to alpha (4 words)
    e: result_ptr,     // Pointer to store result (4 words)
    f: hint_id_ptr,    // Hint stream identifier
    g: is_init_ptr,    // Init flag (0=write a, 1=read a)
}
```

## Column Structures

### General Columns (3)
```rust
struct GeneralCols<T> {
    is_workload_row: T,    // 1 for workload, 0 otherwise
    is_ins_row: T,         // 1 for instruction, 0 otherwise
    timestamp: T,          // Execution timestamp
}
```

### Data Columns (12)
```rust
struct DataCols<T> {
    a_ptr: T,              // Current a-value pointer
    write_a: T,            // 1=write, 0=read for a-values
    b_ptr: T,              // Current b-value pointer
    idx: T,                // Current index (0 to length-1)
    result: [T; 4],        // Rolling hash state
    alpha: [T; 4],         // Field extension element
}
```

### Phase-Specific Widths
- Workload: 27 columns
- Instruction1: 26 columns
- Instruction2: 26 columns

## Memory Access Patterns

### Setup Phase (Instruction1)
```rust
// Read order with timestamps
alpha:    timestamp + 0  (4 words)
length:   timestamp + 1  (1 word)
a_ptr:    timestamp + 2  (1 word)
b_ptr:    timestamp + 3  (1 word)
is_init:  timestamp + 4  (1 word)
```

### Workload Phase
```rust
// For each element i (reverse order in trace)
a[i]:     timestamp + 5      (read or write)
b[i]:     timestamp + 6      (read 4 words)
// timestamp decreases by 2 each row
```

### Result Phase (Instruction2)
```rust
result:   timestamp + 5 + 2*length  (write 4 words)
```

## Field Extension Operations

```rust
// Convert base to extension
fn elem_to_ext<F: Field>(elem: F) -> [F; 4] {
    [elem, F::ZERO, F::ZERO, F::ZERO]
}

// Field arithmetic
FieldExtension::add(a, b)
FieldExtension::subtract(a, b)  
FieldExtension::multiply(a, b)
```

## Rolling Hash Computation

```rust
// Forward execution (for testing)
let mut result = [F::ZERO; 4];
for i in (0..length).rev() {
    result = result * alpha + (b[i] - a[i]);
}

// Reverse trace generation
for i in 0..length {
    // Row i has idx = i, processes element (length-1-i)
    result_next = result * alpha + (b[length-1-i] - a[length-1-i]);
}
```

## Common Code Patterns

### Creating FRI Chip
```rust
let chip = FriReducedOpeningChip::new(
    execution_bus,
    program_bus,
    memory_bridge,
    offline_memory,
    streams,
);
```

### Executing Instruction
```rust
let record = FriReducedOpeningRecord {
    pc: F::from_canonical_u32(from_state.pc),
    start_timestamp: F::from_canonical_u32(from_state.timestamp),
    instruction: instruction.clone(),
    alpha_read: alpha_read.0,
    length_read: length_read.0,
    a_ptr_read: a_ptr_read.0,
    is_init_read: is_init_read.0,
    b_ptr_read: b_ptr_read.0,
    a_rws: a_rws.into_iter().map(|r| r.0).collect(),
    b_reads: b_reads.into_iter().map(|r| r.0).collect(),
    result_write,
};
```

### Memory Operations
```rust
// Read field extension element
let value = memory.read::<4>(addr_space, ptr);

// Write field extension element  
let (record_id, _) = memory.write(addr_space, ptr, value);

// Read single field element
let elem = memory.read_cell(addr_space, ptr);
```

### Constraint Helpers
```rust
// Assert arrays equal
fn assert_array_eq<AB, I1, I2, const N: usize>(
    builder: &mut AB,
    x: [I1; N], 
    y: [I2; N],
) where AB: AirBuilder, I1: Into<AB::Expr>, I2: Into<AB::Expr>
```

## Timing Formulas

```rust
// Total execution time
total_time = 2 * length + INSTRUCTION_READS + 1

// Workload row timestamp (row i)
row_timestamp = start_timestamp + 2 * (length - i)

// Memory operation timestamps
alpha_read:   start_timestamp + 0
length_read:  start_timestamp + 1
a_ptr_read:   start_timestamp + 2
b_ptr_read:   start_timestamp + 3
is_init_read: start_timestamp + 4
a_rw[i]:      start_timestamp + 5 + 2*(length-1-i)
b_read[i]:    start_timestamp + 6 + 2*(length-1-i)
result_write: start_timestamp + 5 + 2*length
```

## Pointer Arithmetic

```rust
// a-values: single field elements
a_address(i) = a_ptr + i

// b-values: field extension elements  
b_address(i) = b_ptr + i * EXT_DEG

// In workload row for index idx:
current_a_ptr = a_ptr + (length - idx)
current_b_ptr = b_ptr + (length - idx) * EXT_DEG
```

## Phase Transitions

```
Start → Workload(0) → Workload(1) → ... → Workload(n-1) → Instruction1 → Instruction2 → End
```

Valid transitions:
- Workload → Workload (idx increases)
- Workload → Instruction1 (when idx = length-1)
- Instruction1 → Instruction2
- Instruction2 → Workload or Disabled

## Testing Utilities

```rust
// Compute expected result
fn compute_fri_mat_opening<F: Field>(
    alpha: [F; 4],
    a: &[F],
    b: &[[F; 4]],
) -> [F; 4] {
    let mut result = [F::ZERO; 4];
    for (&a, &b) in a.iter().zip_eq(b).rev() {
        result = FieldExtension::add(
            FieldExtension::multiply(result, alpha),
            FieldExtension::subtract(b, elem_to_ext(a)),
        );
    }
    result
}
```

## Error Handling

Common errors:
- `ExecutionError`: Invalid instruction format
- Constraint failure: Phase transitions incorrect
- Memory error: Invalid addresses or timing
- Field arithmetic: Overflow in extension field

## Performance Tips

1. **Batch Operations**: Process multiple FRI ops together
2. **Sequential Access**: Keep a/b values contiguous
3. **Reuse Computation**: Cache alpha powers if needed
4. **Minimize Columns**: Current design uses minimal 27

## Security Checklist

- [ ] All memory accesses bounded
- [ ] Field arithmetic constraints complete
- [ ] Phase transitions validated
- [ ] No information leakage
- [ ] Deterministic execution