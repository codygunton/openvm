# OpenVM Transpiler Component AI Documentation

## Overview
The OpenVM transpiler is responsible for converting RISC-V ELF binaries into OpenVM's internal instruction format. It serves as a critical bridge in the compilation pipeline, enabling standard RISC-V programs to execute within the zkVM environment.

## Architecture

### Core Components

#### `Transpiler<F>`
The main transpiler struct that orchestrates instruction conversion:
- Manages a collection of `TranspilerExtension` processors
- Processes RISC-V instructions in 32-bit chunks
- Ensures unambiguous instruction handling across extensions

#### `TranspilerExtension<F>` Trait
Interface for custom instruction transpilation:
```rust
pub trait TranspilerExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>>;
}
```

Key behaviors:
- Returns `None` if instruction isn't recognized
- Never claims instructions handled by other extensions
- Can process multiple RISC-V instructions into single OpenVM instruction

#### `TranspilerOutput<F>`
Result type for transpilation operations:
- `one_to_one`: Single RISC-V → Single OpenVM instruction
- `many_to_one`: Multiple RISC-V → Single OpenVM instruction  
- `gap`: Insert NOPs or gaps in instruction stream

### Key Traits

#### `FromElf`
Enables conversion from ELF binary to executable format:
```rust
pub trait FromElf {
    type ElfContext;
    fn from_elf(elf: Elf, ctx: Self::ElfContext) -> Result<Self, TranspilerError>;
}
```

## Critical Behaviors

### RISC-V Semantics Preservation
- Maintains exact behavior of original RISC-V instructions
- Handles special cases like x0 register writes (converted to NOPs)
- Preserves sign extension for loads and immediates
- Maintains PC-relative addressing correctness

### Memory Safety
- Validates instruction addresses against `MAX_ALLOWED_PC`
- Ensures ELF segments don't exceed guest memory bounds
- Converts memory images from word to byte addressing

### Error Handling
- `TranspilerError::AmbiguousNextInstruction`: Multiple extensions claim same instruction
- `TranspilerError::ParseError`: Instruction cannot be transpiled by any extension

## Extension Development Guidelines

### Implementation Pattern
```rust
impl<F: PrimeField32> TranspilerExtension<F> for MyExtension<F> {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        let insn = instruction_stream[0];
        
        // Check instruction pattern
        if (insn & MASK) == PATTERN {
            // Decode and process
            let vm_insn = decode_and_convert(insn);
            Some(TranspilerOutput::one_to_one(vm_insn))
        } else {
            None
        }
    }
}
```

### Best Practices
1. **Pattern Matching**: Use precise bit masks to identify instructions
2. **Early Return**: Return `None` immediately if instruction doesn't match
3. **Utility Functions**: Leverage existing decode helpers for standard RISC-V formats
4. **Documentation**: Document any multi-instruction patterns your extension handles

### Common Pitfalls
- Don't claim instructions that could be handled by other extensions
- Always validate instruction format before processing
- Handle edge cases like register x0 properly
- Ensure proper sign extension for immediate values

## Performance Considerations
- Keep processor checks efficient with early returns
- Avoid unnecessary allocations in hot transpilation paths
- Use `Rc` for sharing processor instances across transpiler calls
- Minimize pattern matching complexity

## Security Model
- Never allow ambiguous instruction transpilation
- Validate all ELF data before processing
- Enforce memory and PC limits strictly
- Report clear errors for invalid or unrecognized instructions

## Integration Points
- Works with `openvm_instructions` for VM instruction format
- Integrates with `openvm_platform` for system-level operations  
- Uses `openvm_stark_backend` field traits for cryptographic operations
- Processes ELF binaries through the `elf` module

## Testing Strategy
- Test with both valid and malformed ELF files
- Verify function boundary extraction with symbol tables
- Test all instruction format helpers with edge cases
- Ensure memory image conversion preserves data integrity
- Validate extension precedence and ambiguity detection