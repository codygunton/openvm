# Test NaN propagation and edge cases

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: NaN + 1.0 = NaN
    la      a0, float_qnan
    la      a1, float_1_0
    flw     f0, 0(a0)
    flw     f1, 0(a1)
    fadd.s  f2, f0, f1
    fsw     f2, 0(sp)
    lw      t0, 0(sp)
    # Check if result is NaN (exponent=0xFF, mantissa≠0)
    li      t1, 0x7F800000
    and     t2, t0, t1
    bne     t2, t1, fail       # Exponent must be 0xFF
    li      t1, 0x007FFFFF
    and     t2, t0, t1
    beqz    t2, fail           # Mantissa must be non-zero

    # Test: sqrt(-1.0) = NaN
    li      t0, 0xBF800000     # -1.0
    sw      t0, 0(sp)
    flw     f0, 0(sp)
    fsqrt.s f1, f0
    fsw     f1, 0(sp)
    lw      t0, 0(sp)
    # Check if result is NaN
    li      t1, 0x7F800000
    and     t2, t0, t1
    bne     t2, t1, fail
    li      t1, 0x007FFFFF
    and     t2, t0, t1
    beqz    t2, fail

    # Test: 0.0 / 0.0 = NaN
    la      a0, float_pos_zero
    flw     f0, 0(a0)
    flw     f1, 0(a0)
    fdiv.s  f2, f0, f1
    fsw     f2, 0(sp)
    lw      t0, 0(sp)
    # Check if result is NaN
    li      t1, 0x7F800000
    and     t2, t0, t1
    bne     t2, t1, fail
    li      t1, 0x007FFFFF
    and     t2, t0, t1
    beqz    t2, fail

    test_pass

fail:
    test_fail
