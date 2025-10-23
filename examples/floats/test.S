# Comprehensive floating-point test for OpenVM RV32F Extension
# Tests FLW, FSW, FADD.S (basic operations)
# Tests FMADD.S, FMSUB.S, FNMSUB.S, FNMADD.S (fused multiply-add)
# Tests FSQRT.S, FMIN.S, FMAX.S, FSGNJ.S, FSGNJN.S, FSGNJX.S (extended float_alu)
# Tests FCVT.W.S, FCVT.WU.S, FCVT.S.W, FCVT.S.WU (conversions - Task #16)
# Tests FEQ.S, FLT.S, FLE.S (comparisons - Task #16)
# Tests FMV.X.W, FMV.W.X (register moves - Task #16)
# Tests FCLASS.S (classification - Task #16)

.section .float_lib_entry,"aw",@progbits
.align 4
# Float library function pointer at fixed address 0x1EC80
.globl _float_lib_entry
_float_lib_entry:
    .word _zisk_float   # Pointer to float handler function

.section .data
.align 4

float_a:
    .word 0x3f800000    # 1.0 in IEEE 754 single precision
float_b:
    .word 0x40000000    # 2.0 in IEEE 754 single precision
float_3_7:
    .word 0x406ccccd    # 3.7 in IEEE 754
float_pi:
    .word 0x40490fdb    # 3.14159 (approximation of pi)
float_100:
    .word 0x42c80000    # 100.0 in IEEE 754
float_pos_zero:
    .word 0x00000000    # +0.0
float_neg_zero:
    .word 0x80000000    # -0.0
float_pos_inf:
    .word 0x7f800000    # +∞
float_neg_inf:
    .word 0xff800000    # -∞
float_qnan:
    .word 0x7fc00000    # Quiet NaN
result:
    .word 0x00000000    # Storage for result

.section .text
.globl _start

_start:
    # Initialize stack pointer (sp = x2) to a valid address
    # Place stack at high memory (e.g., 512KB = 0x80000)
    lui  sp, 0x80           # sp = 0x80000

    # Load two floats from memory
    la   a0, float_a
    flw  f0, 0(a0)             # f0 = 1.0

    la   a1, float_b
    flw  f1, 0(a1)             # f1 = 2.0

    # Add them: f2 = f0 + f1
    fadd.s f2, f0, f1          # f2 = 1.0 + 2.0 = 3.0

    # Store result back to memory
    la   a2, result
    fsw  f2, 0(a2)             # Store f2 to result

    # Test FMADD.S: (2.0 * 3.0) + 4.0 = 10.0
    li      t0, 0x40000000     # 2.0 in IEEE 754
    li      t1, 0x40400000     # 3.0 in IEEE 754
    li      t2, 0x40800000     # 4.0 in IEEE 754
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    sw      t2, 8(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    flw     f3, 8(sp)
    fmadd.s f4, f0, f1, f3     # f4 = (2.0 * 3.0) + 4.0 = 10.0
    fsw     f4, 12(sp)
    lw      t3, 12(sp)
    li      t4, 0x41200000     # Expected: 10.0
    bne     t3, t4, fail

    # Test FMSUB.S: (2.0 * 3.0) - 4.0 = 2.0
    li      t0, 0x40000000     # 2.0 in IEEE 754
    li      t1, 0x40400000     # 3.0 in IEEE 754
    li      t2, 0x40800000     # 4.0 in IEEE 754
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    sw      t2, 8(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    flw     f3, 8(sp)
    fmsub.s f4, f0, f1, f3     # f4 = (2.0 * 3.0) - 4.0 = 2.0
    fsw     f4, 12(sp)
    lw      t3, 12(sp)
    li      t4, 0x40000000     # Expected: 2.0
    bne     t3, t4, fail

    # Test FNMSUB.S: -(2.0 * 3.0) + 4.0 = -2.0
    li      t0, 0x40000000     # 2.0 in IEEE 754
    li      t1, 0x40400000     # 3.0 in IEEE 754
    li      t2, 0x40800000     # 4.0 in IEEE 754
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    sw      t2, 8(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    flw     f3, 8(sp)
    fnmsub.s f4, f0, f1, f3    # f4 = -(2.0 * 3.0) + 4.0 = -2.0
    fsw     f4, 12(sp)
    lw      t3, 12(sp)
    li      t4, 0xC0000000     # Expected: -2.0
    bne     t3, t4, fail

    # Test FNMADD.S: -(2.0 * 3.0) - 4.0 = -10.0
    li      t0, 0x40000000     # 2.0 in IEEE 754
    li      t1, 0x40400000     # 3.0 in IEEE 754
    li      t2, 0x40800000     # 4.0 in IEEE 754
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    sw      t2, 8(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    flw     f3, 8(sp)
    fnmadd.s f4, f0, f1, f3    # f4 = -(2.0 * 3.0) - 4.0 = -10.0
    fsw     f4, 12(sp)
    lw      t3, 12(sp)
    li      t4, 0xC1200000     # Expected: -10.0
    bne     t3, t4, fail

    # ===== Test FSQRT.S: sqrt(4.0) = 2.0 =====
    li      t0, 0x40800000     # 4.0 in IEEE 754
    sw      t0, 0(sp)
    flw     f0, 0(sp)
    fsqrt.s f1, f0             # f1 = sqrt(4.0) = 2.0
    fsw     f1, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0x40000000     # Expected: 2.0
    bne     t1, t2, fail

    # ===== Test FSQRT.S: sqrt(2.0) ≈ 1.414213562 =====
    li      t0, 0x40000000     # 2.0 in IEEE 754
    sw      t0, 0(sp)
    flw     f0, 0(sp)
    fsqrt.s f1, f0             # f1 = sqrt(2.0) ≈ 1.414213562
    fsw     f1, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0x3FB504F3     # Expected: ~1.414213562
    bne     t1, t2, fail

    # ===== Test FMIN.S: min(1.0, 2.0) = 1.0 =====
    li      t0, 0x3F800000     # 1.0
    li      t1, 0x40000000     # 2.0
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    fmin.s  f2, f0, f1         # f2 = min(1.0, 2.0) = 1.0
    fsw     f2, 8(sp)
    lw      t2, 8(sp)
    li      t3, 0x3F800000     # Expected: 1.0
    bne     t2, t3, fail

    # ===== Test FMIN.S: min(-0.0, +0.0) = -0.0 =====
    li      t0, 0x80000000     # -0.0
    li      t1, 0x00000000     # +0.0
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    fmin.s  f2, f0, f1         # f2 = min(-0.0, +0.0) = -0.0
    fsw     f2, 8(sp)
    lw      t2, 8(sp)
    li      t3, 0x80000000     # Expected: -0.0
    bne     t2, t3, fail

    # ===== Test FMAX.S: max(1.0, 2.0) = 2.0 =====
    li      t0, 0x3F800000     # 1.0
    li      t1, 0x40000000     # 2.0
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    fmax.s  f2, f0, f1         # f2 = max(1.0, 2.0) = 2.0
    fsw     f2, 8(sp)
    lw      t2, 8(sp)
    li      t3, 0x40000000     # Expected: 2.0
    bne     t2, t3, fail

    # ===== Test FMAX.S: max(-0.0, +0.0) = +0.0 =====
    li      t0, 0x80000000     # -0.0
    li      t1, 0x00000000     # +0.0
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    fmax.s  f2, f0, f1         # f2 = max(-0.0, +0.0) = +0.0
    fsw     f2, 8(sp)
    lw      t2, 8(sp)
    li      t3, 0x00000000     # Expected: +0.0
    bne     t2, t3, fail

    # ===== Test FSGNJ.S: magnitude of 1.5 with sign of -2.0 = -1.5 =====
    li      t0, 0x3FC00000     # 1.5
    li      t1, 0xC0000000     # -2.0
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    fsgnj.s f2, f0, f1         # f2 = sign(-2.0) | magnitude(1.5) = -1.5
    fsw     f2, 8(sp)
    lw      t2, 8(sp)
    li      t3, 0xBFC00000     # Expected: -1.5
    bne     t2, t3, fail

    # ===== Test FSGNJN.S: magnitude of 1.5 with negated sign of -2.0 = 1.5 =====
    li      t0, 0x3FC00000     # 1.5
    li      t1, 0xC0000000     # -2.0
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    fsgnjn.s f2, f0, f1        # f2 = ~sign(-2.0) | magnitude(1.5) = 1.5
    fsw     f2, 8(sp)
    lw      t2, 8(sp)
    li      t3, 0x3FC00000     # Expected: 1.5
    bne     t2, t3, fail

    # ===== Test FSGNJX.S: XOR signs of 1.5 and -2.0 = -1.5 =====
    li      t0, 0x3FC00000     # 1.5 (positive)
    li      t1, 0xC0000000     # -2.0 (negative)
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    fsgnjx.s f2, f0, f1        # f2 = sign_xor(1.5, -2.0) = -1.5
    fsw     f2, 8(sp)
    lw      t2, 8(sp)
    li      t3, 0xBFC00000     # Expected: -1.5 (positive XOR negative = negative)
    bne     t2, t3, fail

    # ===== Test FSGNJX.S: XOR signs of 1.5 and 2.0 = 1.5 =====
    li      t0, 0x3FC00000     # 1.5 (positive)
    li      t1, 0x40000000     # 2.0 (positive)
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    fsgnjx.s f2, f0, f1        # f2 = sign_xor(1.5, 2.0) = 1.5
    fsw     f2, 8(sp)
    lw      t2, 8(sp)
    li      t3, 0x3FC00000     # Expected: 1.5 (positive XOR positive = positive)
    bne     t2, t3, fail

    #=================================================================
    # Task #16 Tests: Conversions, Comparisons, Moves, Classification
    #=================================================================

    # ===== Test FCVT.W.S: Convert float to signed int with RTZ rounding =====
    # Test: (int)3.7 = 3 with RTZ (round toward zero)
    la      t0, float_3_7
    flw     f5, 0(t0)          # f5 = 3.7
    fcvt.w.s x10, f5, rtz      # x10 = (int)3.7 = 3 (RTZ mode)
    li      t1, 3
    bne     x10, t1, fail

    # Test: (int)1.0 = 1
    la      t0, float_a
    flw     f5, 0(t0)          # f5 = 1.0
    fcvt.w.s x11, f5, rtz      # x11 = (int)1.0 = 1
    li      t1, 1
    bne     x11, t1, fail

    # ===== Test FCVT.WU.S: Convert float to unsigned int =====
    # Test: (uint)3.7 = 3
    la      t0, float_3_7
    flw     f6, 0(t0)          # f6 = 3.7
    fcvt.wu.s x12, f6, rtz     # x12 = (uint)3.7 = 3
    li      t1, 3
    bne     x12, t1, fail

    # Test: (uint)100.0 = 100
    la      t0, float_100
    flw     f6, 0(t0)          # f6 = 100.0
    fcvt.wu.s x13, f6, rtz     # x13 = (uint)100.0 = 100
    li      t1, 100
    bne     x13, t1, fail

    # ===== Test FCVT.S.W: Convert signed int to float =====
    # Test: (float)42 = 42.0
    li      x14, 42
    fcvt.s.w f7, x14           # f7 = (float)42 = 42.0
    fsw     f7, 0(sp)
    lw      t2, 0(sp)
    li      t3, 0x42280000     # Expected: 42.0
    bne     t2, t3, fail

    # ===== Test FCVT.S.WU: Convert unsigned int to float =====
    # Test: (float)100 = 100.0
    li      x16, 100
    fcvt.s.wu f9, x16          # f9 = (float)100 = 100.0
    fsw     f9, 0(sp)
    lw      t2, 0(sp)
    li      t3, 0x42c80000     # Expected: 100.0
    bne     t2, t3, fail

    # ===== Test FEQ.S: Float equal comparison =====
    # Test: 1.0 == 1.0 → 1 (true)
    la      t0, float_a
    flw     f10, 0(t0)         # f10 = 1.0
    flw     f11, 0(t0)         # f11 = 1.0
    feq.s   x17, f10, f11      # x17 = (1.0 == 1.0) = 1
    li      t1, 1
    bne     x17, t1, fail

    # Test: 1.0 == 2.0 → 0 (false)
    la      t0, float_a
    flw     f10, 0(t0)         # f10 = 1.0
    la      t0, float_b
    flw     f11, 0(t0)         # f11 = 2.0
    feq.s   x18, f10, f11      # x18 = (1.0 == 2.0) = 0
    li      t1, 0
    bne     x18, t1, fail

    # ===== Test FLT.S: Float less than comparison =====
    # Test: 1.0 < 2.0 → 1 (true)
    la      t0, float_a
    flw     f12, 0(t0)         # f12 = 1.0
    la      t0, float_b
    flw     f13, 0(t0)         # f13 = 2.0
    flt.s   x20, f12, f13      # x20 = (1.0 < 2.0) = 1
    li      t1, 1
    bne     x20, t1, fail

    # Test: 2.0 < 1.0 → 0 (false)
    la      t0, float_b
    flw     f12, 0(t0)         # f12 = 2.0
    la      t0, float_a
    flw     f13, 0(t0)         # f13 = 1.0
    flt.s   x21, f12, f13      # x21 = (2.0 < 1.0) = 0
    li      t1, 0
    bne     x21, t1, fail

    # ===== Test FLE.S: Float less than or equal comparison =====
    # Test: 1.0 <= 1.0 → 1 (true)
    la      t0, float_a
    flw     f14, 0(t0)         # f14 = 1.0
    flw     f15, 0(t0)         # f15 = 1.0
    fle.s   x23, f14, f15      # x23 = (1.0 <= 1.0) = 1
    li      t1, 1
    bne     x23, t1, fail

    # ===== Test FMV.X.W: Move float register bits to integer register =====
    # Test: Move pi (3.14159) bits to integer register
    la      t0, float_pi
    flw     f16, 0(t0)         # f16 = 3.14159 (0x40490fdb)
    fmv.x.w x26, f16           # x26 = bit pattern of f16
    li      t1, 0x40490fdb
    bne     x26, t1, fail

    # ===== Test FMV.W.X: Move integer register bits to float register =====
    # Test: Move integer bit pattern to float register
    li      x28, 0x42c80000    # 100.0 bit pattern
    fmv.w.x f17, x28           # f17 = float with bits of x28
    fsw     f17, 0(sp)
    lw      t2, 0(sp)
    li      t3, 0x42c80000
    bne     t2, t3, fail

    # ===== Test FCLASS.S: Classify float values =====
    # FCLASS.S returns a 10-bit mask where exactly one bit is set:
    # Bit 0: -∞, Bit 1: negative normal, Bit 2: negative subnormal
    # Bit 3: -0, Bit 4: +0, Bit 5: positive subnormal
    # Bit 6: positive normal, Bit 7: +∞
    # Bit 8: signaling NaN, Bit 9: quiet NaN

    # Test: Classify +0.0 → bit 4
    la      t0, float_pos_zero
    flw     f19, 0(t0)         # f19 = +0.0
    fclass.s x30, f19          # x30 = classification
    li      t1, 0x010          # Bit 4 set (16 decimal)
    bne     x30, t1, fail

    # Test: Classify -0.0 → bit 3
    la      t0, float_neg_zero
    flw     f20, 0(t0)         # f20 = -0.0
    fclass.s x31, f20          # x31 = classification
    li      t1, 0x008          # Bit 3 set (8 decimal)
    bne     x31, t1, fail

    # Test: Classify normal positive number (1.0) → bit 6
    la      t0, float_a
    flw     f21, 0(t0)         # f21 = 1.0 (normal positive)
    fclass.s t2, f21           # t2 = classification
    li      t1, 0x040          # Bit 6 set (64 decimal)
    bne     t2, t1, fail

    # Test: Classify +∞ → bit 7
    la      t0, float_pos_inf
    flw     f22, 0(t0)         # f22 = +∞
    fclass.s t2, f22           # t2 = classification
    li      t1, 0x080          # Bit 7 set (128 decimal)
    bne     t2, t1, fail

    # Test: Classify -∞ → bit 0
    la      t0, float_neg_inf
    flw     f23, 0(t0)         # f23 = -∞
    fclass.s t2, f23           # t2 = classification
    li      t1, 0x001          # Bit 0 set (1 decimal)
    bne     t2, t1, fail

    # Test: Classify quiet NaN → bit 9
    la      t0, float_qnan
    flw     f24, 0(t0)         # f24 = quiet NaN
    fclass.s t2, f24           # t2 = classification
    li      t1, 0x200          # Bit 9 set (512 decimal)
    bne     t2, t1, fail

    #=================================================================
    # Task #17 Tests: Edge Cases (NaN, ±∞, ±0, Subnormals, Rounding)
    #=================================================================

    # ===== NaN Propagation Tests =====

    # Test: NaN + 1.0 = NaN
    la      t0, float_qnan
    flw     f25, 0(t0)         # f25 = NaN
    la      t0, float_a
    flw     f26, 0(t0)         # f26 = 1.0
    fadd.s  f27, f25, f26      # f27 = NaN + 1.0 = NaN
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    # Check if result is NaN (exponent=0xFF, mantissa≠0)
    li      t2, 0x7F800000
    and     t3, t1, t2
    bne     t3, t2, fail       # Exponent must be 0xFF
    li      t2, 0x007FFFFF
    and     t3, t1, t2
    beqz    t3, fail           # Mantissa must be non-zero

    # Test: sqrt(-1.0) = NaN
    li      t0, 0xBF800000     # -1.0
    sw      t0, 0(sp)
    flw     f25, 0(sp)
    fsqrt.s f26, f25           # f26 = sqrt(-1.0) = NaN
    fsw     f26, 0(sp)
    lw      t1, 0(sp)
    # Check if result is NaN
    li      t2, 0x7F800000
    and     t3, t1, t2
    bne     t3, t2, fail       # Exponent must be 0xFF
    li      t2, 0x007FFFFF
    and     t3, t1, t2
    beqz    t3, fail           # Mantissa must be non-zero

    # Test: 0.0 / 0.0 = NaN
    la      t0, float_pos_zero
    flw     f25, 0(t0)         # f25 = +0.0
    flw     f26, 0(t0)         # f26 = +0.0
    fdiv.s  f27, f25, f26      # f27 = 0.0 / 0.0 = NaN
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    # Check if result is NaN
    li      t2, 0x7F800000
    and     t3, t1, t2
    bne     t3, t2, fail       # Exponent must be 0xFF
    li      t2, 0x007FFFFF
    and     t3, t1, t2
    beqz    t3, fail           # Mantissa must be non-zero

    # ===== Infinity Arithmetic Tests =====

    # Test: ∞ + 1.0 = ∞
    la      t0, float_pos_inf
    flw     f25, 0(t0)         # f25 = +∞
    la      t0, float_a
    flw     f26, 0(t0)         # f26 = 1.0
    fadd.s  f27, f25, f26      # f27 = ∞ + 1.0 = ∞
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0x7F800000     # Expected: +∞
    bne     t1, t2, fail

    # Test: ∞ + ∞ = ∞
    la      t0, float_pos_inf
    flw     f25, 0(t0)         # f25 = +∞
    flw     f26, 0(t0)         # f26 = +∞
    fadd.s  f27, f25, f26      # f27 = ∞ + ∞ = ∞
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0x7F800000     # Expected: +∞
    bne     t1, t2, fail

    # Test: ∞ - ∞ = NaN
    la      t0, float_pos_inf
    flw     f25, 0(t0)         # f25 = +∞
    flw     f26, 0(t0)         # f26 = +∞
    fsub.s  f27, f25, f26      # f27 = ∞ - ∞ = NaN
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    # Check if result is NaN
    li      t2, 0x7F800000
    and     t3, t1, t2
    bne     t3, t2, fail       # Exponent must be 0xFF
    li      t2, 0x007FFFFF
    and     t3, t1, t2
    beqz    t3, fail           # Mantissa must be non-zero

    # Test: ∞ * 0.0 = NaN
    la      t0, float_pos_inf
    flw     f25, 0(t0)         # f25 = +∞
    la      t0, float_pos_zero
    flw     f26, 0(t0)         # f26 = +0.0
    fmul.s  f27, f25, f26      # f27 = ∞ * 0.0 = NaN
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    # Check if result is NaN
    li      t2, 0x7F800000
    and     t3, t1, t2
    bne     t3, t2, fail       # Exponent must be 0xFF
    li      t2, 0x007FFFFF
    and     t3, t1, t2
    beqz    t3, fail           # Mantissa must be non-zero

    # Test: 1.0 / 0.0 = +∞
    la      t0, float_a
    flw     f25, 0(t0)         # f25 = 1.0
    la      t0, float_pos_zero
    flw     f26, 0(t0)         # f26 = +0.0
    fdiv.s  f27, f25, f26      # f27 = 1.0 / 0.0 = +∞
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0x7F800000     # Expected: +∞
    bne     t1, t2, fail

    # Test: -1.0 / 0.0 = -∞
    li      t0, 0xBF800000     # -1.0
    sw      t0, 0(sp)
    flw     f25, 0(sp)         # f25 = -1.0
    la      t0, float_pos_zero
    flw     f26, 0(t0)         # f26 = +0.0
    fdiv.s  f27, f25, f26      # f27 = -1.0 / 0.0 = -∞
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0xFF800000     # Expected: -∞
    bne     t1, t2, fail

    # ===== Signed Zero Tests =====

    # Test: +0.0 + +0.0 = +0.0
    la      t0, float_pos_zero
    flw     f25, 0(t0)         # f25 = +0.0
    flw     f26, 0(t0)         # f26 = +0.0
    fadd.s  f27, f25, f26      # f27 = +0.0 + +0.0 = +0.0
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0x00000000     # Expected: +0.0
    bne     t1, t2, fail

    # Test: +0.0 + -0.0 = +0.0
    la      t0, float_pos_zero
    flw     f25, 0(t0)         # f25 = +0.0
    la      t0, float_neg_zero
    flw     f26, 0(t0)         # f26 = -0.0
    fadd.s  f27, f25, f26      # f27 = +0.0 + -0.0 = +0.0
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0x00000000     # Expected: +0.0
    bne     t1, t2, fail

    # Test: -0.0 + -0.0 = -0.0
    la      t0, float_neg_zero
    flw     f25, 0(t0)         # f25 = -0.0
    flw     f26, 0(t0)         # f26 = -0.0
    fadd.s  f27, f25, f26      # f27 = -0.0 + -0.0 = -0.0
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0x80000000     # Expected: -0.0
    bne     t1, t2, fail

    # Test: +0.0 * -1.0 = -0.0
    la      t0, float_pos_zero
    flw     f25, 0(t0)         # f25 = +0.0
    li      t0, 0xBF800000     # -1.0
    sw      t0, 4(sp)
    flw     f26, 4(sp)         # f26 = -1.0
    fmul.s  f27, f25, f26      # f27 = +0.0 * -1.0 = -0.0
    fsw     f27, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0x80000000     # Expected: -0.0
    bne     t1, t2, fail

    # ===== Subnormal Number Tests =====
    # Smallest positive normal: 0x00800000 (1.175494351e-38)
    # Largest subnormal:        0x007FFFFF (1.175494211e-38)
    # Smallest subnormal:       0x00000001 (1.401298464e-45)

    # Test: Operations with subnormal numbers
    # Test: subnormal + subnormal (should produce valid result)
    li      t0, 0x00000001     # Smallest subnormal
    li      t1, 0x00000001     # Smallest subnormal
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    flw     f25, 0(sp)
    flw     f26, 4(sp)
    fadd.s  f27, f25, f26      # Add two smallest subnormals
    fsw     f27, 8(sp)
    lw      t2, 8(sp)
    li      t3, 0x00000002     # Expected: 2 * smallest subnormal
    bne     t2, t3, fail

    # Test: Smallest normal * 0.5 = largest subnormal
    li      t0, 0x00800000     # Smallest normal
    li      t1, 0x3F000000     # 0.5
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    flw     f25, 0(sp)
    flw     f26, 4(sp)
    fmul.s  f27, f25, f26      # Multiply smallest normal by 0.5
    fsw     f27, 8(sp)
    lw      t2, 8(sp)
    li      t3, 0x00400000     # Expected: largest subnormal / 2
    bne     t2, t3, fail

    # ===== Rounding Mode Tests =====
    # Test different rounding modes with conversion instructions
    # RNE (round to nearest, ties to even) = 0
    # RTZ (round toward zero) = 1
    # RDN (round down) = 2
    # RUP (round up) = 3
    # RMM (round to nearest, ties to max magnitude) = 4

    # Test: Convert 2.5 to int with different rounding modes
    li      t0, 0x40200000     # 2.5
    sw      t0, 0(sp)
    flw     f28, 0(sp)

    # RTZ: 2.5 → 2 (toward zero)
    fcvt.w.s t1, f28, rtz
    li      t2, 2
    bne     t1, t2, fail

    # RDN: 2.5 → 2 (toward -∞)
    fcvt.w.s t1, f28, rdn
    li      t2, 2
    bne     t1, t2, fail

    # RUP: 2.5 → 3 (toward +∞)
    fcvt.w.s t1, f28, rup
    li      t2, 3
    bne     t1, t2, fail

    # RNE: 2.5 → 2 (ties to even)
    fcvt.w.s t1, f28, rne
    li      t2, 2
    bne     t1, t2, fail

    # Test: Convert -2.5 to int with different rounding modes
    li      t0, 0xC0200000     # -2.5
    sw      t0, 0(sp)
    flw     f29, 0(sp)

    # RTZ: -2.5 → -2 (toward zero)
    fcvt.w.s t1, f29, rtz
    li      t2, -2
    bne     t1, t2, fail

    # RDN: -2.5 → -3 (toward -∞)
    fcvt.w.s t1, f29, rdn
    li      t2, -3
    bne     t1, t2, fail

    # RUP: -2.5 → -2 (toward +∞)
    fcvt.w.s t1, f29, rup
    li      t2, -2
    bne     t1, t2, fail

    # RNE: -2.5 → -2 (ties to even)
    fcvt.w.s t1, f29, rne
    li      t2, -2
    bne     t1, t2, fail

    # Test: Rounding in arithmetic operations
    # Test with 3.5 (ties to even should round to 4)
    li      t0, 0x40600000     # 3.5
    sw      t0, 0(sp)
    flw     f30, 0(sp)
    fcvt.w.s t1, f30, rne
    li      t2, 4              # 3.5 rounds to 4 (even)
    bne     t1, t2, fail

    # All tests passed - terminate with exit code 0
    .insn i 0x0b, 0, x0, x0, 0

fail:
    # Test failed - terminate with exit code 1 encoded in instruction
    .insn i 0x0b, 0, x0, x0, 1

# Placeholder for float library (will be linked)
.weak _zisk_float
