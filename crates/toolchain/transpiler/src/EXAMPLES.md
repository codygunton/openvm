# OpenVM Transpiler Examples

## Basic Usage

### Creating a Basic Transpiler
```rust
use openvm_toolchain_transpiler::{Transpiler, FromElf};
use openvm_instructions::exe::VmExe;
use openvm_stark_backend::p3_baby_bear::BabyBear;

// Create a new transpiler without extensions
let transpiler = Transpiler::<BabyBear>::new();

// Convert ELF to executable
let elf = load_elf_from_file("program.elf")?;
let vm_exe = VmExe::from_elf(elf, transpiler)?;
```

### Adding Extensions to Transpiler
```rust
use std::rc::Rc;

// Create transpiler with custom extensions
let transpiler = Transpiler::<BabyBear>::new()
    .with_extension(MyCustomExtension::new())
    .with_processor(Rc::new(AnotherExtension::default()));
```

## Creating Custom Extensions

### Simple Instruction Extension
```rust
use openvm_toolchain_transpiler::{TranspilerExtension, TranspilerOutput};
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

pub struct CustomAluExtension<F> {
    _phantom: std::marker::PhantomData<F>,
}

impl<F: PrimeField32> TranspilerExtension<F> for CustomAluExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let insn = instruction_stream[0];
        
        // Check for custom ADD instruction pattern
        if (insn & 0xfe00707f) == 0x00000033 {
            let rs1 = ((insn >> 15) & 0x1f) as u8;
            let rs2 = ((insn >> 20) & 0x1f) as u8;
            let rd = ((insn >> 7) & 0x1f) as u8;
            
            // Create OpenVM instruction
            let vm_insn = Instruction::new(
                MyCustomOpcode::Add.into(),
                vec![F::from_canonical_u32(rd as u32)],
                vec![F::from_canonical_u32(rs1 as u32), F::from_canonical_u32(rs2 as u32)],
                1, // address_space
            );
            
            Some(TranspilerOutput::one_to_one(vm_insn))
        } else {
            None
        }
    }
}
```

### Multi-Instruction Extension
```rust
impl<F: PrimeField32> TranspilerExtension<F> for ComplexExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        if instruction_stream.len() < 2 {
            return None;
        }
        
        let insn1 = instruction_stream[0];
        let insn2 = instruction_stream[1];
        
        // Check for specific two-instruction pattern
        if is_load_immediate_pattern(insn1, insn2) {
            let combined_immediate = extract_combined_immediate(insn1, insn2);
            let rd = extract_rd(insn2);
            
            let vm_insn = Instruction::new(
                MyOpcode::LoadImmediate.into(),
                vec![F::from_canonical_u32(rd)],
                vec![F::from_canonical_u32(combined_immediate)],
                1,
            );
            
            // Consume both RISC-V instructions
            Some(TranspilerOutput::many_to_one(vm_insn, 2))
        } else {
            None
        }
    }
}
```

### Extension with Gap Handling
```rust
impl<F: PrimeField32> TranspilerExtension<F> for AlignmentExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let insn = instruction_stream[0];
        
        // Handle NOP padding for alignment
        if insn == 0x00000013 { // RISC-V NOP (addi x0, x0, 0)
            // Insert gap instead of actual instruction
            Some(TranspilerOutput::gap(1, 1))
        } else {
            None
        }
    }
}
```

## ELF Processing Examples

### Loading and Transpiling ELF
```rust
use openvm_toolchain_transpiler::elf::Elf;

// Load ELF file
let elf_bytes = std::fs::read("program.elf")?;
let elf = Elf::load(&elf_bytes)?;

// Inspect ELF properties
println!("PC start: 0x{:x}", elf.pc_start);
println!("PC base: 0x{:x}", elf.pc_base);
println!("Instructions: {}", elf.instructions.len());
println!("Memory image size: {}", elf.memory_image.len());

// Transpile with custom transpiler
let transpiler = Transpiler::new()
    .with_extension(MyExtension::new());

let vm_exe = VmExe::from_elf(elf, transpiler)?;
```

### Handling Function Boundaries
```rust
// Access function boundaries from transpiled executable
for (name, bounds) in &vm_exe.fn_bounds {
    println!("Function {}: 0x{:x} - 0x{:x}", 
             name, bounds.start, bounds.end);
}
```

## Error Handling Examples

### Graceful Error Handling
```rust
use openvm_toolchain_transpiler::TranspilerError;

match VmExe::from_elf(elf, transpiler) {
    Ok(vm_exe) => {
        println!("Transpilation successful!");
    },
    Err(TranspilerError::AmbiguousNextInstruction) => {
        eprintln!("Error: Multiple extensions claimed the same instruction");
    },
    Err(TranspilerError::ParseError(insn)) => {
        eprintln!("Error: Unknown instruction 0x{:08x}", insn);
    },
}
```

### Extension Error Prevention
```rust
impl<F: PrimeField32> TranspilerExtension<F> for SafeExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        if instruction_stream.is_empty() {
            return None;
        }
        
        let insn = instruction_stream[0];
        
        // Validate instruction format before claiming
        if !self.is_valid_format(insn) {
            return None;
        }
        
        // Only claim if we can definitely handle it
        if self.can_handle(insn) {
            Some(self.process_instruction(insn))
        } else {
            None
        }
    }
}
```

## Testing Examples

### Extension Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_custom_extension() {
        let extension = CustomExtension::new();
        
        // Test recognized instruction
        let insn = 0x00000033; // ADD x0, x0, x0
        let result = extension.process_custom(&[insn]);
        assert!(result.is_some());
        
        // Test unrecognized instruction
        let unknown_insn = 0xffffffff;
        let result = extension.process_custom(&[unknown_insn]);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_multi_instruction_pattern() {
        let extension = ComplexExtension::new();
        
        let pattern = [0x12345678, 0x87654321];
        let result = extension.process_custom(&pattern);
        
        if let Some(output) = result {
            assert_eq!(output.used_u32s, 2);
            assert_eq!(output.instructions.len(), 1);
        }
    }
}
```

## Advanced Patterns

### Extension with State
```rust
pub struct StatefulExtension<F> {
    state: RefCell<ExtensionState>,
    _phantom: PhantomData<F>,
}

impl<F: PrimeField32> TranspilerExtension<F> for StatefulExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let mut state = self.state.borrow_mut();
        
        // Use state to make transpilation decisions
        if state.should_handle(instruction_stream[0]) {
            let vm_insn = state.process_with_context(instruction_stream[0]);
            Some(TranspilerOutput::one_to_one(vm_insn))
        } else {
            None
        }
    }
}
```

### Chain of Extensions
```rust
// Create a transpiler with a specific order of extensions
let transpiler = Transpiler::new()
    .with_extension(HighPriorityExtension::new())  // Checked first
    .with_extension(StandardExtension::new())      // Checked second
    .with_extension(FallbackExtension::new());     // Checked last
```