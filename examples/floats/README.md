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
# Run a specific operation test
cargo test -p floats-tests --release test_fadd

# Run all FMA tests (pattern matching)
cargo test -p floats-tests --release test_f

# Run only comparison tests
cargo test -p floats-tests --release test_feq test_flt test_fle

# Or just run the integration test suite (fastest)
cargo test -p floats-tests --release --test integration
```

### Proof Generation (Experimental)

A proof test has been added for `test_fadd` that generates and verifies a STARK proof:

```bash
cargo test -p floats-tests --release test_fadd_proof -- --nocapture --show-output
```

**Note**: Proof generation is currently experiencing a configuration mismatch issue (AIR count: 25 vs 15). This is being investigated. The test infrastructure is in place and execution works correctly; only proof generation is affected.

## Test Categories

Each RISC-V floating-point operation has its own test file for precise granularity:

### Basic Operations
| Test File | Instruction | Description |
|-----------|-------------|-------------|
| `test_flw_fsw.S` | FLW, FSW | Float load/store word |
| `test_fadd.S` | FADD.S | Floating-point addition |
| `test_fsub.S` | FSUB.S | Floating-point subtraction |
| `test_fmul.S` | FMUL.S | Floating-point multiplication |
| `test_fdiv.S` | FDIV.S | Floating-point division |

### Fused Multiply-Add Operations
| Test File | Instruction | Description |
|-----------|-------------|-------------|
| `test_fmadd.S` | FMADD.S | Fused multiply-add: (a×b)+c |
| `test_fmsub.S` | FMSUB.S | Fused multiply-subtract: (a×b)-c |
| `test_fnmsub.S` | FNMSUB.S | Negated fused multiply-subtract: -(a×b)+c |
| `test_fnmadd.S` | FNMADD.S | Negated fused multiply-add: -(a×b)-c |

### Comparison Operations
| Test File | Instruction | Description |
|-----------|-------------|-------------|
| `test_feq.S` | FEQ.S | Floating-point equality comparison |
| `test_flt.S` | FLT.S | Floating-point less than |
| `test_fle.S` | FLE.S | Floating-point less than or equal |

### Conversion Operations
| Test File | Instruction | Description |
|-----------|-------------|-------------|
| `test_fcvt_w_s.S` | FCVT.W.S | Convert float to signed integer |
| `test_fcvt_wu_s.S` | FCVT.WU.S | Convert float to unsigned integer |
| `test_fcvt_s_w.S` | FCVT.S.W | Convert signed integer to float |
| `test_fcvt_s_wu.S` | FCVT.S.WU | Convert unsigned integer to float |

### Edge Cases
| Test File | Description |
|-----------|-------------|
| `test_nan.S` | NaN propagation and special values |

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
