# OpenVM Transpiler Integration Patterns

## Integration Overview
The transpiler integrates with multiple OpenVM components to provide seamless RISC-V to OpenVM instruction conversion. This document outlines key integration patterns and best practices.

## Core Integration Points

### 1. OpenVM Instructions Integration
```rust
use openvm_instructions::{
    instruction::Instruction,
    exe::VmExe,
    program::{Program, DEFAULT_PC_STEP},
};

// Transpiler produces Instructions compatible with the VM
let vm_instruction = Instruction::new(
    opcode,
    outputs,
    inputs,
    address_space,
);
```

### 2. OpenVM Platform Integration  
```rust
use openvm_platform::memory::MemoryImage;

// Memory layout integration
let init_memory = elf_memory_image_to_openvm_memory_image(elf.memory_image);
```

### 3. Stark Backend Integration
```rust
use openvm_stark_backend::p3_field::PrimeField32;

// Field-generic transpiler design
impl<F: PrimeField32> TranspilerExtension<F> for MyExtension<F> {
    // Extension works with any compatible field
}
```

## Extension Integration Patterns

### Pattern 1: ISA Extension Integration
```rust
// Integrate with existing RISC-V ISA extensions
pub struct RV32IExtension<F> {
    _phantom: PhantomData<F>,
}

impl<F: PrimeField32> TranspilerExtension<F> for RV32IExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        use crate::util::{from_r_type, from_i_type, from_s_type};
        
        let insn = instruction_stream[0];
        
        match insn & 0x7f {
            0x33 => Some(TranspilerOutput::one_to_one(
                from_r_type(RV32I_ALU, 1, &decode_r_type(insn), true)
            )),
            0x13 => Some(TranspilerOutput::one_to_one(
                from_i_type(RV32I_ALU_IMM, 1, &decode_i_type(insn), true)
            )),
            _ => None,
        }
    }
}
```

### Pattern 2: Custom Circuit Integration
```rust
// Integrate with custom OpenVM circuits
pub struct CustomCircuitExtension<F> {
    opcode_mapping: HashMap<u32, u32>,
    _phantom: PhantomData<F>,
}

impl<F: PrimeField32> TranspilerExtension<F> for CustomCircuitExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let insn = instruction_stream[0];
        
        if let Some(&vm_opcode) = self.opcode_mapping.get(&(insn & 0xfe00707f)) {
            let decoded = decode_instruction(insn);
            let vm_insn = Instruction::new(
                vm_opcode,
                vec![F::from_canonical_u32(decoded.rd as u32)],
                vec![
                    F::from_canonical_u32(decoded.rs1 as u32),
                    F::from_canonical_u32(decoded.rs2 as u32),
                ],
                1, // Custom address space
            );
            Some(TranspilerOutput::one_to_one(vm_insn))
        } else {
            None
        }
    }
}
```

## Memory Integration Patterns

### Memory Layout Conversion
```rust
use crate::util::elf_memory_image_to_openvm_memory_image;

// Convert ELF memory layout to OpenVM format
fn integrate_memory_layout(elf: &Elf) -> MemoryImage {
    // ELF uses word-aligned addresses, OpenVM uses byte-aligned
    let openvm_memory = elf_memory_image_to_openvm_memory_image(elf.memory_image.clone());
    
    // Additional memory validation
    for (&addr, &value) in &openvm_memory {
        assert!(addr < MAX_GUEST_MEMORY, "Memory address out of bounds");
    }
    
    openvm_memory
}
```

### Address Space Integration
```rust
// Different components use different address spaces
const SYSTEM_ADDRESS_SPACE: u32 = 0;
const MEMORY_ADDRESS_SPACE: u32 = 1;  
const CUSTOM_ADDRESS_SPACE: u32 = 2;

impl<F: PrimeField32> TranspilerExtension<F> for AddressSpaceExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let insn = instruction_stream[0];
        
        let address_space = match get_instruction_type(insn) {
            InstructionType::Memory => MEMORY_ADDRESS_SPACE,
            InstructionType::System => SYSTEM_ADDRESS_SPACE,
            InstructionType::Custom => CUSTOM_ADDRESS_SPACE,
        };
        
        Some(TranspilerOutput::one_to_one(
            create_instruction_with_address_space(insn, address_space)
        ))
    }
}
```

## Toolchain Integration

### Build System Integration
```rust
// Integration with cargo-openvm
use openvm_toolchain_transpiler::{Transpiler, FromElf};

pub fn transpile_guest_program<F: PrimeField32>(
    elf_path: &Path, 
    transpiler: Transpiler<F>
) -> Result<VmExe<F>, Box<dyn std::error::Error>> {
    let elf_bytes = std::fs::read(elf_path)?;
    let elf = Elf::load(&elf_bytes)?;
    
    VmExe::from_elf(elf, transpiler).map_err(Into::into)
}
```

### CLI Integration Pattern
```rust 
// Command-line tool integration
pub struct TranspilerConfig {
    pub extensions: Vec<String>,
    pub debug_mode: bool,
    pub optimization_level: u8,
}

impl TranspilerConfig {
    pub fn build_transpiler<F: PrimeField32>(&self) -> Transpiler<F> {
        let mut transpiler = Transpiler::new();
        
        for ext_name in &self.extensions {
            transpiler = match ext_name.as_str() {
                "rv32i" => transpiler.with_extension(RV32IExtension::new()),
                "custom" => transpiler.with_extension(CustomExtension::new()),
                _ => panic!("Unknown extension: {}", ext_name),
            };
        }
        
        transpiler
    }
}
```

## Testing Integration

### Integration Test Framework
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use openvm_stark_backend::p3_baby_bear::BabyBear;
    
    fn create_test_transpiler() -> Transpiler<BabyBear> {
        Transpiler::new()
            .with_extension(TestExtension::new())
            .with_extension(RV32IExtension::new())
    }
    
    #[test]
    fn test_full_elf_transpilation() {
        let transpiler = create_test_transpiler();
        let test_elf = load_test_elf("simple_program.elf");
        
        let vm_exe = VmExe::from_elf(test_elf, transpiler).unwrap();
        
        assert!(!vm_exe.program.instructions.is_empty());
        assert!(vm_exe.pc_start > 0);
    }
    
    #[test]
    fn test_extension_precedence() {
        // Ensure extensions are checked in the correct order
        let transpiler = Transpiler::new()
            .with_extension(HighPriorityExtension::new())
            .with_extension(LowPriorityExtension::new());
            
        let instructions = [0x12345678u32]; // Recognized by both extensions
        let result = transpiler.transpile(&instructions).unwrap();
        
        // Should be handled by high priority extension
        assert_eq!(result[0].unwrap().opcode, HIGH_PRIORITY_OPCODE);
    }
}
```

## Performance Integration

### Optimization Patterns
```rust  
// Efficient extension design for hot paths
pub struct OptimizedExtension<F> {
    // Pre-computed lookup tables
    opcode_table: [Option<u32>; 1024], // Direct lookup by instruction bits
    _phantom: PhantomData<F>,
}

impl<F: PrimeField32> TranspilerExtension<F> for OptimizedExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let insn = instruction_stream[0];
        let index = (insn >> 22) as usize; // Extract key bits
        
        // Fast table lookup instead of pattern matching
        if let Some(opcode) = self.opcode_table.get(index).and_then(|&x| x) {
            Some(TranspilerOutput::one_to_one(
                self.fast_create_instruction(insn, opcode)
            ))
        } else {
            None
        }
    }
}
```

### Memory-Efficient Integration
```rust
// Minimize allocations during transpilation
impl<F: PrimeField32> TranspilerExtension<F> for MemoryEfficientExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        // Reuse pre-allocated buffers
        let mut outputs = self.output_buffer.borrow_mut();
        let mut inputs = self.input_buffer.borrow_mut();
        
        outputs.clear();
        inputs.clear();
        
        // Process instruction without additional allocations
        self.process_into_buffers(instruction_stream[0], &mut outputs, &mut inputs);
        
        Some(TranspilerOutput::one_to_one(
            Instruction::new(self.opcode, outputs.clone(), inputs.clone(), 1)
        ))
    }
}
```

## Error Integration

### Error Propagation Pattern
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtensionError {
    #[error("Invalid instruction format: {0:032b}")]
    InvalidFormat(u32),
    #[error("Unsupported addressing mode")]
    UnsupportedAddressing,
    #[error("Register out of range: {0}")]
    InvalidRegister(u8),
}

// Convert extension errors to transpiler errors
impl From<ExtensionError> for TranspilerError {
    fn from(err: ExtensionError) -> Self {
        match err {
            ExtensionError::InvalidFormat(insn) => TranspilerError::ParseError(insn),
            _ => TranspilerError::ParseError(0), // Generic parse error
        }
    }
}
```

## Configuration Integration

### Runtime Configuration
```rust
// Dynamic extension configuration
pub struct ConfigurableTranspiler<F> {
    base: Transpiler<F>,
    config: TranspilerConfig,
}

impl<F: PrimeField32> ConfigurableTranspiler<F> {
    pub fn from_config(config: TranspilerConfig) -> Self {
        let mut transpiler = Transpiler::new();
        
        // Add extensions based on configuration
        if config.enable_rv32i {
            transpiler = transpiler.with_extension(RV32IExtension::new());
        }
        
        if let Some(custom_config) = &config.custom_extension {
            transpiler = transpiler.with_extension(
                CustomExtension::from_config(custom_config)
            );
        }
        
        Self { base: transpiler, config }
    }
}
```

This integration documentation provides comprehensive patterns for integrating the transpiler with various OpenVM components while maintaining modularity and performance.