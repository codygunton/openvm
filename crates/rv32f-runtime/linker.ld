/* Linker script for OpenVM RV32F runtime library
 * Places float registers, FCSR, and FREG_INST at fixed memory-mapped addresses
 */

SECTIONS
{
    /* Place _openvm_float handler in .text section (contiguous with program code)
     * This allows the transpiler to handle it as part of the main code segment */
    .float_handler : {
        KEEP(*(.float_handler))
    }

    /* Place float register file at 0x18000000 */
    .float_registers 0x18000000 : {
        . = . + 0x80;  /* 32 registers × 4 bytes */
    }

    /* Place FCSR at 0x18000080 */
    .float_csr 0x18000080 : {
        . = . + 0x04;  /* FCSR register */
    }

    /* Place FREG_INST at 0x18000088 */
    .freg_inst 0x18000088 : {
        . = . + 0x04;  /* FREG_INST register */
    }
}
