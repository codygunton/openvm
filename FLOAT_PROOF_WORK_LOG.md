# Float Proof Generation - Work Log

## Current Status

**Branch**: `floats`
**Test**: `cargo test -p floats-tests --release test_fadd_proof`
**Status**: ❌ Proof verification fails with `OodEvaluationMismatch`

### Last Known State
- ✅ Execution completes successfully through FLOAT_RETURN and FSW
- ✅ Arena allocation fixed for synthetic instructions
- ❌ Proof verification fails (constraint violation in one of the float AIRs)

---

## Problems Fixed

### 1. TracingMemory Infinite Loop (Nov 6-7)
**Error**: Preflight execution hung indefinitely in `calculate_splits_and_merges()`

**Root Cause**: When `block_size = 4` and reading with `size = 4, align = 1`, the code used `split_addr = addr.next_multiple_of(4)` which stayed at the same address if already aligned, causing infinite loop.

**Fix**: Changed to `split_addr = addr + align` to ensure progress
- File: `crates/vm/src/system/memory/online/trace.rs:158`

### 2. Excessive Logging (Nov 7)
**Problem**: Test logs grew to 3.9GB → 2GB → still too large

**Fixes Applied**:
1. Removed verbose instruction logging from `interpreter_preflight.rs:183`
2. Configured tracing to only show `interpreter_preflight` module at TRACE level
3. Kept opcode and timestamp logging for debugging

**Result**: Logs now manageable while keeping useful execution trace

### 3. Field Value Constraints (Nov 7)
**Error**: `assertion failed: n < FP::PRIME` with value `0xFFFF_F000` (4,294,963,200) exceeding Monty-31 prime (2^31 - 1)

**Fixes**:
- Changed `from_canonical_u32(0xFFFF_F000)` to `from_wrapped_u32(0xFFFF_F000u32)` in `float_loadstore/core.rs:86,95`
- Removed unused `FLOAT_TRAMPOLINE_PC` constant that also exceeded prime

### 4. Memory Alignment Assertions (Nov 7)
**Error**: `assertion left == right failed, left: 1, right: 4` at `online.rs:501`

**Root Cause**: Memory operations used alignment 1 for 4-byte word accesses, but both `RV32_REGISTER_AS` and `FLOAT_MEM_AS` have `min_block_size = 4`

**Fix**: Global replacement across all float executors:
- Changed all `read::<u8, 4, 1>` → `read::<u8, 4, 4>`
- Changed all `write::<u8, 4, 1>` → `write::<u8, 4, 4>`

**Files**:
- `extensions/floats/circuit/src/float_load/core.rs`
- `extensions/floats/circuit/src/float_store/core.rs`
- `extensions/floats/circuit/src/handler_executor/execution.rs`
- `extensions/floats/circuit/src/float_return/core.rs`
- `extensions/floats/circuit/src/float_csr/core.rs`

### 5. Arena Allocation for Synthetic Instructions (Nov 7) ✅
**Error**: `range end index 34 out of range for slice of length 0` when FLOAT_RETURN tries to allocate from arena

**Root Cause**:
- FLOAT_RETURN is a synthetic instruction not in the ELF program
- During E2 (metered execution), float operations are emulated directly without handler calls
- During E3 (preflight with tracing), float operations DO call handlers, executing FLOAT_RETURN
- Arena capacity is based on E2 execution frequencies
- FLOAT_RETURN has frequency 0 in E2 → arena capacity 0 → panic in E3

**Solution**: Minimum arena capacity for AIRs with zero trace height
- File: `crates/vm/src/arch/vm.rs:473-479`
- Change: Set minimum capacity of 100 rows for any AIR with `trace_height = 0`
- Impact: ~10KB memory overhead for unused AIRs (negligible)
- Commit: `4d67b0b1b`

---

## Current Issue: OodEvaluationMismatch

### Error
```
Failed to generate or verify proof for test_fadd: Vm(Verification(StarkError(OodEvaluationMismatch)))
```

### What This Means
- Execution completed successfully (FLOAT_RETURN executed at timestamp 832)
- Proof was generated
- Proof verification failed - AIR constraints don't hold at random challenge points
- This indicates a constraint violation in one of the float AIR circuits

### Most Likely Culprits
1. **FloatHandlerReturnAir** (most likely) - New code, first time executing
2. **FloatHandlerSetupAir** - Complex multi-operation AIR
3. **FloatLoadStoreAir** - FSW executed after FLOAT_RETURN

### Execution Flow Observed
```
opcode: STOREW | timestamp: 829
opcode: FLOAT_RETURN | timestamp: 832  ← Handler return successful
opcode: FSW | timestamp: 895            ← Float store executed
opcode: LOADW | timestamp: 898
```

The gap from 832 → 895 represents the 63 memory operations in FLOAT_RETURN (31 register reads + 31 register writes + 1 return address read).

---

## Architecture Notes

### Float Execution Model

**E1 (Basic Execution)**:
- Uses `Executor<F>` trait with `GuestMemory`
- No tracing, pure execution

**E2 (Metered Execution)**:
- Uses `MeteredExecutor<F>` trait with `GuestMemory`
- Tracks execution frequencies and trace heights
- Float operations are **emulated** - handler library doesn't actually execute
- FLOAT_RETURN never executes → frequency = 0

**E3 (Preflight Execution with Tracing)**:
- Uses `PreflightExecutor<F, RA>` trait with `TracingMemory`
- Generates trace data for proof
- Float operations **actually call handler** - native code executes
- FLOAT_RETURN executes when handler returns → needs arena capacity

### Memory Address Spaces
- `RV32_REGISTER_AS` (0): Integer registers x0-x31, min_block_size=4
- `FLOAT_MEM_AS` (2): Float registers + handler state, min_block_size=4

### Float Handler Memory Layout (FLOAT_MEM_AS)
```
FLOAT_REGISTER_BASE (0x1000000):
  FREG_FIRST_OFFSET (0x100): f0-f31 (32 registers × 8 bytes = 256 bytes)
  FCSR_OFFSET (0x200): FCSR register

FLOAT_LIB_ENTRY_PTR (0x1001000): Handler entry point
FLOAT_INST_ADDR (0x1001100): Current instruction encoding
FLOAT_RETURN_ADDR (0x1001108): Saved return address
FLOAT_X0_BACKUP (0x1002000): x1-x31 backup (31 × 8 bytes)
```

---

## Key Files Modified

### Core Infrastructure
- `crates/vm/src/system/memory/online/trace.rs` - Fixed infinite loop in split/merge calculation
- `crates/vm/src/arch/vm.rs` - Added minimum arena capacity for synthetic instructions
- `crates/vm/src/arch/interpreter_preflight.rs` - Reduced verbose logging

### Float Extension
- `extensions/floats/circuit/src/constants.rs` - Removed invalid FLOAT_TRAMPOLINE_PC
- `extensions/floats/circuit/src/float_loadstore/core.rs` - Fixed field wrapping for sign extension
- `extensions/floats/circuit/src/float_load/core.rs` - Fixed memory alignment
- `extensions/floats/circuit/src/float_store/core.rs` - Fixed memory alignment
- `extensions/floats/circuit/src/float_csr/core.rs` - Fixed memory alignment
- `extensions/floats/circuit/src/float_return/core.rs` - Fixed memory alignment
- `extensions/floats/circuit/src/handler_executor/execution.rs` - Fixed memory alignment

### Test Configuration
- `examples/floats/tests/integration.rs` - Configured selective tracing

---

## Next Steps

1. **Debug OodEvaluationMismatch**:
   - Identify which AIR is failing
   - Check if FloatHandlerReturn adapter AIR is properly implemented
   - Verify memory bridge interactions are correct
   - Check timestamp deltas match actual execution

2. **Missing AIR Implementations**:
   - `extensions/floats/circuit/src/float_return/adapter.rs` may be incomplete
   - `extensions/floats/circuit/src/float_handler_setup/` adapter may have issues

3. **Verification Strategy**:
   - Run with `RUST_BACKTRACE=full` to get more detail
   - Add debug logging to identify failing AIR
   - Check if trace generation matches AIR expectations
   - Verify all memory operations are properly constrained

---

## Useful Commands

```bash
# Run proof test (takes ~30 seconds)
cargo test -p floats-tests --release test_fadd_proof -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test -p floats-tests --release test_fadd_proof -- --nocapture

# Run simple execution test (no proof)
cargo test -p floats-tests --release test_fadd -- --nocapture

# Check git status
git status

# View recent commits
git log --oneline -10
```

---

## Reference: Execution Flow

1. **Float operation (e.g., FADD.S)** executes:
   - FloatHandlerExecutor saves x1-x31 to FLOAT_X0_BACKUP
   - Writes instruction encoding to FLOAT_INST_ADDR
   - Writes return address to FLOAT_RETURN_ADDR
   - Jumps to handler library

2. **Handler library** (native RISC-V code):
   - Reads instruction from FLOAT_INST_ADDR
   - Executes float operation using native ISA
   - Writes result to float registers
   - May write to x0-x31 for int-writing ops
   - Executes FLOAT_RETURN instruction

3. **FLOAT_RETURN** executes:
   - Reads x1-x31 from FLOAT_X0_BACKUP
   - Restores all integer registers
   - Reads return address from FLOAT_RETURN_ADDR
   - Jumps back to caller

4. **Caller** continues:
   - Sees result in float registers (or integer registers for int-writing ops)
   - Continues normal execution
