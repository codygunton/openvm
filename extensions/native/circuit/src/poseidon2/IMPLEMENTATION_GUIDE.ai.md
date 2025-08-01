# Native Poseidon2 Implementation Guide

## Overview
This guide provides detailed implementation patterns for working with the Native Poseidon2 component in OpenVM. It covers instruction implementation, trace generation, constraint development, and integration patterns.

## Core Implementation Patterns

### 1. Implementing a New Poseidon2-Based Instruction

#### Step 1: Define the Opcode
```rust
// In openvm_native_compiler
pub enum CustomPoseidon2Opcode {
    HASH_CHAIN = 0x40,
}
```

#### Step 2: Create Executor
```rust
impl<F: PrimeField32> InstructionExecutor<F> for NativePoseidon2Chip<F, SBOX_REGISTERS> {
    fn execute(
        &mut self,
        instruction: Instruction<F>,
        from_state: ExecutionState<u32>,
        ctx: &mut ExecutionContext<F>,
    ) -> Result<ExecutionState<u32>, ExecutionError> {
        let opcode = instruction.opcode.local_opcode_idx(self.air.offset);
        
        match opcode {
            HASH_CHAIN => self.execute_hash_chain(instruction, from_state, ctx),
            _ => self.execute_existing(instruction, from_state, ctx),
        }
    }
}
```

#### Step 3: Implement Execution Logic
```rust
fn execute_hash_chain(
    &mut self,
    instruction: Instruction<F>,
    from_state: ExecutionState<u32>,
    ctx: &mut ExecutionContext<F>,
) -> Result<ExecutionState<u32>, ExecutionError> {
    // Read operands
    let input_ptr = instruction.operands.a();
    let output_ptr = instruction.operands.b();
    let chain_length = instruction.operands.c();
    
    // Create execution record
    let mut record = HashChainRecord {
        instruction,
        from_state,
        chain_steps: vec![],
    };
    
    // Execute chain
    let mut current = self.read_chunk(ctx.memory, input_ptr)?;
    for i in 0..chain_length {
        let next = self.poseidon2_permute(current);
        record.chain_steps.push(ChainStep { input: current, output: next });
        current = next;
    }
    
    // Write result
    self.write_chunk(ctx.memory, output_ptr, current)?;
    
    // Update state
    let to_state = ExecutionState {
        pc: from_state.pc + DEFAULT_PC_STEP,
        timestamp: ctx.timestamp,
    };
    
    self.records.push(ExecutionRecord::HashChain(record));
    Ok(to_state)
}
```

### 2. Trace Generation Patterns

#### Row Type Selection
```rust
fn generate_trace_row<SC: StarkGenericConfig>(
    &self,
    row_index: usize,
    cols: &mut NativePoseidon2Cols<Val<SC>, SBOX_REGISTERS>,
) {
    match &self.execution_type[row_index] {
        ExecutionType::Simple(record) => {
            self.generate_simple_row(record, cols);
        }
        ExecutionType::TopLevel(record) => {
            self.generate_top_level_row(record, cols);
        }
        ExecutionType::InsideRow(record) => {
            self.generate_inside_row(record, cols);
        }
    }
}
```

#### Column Assignment Pattern
```rust
fn generate_top_level_row<F: Field>(
    &self,
    record: &TopLevelRecord<F>,
    cols: &mut NativePoseidon2Cols<F, SBOX_REGISTERS>,
) {
    // Set row type flags
    cols.incorporate_row = F::ONE;
    cols.incorporate_sibling = F::ZERO;
    cols.inside_row = F::ZERO;
    cols.simple = F::ZERO;
    
    // Cast specific columns
    let specific: &mut TopLevelSpecificCols<F> = cols.specific.borrow_mut();
    
    // Populate specific columns
    specific.sibling = record.sibling;
    specific.index_bit = record.index_bit;
    specific.current_height = F::from_canonical_usize(record.height);
    
    // Set common columns
    cols.start_timestamp = record.timestamp;
    cols.very_first_timestamp = record.instruction_start;
    
    // Generate Poseidon2 subcols
    self.generate_poseidon2_cols(&record.input, &mut cols.inner);
}
```

### 3. Memory Operation Patterns

#### Safe Memory Read with Dereferencing
```rust
fn read_array_with_deref<F: Field>(
    memory: &mut OfflineMemory<F>,
    pointer_addr: F,
    timestamp: &mut u32,
) -> Result<Vec<F>, MemoryError> {
    // Read array pointer
    let (array_ptr, ptr_aux) = memory.read(*timestamp, pointer_addr, F::from_canonical_u32(AS::Native));
    *timestamp += 1;
    
    // Read array length
    let (array_len, len_aux) = memory.read(*timestamp, array_ptr + F::ONE, F::from_canonical_u32(AS::Native));
    *timestamp += 1;
    
    // Read array elements
    let mut elements = Vec::new();
    for i in 0..array_len.as_canonical_u32() {
        let (elem, elem_aux) = memory.read(
            *timestamp,
            array_ptr + F::from_canonical_u32(2 + i),
            F::from_canonical_u32(AS::Native),
        );
        elements.push(elem);
        *timestamp += 1;
    }
    
    Ok(elements)
}
```

#### Batch Memory Operations
```rust
fn read_matrix_rows<F: Field>(
    memory: &mut OfflineMemory<F>,
    row_pointers: &[F],
    timestamp: &mut u32,
) -> Vec<Vec<F>> {
    row_pointers
        .par_iter()
        .map(|&ptr| {
            let local_timestamp = timestamp.fetch_add(100); // Reserve timestamp range
            read_array_with_deref(memory, ptr, &mut local_timestamp).unwrap()
        })
        .collect()
}
```

### 4. AIR Constraint Patterns

#### Row Type Constraints
```rust
fn eval_row_types<AB: AirBuilder<F = F>>(&self, builder: &mut AB) {
    let main = builder.main();
    let current = main.row_slice(0);
    
    // Extract row type flags
    let is_simple = current[Self::COL_SIMPLE];
    let is_incorporate_row = current[Self::COL_INCORPORATE_ROW];
    let is_incorporate_sibling = current[Self::COL_INCORPORATE_SIBLING];
    let is_inside_row = current[Self::COL_INSIDE_ROW];
    
    // At most one flag active
    let sum_flags = is_simple + is_incorporate_row + is_incorporate_sibling + is_inside_row;
    builder.assert_bool(sum_flags);
    
    // Flag consistency constraints
    builder.when(is_incorporate_row).assert_one(current[Self::COL_TOP_LEVEL]);
    builder.when(is_inside_row).assert_zero(current[Self::COL_TOP_LEVEL]);
}
```

#### Bus Communication Constraints
```rust
fn eval_bus_communication<AB: InteractionBuilder<F = F>>(&self, builder: &mut AB) {
    let current = builder.main().row_slice(0);
    
    // InsideRow sends to TopLevel
    builder
        .when(current[Self::COL_END_INSIDE_ROW])
        .push_send(
            self.internal_bus.0,
            vec![
                current[Self::COL_HASH_RESULT + 0],
                current[Self::COL_HASH_RESULT + 1],
                // ... remaining hash elements
            ],
            current[Self::COL_MULTIPLICITY],
        );
    
    // TopLevel receives from InsideRow
    builder
        .when(current[Self::COL_INCORPORATE_ROW])
        .push_receive(
            self.internal_bus.0,
            vec![
                current[Self::COL_RECEIVED_HASH + 0],
                current[Self::COL_RECEIVED_HASH + 1],
                // ... remaining hash elements
            ],
            current[Self::COL_MULTIPLICITY],
        );
}
```

#### Memory Consistency Constraints
```rust
fn eval_memory_operations<AB: AirBuilder<F = F>>(&self, builder: &mut AB) {
    let current = builder.main().row_slice(0);
    let memory_cols = current[Self::MEMORY_START..Self::MEMORY_END];
    
    // Constrain read auxiliary columns
    self.memory_bridge.read(
        MemoryAddress::new(
            current[Self::COL_ADDRESS_SPACE],
            current[Self::COL_POINTER],
        ),
        memory_cols,
        current[Self::COL_TIMESTAMP],
        &current[Self::COL_READ_VALUE],
    );
}
```

### 5. Testing Patterns

#### Instruction Test Template
```rust
#[test]
fn test_custom_instruction() {
    let mut chip = create_test_chip();
    let mut memory = OfflineMemory::new();
    
    // Setup test data
    let test_input = [F::from(1u32); CHUNK];
    let input_ptr = F::from(0x1000u32);
    write_array(&mut memory, input_ptr, &test_input);
    
    // Create instruction
    let instruction = Instruction {
        opcode: CustomPoseidon2Opcode::HASH_CHAIN.into(),
        operands: Operands::new(input_ptr, output_ptr, chain_length),
    };
    
    // Execute
    let result = chip.execute(instruction, state, &mut ctx)?;
    
    // Verify
    let output = read_array(&memory, output_ptr);
    assert_eq!(output, expected_output);
}
```

#### Trace Verification
```rust
fn verify_trace_constraints<F: Field>(trace: &RowMajorMatrix<F>) {
    for i in 0..trace.height() {
        let row = trace.row_slice(i);
        
        // Verify row type exclusivity
        let flags_sum = row[COL_SIMPLE] + row[COL_INCORPORATE_ROW] + 
                       row[COL_INCORPORATE_SIBLING] + row[COL_INSIDE_ROW];
        assert!(flags_sum == F::ZERO || flags_sum == F::ONE);
        
        // Verify timestamp ordering
        if i > 0 {
            let prev_row = trace.row_slice(i - 1);
            assert!(row[COL_TIMESTAMP] >= prev_row[COL_TIMESTAMP]);
        }
    }
}
```

### 6. Performance Optimization Patterns

#### Parallel Trace Generation
```rust
fn generate_trace_parallel(&self) -> RowMajorMatrix<F> {
    let width = self.trace_width();
    let height = self.trace_height();
    
    let mut trace = RowMajorMatrix::new(vec![F::ZERO; width * height], width);
    
    trace
        .par_rows_mut()
        .enumerate()
        .for_each(|(i, row)| {
            let mut cols = NativePoseidon2Cols::from_slice_mut(row);
            self.generate_trace_row(i, &mut cols);
        });
    
    trace
}
```

#### Memory Access Batching
```rust
fn batch_memory_reads<F: Field>(
    memory: &mut OfflineMemory<F>,
    addresses: &[F],
    timestamp_start: u32,
) -> Vec<(F, MemoryReadAuxCols<F>)> {
    addresses
        .iter()
        .enumerate()
        .map(|(i, &addr)| {
            memory.read(
                timestamp_start + i as u32,
                addr,
                F::from_canonical_u32(AS::Native),
            )
        })
        .collect()
}
```

## Integration Examples

### Adding to VM Configuration
```rust
impl VmConfig {
    pub fn with_native_poseidon2(mut self) -> Self {
        let chip = NativePoseidon2Chip::new(
            self.system.memory_controller.clone(),
            Poseidon2Config::default(),
        );
        
        self.chips.push(Box::new(chip));
        self
    }
}
```

### Custom Instruction Compiler Support
```rust
impl NativePoseidon2Compiler {
    pub fn compile_hash_chain(
        &self,
        input: ValueOrConst,
        length: u32,
    ) -> CompileResult {
        let instruction = Instruction {
            opcode: HASH_CHAIN.into(),
            operands: Operands::new(
                input.to_operand(),
                self.alloc_result(),
                F::from(length),
            ),
        };
        
        self.emit(instruction)
    }
}
```

## Best Practices

### 1. Error Handling
- Always validate memory addresses before access
- Check array bounds explicitly
- Return meaningful error messages

### 2. Record Keeping
- Store all auxiliary data for constraint verification
- Maintain complete execution history
- Enable reconstruction of execution flow

### 3. Constraint Completeness
- Every computation must be constrained
- No implicit assumptions about values
- Test with random and adversarial inputs

### 4. Documentation
- Document non-obvious algorithmic choices
- Explain performance trade-offs
- Maintain examples for complex patterns