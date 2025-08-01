# Keccak-256 Circuit Quick Reference

## Core Types

### KeccakVmChip<F>
Main chip implementing Keccak-256 instruction execution.
```rust
pub struct KeccakVmChip<F: PrimeField32> {
    pub air: KeccakVmAir,
    pub records: Vec<KeccakRecord<F>>,
    pub bitwise_lookup_chip: SharedBitwiseOperationLookupChip<8>,
    offset: usize,
    offline_memory: Arc<Mutex<OfflineMemory<F>>>,
}
```

### KeccakRecord<F>
Record of a single Keccak-256 instruction execution.
```rust
pub struct KeccakRecord<F> {
    pub pc: F,                                          // Program counter
    pub dst_read: RecordId,                            // Destination register read
    pub src_read: RecordId,                            // Source register read  
    pub len_read: RecordId,                            // Length register read
    pub input_blocks: Vec<KeccakInputBlock>,           // Input data blocks
    pub digest_writes: [RecordId; KECCAK_DIGEST_WRITES], // Output writes
}
```

### KeccakInputBlock
Single input block (up to 136 bytes) with padding.
```rust
pub struct KeccakInputBlock {
    pub reads: Vec<RecordId>,                          // Memory reads
    pub partial_read_idx: Option<usize>,               // Index of partial read
    pub padded_bytes: [u8; KECCAK_RATE_BYTES],        // Padded input bytes
    pub remaining_len: usize,                          // Bytes left to process
    pub src: usize,                                    // Source address
    pub is_new_start: bool,                           // First block flag
}
```

## Key Constants

```rust
// Keccak parameters
pub const KECCAK_WIDTH_BYTES: usize = 200;    // Total state size
pub const KECCAK_RATE_BYTES: usize = 136;     // Rate (input) portion
pub const KECCAK_CAPACITY_BYTES: usize = 64;  // Capacity portion
pub const KECCAK_DIGEST_BYTES: usize = 32;    // Output size

// Memory access
const KECCAK_REGISTER_READS: usize = 3;       // dst, src, len
const KECCAK_WORD_SIZE: usize = 4;            // Bytes per memory op
const KECCAK_ABSORB_READS: usize = 34;        // Reads per input block
const KECCAK_DIGEST_WRITES: usize = 8;        // Writes for output

// Derived values
pub const KECCAK_RATE_U16S: usize = 68;       // Rate in 16-bit limbs
pub const NUM_ABSORB_ROUNDS: usize = 17;      // Rate in 64-bit words
```

## Instruction Format

```
KECCAK256 dst, src, len
```
- `dst`: Memory address to write 32-byte hash
- `src`: Memory address of input data
- `len`: Length of input in bytes

## API Methods

### Chip Construction
```rust
impl<F: PrimeField32> KeccakVmChip<F> {
    pub fn new(
        execution_bus: ExecutionBus,
        program_bus: ProgramBus,
        memory_bridge: MemoryBridge,
        address_bits: usize,
        bitwise_lookup_chip: SharedBitwiseOperationLookupChip<8>,
        offset: usize,
        offline_memory: Arc<Mutex<OfflineMemory<F>>>,
    ) -> Self
}
```

### Instruction Execution
```rust
impl<F: PrimeField32> InstructionExecutor<F> for KeccakVmChip<F> {
    fn execute(
        &mut self,
        memory: &mut MemoryController<F>,
        instruction: &Instruction<F>,
        from_state: ExecutionState<u32>,
    ) -> Result<ExecutionState<u32>, ExecutionError>
}
```

### Trace Generation
```rust
impl<SC: StarkGenericConfig> Chip<SC> for KeccakVmChip<Val<SC>> {
    fn generate_air_proof_input(self) -> AirProofInput<SC>
}
```

## Column Structure

### KeccakVmCols<T>
Main trace columns (must maintain order!).
```rust
pub struct KeccakVmCols<T> {
    pub inner: KeccakPermCols<T>,              // Keccak-f columns (FIRST!)
    pub sponge: KeccakSpongeCols<T>,           // Sponge/padding columns
    pub instruction: KeccakInstructionCols<T>, // Instruction data
    pub mem_oc: KeccakMemoryCols<T>,          // Memory auxiliary
}
```

### Helper Methods
```rust
impl<T: Copy> KeccakVmCols<T> {
    pub fn remaining_len(&self) -> T
    pub fn is_new_start(&self) -> T  
    pub fn postimage(&self, y: usize, x: usize, limb: usize) -> T
    pub fn is_first_round(&self) -> T
    pub fn is_last_round(&self) -> T
}
```

## AIR Constraint Methods

### Main Evaluation
```rust
impl<AB: InteractionBuilder> Air<AB> for KeccakVmAir {
    fn eval(&self, builder: &mut AB)
}
```

### Constraint Helpers
```rust
impl KeccakVmAir {
    // Evaluate Keccak-f permutation
    pub fn eval_keccak_f<AB: AirBuilder>(&self, builder: &mut AB)
    
    // Enforce padding rules
    pub fn constrain_padding<AB: AirBuilder>(&self, builder: &mut AB, 
        local: &KeccakVmCols<AB::Var>, next: &KeccakVmCols<AB::Var>)
    
    // XOR absorption constraints
    pub fn constrain_absorb<AB: InteractionBuilder>(&self, builder: &mut AB,
        local: &KeccakVmCols<AB::Var>, next: &KeccakVmCols<AB::Var>)
    
    // Instruction execution
    pub fn eval_instruction<AB: InteractionBuilder>(&self, builder: &mut AB,
        local: &KeccakVmCols<AB::Var>, 
        register_aux: &[MemoryReadAuxCols<AB::Var>; 3]) -> AB::Expr
    
    // Memory read constraints  
    pub fn constrain_input_read<AB: InteractionBuilder>(&self, builder: &mut AB,
        local: &KeccakVmCols<AB::Var>, start_read_timestamp: AB::Expr,
        mem_aux: &[MemoryReadAuxCols<AB::Var>; 34]) -> AB::Expr
    
    // Memory write constraints
    pub fn constrain_output_write<AB: InteractionBuilder>(&self, builder: &mut AB,
        local: &KeccakVmCols<AB::Var>, start_write_timestamp: AB::Expr,
        mem_aux: &[MemoryWriteAuxCols<AB::Var, 4>; 8])
}
```

## Utility Functions

```rust
// Wrapper for Keccak-f permutation
pub fn keccak_f(state: [u64; 25]) -> [u64; 25]

// Standard Keccak-256 hash
pub fn keccak256(input: &[u8]) -> [u8; 32]

// Calculate number of Keccak-f rounds needed
pub fn num_keccak_f(byte_len: usize) -> usize
```

## Extension Configuration

### Basic Setup
```rust
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Keccak256;

impl<F: PrimeField32> VmExtension<F> for Keccak256 {
    type Executor = Keccak256Executor<F>;
    type Periphery = Keccak256Periphery<F>;
}
```

### Full VM Config
```rust
#[derive(VmConfig)]
pub struct Keccak256Rv32Config {
    #[system] pub system: SystemConfig,
    #[extension] pub rv32i: Rv32I,
    #[extension] pub rv32m: Rv32M,
    #[extension] pub io: Rv32Io,
    #[extension] pub keccak: Keccak256,
}
```

## Usage Patterns

### In Guest Code
```rust
// Direct hash computation
let hash = keccak256(b"Hello, World!");

// With custom memory locations
let input = b"data to hash";
let mut output = [0u8; 32];
unsafe {
    asm!(
        "ecall",
        in("a0") output.as_mut_ptr(),
        in("a1") input.as_ptr(),
        in("a2") input.len(),
        in("t0") KECCAK256_OPCODE,
    );
}
```

### Trace Height Calculation
```rust
let num_blocks = (input_len + KECCAK_RATE_BYTES - 1) / KECCAK_RATE_BYTES;
let trace_height = num_blocks * 24; // 24 rounds per permutation
```

### Timestamp Delta
```rust
let timestamp_delta = len + (KECCAK_REGISTER_READS + KECCAK_ABSORB_READS + KECCAK_DIGEST_WRITES);
// For each instruction: len + 3 + 34 + 8 = len + 45
```

## Common Patterns

### Multi-block Processing
```rust
for (block_idx, block) in input_blocks.iter().enumerate() {
    if block_idx != 0 {
        // Add register read timestamps for continuation
        memory.increment_timestamp_by(KECCAK_REGISTER_READS as u32);
    }
    // Process block...
}
```

### Padding Application
```rust
if remaining_len == KECCAK_RATE_BYTES - 1 {
    // Single byte padding
    padded_bytes[remaining_len] = 0x81;
} else {
    // Multi-byte padding  
    padded_bytes[remaining_len] = 0x01;
    padded_bytes[KECCAK_RATE_BYTES - 1] = 0x80;
}
```

### State Decomposition
```rust
// Convert 16-bit limb to bytes for XOR
let hi = state_hi[i];
let lo = state_limb - hi * 256;
// Now hi and lo are both bytes
```

## Debugging

### Check Padding
```rust
// Verify padding bytes follow 10*1 rule
for i in 0..KECCAK_RATE_BYTES {
    if is_padding_byte[i] {
        match (i == first_padding_idx, i == KECCAK_RATE_BYTES - 1) {
            (true, true) => assert_eq!(block_bytes[i], 0x81),
            (true, false) => assert_eq!(block_bytes[i], 0x01),
            (false, true) => assert_eq!(block_bytes[i], 0x80),
            (false, false) => assert_eq!(block_bytes[i], 0x00),
        }
    }
}
```

### Verify State Transition
```rust
// Check absorption is correct
let mut expected_state = prev_state;
for i in 0..NUM_ABSORB_ROUNDS {
    expected_state[i] ^= u64::from_le_bytes(input_bytes[i*8..(i+1)*8]);
}
keccakf(&mut expected_state);
assert_eq!(expected_state, next_state);
```