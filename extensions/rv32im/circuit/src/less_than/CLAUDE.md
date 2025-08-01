# LessThan Component - Implementation Guidelines

## Component Purpose

This component implements signed and unsigned less-than comparison operations (SLT, SLTU) for RISC-V 32-bit instructions in a zero-knowledge proof-friendly manner. It's part of the RV32IM extension providing essential comparison functionality.

## Code Standards

### When Modifying This Component

1. **Maintain Comparison Logic**: The algorithm must find the most significant differing limb
2. **Preserve Sign Handling**: SLT must correctly handle two's complement representation
3. **Use Bitwise Lookups**: All range checks must go through the bitwise operation lookup table
4. **Boolean Constraints**: Ensure comparison result and markers are properly boolean-constrained

### Testing Requirements

When making changes:
- Run all existing tests: `cargo test -p openvm-rv32im-circuit less_than`
- Add tests for edge cases (equal values, sign boundaries)
- Verify both signed and unsigned comparisons
- Test with maximum and minimum values

### Common Pitfalls

1. **Sign Extension**: Ensure MSB is properly interpreted for signed comparisons
2. **Difference Marker**: Exactly one marker should be set (or none if equal)
3. **Field Arithmetic**: Negative MSB values must use proper field representation
4. **Range Validation**: All MSB values must be validated according to signed/unsigned mode

## Architecture Decisions

### Why Limb-Based Comparison?
- Enables comparison in field arithmetic
- Allows separate handling of sign bit
- Compatible with memory system layout

### Why Difference Marker Approach?
- Efficiently identifies comparison point
- Enables single-pass algorithm
- Simplifies constraint system

### Why Separate MSB Handling?
- Clean sign/unsigned distinction
- Efficient range checking
- Avoids complex sign extension logic

## Integration Notes

### With Memory System
- Reads two operands through adapter
- Writes single bit result (0 or 1) in lowest limb
- Upper limbs of result are always zero

### With Bitwise Lookup
- Range checks MSB values
- Validates difference values are non-zero
- Ensures all values stay within bounds

### With VM Framework
- Implements `VmCoreChip` trait
- Provides proper trace generation
- Handles instruction decoding

## Performance Optimization

1. **Early Termination**: Check limbs from MSB to LSB
2. **Minimal Constraints**: Only essential validation in AIR
3. **Efficient Range Checks**: Batch lookup requests
4. **Optimized Layout**: Align columns for cache efficiency

## Future Considerations

When extending:
- Consider adding greater-than variants
- Maintain the same comparison algorithm
- Keep sign handling consistent
- Document any new edge cases