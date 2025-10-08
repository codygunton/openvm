/* Linker script for OpenVM RV32F runtime library
 * Places the dispatch table at a fixed address for indirect calls
 */

SECTIONS
{
    /* Place dispatch table at fixed address 0xE0000000 */
    .float_dispatch_table 0xE0000000 : {
        KEEP(*(.float_dispatch_table))
    }

    /* Place float register file at 0xC0000000 */
    .float_registers 0xC0000000 : {
        . = . + 0x80;  /* 32 registers * 4 bytes */
    }

    /* Place FCSR at 0xC0000080 */
    .float_csr 0xC0000080 : {
        . = . + 4;  /* 1 register */
    }
}
