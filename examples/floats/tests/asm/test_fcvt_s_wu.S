# Test FCVT.S.WU (convert unsigned integer to float)

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: (float)100 = 100.0
    li      x10, 100
    fcvt.s.wu f0, x10
    fsw     f0, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0x42c80000  # Expected: 100.0
    bne     t0, t1, fail

    # Test: (float)1 = 1.0
    li      x10, 1
    fcvt.s.wu f0, x10
    fsw     f0, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0x3f800000  # Expected: 1.0
    bne     t0, t1, fail

    # Test: (float)0 = 0.0
    li      x10, 0
    fcvt.s.wu f0, x10
    fsw     f0, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0x00000000  # Expected: 0.0
    bne     t0, t1, fail

    test_pass

fail:
    test_fail
