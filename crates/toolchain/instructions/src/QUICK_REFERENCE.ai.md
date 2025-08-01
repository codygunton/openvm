# QUICK_REFERENCE: OpenVM Instructions

## Core Types

### Instruction<F>
```rust
pub struct Instruction<F> {
    pub opcode: VmOpcode,
    pub a: F, pub b: F, pub c: F, pub d: F,
    pub e: F, pub f: F, pub g: F,
}
```

### VmOpcode
```rust
pub struct VmOpcode(usize);
```

### Program<F>
```rust
pub struct Program<F> {
    pub instructions_and_debug_infos: Vec<Option<(Instruction<F>, Option<DebugInfo>)>>,
    pub step: u32,
    pub pc_base: u32,
}
```

### VmExe<F>
```rust
pub struct VmExe<F> {
    pub program: Program<F>,
    pub pc_start: u32,
    pub init_memory: MemoryImage<F>,
    pub fn_bounds: FnBounds,
}
```

## Creating Instructions

```rust
// 5 operands from signed integers
Instruction::from_isize(opcode, a, b, c, d, e)

// N operands from unsigned integers  
Instruction::from_usize(opcode, [operands...])

// All 7 operands from signed integers
Instruction::large_from_isize(opcode, a, b, c, d, e, f, g)

// Phantom instruction
Instruction::phantom(discriminant, a, b, c_upper)

// Debug phantom
Instruction::debug(discriminant)

// Default (all zeros)
Instruction::default()
```

## Creating Programs

```rust
// Empty program
Program::new_empty(step, pc_base)

// From instructions (no debug info)
Program::from_instructions(&instructions)
Program::new_without_debug_infos(&instructions, step, pc_base)

// From optional instructions
Program::new_without_debug_infos_with_option(&instructions, step, pc_base)

// With debug info
Program::from_instructions_and_debug_infos(&instructions, &debug_infos)

// Add instruction
program.push_instruction(instruction)
program.push_instruction_and_debug_info(instruction, debug_info)

// Merge programs
program.append(other_program)

// Strip debug info
program.strip_debug_infos()
```

## Program Operations

```rust
// Get info
program.len()                       // Total slots
program.is_empty()                  // Check if empty
program.num_defined_instructions()  // Non-None count

// Get instructions
program.defined_instructions()      // Vec<Instruction<F>>
program.debug_infos()              // Vec<Option<DebugInfo>>
program.enumerate_by_pc()          // Vec<(pc, inst, debug)>

// Access by index
program.get_instruction_and_debug_info(index)
```

## Creating Executables

```rust
// Basic executable
VmExe::new(program)
VmExe::from(program)

// With configuration
exe.with_pc_start(pc)
exe.with_init_memory(memory)

// Add function bounds
exe.fn_bounds.insert(pc, FnBound {
    start: 0x1000,
    end: 0x2000,
    name: "main".to_string(),
});
```

## Opcodes

### System Opcodes
```rust
SystemOpcode::TERMINATE   // offset 0x0
SystemOpcode::PHANTOM     // offset 0x0

PublishOpcode::PUBLISH    // offset 0x020

UnimplementedOpcode::REPLACE_ME  // offset 0xdeadaf
```

### LocalOpcode Trait
```rust
trait LocalOpcode {
    const CLASS_OFFSET: usize;
    fn from_usize(value: usize) -> Self;
    fn local_usize(&self) -> usize;
    fn global_opcode(&self) -> VmOpcode;
}
```

### VmOpcode Methods
```rust
opcode.as_usize()                    // Get raw value
opcode.local_opcode_idx(offset)      // Get local index
opcode.to_field::<F>()              // Convert to field element
VmOpcode::from_usize(value)         // Create from usize
```

## Phantom Instructions

### System Phantoms
```rust
SysPhantom::Nop         // 0 - No operation
SysPhantom::DebugPanic  // 1 - Panic with backtrace
SysPhantom::CtStart     // 2 - Start tracing
SysPhantom::CtEnd       // 3 - End tracing
```

### Creating Phantom Instructions
```rust
// Debug phantom
Instruction::debug(PhantomDiscriminant(SysPhantom::Nop as u16))

// Phantom with data
Instruction::phantom(
    PhantomDiscriminant(0x100),
    field_a,
    field_b,
    0x1234  // c_upper
)
```

## Constants

### Program Counter
```rust
PC_BITS: usize = 30                    // 30-bit PC
DEFAULT_PC_STEP: u32 = 4               // RISC-V compatible
MAX_ALLOWED_PC: u32 = (1 << 30) - 1    // ~1 billion
```

### RISC-V
```rust
RV32_REGISTER_NUM_LIMBS: usize = 4     // 32-bit as 4×8-bit
RV32_CELL_BITS: usize = 8              // 8 bits per limb
RV32_IMM_AS: u32 = 0                   // Immediate address space
RV32_REGISTER_AS: u32 = 1              // Register address space
RV32_MEMORY_AS: u32 = 2                // Memory address space
```

### Instructions
```rust
NUM_OPERANDS: usize = 7                // Operands per instruction
```

## Utilities

### Field Conversion
```rust
// Convert signed to field (handles negatives)
isize_to_field::<F>(value)

// Parse BigUint from string (auto-detects base)
parse_biguint_auto("0x1234")    // hex
parse_biguint_auto("0b1010")    // binary
parse_biguint_auto("42")        // decimal
```

## Memory Types

```rust
// Memory image: (address_space, address) -> value
type MemoryImage<F> = BTreeMap<(u32, u32), F>;

// Function bounds: pc -> metadata
type FnBounds = BTreeMap<u32, FnBound>;
```

## Debug Info

```rust
pub struct DebugInfo {
    pub dsl_instruction: String,
    pub trace: Option<Backtrace>,
}

DebugInfo::new(dsl_instruction, trace)
```

## Common Patterns

### Define Custom Opcodes
```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, 
         EnumCount, EnumIter, FromRepr, LocalOpcode)]
#[opcode_offset = 0x100]
#[repr(usize)]
pub enum MyOpcode {
    OP1,
    OP2,
}
```

### Create Instruction Helper
```rust
fn make_add<F: Field>(dst: usize, src1: usize, src2: usize) -> Instruction<F> {
    Instruction::from_usize(
        MyOpcode::ADD.global_opcode(),
        [dst, src1, src2]
    )
}
```

### Initialize Memory
```rust
let mut memory: MemoryImage<F> = BTreeMap::new();
memory.insert((RV32_REGISTER_AS, 2), sp_value);     // Stack pointer
memory.insert((RV32_MEMORY_AS, addr), data_value);  // Data
```

### Iterate Instructions
```rust
// With PC
for (pc, inst, debug) in program.enumerate_by_pc() {
    println!("PC {:#x}: {:?}", pc, inst.opcode);
}

// Just instructions
for inst in program.defined_instructions() {
    process_instruction(&inst);
}
```

### Find Instruction at PC
```rust
let index = ((pc - program.pc_base) / program.step) as usize;
if let Some((inst, debug)) = program.get_instruction_and_debug_info(index) {
    // Found instruction
}
```

## Serialization

```rust
// Serialize program (excludes debug info)
let bytes = bitcode::serialize(&program)?;

// Deserialize
let program: Program<F> = bitcode::deserialize(&bytes)?;

// JSON for executables
let json = serde_json::to_string(&exe)?;
```

## Type Constraints

```rust
// Most functions require:
F: Field                           // Basic field operations
F: Serialize                       // For serialization
F: for<'de> Deserialize<'de>      // For deserialization  
F: Ord                            // For BTreeMap in VmExe
```