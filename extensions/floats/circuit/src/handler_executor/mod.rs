mod operation;
mod operations;
mod execution;

pub use operation::FloatOperation;
pub use operations::{FmaOp, AluOp, ConvertOp, CompareOp, MoveOp, ClassOp};
pub use execution::FloatHandlerExecutor;
