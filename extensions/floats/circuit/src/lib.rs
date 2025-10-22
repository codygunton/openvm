//! The RISC-V F extension
//!
//! **Note**: This extension does NOT implement custom ZK circuits.
//! Instead, it uses software emulation - the portable float library
//! (lib-float from Zisk) implements IEEE 754 operations using
//! standard rv32im instructions.
//!

pub mod extension;

// Re-export for consistency with extension pattern
pub use extension::FloatsExtension;
