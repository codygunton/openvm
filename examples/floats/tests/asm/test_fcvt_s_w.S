# Test FCVT.S.W (convert signed integer to float)

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: (float)42 = 42.0
    li      x10, 42
    fcvt.s.w f0, x10
    fsw     f0, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0x42280000  # Expected: 42.0
    bne     t0, t1, fail

    # Test: (float)1 = 1.0
    li      x10, 1
    fcvt.s.w f0, x10
    fsw     f0, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0x3f800000  # Expected: 1.0
    bne     t0, t1, fail

    # Test: (float)-5 = -5.0
    li      x10, -5
    fcvt.s.w f0, x10
    fsw     f0, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0xc0a00000  # Expected: -5.0
    bne     t0, t1, fail

    test_pass

fail:
    test_fail
