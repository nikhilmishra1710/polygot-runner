use crate::{
    cgroup::{ExecutionCgroup, read_cgroup_metrics},
    error::WorkerError,
    model::{
        ExecutionMetrics, ExecutionReport, ExecutionResult, ExecutionStatus, Output,
        TerminationReason,
    },
    runtime::{
        EventPipeline, ForkBackend, ProcessLauncher, Stream, kill_process_group_id,
        reader::spawn_reader, std_backend::StdProcessBackend,
    },
};
use std::os::unix::process::ExitStatusExt;
use std::{
    thread,
    time::{Duration, Instant},
};
use tracing::{debug, info, trace, warn};

use super::RuntimeCommand;

use std::fs;
use std::path::Path;

pub fn is_cgroup_oom(cgroup_path: &Path) -> bool {
    let events_path = cgroup_path.join("memory.events");
    if let Ok(content) = fs::read_to_string(events_path) {
        for line in content.lines() {
            // Line format: "oom_kill 1"
            if line.starts_with("oom_kill") {
                if let Some(count_str) = line.split_whitespace().nth(1) {
                    if let Ok(count) = count_str.parse::<u64>() {
                        return count > 0;
                    }
                }
            }
        }
    }
    false
}

pub struct NativeProcessRunner;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

impl NativeProcessRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, command: RuntimeCommand) -> Result<ExecutionReport, WorkerError> {
        let launcher = ProcessLauncher::new(ForkBackend);

        let cgroup = ExecutionCgroup::create()?;
        cgroup.set_memory_limit(command.limits.memory_bytes)?;
        cgroup.set_pid_limit(command.limits.pids_max)?;

        let mut process = launcher.launch(&command, &cgroup)?;

        let stdout = process
            .stdout()
            .ok_or_else(|| std::io::Error::other("stdout unavailable"))?;

        let stderr = process
            .stderr()
            .ok_or_else(|| std::io::Error::other("stderr unavailable"))?;

        let pipeline = EventPipeline::new();

        let stdout_handle = spawn_reader(stdout, Stream::Stdout, pipeline.sender());

        let stderr_handle = spawn_reader(stderr, Stream::Stderr, pipeline.sender());

        let start_time = Instant::now();
        let mut wall_timeout_occurred = false;

        let exit_status = loop {
            if let Some(status) = process.try_wait()? {
                break status;
            }

            if start_time.elapsed() >= command.limits.wall_time {
                warn!("Execution wall time limit reached; forcefully killing process group");
                wall_timeout_occurred = true;
                let _ = kill_process_group_id(process.pid() as u32);
                break process.wait()?;
            }

            thread::sleep(POLL_INTERVAL);
        };

        let total_wall_time = start_time.elapsed();

        // Drain standard streams
        let _ = stdout_handle.join();
        let _ = stderr_handle.join();
        let (stdout_data, stderr_data) = pipeline.finish()?.into_output();

        // Step 3: Collect cgroup metrics before destroying cgroup
        let (max_rss_bytes, peak_pids, cpu_time) = read_cgroup_metrics(&cgroup.path());
        let is_oom = is_cgroup_oom(&cgroup.path());

        // Step 1: Classify termination reason
        let termination = if wall_timeout_occurred {
            TerminationReason::WallTimeout
        } else if is_oom {
            TerminationReason::OomKilled
        } else if let Some(sig) = exit_status.signal() {
            match sig {
                libc::SIGXCPU => TerminationReason::CpuLimit,
                libc::SIGSYS => TerminationReason::SeccompViolation,
                libc::SIGKILL if is_oom => TerminationReason::MemoryLimit,
                _ => TerminationReason::Signal(sig),
            }
        } else if let Some(code) = exit_status.code() {
            match code - 128 {
                libc::SIGXCPU => TerminationReason::CpuLimit,
                libc::SIGSYS => TerminationReason::SeccompViolation,
                _ => TerminationReason::ExitCode(code),
            }
        } else {
            TerminationReason::ExitCode(1)
        };

        info!(
            reason = ?termination,
            wall_time_ms = total_wall_time.as_millis(),
            max_rss_bytes = max_rss_bytes,
            "Execution finished"
        );

        Ok(ExecutionReport {
            output: Output {
                stdout: stdout_data,
                stderr: stderr_data,
            },
            termination,
            metrics: ExecutionMetrics {
                wall_time: total_wall_time,
                cpu_time,
                max_rss_bytes,
                peak_pids,
            },
        })
    }
}
