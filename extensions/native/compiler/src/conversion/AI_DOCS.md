# Conversion Module Technical Documentation

## Module Overview
The conversion module (`openvm_native_compiler::conversion`) is the final stage of the OpenVM native compiler pipeline. It transforms assembly instructions into executable VM instructions with proper encoding and addressing.

## Core Types

### `CompilerOptions`
Configuration struct controlling the conversion process:
```rust
pub struct CompilerOptions {
    pub word_size: usize,           // Memory alignment (default: 8)
    pub enable_cycle_tracker: bool,  // Performance tracking (default: false)
}
```

### `AS` (Address Specifier)
Enum representing operand addressing modes:
```rust
pub enum AS {
    Immediate = 0,  // Direct value operand
    Native = 4,     // Memory address operand
}
```

## Key Functions

### `convert_instruction<F, EF>`
Converts a single assembly instruction to one or more VM instructions.

**Parameters:**
- `instruction: AsmInstruction<F, EF>` - The assembly instruction to convert
- `debug_info: Option<DebugInfo>` - Debug metadata
- `pc: F` - Current program counter value
- `labels: impl Fn(F) -> F` - Label resolution function
- `options: &CompilerOptions` - Conversion options

**Returns:** `Program<F>` - Converted VM instructions

### `convert_program<F, EF>`
Converts an entire assembly program to VM format.

**Process:**
1. Initializes register 0 to zero
2. Calculates block starting addresses
3. Converts each instruction with proper PC tracking
4. Resolves labels to absolute addresses

## Instruction Categories

### Memory Operations
- **LoadFI/LoadEI**: Load field/extension from memory
- **StoreFI/StoreEI**: Store field/extension to memory
- **Addressing**: `mem[dst] ← mem[mem[src] + index * size + offset]`

### Control Flow
- **Jump**: Unconditional branch with link register
- **Bne/Beq**: Conditional branches (not equal/equal)
- **BneE/BeqE**: Extension field comparisons
- **Label Resolution**: Converts labels to PC-relative offsets

### Arithmetic Operations

#### Field Operations
- **AddF/SubF/MulF/DivF**: Binary operations on memory operands
- **AddFI/SubFI/MulFI/DivFI**: Operations with immediate values
- **SubFIN/DivFIN**: Reverse immediate operations

#### Extension Field Operations
- **AddE/SubE**: Addition/subtraction in extension field
- **MulE/DivE**: Multiplication/division using BBE4 opcodes

### Special Operations

#### Poseidon2 Hash
- **Compress**: 2-to-1 compression function
- **Permute**: Full state permutation

#### FRI Operations
- **FriReducedOpening**: Polynomial opening verification

#### Batch Verification
- **VerifyBatchFelt/Ext**: Merkle proof verification

#### System Operations
- **Halt**: Successful termination
- **Trap**: Error termination with debug panic
- **CycleTracker**: Performance measurement hooks

### Hint Operations
- **HintInputVec**: Prepare input vector
- **HintFelt**: Prepare field element
- **HintBits**: Bit decomposition hints
- **StoreHintWordI/ExtI**: Store hints to memory

## Memory Model

### Addressing
- Base address from register (Native mode)
- Index-based offsets: `base + index * size + offset`
- Word size alignment enforced by `CompilerOptions`

### Address Specifiers
- `AS::Native` (4): Memory indirect addressing
- `AS::Immediate` (0): Direct value operands

## Extension Field Handling

### Multi-word Operations
Extension field elements span multiple words (dimension EF::D):
- Operations iterate over all components
- Branch instructions handle all components sequentially
- PC increments account for multi-instruction sequences

### Branch Optimization
- **BneE**: Early exit on first mismatch
- **BeqE**: Requires all components to match

## Program Structure

### Initialization
Every program starts with:
```
mem[0] ← 0  // Initialize register 0
```

### Block Layout
- Each block starts at aligned PC addresses
- PC increments by `DEFAULT_PC_STEP` (4)
- Labels resolved to block start addresses

## Debug Support

### Debug Information
- Preserved through conversion process
- Attached to each generated instruction
- Enables source-level debugging

### Phantom Instructions
Special no-op instructions for:
- Debug prints (`Print`)
- Cycle tracking (`CtStart/CtEnd`)
- Panic handling (`DebugPanic`)

## Optimization Considerations

### Instruction Fusion
Some ASM instructions expand to multiple VM instructions:
- Extension field branches (4 comparisons)
- Extension field prints (4 outputs)

### Address Calculation
- Compile-time offset computation where possible
- Efficient encoding of immediate values
- Minimal register usage

## Error Handling

### Range Checks
- Bit width validation (0-16 for x, 0-14 for y)
- Compile-time assertions for invalid ranges

### Type Safety
- Generic over prime fields (F: PrimeField32)
- Extension field compatibility (EF: ExtensionField<F>)

## Integration Points

### Input Format
Accepts `AssemblyCode<F, EF>` containing:
- Blocks of instructions
- Debug information
- Label definitions

### Output Format
Produces `Program<F>` with:
- Linear instruction sequence
- Embedded debug metadata
- Resolved addresses

## Performance Notes

### Cycle Tracking
When enabled via `CompilerOptions`:
- Inserts tracking phantoms at start/end markers
- No overhead when disabled
- Zero runtime cost (phantom instructions)

### Memory Efficiency
- Compact instruction encoding
- Shared debug information
- Minimal memory allocations during conversion