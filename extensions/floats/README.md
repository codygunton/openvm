# OpenVM Floats Extension

RISC-V F extension (single-precision floating point) support for OpenVM.

## Features

- **Complete F Extension**: All RV32F instructions supported
- **IEEE 754 Compliant**: Correct NaN propagation, infinity handling, rounding modes
- **Berkeley SoftFloat**: Battle-tested library (45K+ lines of verified code)
- **Transparent Integration**: Just use Rust `f32` types

## Architecture

This extension integrates the Zisk float library, which wraps Berkeley SoftFloat
with RISC-V instruction decoding. Float registers are memory-mapped at 0x1F001000.

### Instruction Flow

1. **FLW/FSW** (load/store): Direct memory operations to float register file
2. **FADD/FMUL/etc** (ALU): Call into library via transpiler-generated sequence
3. **Library execution**: Decodes instruction, performs operation via SoftFloat

### Memory Map

- **0x1F001000 - 0x1F00107F**: Float registers f0-f31 (32 × 4 bytes)
- **0x1F001108**: Raw instruction storage (FREG_INST)
- **0x1F008018**: fcsr register (control/status)
- **0x1FFFF000**: Float library function pointer (high memory)

### Transpiler Architecture

The transpiler intercepts F extension instructions and generates:
- **FLW/FSW**: Direct LOADW/STOREW to float register memory region
- **Float ALU ops**: Sequence of instructions:
  1. Store raw instruction to 0x1F001108
  2. Load library pointer from 0x1FFFF000 (LUI + LW)
  3. Call library via JALR (saves return address to x1)

## Usage

Add to your guest program's `Cargo.toml`:

```toml
[dependencies]
openvm = { workspace = true }
openvm-floats-guest = { workspace = true }
```

Enable in `openvm.toml`:

```toml
[app_vm_config.rv32f]
```

Use floats in your program:

```rust
#![no_main]
#![no_std]

openvm::entry!(main);

pub fn main() {
    let a: f32 = 1.5;
    let b: f32 = 2.5;
    let sum = a + b;  // Compiles to FADD.S instruction
}
```

## Implementation Status

- ✅ FLW, FSW (load/store) - Fully implemented
- ✅ FADD, FSUB, FMUL, FDIV, etc. - Call library via high-memory function pointer
  - Function pointer stored at 0x1FFFF000 (near 512MB limit)
  - Provides ~480MB buffer between typical programs and reserved area
- ✅ Zisk library compilation - Successfully compiling to rv32imf
  - Library compiles to ~937KB static archive (libziskfloat.a)
  - Contains all SoftFloat f32 operations
  - Uses riscv64-elf-gcc with -march=rv32imf flag
- ⚠️  Runtime integration - Still needs work
  - Function pointer must be initialized at 0x1FFFF000 at program startup
  - Library must be modified to return via `jalr x0, x1, 0`
  - Linking mechanism needs to be set up

## Limitations

- Runtime integration not yet complete (library needs to be linked and initialized)
- Only single precision (f32) supported (double precision f64 is future work)
- No fused multiply-add yet (FMADD/FMSUB/FNMADD/FNMSUB implemented but untested)
- Programs larger than ~480MB will collide with reserved float region

## Build Requirements

- ✅ `riscv64-elf-gcc` compiler (installed and working)
- ✅ Compilation to rv32imf working via build.rs
- ⚠️  Custom target: `riscv32imf-openvm-zkvm-elf` (needs to be created/installed)

## Next Steps

1. Set up automated C library compilation in build system
2. Modify Zisk library return mechanism to use x1 (JALR convention)
3. Add runtime initialization to store library address at 0x1FFFF000
4. Test with complete float program
5. Add pre-link safety check for program size

## References

- [Zisk Float Library](https://github.com/0xPolygonHermez/zisk)
- [Berkeley SoftFloat](http://www.jhauser.us/arithmetic/SoftFloat.html)
- [RISC-V F Extension Spec](https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf)
