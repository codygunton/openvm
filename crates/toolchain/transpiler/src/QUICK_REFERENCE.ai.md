# Quick Reference: OpenVM Transpiler

## Core Types

### Transpiler
```rust
use openvm_transpiler::Transpiler;

// Create and configure
let transpiler = Transpiler::<F>::new()
    .with_extension(ext1)
    .with_extension(ext2);

// Transpile instructions
let result = transpiler.transpile(&instructions)?;
```

### ELF Parsing
```rust
use openvm_transpiler::elf::Elf;

// Parse ELF file
let elf = Elf::decode(&elf_bytes, max_memory)?;

// Access components
let instructions = elf.instructions;     // Vec<u32>
let pc_start = elf.pc_start;            // u32
let memory = elf.memory_image;          // BTreeMap<u32, u32>
let functions = elf.fn_bounds;          // FnBounds
```

### TranspilerExtension Trait
```rust
use openvm_transpiler::{TranspilerExtension, TranspilerOutput};

impl<F: PrimeField32> TranspilerExtension<F> for MyExt {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        // Return None if not recognized
        // Return Some(output) if handled
    }
}
```

### TranspilerOutput Constructors
```rust
// One RISC-V → One OpenVM
TranspilerOutput::one_to_one(instruction)

// Many RISC-V → One OpenVM  
TranspilerOutput::many_to_one(instruction, used_u32s)

// Create instruction gap
TranspilerOutput::gap(gap_length, used_u32s)
```

## Utility Functions

### Instruction Format Helpers
```rust
use openvm_transpiler::util::*;

// R-type: op rd, rs1, rs2
from_r_type(opcode, addr_space, &decoded, allow_rd_zero)

// I-type: op rd, rs1, imm
from_i_type(opcode, &decoded)

// Load: lw rd, offset(rs1)
from_load(opcode, &decoded)

// I-type with shift: slli rd, rs1, shamt
from_i_type_shamt(opcode, &decoded)

// S-type: sw rs2, offset(rs1)
from_s_type(opcode, &decoded)

// B-type: beq rs1, rs2, offset
from_b_type(opcode, &decoded)

// J-type: jal rd, offset
from_j_type(opcode, &decoded)

// U-type: lui rd, imm
from_u_type(opcode, &decoded)

// Special instructions
nop::<F>()      // No operation
unimp::<F>()    // Exit with code 2
```

### Memory Conversion
```rust
use openvm_transpiler::util::elf_memory_image_to_openvm_memory_image;

// Convert ELF memory to OpenVM format
let vm_memory = elf_memory_image_to_openvm_memory_image(elf.memory_image);
```

## FromElf Trait
```rust
use openvm_transpiler::FromElf;
use openvm_instructions::exe::VmExe;

// Convert ELF to executable
let exe = VmExe::from_elf(elf, transpiler)?;
```

## Error Types
```rust
use openvm_transpiler::transpiler::TranspilerError;

match error {
    TranspilerError::AmbiguousNextInstruction => {
        // Multiple processors claimed instruction
    },
    TranspilerError::ParseError(insn) => {
        // Unrecognized instruction: insn
    }
}
```

## Constants (via dependencies)
```rust
use openvm_instructions::riscv::{
    RV32_REGISTER_NUM_LIMBS,  // 4
    RV32_CELL_BITS,          // 8
    RV32_IMM_AS,             // 0
    RV32_REGISTER_AS,        // 1
    RV32_MEMORY_AS,          // 2
};

use openvm_platform::WORD_SIZE;  // 4

use openvm_instructions::program::{
    DEFAULT_PC_STEP,         // 4
    MAX_ALLOWED_PC,          // (1 << 30) - 1
};
```

## Common Patterns

### Basic Extension
```rust
struct MyExtension;

impl<F: PrimeField32> TranspilerExtension<F> for MyExtension {
    fn process_custom(&self, stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let insn = stream[0];
        
        // Quick reject
        if (insn & 0x7f) != MY_OPCODE {
            return None;
        }
        
        // Decode and convert
        let decoded = decode(insn);
        let vm_insn = convert(decoded);
        
        Some(TranspilerOutput::one_to_one(vm_insn))
    }
}
```

### Full Transpilation Flow
```rust
// 1. Read ELF
let elf_bytes = std::fs::read("program.elf")?;

// 2. Parse ELF
let elf = Elf::decode(&elf_bytes, GUEST_MAX_MEM)?;

// 3. Create transpiler
let transpiler = Transpiler::<BabyBear>::new()
    .with_extension(Rv32Extension::new());

// 4. Convert to executable
let exe = VmExe::from_elf(elf, transpiler)?;
```

### Instruction Building
```rust
use openvm_instructions::instruction::Instruction;
use openvm_instructions::VmOpcode;

// Manual instruction creation
let insn = Instruction::new(
    VmOpcode::from_usize(opcode),
    a, b, c,  // Main operands
    d, e,     // Flags/modes
    f, g,     // Additional data
);

// Using helpers for standard formats
let insn = from_r_type(opcode, RV32_REGISTER_AS, &r_decoded, false);
```

## Quick Checks

### Valid RISC-V Instruction
```rust
fn is_valid_rv32(insn: u32) -> bool {
    // Check for compressed (16-bit) - not supported
    if insn & 0x3 != 0x3 {
        return false;
    }
    
    // Check opcode is not all 1s
    if insn & 0x7f == 0x7f {
        return false;
    }
    
    true
}
```

### Register Validation
```rust
fn validate_register(reg: usize) -> bool {
    reg < 32  // x0-x31
}

fn should_write_register(reg: usize) -> bool {
    reg != 0  // x0 is always zero
}
```

### Memory Alignment
```rust
fn is_word_aligned(addr: u32) -> bool {
    addr & 0x3 == 0
}

fn align_to_word(addr: u32) -> u32 {
    addr & !0x3
}
```

## Debug Tips

### Trace Transpilation
```rust
let transpiler = Transpiler::<F>::new()
    .with_processor(Rc::new(DebugExtension));

struct DebugExtension;
impl<F: PrimeField32> TranspilerExtension<F> for DebugExtension {
    fn process_custom(&self, stream: &[u32]) -> Option<TranspilerOutput<F>> {
        eprintln!("Transpiling: 0x{:08x}", stream[0]);
        None  // Don't actually handle
    }
}
```

### Inspect ELF
```rust
let elf = Elf::decode(&bytes, max_mem)?;
println!("Entry: 0x{:08x}", elf.pc_start);
println!("Base: 0x{:08x}", elf.pc_base);
println!("Instructions: {}", elf.instructions.len());
println!("Memory entries: {}", elf.memory_image.len());
```