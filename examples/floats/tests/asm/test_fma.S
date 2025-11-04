# Test fused multiply-add operations: FMADD.S, FMSUB.S, FNMSUB.S, FNMADD.S

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test FMADD.S: (2.0 * 3.0) + 4.0 = 10.0
    la      a0, float_2_0
    la      a1, float_3_0
    la      a2, float_4_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    flw     f2, 0(a2)
    fmadd.s f3, f0, f1, f2  # f3 = (2.0 * 3.0) + 4.0 = 10.0
    fsw     f3, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0x41200000  # Expected: 10.0
    bne     t0, t1, fail

    # Test FMSUB.S: (2.0 * 3.0) - 4.0 = 2.0
    la      a0, float_2_0
    la      a1, float_3_0
    la      a2, float_4_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    flw     f2, 0(a2)
    fmsub.s f3, f0, f1, f2  # f3 = (2.0 * 3.0) - 4.0 = 2.0
    fsw     f3, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0x40000000  # Expected: 2.0
    bne     t0, t1, fail

    # Test FNMSUB.S: -(2.0 * 3.0) + 4.0 = -2.0
    la      a0, float_2_0
    la      a1, float_3_0
    la      a2, float_4_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    flw     f2, 0(a2)
    fnmsub.s f3, f0, f1, f2 # f3 = -(2.0 * 3.0) + 4.0 = -2.0
    fsw     f3, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0xC0000000  # Expected: -2.0
    bne     t0, t1, fail

    # Test FNMADD.S: -(2.0 * 3.0) - 4.0 = -10.0
    la      a0, float_2_0
    la      a1, float_3_0
    la      a2, float_4_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    flw     f2, 0(a2)
    fnmadd.s f3, f0, f1, f2 # f3 = -(2.0 * 3.0) - 4.0 = -10.0
    fsw     f3, 0(sp)
    lw      t0, 0(sp)
    li      t1, 0xC1200000  # Expected: -10.0
    bne     t0, t1, fail

    test_pass

fail:
    test_fail
