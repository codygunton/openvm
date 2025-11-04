# Test FEQ.S (floating-point equality comparison)

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: 1.0 == 1.0 → 1 (true)
    la      a0, float_1_0
    flw     f0, 0(a0)
    flw     f1, 0(a0)
    feq.s   t0, f0, f1
    li      t1, 1
    bne     t0, t1, fail

    # Test: 1.0 == 2.0 → 0 (false)
    la      a0, float_1_0
    la      a1, float_2_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    feq.s   t0, f0, f1
    li      t1, 0
    bne     t0, t1, fail

    test_pass

fail:
    test_fail
