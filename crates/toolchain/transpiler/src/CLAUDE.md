# Claude Instructions for OpenVM Transpiler

## Component Overview
The transpiler is responsible for converting RISC-V ELF binaries into OpenVM's internal instruction format. This is a critical component in the compilation pipeline that enables standard RISC-V programs to run in the zkVM.

## Key Responsibilities

### When Working on the Transpiler
1. **Maintain RISC-V Semantics**: Any transpilation must preserve the exact behavior of the original RISC-V instructions
2. **Handle Edge Cases**: Pay special attention to:
   - x0 register writes (should become NOPs)
   - Sign extension for loads and immediates
   - PC-relative addressing
   - Memory alignment requirements

### Extension Development
When implementing new `TranspilerExtension`:
1. Return `None` if the instruction isn't recognized
2. Never claim an instruction that could be handled by another extension
3. Use the provided utility functions for standard RISC-V formats
4. Document any multi-instruction patterns your extension handles

### Memory Safety
- Always validate memory addresses against `MAX_ALLOWED_PC` for instructions
- Ensure ELF segments don't exceed guest memory bounds
- Convert memory images correctly (word to byte addressing)

## Common Patterns

### Adding a New Instruction Type
```rust
// 1. Check instruction pattern
if (insn & MASK) == PATTERN {
    // 2. Decode instruction fields
    let decoded = decode_instruction(insn);
    
    // 3. Use appropriate utility function
    let vm_insn = from_r_type(opcode, address_space, &decoded, allow_rd_zero);
    
    // 4. Return transpiler output
    Some(TranspilerOutput::one_to_one(vm_insn))
} else {
    None
}
```

### Handling Special Cases
- **NOP Generation**: When rd=x0 for ALU operations
- **Sign Extension**: Use the sign flag in instruction encoding
- **Immediate Handling**: Apply appropriate masks (12-bit, 20-bit, etc.)

## Testing Requirements
1. Test with both valid and malformed ELF files
2. Verify function boundary extraction with symbol tables
3. Test all instruction format helpers with edge cases
4. Ensure memory image conversion preserves data

## Performance Considerations
- Keep processor checks efficient (early return on mismatch)
- Avoid unnecessary allocations in hot paths
- Use `Rc` for sharing processor instances

## Security Notes
- Never allow ambiguous instruction transpilation
- Validate all ELF data before processing
- Enforce memory and PC limits strictly
- Report clear errors for invalid instructions