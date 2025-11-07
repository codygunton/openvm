/* Minimal linker script for OpenVM float test */

OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x10000, LENGTH = 512M
}

PHDRS
{
    text PT_LOAD FLAGS(5);  /* R+X = 5 (read + execute) */
    data PT_LOAD FLAGS(6);  /* RW = 6 (read + write) */
    float_entry PT_LOAD FLAGS(6);  /* Separate segment for float lib entry to avoid padding */
}

SECTIONS
{
    . = 0x10000;

    .text : {
        *(.text._start)
        *(.text*)
    } > RAM : text

    .rodata : {
        *(.rodata*)
    } > RAM : data

    .data : {
        *(.data*)
        *(.sdata*)
    } > RAM : data

    .bss : {
        *(.bss*)
        *(.sbss*)
    } > RAM : data

    /* Place float library entry pointer at fixed address to match FLOAT_LIB_ENTRY_PTR constant */
    .float_lib_entry 0x00100000 : {
        KEEP(*(.float_lib_entry))
    } > RAM : float_entry

    /* Place FCSR register at fixed address to match FLOAT_CSR_FCSR constant (0x00208018) */
    .float_csr 0x00208018 : {
        KEEP(*(.float_csr))
    } > RAM : data

    /DISCARD/ : {
        *(.comment)
        *(.riscv.attributes)
    }
}
