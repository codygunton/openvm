# AIR Playground Implementation Notes

Notes from building all 23 OpenVM AIR playgrounds.

---

## Phase 1: Example Playgrounds (Tasks #0-3)

### Task #0: Limb Arithmetic AIR (`limb-arithmetic.html`)

- **Size**: 26,177 bytes
- **Approach**: Displayed as 4-row table (one per limb) rather than traditional trace rows. This is a departure from the Fibonacci/Range-Check pattern but makes the carry chain much more visually clear.
- **SUB mode**: Implemented as borrow propagation — constraint flips to `result[i] + b[i] + carry_in = a[i] + carry[i] * 256`. All sidebar labels and constraint formulas dynamically update between "carry" and "borrow" terminology.
- **CSS additions**: Added `.concept`/`.concept-title`, `.sel-all`, `.bit-view`, `.carry-active`/`.carry-zero`, `--cyan`/`--cyan-dim` vars. These CSS additions are reused by subsequent playgrounds.
- **No issues encountered** — clean build.

### Task #1: Adapter/Core Pattern (`adapter-core.html`)

- **Size**: 30,843 bytes
- **Design decision**: Split main panel into 3 visually distinct sections with colored left borders (purple=adapter, green=core, cyan=bus). This makes the adapter/core boundary immediately visible.
- **5 operations**: ADD, SUB, XOR, OR, AND all work. Bitwise ops (XOR/OR/AND) use direct limb-level bitwise operations with carry forced to 0. Arithmetic ops (ADD/SUB) use carry chain.
- **Bus interactions**: Shown as a non-editable table with colored badges. Timestamps auto-computed as pc+0, pc+1, pc+2, pc+3 for the 3 memory ops + next execution state.
- **No issues encountered** — clean build.

### Task #2: Memory Bus (Offline Checker) (`memory-bus.html`)

- **Size**: 25,109 bytes
- **Preset approach**: 3 presets (Simple read/write, Multiple addresses, Overwrite) instead of free-form editing. This keeps the playground focused and avoids complex input validation.
- **Auto-generated init/final ops**: `buildAllOps()` automatically adds boundary operations per unique address.
- **Bus matching**: Uses greedy tuple matching — for each send, finds first unmatched receive with same tuple.
- **No issues encountered** — clean build.

### Task #3: Execution Chain (`execution-chain.html`)

- **Size**: 28,527 bytes
- **Chain visualization**: Horizontal CSS box flow with arrows between boxes. VmConnector boxes have purple borders, instruction boxes have blue borders.
- **Timestamp calculation**: Each instruction consumes 3 timestamps, advancing by 3 per instruction.
- **TERMINATE handling**: TERMINATE instruction sends back to VmConnector Row 1, closing the cycle.
- **No issues encountered** — clean build.

---

## Phase 2: RISC-V (RV32IM) Playgrounds

### Task #4: ALU (ADD/SUB/XOR/OR/AND) (`rv32im-alu.html`)

- **Size**: 25,804 bytes
- **One-hot multiplexing**: Five boolean flags, exactly one set per operation. Constraints C1-C4 (carry equations), C5-C8 (carry boolean), C9 (flag sum), C10 (flags boolean).
- **SUB carry normalization**: SUB uses the constraint `a[i]+c[i]+carry[i-1] - b[i] - carry[i]*256 = 0`, requiring careful carry re-derivation.
- **BitwiseOperationLookupBus**: For ADD/SUB sends `(a[i], a[i], 0)` for range checking. For XOR/OR/AND sends `(b[i], c[i], b[i]^c[i])`.
- **No issues encountered** — clean build. Verified with manual trace-through of 0x4A3B2C1D + 0x1234ABCD.

### Task #5: Shift (SLL/SRL/SRA) (`rv32im-shift.html`)

- **Size**: 31,444 bytes
- **Shift decomposition**: Split into `limb_shift` (whole bytes, 0-3) and `bit_shift` (intra-byte, 0-7). Both represented as one-hot marker arrays.
- **SRA sign extension**: `b_sign = (b[3] >> 7) & 1`. For SRA with MSB set, carry above the top limb fills with `bSign * (bitMultiplier - 1)` instead of 0.
- **Constraint structure**: 11 constraints total — one-hot markers (C1-C2), marker/value consistency (C3-C4), operation flag sum (C5), and per-limb carry equations (C6-C9) that differ between SLL and SRL/SRA.
- **Verified SLL and SRL**: Both trace-through tests pass (0x12345678 << 5 = 0x468ACF00, >>> 5 = 0x0091A2B3). SRA with 0x80000000 >> 5 = 0xFC000000 also verified.
- **No issues encountered** — clean build.

### Task #6: Multiply (MUL/MULH/MULHU) (`rv32im-multiply.html`)

- **Size**: 29,813 bytes
- **Cross-product matrix visualization**: 5x5 CSS grid showing `b[j]*c[k]` products, color-coded by anti-diagonal (result limb contribution). This is the standout visual feature.
- **RangeTupleCheckerBus**: Unlike ALU which uses BitwiseOperationLookup, multiply uses tuple range checks with maxCarry = [254, 508, 763, 1019] because cross-product sums can exceed 255.
- **MULH/MULHU**: Full 8-position schoolbook multiplication. MULH uses sign-extended 8-limb multiply; MULHU uses unsigned.
- **No issues encountered** — clean build.

### Task #7: Divide/Remainder (DIV/REM/DIVU/REMU) (`rv32im-divrem.html`)

- **Size**: 38,763 bytes (second-largest playground)
- **41 core columns**: b[4], c[4], q[4], r[4], r_prime[4], r_inv[4], lt_marker[4], carries, signs, flags — the most complex chip in RV32IM.
- **Division by verification**: Instead of computing q and r, the prover provides them as witness values and proves `b = c*q + r` via schoolbook multiplication carry chain plus `r < c` via lt_marker scan.
- **Edge cases**: Zero divisor (q=0xFFFFFFFF, r=b), signed overflow (INT_MIN/-1), and r_zero flag all handled.
- **Verified against Rust test vectors**: Exact match with `run_divrem_unsigned_sanity_test` and `run_divrem_signed_sanity_test` in the Rust circuit tests.
- **No issues encountered** — clean build.

### Task #8: Branch (BEQ/BNE/BLT/BGE/BLTU/BGEU) (`rv32im-branch.html`)

- **Size**: 37,956 bytes
- **Two AIR families in one playground**: BranchEqualCoreAir (BEQ/BNE) and BranchLessThanCoreAir (BLT/BGE/BLTU/BGEU). The main panel dynamically switches between equality and less-than modes.
- **Inverse witness technique**: For BEQ/BNE, `diff_inv_marker[i] = 1/(a[i]-b[i])` at the first differing limb. The sum `cmp_eq + Σ(a[i]-b[i])*marker[i]` must equal 1.
- **MSB signed comparison**: For BLT/BGE, `a_msb_f = a[3] - sign_bit * 256` transforms the MSB into a signed field element. `diff_marker` scans MSB-to-LSB and `diff_val = (b[i]-a[i]) * (2*cmp_lt - 1)`.
- **Extended GCD**: Needed `modInv(a, P)` via extended Euclidean algorithm for the inverse witness computation.
- **No issues encountered** — clean build.

### Task #9: Load/Store (LW/LH/LB/SW/SH/SB) (`rv32im-memory.html`)

- **Size**: 36,247 bytes
- **8 operations**: LW, LH, LHU, LB, LBU, SW, SH, SB with shift amount 0-3. Auto-disables invalid shift values per operation (e.g., LW locks to 0, LH caps at 2).
- **Three visualization modes**: Byte selection (loads), sign extension comparison (LB vs LBU, LH vs LHU), and store merge preview (SB/SH/SW).
- **Constraint evaluation**: write_data[i] - expected[i] mod 256 per byte, with editable write_data to intentionally break constraints.
- **No issues encountered** — clean build.

### Task #10: Jump (JAL/JALR/LUI/AUIPC) (`rv32im-jump.html`)

- **Size**: 44,764 bytes (largest playground)
- **4 distinct instructions**: Each has different column layouts, constraint sets, and visualizations. The sidebar dynamically updates AIR structure and constraints when switching instructions.
- **JALR sign extension**: 12-bit immediate sign-extended to 32 bits, added to rs1, with LSB cleared.
- **AUIPC carry chain**: Shows byte-by-byte carry propagation for PC + (imm << 12).
- **LUI always-zero byte**: byte 0 of rd is always 0 for LUI (since imm << 12 zeros the bottom 12 bits).
- **No issues encountered** — clean build.

### Task #11: Keccak-256 (`keccak256.html`)

- **Size**: 31,902 bytes
- **Structural playground**: Unlike the other RV32IM playgrounds, Keccak doesn't implement the full permutation. Instead it visualizes the AIR structure: padding, absorption XOR, sponge state layout, and the 24-round timeline.
- **Padding visualization**: 136-byte grid with color coding: blue (message), yellow (pad 0x01 and 0x80), dim (zero fill). Uses 10*1 Keccak padding scheme.
- **XOR absorption**: Interactive table with editable state bytes — users can change pre-state values and see XOR results update with BitwiseOperationLookup verification tuples.
- **24-round timeline**: Round 0 (green, "instruction fires here") through round 23 (yellow, "digest extracted here") with Keccak-f operations listed.
- **Precomputed digests**: Known hashes for "hello", "abc", "", "hello world" displayed in the digest section.
- **No issues encountered** — clean build.

---

## Phase 3: System AIRs + Crypto Extensions + RV32IM Gaps

### Task #12: Set Less Than (`rv32im-slt.html`)

- **Size**: 37,785 bytes
- **15 core columns**: b[4], c[4], cmp_result, b_msb_f, c_msb_f, diff_marker[4], diff_val
- **9 constraint groups** (C1-C9): flag booleans, is_valid, cmp_result boolean, MSB transformation, diff_marker booleans, prefix scan, diff_val matching, prefix_sum boolean, no-marker→zero result
- **Shares diff_marker technique with Branch**: Same MSB-to-LSB scan, but writes result to register instead of branching
- **Bus**: BitwiseOperationLookupBus (2 interactions: MSB range check, diff_val non-zero check)
- **No issues encountered** — clean build, JS syntax verified.

### Task #13: Load Sign Extend (`rv32im-loadsignext.html`)

- **Size**: 37,783 bytes
- **11 core columns**: 3 opcode flags (loadb_flag0, loadb_flag1, loadh_flag), 2 bit columns, shifted_read_data[4], prev_data[4]
- **Three-flag design**: loadb_flag0 (LB shift&1=0), loadb_flag1 (LB shift&1=1), loadh_flag (LH)
- **Sign extension visualization**: Green=data bytes, yellow=sign fill (0x00/0xFF)
- **Bus**: VariableRangeCheckerBus (7-bit range check for MSB)
- **No issues encountered** — clean build.

### Task #14: Program Bus (`system-program.html`)

- **Size**: 35,006 bytes
- **10 columns**: 9 cached main (pc, opcode, a-g) + 1 common main (exec_freq)
- **Zero polynomial constraints** — only bus interaction (LookupBus)
- **3 example programs** with exec_freq visualization
- **"What If" section**: Toggle to remove instruction rows and see bus balance fail
- **No issues encountered** — clean build.

### Task #15: VM Connector (`system-connector.html`)

- **Size**: 37,825 bytes
- **Fixed 2 rows**: begin state + end state
- **5 main columns** + 1 preprocessed column + 4 public values
- **Constraint-rich**: C1-C4 (PV matching), C5 (initial timestamp), C6-C9 (timestamp range decomposition)
- **Segment boundary visualization**: Flow diagram with begin/end state boxes
- **No issues encountered** — clean build.

### Task #16: Phantom Opcodes (`system-phantom.html`)

- **Size**: 33,456 bytes
- **6 columns**: pc, operands[3], timestamp, is_valid
- **Zero local constraints** — only ExecutionBridge bus interaction
- **Key insight highlighted**: discriminant is invisible to verifier
- **Execution vs. Constraint comparison**: Two-column showing prover-time vs verifier-time views
- **No issues encountered** — clean build.

### Task #17: Memory Subsystem (`system-memory.html`)

- **Size**: 50,057 bytes (largest playground)
- **Two AIRs in one playground**: VolatileBoundaryAir + AccessAdapterAir
- **Tab-switching interface**: Toggle between Boundary and Access Adapter views
- **Split/merge visualization**: Block diagrams showing 4-byte→1-byte splits and reverse merges
- **Memory bus balance**: Full ledger showing send/receive pair matching
- **No issues encountered** — clean build.

### Task #18: Modular Field Arithmetic (`modular-arithmetic.html`)

- **Size**: 40,226 bytes
- **BigInt arithmetic**: Uses JavaScript BigInt for exact 256-bit operations
- **secp256k1 prime**: Full 256-bit modulus displayed and used in computations
- **5 operations**: ADD, SUB, MUL, DIV (via extended GCD), IS_EQ
- **Carry-chain visualization**: Per-limb carry propagation for modular reduction
- **Runtime modulus concept**: Explains SETUP-time configuration
- **No issues encountered** — clean build.

### Task #19: Elliptic Curve Weierstrass (`ecc-weierstrass.html`)

- **Size**: 36,350 bytes
- **Small field visualization**: Uses p=97, curve y²=x³+7 for interactive exploration
- **Canvas point plot**: All 78 affine curve points displayed with P1, P2, P3 highlighted
- **EC_ADD and EC_DOUBLE**: Step-by-step lambda computation with modular inverse
- **4 presets** with verified on-curve points
- **Editable witness trace**: Modify lambda/x3/y3 to see constraint violations
- **No issues encountered** — clean build.

---

## Common Findings

### Pattern Consistency
All 23 playgrounds follow the Fibonacci AIR template pattern:
- Same CSS variables, class names, two-column layout
- Same editable cell pattern (`.cell-input`, `.modified`, `onchange` handlers)
- Same state management (`cleanTrace`/`editedTrace`, `recompute()`, `resetTrace()`, `hasEdits()`)
- Same constraint evaluation coloring (`.c-pass` green, `.c-fail` red, `.c-inactive` dash)

### CSS Evolution
The playgrounds independently defined common CSS extensions:
- `.callout`/`.callout-title`/`.callout-body` concept boxes
- `.radio-group` styled as segmented buttons
- `.summary-block` for 32-bit value display with hex/decimal/limbs
- `.flag-display` for one-hot flag visualization
- `.bus-table` for bus interaction tables

### Index Organization (4 sections)
- `.card` (no class) — Examples (blue/default)
- `.card.sys` — System AIRs (purple left border)
- `.card.rv32` — RISC-V instructions (green left border)
- `.card.crypto` — Crypto extensions (yellow left border)

### Constraint Fidelity
All playgrounds were built by reading the actual Rust AIR implementations. Constraint formulas match the `eval()` methods exactly.

### Self-Contained
All 23 playgrounds are fully self-contained single HTML files with no external dependencies.

### JS Syntax
All 23 files pass `node --check` syntax validation.

### Size Distribution
- Smallest: fibonacci-air.html (16KB)
- Largest: system-memory.html (50KB)
- Total: ~740KB for all 24 files (23 playgrounds + index)

### Complete Conceptual Coverage
The 23 playgrounds cover everything needed to prove an Ethereum transaction:

**Examples (1-7)**: Fibonacci AIR → Range Check → LogUp → Limb Arithmetic → Adapter/Core → Memory Bus → Execution Chain

**System (8-11)**: Program Bus → VM Connector → Phantom Opcodes → Memory Subsystem

**RISC-V (12-20)**: ALU → Shift → Multiply → DivRem → Branch → SLT → LoadSignExtend → Load/Store → Jump

**Crypto (21-23)**: Keccak-256 → Modular Arithmetic → ECC Weierstrass
