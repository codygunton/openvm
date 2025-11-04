# Test floating-point comparison operations: FEQ.S, FLT.S, FLE.S

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test FEQ.S: 1.0 == 1.0 → 1 (true)
    la      a0, float_1_0
    flw     f0, 0(a0)
    flw     f1, 0(a0)
    feq.s   t0, f0, f1
    li      t1, 1
    bne     t0, t1, fail

    # Test FEQ.S: 1.0 == 2.0 → 0 (false)
    la      a0, float_1_0
    la      a1, float_2_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    feq.s   t0, f0, f1
    li      t1, 0
    bne     t0, t1, fail

    # Test FLT.S: 1.0 < 2.0 → 1 (true)
    la      a0, float_1_0
    la      a1, float_2_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    flt.s   t0, f0, f1
    li      t1, 1
    bne     t0, t1, fail

    # Test FLT.S: 2.0 < 1.0 → 0 (false)
    la      a0, float_2_0
    la      a1, float_1_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    flt.s   t0, f0, f1
    li      t1, 0
    bne     t0, t1, fail

    # Test FLE.S: 1.0 <= 1.0 → 1 (true)
    la      a0, float_1_0
    flw     f0, 0(a0)
    flw     f1, 0(a0)
    fle.s   t0, f0, f1
    li      t1, 1
    bne     t0, t1, fail

    # Test FLE.S: 2.0 <= 1.0 → 0 (false)
    la      a0, float_2_0
    la      a1, float_1_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    fle.s   t0, f0, f1
    li      t1, 0
    bne     t0, t1, fail

    test_pass

fail:
    test_fail
