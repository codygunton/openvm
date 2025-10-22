# Simple edge case test - minimal version to debug
.section .data
.align 4
.globl _float_lib_entry
_float_lib_entry:
    .word _zisk_float

float_a:
    .word 0x3f800000    # 1.0
float_b:
    .word 0x40000000    # 2.0
float_qnan:
    .word 0x7fc00000    # Quiet NaN

.section .text
.globl _start

_start:
    lui  sp, 0x80      # sp = 0x80000

    # Test 1: Simple addition
    la   a0, float_a
    flw  f0, 0(a0)
    la   a1, float_b
    flw  f1, 0(a1)
    fadd.s f2, f0, f1

    # Test 2: NaN + 1.0 = NaN
    la      t0, float_qnan
    flw     f3, 0(t0)         # f3 = NaN
    la      t0, float_a
    flw     f4, 0(t0)         # f4 = 1.0
    fadd.s  f5, f3, f4        # f5 = NaN + 1.0 = NaN

    # Success
    li   a0, 0
    .insn 4, 0x000b            # TERMINATE

fail:
    li   a0, 1
    .insn 4, 0x000b            # TERMINATE

.weak _zisk_float
