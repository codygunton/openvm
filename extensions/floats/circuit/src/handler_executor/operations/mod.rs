mod fma;
mod alu;
mod convert;
mod compare;
mod r#move;  // 'move' is a keyword
mod class;

pub use fma::FmaOp;
pub use alu::AluOp;
pub use convert::ConvertOp;
pub use compare::CompareOp;
pub use r#move::MoveOp;
pub use class::ClassOp;
