# OpenVM Float Minimal Test

Minimal test for OpenVM RV32F (floating-point) extension with SoftFloat library.

## Files

- **test.S** - Assembly test with FLW, FADD.S, and FSW instructions
- **build.sh** - Build script that compiles test + float library + SoftFloat
- **link.ld** - Linker script for RV32IMF
- **openvm.toml** - OpenVM configuration enabling rv32f extension
- **ucmpdi2.c** - RV32 implementation of `__ucmpdi2` (missing from libgcc)

## Build

```bash
./build.sh
```

This compiles:
1. test.S (the minimal float test)
2. float.c (zisk float handler library)
3. 70+ SoftFloat source files
4. ucmpdi2.c (compiler builtin for RV32)

Output: `build/test.elf`

## Run with OpenVM

```bash
cargo run --release --bin cargo-openvm -- openvm run --exe build/test.elf --config openvm.toml
```

## What it tests

1. Initializes float library pointer at 0x1FFFF000
2. Loads two floats: 1.0 and 2.0
3. Adds them using FADD.S (expands to 9 OpenVM instructions calling SoftFloat)
4. Stores result (3.0) back to memory
5. Exits with code 0

## Architecture

- **Target**: rv32imf (RV32I + M extension + F extension)
- **ABI**: ilp32f (integer 32-bit, long 64-bit, float in FPU registers)
- **Float implementation**: Software library via JALR calls to SoftFloat
