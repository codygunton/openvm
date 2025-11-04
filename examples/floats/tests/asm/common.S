# Common definitions for all float tests
# This file should be included at the start of each test

.section .float_lib_entry,"aw",@progbits
.align 4
# Float library function pointer at fixed address 0x00100000
.globl _float_lib_entry
_float_lib_entry:
    .word _zisk_float   # Pointer to float handler function

.section .data
.align 4

# Common float constants
float_1_0:
    .word 0x3f800000    # 1.0
float_2_0:
    .word 0x40000000    # 2.0
float_3_0:
    .word 0x40400000    # 3.0
float_4_0:
    .word 0x40800000    # 4.0
float_pos_zero:
    .word 0x00000000    # +0.0
float_neg_zero:
    .word 0x80000000    # -0.0
float_pos_inf:
    .word 0x7f800000    # +∞
float_neg_inf:
    .word 0xff800000    # -∞
float_qnan:
    .word 0x7fc00000    # Quiet NaN

# Placeholder for float library (will be linked)
.weak _zisk_float

# Macros for test exit
.macro test_pass
    .insn i 0x0b, 0, x0, x0, 0
.endm

.macro test_fail
    .insn i 0x0b, 0, x0, x0, 1
.endm
