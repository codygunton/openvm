# FRI Component Implementation Guide

## Overview

This guide provides detailed implementation patterns for working with the FRI (Fast Reed-Solomon Interactive Oracle Proof) component in OpenVM. The FRI reduced opening operation is a core primitive for STARK proof systems.

## Core Implementation Structure

### 1. FRI Chip Definition

```rust
pub struct FriReducedOpeningChip<F: Field> {
    air: FriReducedOpeningAir,
    pub records: Vec<FriReducedOpeningRecord<F>>,
    pub height: usize,
    offline_memory: Arc<Mutex<OfflineMemory<F>>>,
    streams: Arc<Mutex<Streams<F>>>,
}
```

Key components:
- `air`: Algebraic Intermediate Representation for constraints
- `records`: Execution records for trace generation
- `height`: Total trace rows used
- `offline_memory`: Memory verification system
- `streams`: Hint streaming for witness generation

### 2. Column Structure

The FRI component uses three types of column layouts that share memory:

```rust
// Workload columns (27 total)
#[repr(C)]
struct WorkloadCols<T> {
    prefix: PrefixCols<T>,              // 16 columns
    a_aux: MemoryWriteAuxCols<T, 1>,   // 10 columns
    b: [T; EXT_DEG],                    // 4 columns
    b_aux: MemoryReadAuxCols<T>,       // 7 columns
}

// Instruction1 columns (26 total)
#[repr(C)]
struct Instruction1Cols<T> {
    prefix: PrefixCols<T>,              // 16 columns
    pc: T,
    a_ptr_ptr: T,
    a_ptr_aux: MemoryReadAuxCols<T>,
    b_ptr_ptr: T,
    b_ptr_aux: MemoryReadAuxCols<T>,
    write_a_x_is_first: T,              // Extraneous
}

// Instruction2 columns (26 total)
#[repr(C)]
struct Instruction2Cols<T> {
    general: GeneralCols<T>,            // 3 columns
    is_first: T,
    length_ptr: T,
    length_aux: MemoryReadAuxCols<T>,
    alpha_ptr: T,
    alpha_aux: MemoryReadAuxCols<T>,
    result_ptr: T,
    result_aux: MemoryWriteAuxCols<T, EXT_DEG>,
    hint_id_ptr: T,
    is_init_ptr: T,
    is_init_aux: MemoryReadAuxCols<T>,
    write_a_x_is_first: T,              // Extraneous
}
```

### 3. Execution Flow

#### Phase 1: Instruction Execution

```rust
impl<F: PrimeField32> InstructionExecutor<F> for FriReducedOpeningChip<F> {
    fn execute(
        &mut self,
        memory: &mut MemoryController<F>,
        instruction: &Instruction<F>,
        from_state: ExecutionState<u32>,
    ) -> Result<ExecutionState<u32>, ExecutionError> {
        // 1. Decode instruction operands
        let &Instruction {
            a: a_ptr_ptr,
            b: b_ptr_ptr,
            c: length_ptr,
            d: alpha_ptr,
            e: result_ptr,
            f: hint_id_ptr,
            g: is_init_ptr,
            ..
        } = instruction;

        // 2. Read configuration values
        let alpha_read = memory.read(addr_space, alpha_ptr);
        let length_read = memory.read_cell(addr_space, length_ptr);
        let a_ptr_read = memory.read_cell(addr_space, a_ptr_ptr);
        let b_ptr_read = memory.read_cell(addr_space, b_ptr_ptr);
        let is_init_read = memory.read_cell(addr_space, is_init_ptr);

        // 3. Optional: Get hint data for a-values
        let data = if is_init == 0 {
            let mut streams = self.streams.lock().unwrap();
            let hint_stream = &mut streams.hint_space[hint_id];
            hint_stream.drain(0..length).collect()
        } else {
            vec![]
        };

        // 4. Process elements (forward order for execution)
        let mut a_rws = Vec::with_capacity(length);
        let mut b_reads = Vec::with_capacity(length);
        
        for i in 0..length {
            // Read or write a[i]
            let a_rw = if is_init == 0 {
                memory.write_cell(addr_space, a_ptr + F::from_canonical_usize(i), data[i])
            } else {
                memory.read_cell(addr_space, a_ptr + F::from_canonical_usize(i))
            };
            
            // Read b[i] (field extension element)
            let b_read = memory.read::<EXT_DEG>(
                addr_space, 
                b_ptr + F::from_canonical_usize(EXT_DEG * i)
            );
            
            a_rws.push(a_rw);
            b_reads.push(b_read);
        }

        // 5. Compute result (reverse order for rolling hash)
        let mut result = [F::ZERO; EXT_DEG];
        for (a_rw, b_read) in a_rws.iter().rev().zip_eq(b_reads.iter().rev()) {
            result = FieldExtension::add(
                FieldExtension::multiply(result, alpha),
                FieldExtension::subtract(b_read.1, elem_to_ext(a_rw.1)),
            );
        }

        // 6. Write result
        let (result_write, _) = memory.write(addr_space, result_ptr, result);

        // 7. Create execution record
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
        
        self.height += record.get_height();
        self.records.push(record);

        Ok(ExecutionState {
            pc: from_state.pc + DEFAULT_PC_STEP,
            timestamp: memory.timestamp(),
        })
    }
}
```

#### Phase 2: Trace Generation

```rust
fn record_to_rows<F: PrimeField32>(
    record: FriReducedOpeningRecord<F>,
    aux_cols_factory: &MemoryAuxColsFactory<F>,
    slice: &mut [F],
    memory: &OfflineMemory<F>,
) {
    // Extract record data
    let length = length_read.data_at(0).as_canonical_u32() as usize;
    let alpha: [F; EXT_DEG] = alpha_read.data_slice().try_into().unwrap();
    let a_ptr = a_ptr_read.data_at(0);
    let b_ptr = b_ptr_read.data_at(0);
    let write_a = F::ONE - is_init;

    // Generate workload rows (reverse order for trace)
    let mut result = [F::ZERO; EXT_DEG];
    
    for (i, (&a_record_id, &b_record_id)) in record.a_rws.iter()
        .rev()
        .zip_eq(record.b_reads.iter().rev())
        .enumerate() 
    {
        let a_rw = memory.record_by_id(a_record_id);
        let b_read = memory.record_by_id(b_record_id);
        
        let start = i * OVERALL_WIDTH;
        let cols: &mut WorkloadCols<F> = slice[start..start + WL_WIDTH].borrow_mut();
        
        *cols = WorkloadCols {
            prefix: PrefixCols {
                general: GeneralCols {
                    is_workload_row: F::ONE,
                    is_ins_row: F::ZERO,
                    timestamp: record.start_timestamp + F::from_canonical_usize((length - i) * 2),
                },
                a_or_is_first: a,
                data: DataCols {
                    a_ptr: a_ptr + F::from_canonical_usize(length - i),
                    write_a,
                    b_ptr: b_ptr + F::from_canonical_usize((length - i) * EXT_DEG),
                    idx: F::from_canonical_usize(i),
                    result,
                    alpha,
                },
            },
            a_aux: /* memory auxiliary columns */,
            b: b_read.data_slice().try_into().unwrap(),
            b_aux: aux_cols_factory.make_read_aux_cols(b_read),
        };
        
        // Update rolling hash for next iteration
        result = FieldExtension::add(
            FieldExtension::multiply(result, alpha),
            FieldExtension::subtract(b, elem_to_ext(a)),
        );
    }

    // Generate Instruction1 row
    {
        let start = length * OVERALL_WIDTH;
        let cols: &mut Instruction1Cols<F> = slice[start..start + INS_1_WIDTH].borrow_mut();
        
        *cols = Instruction1Cols {
            prefix: PrefixCols {
                general: GeneralCols {
                    is_workload_row: F::ZERO,
                    is_ins_row: F::ONE,
                    timestamp: record.start_timestamp,
                },
                a_or_is_first: F::ONE,  // is_first flag
                data: DataCols {
                    a_ptr,
                    write_a,
                    b_ptr,
                    idx: F::from_canonical_usize(length),
                    result,
                    alpha,
                },
            },
            pc: record.pc,
            a_ptr_ptr,
            a_ptr_aux,
            b_ptr_ptr,
            b_ptr_aux,
            write_a_x_is_first: write_a,
        };
    }

    // Generate Instruction2 row
    {
        let start = (length + 1) * OVERALL_WIDTH;
        let cols: &mut Instruction2Cols<F> = slice[start..start + INS_2_WIDTH].borrow_mut();
        
        *cols = Instruction2Cols {
            general: GeneralCols {
                is_workload_row: F::ZERO,
                is_ins_row: F::ONE,
                timestamp: record.start_timestamp,
            },
            is_first: F::ZERO,
            length_ptr,
            length_aux,
            alpha_ptr,
            alpha_aux,
            result_ptr,
            result_aux,
            hint_id_ptr,
            is_init_ptr,
            is_init_aux,
            write_a_x_is_first: F::ZERO,
        };
    }
}
```

### 4. AIR Constraints Implementation

```rust
impl<AB: InteractionBuilder> Air<AB> for FriReducedOpeningAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let next = main.row_slice(1);
        
        // Evaluate different constraint sets
        self.eval_general(builder, local.deref(), next.deref());
        self.eval_workload_row(builder, local.deref(), next.deref());
        self.eval_instruction1_row(builder, local.deref(), next.deref());
        self.eval_instruction2_row(builder, local.deref(), next.deref());
    }
}
```

#### Workload Row Constraints

```rust
fn eval_workload_row<AB: InteractionBuilder>(
    &self,
    builder: &mut AB,
    local_slice: &[AB::Var],
    next_slice: &[AB::Var],
) {
    let local: &WorkloadCols<AB::Var> = local_slice[..WL_WIDTH].borrow();
    let next: &PrefixCols<AB::Var> = next_slice[..PREFIX_WIDTH].borrow();
    
    let multiplicity = local.prefix.general.is_workload_row;
    
    // Constrain write_a to be boolean
    builder.when(multiplicity).assert_bool(local_data.write_a);
    
    // Memory operations based on write_a flag
    // Read a when write_a is 0
    self.memory_bridge
        .read(
            MemoryAddress::new(native_as.clone(), next.data.a_ptr),
            [local.prefix.a_or_is_first],
            start_timestamp + ptr_reads,
            local.a_aux.as_ref(),
        )
        .eval(builder, (AB::Expr::ONE - local_data.write_a) * multiplicity);
    
    // Write a when write_a is 1
    self.memory_bridge
        .write(
            MemoryAddress::new(native_as.clone(), next.data.a_ptr),
            [local.prefix.a_or_is_first],
            start_timestamp + ptr_reads,
            &local.a_aux,
        )
        .eval(builder, local_data.write_a * multiplicity);
    
    // Always read b
    self.memory_bridge
        .read(
            MemoryAddress::new(native_as, next.data.b_ptr),
            local.b,
            start_timestamp + ptr_reads + AB::Expr::ONE,
            &local.b_aux,
        )
        .eval(builder, multiplicity);
    
    // Transition constraints
    {
        let mut when_transition = builder.when_transition();
        let mut builder = when_transition.when(local.prefix.general.is_workload_row);
        
        // Timestamp decreases by 2
        builder.assert_eq(
            local.prefix.general.timestamp,
            start_timestamp + AB::Expr::TWO,
        );
        
        // Index increases by 1
        builder.assert_eq(local_data.idx + AB::Expr::ONE, next.data.idx);
        
        // Alpha remains constant
        assert_array_eq(&mut builder, local_data.alpha, next.data.alpha);
        
        // Pointer updates
        builder.assert_eq(local_data.a_ptr, next.data.a_ptr + AB::F::ONE);
        builder.assert_eq(
            local_data.b_ptr,
            next.data.b_ptr + AB::F::from_canonical_usize(EXT_DEG),
        );
        
        // Rolling hash constraint
        let mut expected_result = FieldExtension::multiply(local_data.result, local_data.alpha);
        expected_result
            .iter_mut()
            .zip(local.b.iter())
            .for_each(|(e, b)| {
                *e += (*b).into();
            });
        expected_result[0] -= local.prefix.a_or_is_first.into();
        assert_array_eq(&mut builder, expected_result, next.data.result);
    }
}
```

## Common Patterns

### 1. Field Extension Operations

```rust
// Addition
let sum = FieldExtension::add(a, b);

// Subtraction  
let diff = FieldExtension::subtract(a, b);

// Multiplication
let prod = FieldExtension::multiply(a, b);

// Convert base field to extension
let ext = elem_to_ext(base_elem);  // [elem, 0, 0, 0]
```

### 2. Memory Access Patterns

```rust
// Reading field extension element (4 words)
let ext_value = memory.read::<EXT_DEG>(addr_space, base_ptr);

// Writing field extension element
let (record_id, _) = memory.write(addr_space, base_ptr, ext_value);

// Reading single field element
let elem = memory.read_cell(addr_space, ptr);
```

### 3. Timestamp Management

```rust
// Initial timestamp from instruction
let start_timestamp = from_state.timestamp;

// Memory operations consume time
let after_reads = start_timestamp + INSTRUCTION_READS;

// Each workload row takes 2 time units
let workload_timestamp = start_timestamp + (length - i) * 2;

// Final timestamp after all operations
let end_timestamp = start_timestamp + 2 * length + INSTRUCTION_READS + 1;
```

### 4. Constraint Helpers

```rust
// Assert arrays are equal element-wise
fn assert_array_eq<AB: AirBuilder, I1, I2, const N: usize>(
    builder: &mut AB,
    x: [I1; N],
    y: [I2; N],
) {
    for (x, y) in zip_eq(x, y) {
        builder.assert_eq(x, y);
    }
}
```

## Testing Patterns

### Basic Test Structure

```rust
#[test]
fn fri_mat_opening_air_test() {
    // 1. Setup test environment
    let mut tester = VmChipTestBuilder::default();
    let streams = Arc::new(Mutex::new(Streams::default()));
    let mut chip = FriReducedOpeningChip::new(
        tester.execution_bus(),
        tester.program_bus(),
        tester.memory_bridge(),
        tester.offline_memory_mutex_arc(),
        streams.clone(),
    );

    // 2. Generate test data
    let alpha = [F::from_canonical_u32(1), F::from_canonical_u32(2), ...];
    let a = vec![F::from_canonical_u32(10), ...];
    let b = vec![[F::from_canonical_u32(20), ...], ...];

    // 3. Setup memory
    tester.write(address_space, alpha_pointer, alpha);
    tester.write_cell(address_space, length_pointer, F::from_canonical_usize(length));
    // ... more setup

    // 4. Execute instruction
    tester.execute(
        &mut chip,
        &Instruction::from_usize(
            FRI_REDUCED_OPENING.global_opcode(),
            [a_ptr_ptr, b_ptr_ptr, length_ptr, alpha_ptr, result_ptr, hint_id, is_init_ptr],
        ),
    );

    // 5. Verify result
    let expected = compute_fri_mat_opening(alpha, &a, &b);
    assert_eq!(expected, tester.read(address_space, result_pointer));

    // 6. Run constraint verification
    let mut tester = tester.build().load(chip).finalize();
    tester.simple_test().expect("Verification failed");
}
```

## Performance Optimization

### 1. Memory Layout
- Store b-values contiguously for sequential access
- Align field extension elements on 4-word boundaries
- Group related pointers for cache efficiency

### 2. Computation Ordering
- Process in reverse for trace generation
- Batch similar operations together
- Minimize intermediate storage

### 3. Constraint Optimization
- Use unified column structure (27 max)
- Share columns between phases
- Avoid redundant constraints

## Debugging Guide

### Common Issues and Solutions

1. **Wrong Result**
   - Check iteration order (must be reverse for rolling hash)
   - Verify field extension arithmetic
   - Ensure proper index calculations

2. **Constraint Failures**
   - Verify phase flags are exclusive
   - Check timestamp calculations
   - Ensure memory operations use correct addresses

3. **Memory Errors**
   - Validate pointer arithmetic
   - Check address space usage
   - Ensure proper read/write modes

### Debugging Tools

```rust
// Print intermediate values
tracing::debug!("Rolling hash at step {}: {:?}", i, result);

// Verify memory state
let mem_value = tester.read_cell(address_space, ptr);
tracing::debug!("Memory at {}: {:?}", ptr, mem_value);

// Check instruction decoding
tracing::debug!("Instruction operands: {:?}", 
    [a_ptr_ptr, b_ptr_ptr, length_ptr, alpha_ptr, result_ptr, hint_id, is_init_ptr]
);
```

## Advanced Topics

### 1. Hint Streaming Integration
- Hints provide witness data for a-values
- Controlled by is_init flag
- Enables efficient proof generation

### 2. Batch Processing
- Multiple FRI operations can share setup
- Amortize memory access costs
- Improve cache utilization

### 3. Custom Optimizations
- Specialized handling for common lengths
- Precomputed alpha powers
- Vectorized field arithmetic

This implementation guide provides the foundation for working with the FRI component. Always prioritize correctness and security over optimization.