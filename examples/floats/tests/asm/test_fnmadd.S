# Test FNMADD.S (fused negated multiply-add: -(a*b) - c)

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: -(2.0 * 3.0) - 4.0 = -10.0
    la      a0, float_2_0
    la      a1, float_3_0
    la      a2, float_4_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    flw     f2, 0(a2)
    fnmadd.s f3, f0, f1, f2
    fsw     f3, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0xC1200000  # Expected: -10.0
    bne     t0, t1, fail

    test_pass

fail:
    test_fail
