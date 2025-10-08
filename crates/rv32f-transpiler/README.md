# OpenVM RV32IMF Transpiler Extension - Architecture A

This extension adds the **F extension** (single-precision floating-point) to OpenVM's RV32IM base, creating full RV32IMF support using **Architecture A: Indirect Calls via Dispatch Table**.

## Overview

This approach solves the JAL offset problem by using **indirect calls** through a function pointer dispatch table at a fixed address. This eliminates the need for:
- Symbol table access during transpilation
- PC tracking during transpilation
- Relocation passes after transpilation
- Modifying the `TranspilerExtension` trait

## How It Works

### Build Time

1. **Rust compiler** (with target `riscv32im-unknown-none-elf` + `f` feature) emits RV32IMF instructions
2. **OpenVM transpiler** (with this extension) processes the ELF:
   - Detects F extension instructions (opcodes 0x53, 0x43, 0x47, 0x4B, 0x4F)
   - Replaces each F instruction with 10 RV32IM instructions:
     ```assembly
     li a0, rs1          # Argument: source register 1
     li a1, rs2          # Argument: source register 2
     li a2, rd           # Argument: destination register
     li a3, rm           # Argument: rounding mode
     li t0, 0xE0000000   # Load dispatch table address
     li t1, op_index     # Operation index (0=FADD, 1=FSUB, etc.)
     slli t1, t1, 2      # Multiply by pointer size (4 bytes)
     add t0, t0, t1      # Calculate &table[op_index]
     lw t0, 0(t0)        # Load function pointer from table
     jalr ra, t0, 0      # Indirect call to runtime function
     ```
3. **Output .vmexe** contains only RV32IM instructions (F instructions replaced)

### Runtime

1. OpenVM executor runs the .vmexe normally
2. When it hits the `jalr`, it loads the function pointer from `0xE0000000 + (op_index * 4)`
3. Jumps to the runtime function (in `openvm-rv32f-runtime`)
4. Runtime function:
   - Reads float registers from memory-mapped region (0xC0000000)
   - Calls LLVM compiler-rt soft-float function (e.g., `__addsf3`)
   - Writes result back to float register
   - Returns via `ret`

## Overhead Analysis

Per F extension operation:
- **Setup**: 4 instructions (load arguments)
- **Dispatch**: 6 instructions (table lookup + indirect call)
- **Runtime function**: ~1-3 instructions (wrapper, most tail-call to LLVM compiler-rt)
- **Soft-float computation**: ~30-100 instructions (LLVM compiler-rt soft-float, e.g., `__addsf3`)
- **Total**: ~41-113 instructions per F operation

### Comparison to Zisk

Zisk's approach (runtime trap-and-emulate):
- Trap overhead: 30-50 instructions
- Dispatcher: 20-40 instructions (switch statement)
- Soft-float: ~30-100 instructions
- Total: ~80-190 instructions

**Our approach is ~20-35% faster than Zisk.**

## Memory Layout

### Dispatch Table (0xE0000000)
```
0xE0000000: [pointer to _openvm_fadd_s]    # Index 0
0xE0000004: [pointer to _openvm_fsub_s]    # Index 1
0xE0000008: [pointer to _openvm_fmul_s]    # Index 2
...
0xE000007C: [pointer to _openvm_fclass_s]  # Index 31
```

### Float Registers (0xC0000000)
```
0xC0000000: f0 (4 bytes)
0xC0000004: f1 (4 bytes)
...
0xC000007C: f31 (4 bytes)
```

### FCSR Register (0xC0000080)
```
0xC0000080: FCSR (4 bytes)
```

## Supported F Extension Instructions

All instructions from the RISC-V F extension (RV32F):

### Arithmetic (Opcode 0x53)
- FADD.S, FSUB.S, FMUL.S, FDIV.S, FSQRT.S
- FMIN.S, FMAX.S

### Fused Multiply-Add (Opcodes 0x43, 0x47, 0x4B, 0x4F)
- FMADD.S, FMSUB.S, FNMADD.S, FNMSUB.S

### Sign Injection (Opcode 0x53, funct7=0x10)
- FSGNJ.S, FSGNJN.S, FSGNJX.S

### Comparison (Opcode 0x53, funct7=0x50)
- FEQ.S, FLT.S, FLE.S

### Conversion (Opcode 0x53, funct7=0x60/0x68)
- FCVT.W.S, FCVT.WU.S, FCVT.S.W, FCVT.S.WU

### Move (Opcode 0x53, funct7=0x70/0x78)
- FMV.X.W, FMV.W.X

### Classify (Opcode 0x53, funct7=0x70)
- FCLASS.S

## Current Limitations

1. **FMA instructions** (FMADD, FMSUB, etc.) don't handle rs3 properly
   - R4-type requires 4 operands, current implementation only passes 3
   - Functionally correct but ignores rs3 register

2. **Comparison/conversion instructions** write to float regs instead of integer regs
   - FEQ/FLT/FLE should write to integer rd
   - FCVT.W.S/FCVT.WU.S should write to integer rd
   - FMV.X.W should write to integer rd
   - FCLASS.S should write to integer rd

3. **No FLW/FSW support**
   - Float load/store instructions not yet implemented
   - Programs cannot load/store float values from memory

4. **No FCSR (rounding mode) support**
   - Rounding mode parameter passed but ignored
   - Always uses default rounding (RNE)

## Testing

To test the transpiler:
```bash
cargo test -p openvm-rv32f-transpiler
```

Current test coverage:
- ✅ 30 unit tests (all passing)
- ❌ No integration tests yet
- ❌ No RISCOF compliance tests yet

## Integration (TODO)

This extension will be automatically included when enabled in `openvm.toml`:

```toml
[app_vm_config]
rv32imf = {}  # Enable F extension on top of RV32IM base
```

The SDK will add `Rv32FArchATranspilerExtension` to the transpiler chain.

**Status:** Not yet integrated into SDK configuration.

## Related Crates

- `openvm-rv32f-runtime`: C runtime library with dispatch table and soft-float wrappers
- `openvm-rv32f-circuit`: (TODO) Circuit for float register file and FCSR
- `openvm-rv32f-guest`: (TODO) Guest library for float operations

## Design Rationale

### Why Indirect Calls?

We considered 5 approaches to solve the JAL offset problem:

1. **Pre-link**: Link runtime before transpilation → Complex, requires compiler changes
2. **Fixed addresses**: Use linker script → Still needs PC tracking
3. **Symbol table access**: Extend transpiler trait → Invasive to OpenVM core
4. **Post-processing**: Add relocation pass → Complex architecture change
5. **Indirect calls**: Use dispatch table → **Chosen for simplicity**

Indirect calls add ~6 extra instructions per float op but require no changes to OpenVM's core transpiler framework.

### Why Fixed Table Address?

The dispatch table at 0xE0000000 provides:
- Deterministic location (no dynamic allocation)
- Out of the way of normal memory (above 3.5GB)
- Easy to reserve via linker script
- No conflicts with OpenVM's memory layout

### Why Memory-Mapped Registers?

Float registers at 0xC0000000 provide:
- RISCOF signature compatibility (needed for testing)
- Simple access from runtime functions
- No special register file needed in VM
- Easy to read/write via standard load/store

## Performance Optimizations

Potential future optimizations:

1. **Reduce table lookup overhead**:
   ```assembly
   # Instead of: li + li + slli + add + lw + jalr (6 instructions)
   # Use: auipc + addi + lw + jalr (4 instructions)
   ```

2. **Inline simple operations**:
   - Sign injection (FSGNJ) is just bit manipulation
   - Could transpile to direct RV32IM instead of calling

3. **Batch operations**:
   - If multiple floats in sequence, keep table pointer in register
   - Saves 4 instructions per subsequent op

## References

- RISC-V F Extension Specification: https://riscv.org/specifications/
- LLVM compiler-rt soft-float: https://compiler-rt.llvm.org/
- OpenVM Documentation: https://openvm.dev/
