/// Shared utilities for float adapters

use crate::constants::*;

// Re-export from RISC-V instructions for use in AIR implementations
pub use openvm_instructions::riscv::RV32_REGISTER_NUM_LIMBS;

/// Calculate the memory address for a float register
#[inline]
pub fn float_register_address(reg: u8) -> u32 {
    float_reg_addr(reg)
}

/// Sign extension for 12-bit immediate values
/// Takes a 12-bit value and sign-extends it to 32 bits
#[inline]
pub fn sign_extend_12bit(imm: u16) -> i32 {
    // Extract sign bit (bit 11)
    let sign_bit = (imm >> 11) & 1;
    if sign_bit == 1 {
        // Negative: set upper 20 bits to 1
        (imm as i32) | !0xFFF
    } else {
        // Positive: upper bits already 0
        imm as i32
    }
}

/// Calculate address with overflow detection
/// Returns (result, overflow_occurred)
#[inline]
pub fn calculate_address_with_overflow(base: u32, offset: i32) -> (u32, bool) {
    base.overflowing_add(offset as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_extend_positive() {
        assert_eq!(sign_extend_12bit(0x7FF), 0x7FF);  // Max positive 12-bit
        assert_eq!(sign_extend_12bit(0x000), 0);      // Zero
        assert_eq!(sign_extend_12bit(0x100), 0x100);  // Mid positive
    }

    #[test]
    fn test_sign_extend_negative() {
        assert_eq!(sign_extend_12bit(0x800), -2048);  // -2048 (min negative)
        assert_eq!(sign_extend_12bit(0xFFF), -1);     // -1
        assert_eq!(sign_extend_12bit(0xF00), -256);   // -256
    }

    #[test]
    fn test_address_overflow() {
        let (addr, overflow) = calculate_address_with_overflow(0xFFFF_FFFF, 1);
        assert_eq!(addr, 0);
        assert!(overflow);

        let (addr, overflow) = calculate_address_with_overflow(0x1000, 0x100);
        assert_eq!(addr, 0x1100);
        assert!(!overflow);
    }
}
