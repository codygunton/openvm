use openvm_instructions::LocalOpcode;
use openvm_instructions_derive::LocalOpcode;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, FromRepr};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
         EnumCount, EnumIter, FromRepr, LocalOpcode, Serialize, Deserialize)]
#[opcode_offset = 0x300]
#[repr(usize)]
pub enum FloatOpcode {
    FLW = 0,     // Float Load Word (0x300)
    FSW = 1,     // Float Store Word (0x301)
    FADD = 2,    // Float Add (0x302)
    FSUB = 3,    // Float Subtract (0x303)
    FMUL = 4,    // Float Multiply (0x304)
    FDIV = 5,    // Float Divide (0x305)
    FSQRT = 6,   // Float Square Root (0x306)
    FMINMAX = 7, // Float Min/Max (0x307)
    FSGNJ = 8,   // Float Sign Injection (0x308)
    FMADD = 9,   // Fused Multiply-Add (0x309)
    FMSUB = 10,  // Fused Multiply-Sub (0x30A)
    FNMSUB = 11, // Fused Negated Multiply-Sub (0x30B)
    FNMADD = 12, // Fused Negated Multiply-Add (0x30C)
    FCVTWS = 0xD,  // Float Convert to Word/Unsigned (0x30D) - FCVT.W.S, FCVT.WU.S
    FCVTSW = 0xE,  // Float Convert from Word/Unsigned (0x30E) - FCVT.S.W, FCVT.S.WU
    FCMP = 0x0F, // Float Compare - FEQ.S, FLT.S, FLE.S (0x30F)
    FMVXW = 0x10, // Float Move to Integer (0x310)
    FMVWX = 0x11, // Float Move from Integer (0x311)
    FCLASS = 0x12, // Float Classify (0x312)
    FCSR = 0x13, // Float CSR Access - FRCSR, FSCSR (0x313)
    FLOAT_RETURN = 0x14, // Float Handler Return (0x314) - Restores caller-saved registers
}
