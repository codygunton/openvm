# Test FCVT.WU.S (convert float to unsigned integer)

.include "tests/asm/common.S"

.section .data
float_3_7:
    .word 0x406ccccd    # 3.7 in IEEE 754
float_100:
    .word 0x42c80000    # 100.0 in IEEE 754

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: (uint)3.7 = 3
    la      a0, float_3_7
    flw     f0, 0(a0)
    fcvt.wu.s x10, f0, rtz
    li      t0, 3
    bne     x10, t0, fail

    # Test: (uint)100.0 = 100
    la      a0, float_100
    flw     f0, 0(a0)
    fcvt.wu.s x11, f0, rtz
    li      t0, 100
    bne     x11, t0, fail

    # Test: (uint)1.0 = 1
    la      a0, float_1_0
    flw     f0, 0(a0)
    fcvt.wu.s x12, f0, rtz
    li      t0, 1
    bne     x12, t0, fail

    test_pass

fail:
    test_fail
