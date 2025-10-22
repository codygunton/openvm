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
}
