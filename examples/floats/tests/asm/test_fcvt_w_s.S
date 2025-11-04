# Test FCVT.W.S (convert float to signed integer)

.include "tests/asm/common.S"

.section .data
float_3_7:
    .word 0x406ccccd    # 3.7 in IEEE 754

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: (int)3.7 = 3 with RTZ (round toward zero)
    la      a0, float_3_7
    flw     f0, 0(a0)
    fcvt.w.s x10, f0, rtz
    li      t0, 3
    bne     x10, t0, fail

    # Test: (int)1.0 = 1
    la      a0, float_1_0
    flw     f0, 0(a0)
    fcvt.w.s x11, f0, rtz
    li      t0, 1
    bne     x11, t0, fail

    # Test: (int)2.0 = 2
    la      a0, float_2_0
    flw     f0, 0(a0)
    fcvt.w.s x12, f0, rtz
    li      t0, 2
    bne     x12, t0, fail

    test_pass

fail:
    test_fail
