# OpenVM Float Example (Rust)

Rust-based example for OpenVM RV32F (floating-point) extension with SoftFloat library.

## Files

- **src/main.rs** - Rust program with f32 operations
- **build.rs** - Cargo build script that compiles C float library + SoftFloat
- **riscv32imf-unknown-none-elf.json** - Custom Rust target for rv32imf
- **.cargo/config.toml** - Cargo configuration with linker flags
- **Cargo.toml** - Rust project configuration
- **link.ld** - Linker script for RV32IMF
- **openvm.toml** - OpenVM configuration enabling rv32f extension

## Build with Single Cargo Command

### From the examples/floats directory:

```bash
cd examples/floats
cargo +nightly build --release -Zbuild-std=core
```

### From the base directory:

```bash
(cd examples/floats && cargo +nightly build --release -Zbuild-std=core)
```

This single command automatically:
1. Runs build.rs which compiles float.c + 70+ SoftFloat files
2. Creates libfloatall.a static library
3. Compiles Rust code to rv32imf using custom target
4. Links everything with proper rv32imf ABI

Output: `examples/floats/target/riscv32imf-unknown-none-elf/release/floats-example`

## Run with OpenVM

Since this is a fork without cargo-openvm, you'll need to use the OpenVM CLI directly or integrate with your existing build system.

## What it tests

1. Initializes float library pointer at 0x10000000
2. Performs f32 addition: 1.0 + 2.0
3. Verifies result equals 3.0
4. Exits with code 0 on success, 1 on failure

## Architecture

- **Target**: rv32imf (RV32I + M extension + F extension)
- **ABI**: ilp32f (integer 32-bit, long 64-bit, float in FPU registers)
- **Float implementation**: Software library via JALR calls to SoftFloat
- **Language**: Rust (no_std) with C float library

## Technical Details

The custom target JSON specifies rv32imf with:
- Features: +m,+f (multiply/divide + single-precision float)
- Data layout: 32-bit pointers, 64-bit i64, natural alignment
- Linker: riscv64-elf-gcc with custom linker script

The Rust code compiles to native rv32imf instructions (FADD.S, etc.), which the OpenVM transpiler expands to call the SoftFloat handler.
