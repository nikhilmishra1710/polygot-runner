use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    RuntimeError,
    TimeLimitExceeded,
    ResourceLimitExceeded(ResourceLimit),
    CompilationError,
    InternalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLimit {
    Cpu,
    FileSize,
    ProcessCount,
    OpenFiles,
}

impl fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionStatus::Success => write!(f, "Success"),
            ExecutionStatus::RuntimeError => write!(f, "Runtime Error"),
            ExecutionStatus::TimeLimitExceeded => write!(f, "Time Limit Exceeded"),
            ExecutionStatus::CompilationError => write!(f, "Compilation Error"),
            ExecutionStatus::ResourceLimitExceeded(resource_limit) => {
                write!(f, "ResourceLimitExceeded Error")
            }
            ExecutionStatus::InternalError => write!(f, "Internal Error"),
        }
    }
}
