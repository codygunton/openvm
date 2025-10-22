# Test for FMV.X.W and FMV.W.X instructions
# These instructions move bit patterns between float and integer registers

.section .data
.align 4
# Float library function pointer - MUST be first in .data for easy access
.globl _float_lib_entry
_float_lib_entry:
    .word _zisk_float   # Pointer to float handler function

test_value:
    .word 0x40490FDB    # Pi (3.14159265) in IEEE 754 single precision

.section .text
.globl _start

_start:
    # Initialize stack pointer
    lui  sp, 0x80           # sp = 0x80000

    # Test FMV.W.X (int to float)
    # Load integer value into x10
    li   x10, 0x40490FDB    # Pi as an integer bit pattern

    # Move from integer register to float register
    fmv.w.x f0, x10         # f0 = bit pattern from x10

    # Test FMV.X.W (float to int)
    # Move from float register back to integer register
    fmv.x.w x11, f0         # x11 = bit pattern from f0

    # Verify the values match
    bne  x10, x11, fail

    # Success - exit with code 0
    li   a0, 0
    .insn 4, 0x000b

fail:
    # Exit with code 1
    li   a0, 1
    .insn 4, 0x000b

# Placeholder for float library (will be linked)
.weak _zisk_float
