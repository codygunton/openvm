# Test basic floating-point operations: FLW, FSW, FADD.S

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    # Initialize stack pointer
    lui  sp, 0x80           # sp = 0x80000

    # Test 1: Load and store
    la   a0, float_1_0
    flw  f0, 0(a0)          # f0 = 1.0
    fsw  f0, 0(sp)          # Store back
    lw   t0, 0(sp)
    li   t1, 0x3f800000     # Expected: 1.0
    bne  t0, t1, fail

    # Test 2: FADD.S (1.0 + 2.0 = 3.0)
    la   a0, float_1_0
    flw  f0, 0(a0)          # f0 = 1.0
    la   a1, float_2_0
    flw  f1, 0(a1)          # f1 = 2.0
    fadd.s f2, f0, f1       # f2 = 3.0
    fsw  f2, 0(sp)
    lw   t0, 0(sp)
    li   t1, 0x40400000     # Expected: 3.0
    bne  t0, t1, fail

    # Test 3: FSUB.S (3.0 - 1.0 = 2.0)
    la   a0, float_3_0
    flw  f0, 0(a0)          # f0 = 3.0
    la   a1, float_1_0
    flw  f1, 0(a1)          # f1 = 1.0
    fsub.s f2, f0, f1       # f2 = 2.0
    fsw  f2, 0(sp)
    lw   t0, 0(sp)
    li   t1, 0x40000000     # Expected: 2.0
    bne  t0, t1, fail

    # Test 4: FMUL.S (2.0 * 3.0 = 6.0)
    la   a0, float_2_0
    flw  f0, 0(a0)          # f0 = 2.0
    la   a1, float_3_0
    flw  f1, 0(a1)          # f1 = 3.0
    fmul.s f2, f0, f1       # f2 = 6.0
    fsw  f2, 0(sp)
    lw   t0, 0(sp)
    li   t1, 0x40c00000     # Expected: 6.0
    bne  t0, t1, fail

    # Test 5: FDIV.S (4.0 / 2.0 = 2.0)
    la   a0, float_4_0
    flw  f0, 0(a0)          # f0 = 4.0
    la   a1, float_2_0
    flw  f1, 0(a1)          # f1 = 2.0
    fdiv.s f2, f0, f1       # f2 = 2.0
    fsw  f2, 0(sp)
    lw   t0, 0(sp)
    li   t1, 0x40000000     # Expected: 2.0
    bne  t0, t1, fail

    test_pass

fail:
    test_fail
