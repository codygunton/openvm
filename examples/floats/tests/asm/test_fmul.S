# Test FMUL.S (floating-point multiplication)

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: 2.0 * 3.0 = 6.0
    la   a0, float_2_0
    flw  f0, 0(a0)
    la   a1, float_3_0
    flw  f1, 0(a1)
    fmul.s f2, f0, f1
    fsw  f2, 0(sp)
    lw   t0, 0(sp)
    li   t1, 0x40c00000     # Expected: 6.0
    bne  t0, t1, fail

    test_pass

fail:
    test_fail
