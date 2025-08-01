# Keccak-256 Circuit Implementation Guide

## Overview

This guide provides detailed implementation instructions for the Keccak-256 circuit component in OpenVM. The implementation follows a modular architecture that separates execution logic, constraint generation, and trace building.

## Implementation Steps

### 1. Setting Up the Chip Structure

```rust
pub struct KeccakVmChip<F: PrimeField32> {
    pub air: KeccakVmAir,
    pub records: Vec<KeccakRecord<F>>,
    pub bitwise_lookup_chip: SharedBitwiseOperationLookupChip<8>,
    offset: usize,
    offline_memory: Arc<Mutex<OfflineMemory<F>>>,
}
```

**Key Design Decisions:**
- Use shared bitwise lookup chip for XOR operations (more efficient than inline computation)
- Store execution records for later trace generation
- Thread-safe offline memory access via Arc<Mutex<>>

### 2. Implementing Instruction Execution

The `InstructionExecutor` trait implementation handles the runtime execution:

```rust
fn execute(&mut self, memory: &mut MemoryController<F>, instruction: &Instruction<F>, from_state: ExecutionState<u32>) -> Result<ExecutionState<u32>, ExecutionError>
```

**Execution Flow:**
1. Read registers (dst, src, len) from memory
2. For each input block (136 bytes):
   - Read input data from memory
   - Update Keccak state
   - Handle padding for final block
3. Write 32-byte digest to memory
4. Track all memory accesses in records

**Critical Implementation Details:**
- Timestamp management: Increment by exact memory access count
- Partial reads: Handle input lengths not divisible by 4
- Padding: Apply 10*1 rule correctly for edge cases

### 3. Defining the AIR Constraints

The AIR implementation enforces correctness through several constraint systems:

#### 3.1 Keccak-f Permutation
```rust
pub fn eval_keccak_f<AB: AirBuilder>(&self, builder: &mut AB) {
    let keccak_f_air = KeccakAir {};
    let mut sub_builder = SubAirBuilder::<AB, KeccakAir, AB::Var>::new(builder, 0..NUM_KECCAK_PERM_COLS);
    keccak_f_air.eval(&mut sub_builder);
}
```
- Reuse p3-keccak-air for the core permutation
- Columns must be placed at the beginning of the trace

#### 3.2 Padding Constraints
```rust
pub fn constrain_padding<AB: AirBuilder>(&self, builder: &mut AB, local: &KeccakVmCols<AB::Var>, next: &KeccakVmCols<AB::Var>)
```
- Enforce 10*1 padding rule
- Single byte: 0x81
- Multiple bytes: 0x01...0x80
- Validate is_padding_byte transitions

#### 3.3 State Absorption
```rust
pub fn constrain_absorb<AB: InteractionBuilder>(&self, builder: &mut AB, local: &KeccakVmCols<AB::Var>, next: &KeccakVmCols<AB::Var>)
```
- XOR input bytes with state using lookup tables
- Handle 16-bit to 8-bit decomposition
- Maintain capacity portion unchanged

### 4. Trace Generation Strategy

The trace generator builds the execution trace for proving:

```rust
fn generate_air_proof_input(self) -> AirProofInput<SC>
```

**Key Steps:**
1. Collect all execution records
2. Generate Keccak-f traces using p3-keccak-air
3. Populate instruction and sponge columns
4. Generate memory auxiliary columns
5. Handle state transitions between blocks

**Performance Optimizations:**
- Parallel trace generation using rayon
- Pre-calculate state differences
- Batch bitwise lookup requests

### 5. Column Layout Design

```rust
#[repr(C)]
pub struct KeccakVmCols<T> {
    pub inner: KeccakPermCols<T>,      // Must be first!
    pub sponge: KeccakSpongeCols<T>,
    pub instruction: KeccakInstructionCols<T>,
    pub mem_oc: KeccakMemoryCols<T>,
}
```

**Layout Considerations:**
- KeccakPermCols must be first for sub-AIR to work
- Group related columns together
- Minimize padding with repr(C)
- Use AlignedBorrow for zero-copy access

### 6. Memory Access Pattern

**Register Reads (once per instruction):**
1. dst pointer → extract destination address
2. src pointer → extract source address  
3. len pointer → extract input length

**Input Reads (per 136-byte block):**
- Read up to 34 4-byte words
- Handle partial final word
- Track timestamps sequentially

**Output Writes (once per instruction):**
- Write 8 4-byte words for 32-byte digest
- Sequential timestamps after input reads

### 7. Integration with VM Extension System

```rust
impl<F: PrimeField32> VmExtension<F> for Keccak256 {
    fn build(&self, builder: &mut VmInventoryBuilder<F>) -> Result<VmInventory<Self::Executor, Self::Periphery>, VmInventoryError>
```

**Setup Requirements:**
1. Register opcode handler for KECCAK256
2. Create or reuse bitwise lookup chip
3. Configure memory bridge and buses
4. Set address bit limits

### 8. Testing Strategy

**Unit Tests:**
- Single block hashing
- Multi-block hashing  
- Edge cases (empty input, rate-1 length)
- Padding verification

**Integration Tests:**
- Full VM execution with Keccak instructions
- Memory consistency checks
- Trace validity verification

### 9. Common Pitfalls and Solutions

**Pitfall 1: Incorrect Timestamp Management**
- Solution: Calculate exact timestamp delta based on memory accesses
- Use KeccakVmAir::timestamp_change() for consistency

**Pitfall 2: Padding Edge Cases**
- Solution: Handle rate-1 length specially (single byte 0x81)
- Test extensively with boundary lengths

**Pitfall 3: State Transition Bugs**
- Solution: Clear separation between rounds and blocks
- Validate is_new_start transitions

**Pitfall 4: Memory Alignment Issues**
- Solution: Handle partial reads explicitly
- Use separate partial_block columns

### 10. Performance Optimization Tips

1. **Minimize Constraint Degree:**
   - Precompute common expressions
   - Use auxiliary columns for high-degree terms
   
2. **Reduce Trace Width:**
   - Share columns between mutually exclusive operations
   - Pack boolean flags efficiently

3. **Optimize Lookups:**
   - Batch XOR operations
   - Reuse bitwise chip across extensions

4. **Parallel Processing:**
   - Generate traces in parallel by block
   - Batch memory auxiliary column generation

## Advanced Topics

### Custom Padding Schemes
While this implementation uses standard Keccak padding, the architecture supports custom padding by modifying `constrain_padding()`.

### Variable Output Lengths
The current implementation fixes output at 32 bytes, but can be extended for variable-length outputs by modifying digest write logic.

### Integration with Other Hash Functions
The sponge construction pattern can be adapted for other hash functions by replacing the permutation and adjusting parameters.

## Debugging Tips

1. **Trace Validation:**
   - Verify permutation output matches expected values
   - Check padding bytes in trace
   - Validate memory timestamps

2. **Constraint Debugging:**
   - Test constraints with known good traces
   - Use assertion helpers for complex expressions
   - Log intermediate values during execution

3. **Memory Consistency:**
   - Track all RecordIds through execution
   - Verify offline memory contains expected data
   - Check timestamp ordering

## Conclusion

This implementation provides an efficient, proven-secure Keccak-256 hasher for the OpenVM zkVM. The modular design allows for easy testing, debugging, and potential extensions while maintaining high performance for proving.