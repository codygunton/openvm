# Test FSUB.S (floating-point subtraction)

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: 3.0 - 1.0 = 2.0
    la   a0, float_3_0
    flw  f0, 0(a0)
    la   a1, float_1_0
    flw  f1, 0(a1)
    fsub.s f2, f0, f1
    fsw  f2, 0(sp)
    lw   t0, 0(sp)
    li   t1, 0x40000000     # Expected: 2.0
    bne  t0, t1, fail

    test_pass

fail:
    test_fail
