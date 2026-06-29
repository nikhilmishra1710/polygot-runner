#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    RuntimeError,
    InternalError,
}