# Float CSR initialization
# This file ensures the FCSR register location is present in the ELF
# so TracingMemory can properly handle accesses to it

.section .float_csr, "aw", @progbits
.align 4
.global _float_csr_fcsr
_float_csr_fcsr:
    .word 0x00000000  # FCSR initialized to 0 (rounding mode = RNE)
