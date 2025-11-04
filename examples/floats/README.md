# OpenVM Floating-Point Test Suite

Comprehensive test suite for the OpenVM RV32F floating-point extension using assembly tests with the SoftFloat library.

## Structure

- **`tests/asm/`** - Assembly test files
  - `common.S` - Common definitions and macros shared by all tests
  - `test_*.S` - Individual test files for different float operations
- **`build.rs`** - Compiles assembly tests and links with SoftFloat library
- **`tests/integration.rs`** - Rust test harness that executes compiled tests
- **`link.ld`** - Linker script for RV32IMF

## Running Tests

**Recommended:** Run from repository root with package filter (minimal rebuilding):

```bash
cargo test -p floats-tests --release
```

Or from the `examples/floats` directory:

```bash
cargo test --release
```

This will:
1. Compile all `test_*.S` files in `tests/asm/`
2. Link them with the float library and SoftFloat
3. Run each test using OpenVM
4. Report pass/fail status

### Running Individual Tests

```bash
# From repository root
cargo test -p floats-tests --release test_basic_ops

# Or just run the integration tests (fastest)
cargo test -p floats-tests --release --test integration

# Run specific test
cargo test -p floats-tests --release test_fma
```

## Test Categories

| Test File | Description |
|-----------|-------------|
| `test_basic_ops.S` | Basic floating-point operations (FLW, FSW, FADD.S, FSUB.S, FMUL.S, FDIV.S) |
| `test_fma.S` | Fused multiply-add operations (FMADD.S, FMSUB.S, FNMSUB.S, FNMADD.S) |
| `test_comparisons.S` | Comparison operations (FEQ.S, FLT.S, FLE.S) |
| `test_nan.S` | NaN propagation and edge cases |

## Adding New Tests

1. Create a new file in `tests/asm/test_<name>.S`
2. Include the common header:
   ```assembly
   .include "tests/asm/common.S"
   ```
3. Write your test code
4. Use `test_pass` macro for success, `test_fail` for failure
5. Add the test to `tests/integration.rs`:
   ```rust
   asm_test!(test_<name>);
   ```
6. Run `cargo test --release`

## Example Test

```assembly
.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # Initialize stack

    # Your test code here
    la   a0, float_1_0
    flw  f0, 0(a0)
    # ... test logic ...

    test_pass              # Exit with success

fail:
    test_fail              # Exit with failure
```

## Architecture

- **Target**: rv32imf (RV32I + M extension + F extension)
- **ABI**: ilp32f (32-bit integer, single-precision float in FPU registers)
- **Float Library Entry**: 0x00100000 (defined in linker script)
- **Float Implementation**: Software library via JALR calls to SoftFloat

## Dependencies

- `riscv64-elf-gcc` - RISC-V cross-compiler
- SoftFloat library (automatically compiled from `extensions/floats/guest/vendor/zisk/lib-float/`)
