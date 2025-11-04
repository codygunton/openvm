# Test FLW (float load word) and FSW (float store word)

.include "tests/asm/common.S"

.section .text
.globl _start

_start:
    lui  sp, 0x80           # sp = 0x80000

    # Test: Load and store a float value
    la   a0, float_1_0
    flw  f0, 0(a0)          # f0 = 1.0
    fsw  f0, 0(sp)          # Store to stack
    lw   t0, 0(sp)          # Load as integer to verify
    li   t1, 0x3f800000     # Expected: 1.0 bit pattern
    bne  t0, t1, fail

    test_pass

fail:
    test_fail
