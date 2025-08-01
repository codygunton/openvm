# Implementation Guide: OpenVM Transpiler

## Overview
This guide provides detailed implementation guidance for working with and extending the OpenVM transpiler component.

## Core Implementation Areas

### 1. Implementing a Custom TranspilerExtension

#### Basic Structure
```rust
use openvm_transpiler::{TranspilerExtension, TranspilerOutput};
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

pub struct MyCustomExtension {
    // Extension state if needed
}

impl<F: PrimeField32> TranspilerExtension<F> for MyCustomExtension {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let insn = instruction_stream[0];
        
        // 1. Pattern matching
        if !matches_my_pattern(insn) {
            return None;
        }
        
        // 2. Decode instruction
        let decoded = decode_my_instruction(insn);
        
        // 3. Generate OpenVM instruction(s)
        let vm_insn = create_vm_instruction(decoded);
        
        // 4. Return output
        Some(TranspilerOutput::one_to_one(vm_insn))
    }
}
```

#### Multi-Instruction Patterns
```rust
fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
    // Check if we have enough instructions
    if instruction_stream.len() < 2 {
        return None;
    }
    
    // Match a two-instruction pattern
    if is_first_part(instruction_stream[0]) && is_second_part(instruction_stream[1]) {
        let combined_insn = combine_instructions(
            instruction_stream[0],
            instruction_stream[1]
        );
        
        // Consume 2 RISC-V instructions, produce 1 OpenVM instruction
        Some(TranspilerOutput::many_to_one(combined_insn, 2))
    } else {
        None
    }
}
```

### 2. ELF Processing Implementation

#### Custom ELF Validation
```rust
use openvm_transpiler::elf::Elf;

fn validate_custom_elf(elf_bytes: &[u8]) -> Result<(), String> {
    let elf = Elf::decode(elf_bytes, GUEST_MAX_MEM)?;
    
    // Custom validation checks
    if elf.pc_start & 0x3 != 0 {
        return Err("PC start must be 4-byte aligned".into());
    }
    
    // Check for required sections
    for addr in required_addresses {
        if !elf.memory_image.contains_key(&addr) {
            return Err(format!("Missing required data at 0x{:08x}", addr));
        }
    }
    
    Ok(())
}
```

#### Function Boundary Processing
```rust
#[cfg(feature = "function-span")]
fn process_function_info(elf: &Elf) -> HashMap<String, (u32, u32)> {
    elf.fn_bounds
        .iter()
        .map(|(_, bound)| {
            (bound.name.clone(), (bound.start, bound.end))
        })
        .collect()
}
```

### 3. Instruction Format Handling

#### Custom R-Type Variant
```rust
use openvm_transpiler::util::from_r_type;
use rrs_lib::instruction_formats::RType;

fn transpile_custom_r_type<F: PrimeField32>(
    dec_insn: &RType,
    custom_opcode: usize,
) -> Instruction<F> {
    // Handle special cases
    if dec_insn.rd == 0 {
        return nop(); // Don't write to x0
    }
    
    // Check for custom encoding in funct fields
    let address_space = match dec_insn.funct7 {
        0x00 => RV32_REGISTER_AS,
        0x01 => RV32_MEMORY_AS,
        _ => return unimp(), // Unknown encoding
    };
    
    from_r_type(custom_opcode, address_space, dec_insn, true)
}
```

#### Immediate Handling
```rust
fn process_custom_immediate<F: PrimeField32>(imm: i32) -> F {
    // Custom immediate processing
    let processed = match imm {
        i if i < 0 => {
            // Sign extend negative values
            F::from_canonical_u32(((i as u32) & 0xffffff) | 0xff000000)
        },
        i => {
            // Zero extend positive values
            F::from_canonical_u32(i as u32 & 0xffffff)
        }
    };
    processed
}
```

### 4. Memory Image Conversion

#### Custom Memory Layout
```rust
use std::collections::BTreeMap;
use openvm_instructions::exe::MemoryImage;

fn create_custom_memory_image<F: PrimeField32>(
    elf_memory: BTreeMap<u32, u32>,
    custom_data: &[u8],
) -> MemoryImage<F> {
    let mut image = MemoryImage::new();
    
    // Standard ELF memory
    for (addr, word) in elf_memory {
        for (i, byte) in word.to_le_bytes().into_iter().enumerate() {
            image.insert(
                (RV32_MEMORY_AS, addr + i as u32),
                F::from_canonical_u8(byte)
            );
        }
    }
    
    // Add custom data at specific location
    let custom_base = 0x10000000;
    for (i, &byte) in custom_data.iter().enumerate() {
        image.insert(
            (RV32_MEMORY_AS, custom_base + i as u32),
            F::from_canonical_u8(byte)
        );
    }
    
    image
}
```

### 5. Error Handling Best Practices

#### Detailed Error Reporting
```rust
use openvm_transpiler::transpiler::TranspilerError;
use thiserror::Error;

#[derive(Error, Debug)]
enum CustomTranspilerError {
    #[error("Invalid instruction at PC 0x{pc:08x}: {insn:08x}")]
    InvalidInstruction { pc: u32, insn: u32 },
    
    #[error("Unsupported instruction variant: {variant}")]
    UnsupportedVariant { variant: String },
    
    #[error("Memory overflow at address 0x{addr:08x}")]
    MemoryOverflow { addr: u32 },
}

fn transpile_with_context(
    instructions: &[u32],
    pc_base: u32,
) -> Result<Vec<Option<Instruction<F>>>, CustomTranspilerError> {
    let mut result = Vec::new();
    let mut pc = pc_base;
    
    for &insn in instructions {
        match transpile_single(insn) {
            Ok(vm_insn) => result.push(Some(vm_insn)),
            Err(e) => {
                return Err(CustomTranspilerError::InvalidInstruction { pc, insn });
            }
        }
        pc += 4;
    }
    
    Ok(result)
}
```

## Testing Strategies

### 1. Unit Testing Extensions
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_custom_extension() {
        let ext = MyCustomExtension::new();
        
        // Test recognized instruction
        let insn = 0x12345678; // Your custom encoding
        let result = ext.process_custom(&[insn]);
        assert!(result.is_some());
        
        // Test unrecognized instruction
        let insn = 0x00000013; // Standard ADDI
        let result = ext.process_custom(&[insn]);
        assert!(result.is_none());
    }
}
```

### 2. Integration Testing
```rust
#[test]
fn test_full_transpilation() {
    let transpiler = Transpiler::<BabyBear>::new()
        .with_extension(MyCustomExtension::new());
    
    let instructions = vec![
        0x00000013, // nop
        0x12345678, // custom instruction
    ];
    
    let result = transpiler.transpile(&instructions).unwrap();
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], Some(insn) if insn.opcode == SystemOpcode::PHANTOM.global_opcode()));
}
```

## Performance Optimization

### 1. Efficient Pattern Matching
```rust
impl<F: PrimeField32> TranspilerExtension<F> for OptimizedExtension {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let insn = instruction_stream[0];
        
        // Use bit masks for fast rejection
        const OPCODE_MASK: u32 = 0x7f;
        const MY_OPCODE: u32 = 0x6b;
        
        if (insn & OPCODE_MASK) != MY_OPCODE {
            return None; // Fast path rejection
        }
        
        // Detailed decoding only after initial match
        // ...
    }
}
```

### 2. Batch Processing
```rust
fn transpile_batch<F: PrimeField32>(
    transpiler: &Transpiler<F>,
    chunks: Vec<Vec<u32>>,
) -> Vec<Result<Vec<Option<Instruction<F>>>, TranspilerError>> {
    chunks.into_iter()
        .map(|chunk| transpiler.transpile(&chunk))
        .collect()
}
```

## Common Pitfalls and Solutions

### 1. Register x0 Handling
**Problem**: Writing to x0 should be a no-op
**Solution**: Always check `rd == 0` and return `nop()`

### 2. Sign Extension
**Problem**: Incorrect sign extension for negative immediates
**Solution**: Use proper masks and sign extension logic

### 3. Memory Alignment
**Problem**: Unaligned memory access
**Solution**: Validate addresses are word-aligned

### 4. Instruction Ambiguity
**Problem**: Multiple extensions claim the same instruction
**Solution**: Use specific patterns and coordinate between extensions

## Advanced Topics

### 1. Stateful Extensions
```rust
struct StatefulExtension<F> {
    state: RefCell<ExtensionState>,
    _phantom: PhantomData<F>,
}

impl<F: PrimeField32> TranspilerExtension<F> for StatefulExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let mut state = self.state.borrow_mut();
        // Use and update state during transpilation
        // ...
    }
}
```

### 2. Debug Information Preservation
```rust
fn preserve_debug_info<F: PrimeField32>(
    original_pc: u32,
    vm_instruction: Instruction<F>,
) -> Instruction<F> {
    // Attach debug information
    vm_instruction.with_debug_info(DebugInfo {
        trace: Some(format!("PC: 0x{:08x}", original_pc)),
        dsl: None,
    })
}
```