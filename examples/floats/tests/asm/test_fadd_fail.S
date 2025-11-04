# Test FADD.S failure case (intentionally wrong expected value)
# This test is designed to fail to verify error detection

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: 1.0 + 2.0 = 3.0, but we expect wrong value (4.0)
    la   a0, float_1_0
    flw  f0, 0(a0)
    la   a1, float_1_0 
    flw  f1, 0(a1)
    fadd.s f2, f0, f1
    fsw  f2, 0(sp)
    lw   t0, 0(sp)
    li   t1, 0x40400000 # Expect 3 but result is 2, so we hit test_fail
    bne  t0, t1, fail

    test_pass

fail:
    test_fail
