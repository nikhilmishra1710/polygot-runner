use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminationReason {
    /// Normal process exit with a return code (0 for success, non-zero for failure).
    ExitCode(i32),
    /// Process was terminated by an unhandled signal (e.g., SIGSEGV = 11).
    Signal(i32),
    /// Wall clock execution time limit breached.
    WallTimeout,
    /// Process hit its allocated CPU execution time limit (SIGXCPU).
    CpuLimit,
    /// Cgroup memory ceiling breached (OOM killer triggered).
    MemoryLimit,
    /// Kernel Out-Of-Memory killer fired in cgroup.
    OomKilled,
    /// Process attempted a restricted syscall caught by seccomp (SIGSYS).
    SeccompViolation,
}

impl TerminationReason {
    pub fn is_success(&self) -> bool {
        matches!(self, TerminationReason::ExitCode(0))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Output {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Total real-world elapsed time.
    pub wall_time: Duration,
    /// Total CPU time used (user + system space).
    pub cpu_time: Duration,
    /// Peak resident set size (memory) in bytes.
    pub max_rss_bytes: u64,
    /// Peak process/thread count during execution.
    pub peak_pids: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub output: Output,
    pub termination: TerminationReason,
    pub metrics: ExecutionMetrics,
}
