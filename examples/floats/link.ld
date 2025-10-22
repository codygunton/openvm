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

    /* Place float library entry pointer at fixed high address to avoid conflicts */
    . = 0x80000;
    .float_lib_entry : {
        KEEP(*(.float_lib_entry))
    } > RAM : data

    .data : {
        *(.data*)
        *(.sdata*)
    } > RAM : data

    .bss : {
        *(.bss*)
        *(.sbss*)
    } > RAM : data

    /DISCARD/ : {
        *(.comment)
        *(.riscv.attributes)
    }
}
