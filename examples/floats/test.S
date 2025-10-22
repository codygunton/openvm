# Minimal floating-point test for OpenVM
# Tests FLW (float load) and FADD.S (float add) instructions

.section .data
.align 4
# Float library function pointer - MUST be first in .data for easy access
.globl _float_lib_entry
_float_lib_entry:
    .word _zisk_float   # Pointer to float handler function

float_a:
    .word 0x3f800000    # 1.0 in IEEE 754 single precision
float_b:
    .word 0x40000000    # 2.0 in IEEE 754 single precision
result:
    .word 0x00000000    # Storage for result

.section .text
.globl _start

_start:
    # Initialize stack pointer (sp = x2) to a valid address
    # Place stack at high memory (e.g., 512KB = 0x80000)
    lui  sp, 0x80           # sp = 0x80000

    # Load two floats from memory
    la   a0, float_a
    flw  f0, 0(a0)             # f0 = 1.0

    la   a1, float_b
    flw  f1, 0(a1)             # f1 = 2.0

    # Add them: f2 = f0 + f1
    fadd.s f2, f0, f1          # f2 = 1.0 + 2.0 = 3.0

    # another addition
    fadd.s f0, f1, f2 
    

    # Terminate with exit code 0
    # OpenVM TERMINATE instruction: opcode=0x0b, funct3=0, imm=exit_code
    .insn 4, 0x000b            # TERMINATE with exit code 0

# Placeholder for float library (will be linked)
.weak _zisk_float
