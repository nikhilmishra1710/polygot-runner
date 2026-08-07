use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    RuntimeError,
    TimeLimitExceeded,
    CompilationError,
    InternalError,
    SeccompViolation,
    MemoryLimitExceeded,
    CpuLimitExcedded,
}

impl fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionStatus::Success => write!(f, "Success"),
            ExecutionStatus::RuntimeError => write!(f, "Runtime Error"),
            ExecutionStatus::TimeLimitExceeded => write!(f, "Time Limit Exceeded"),
            ExecutionStatus::CompilationError => write!(f, "Compilation Error"),
            ExecutionStatus::InternalError => write!(f, "Internal Error"),
            ExecutionStatus::SeccompViolation => write!(f, "Seccomp Violation"),
            ExecutionStatus::MemoryLimitExceeded => write!(f, "Memory Limit Violation"),
            ExecutionStatus::CpuLimitExcedded => write!(f, "CPU Limit error"),
        }
    }
}
