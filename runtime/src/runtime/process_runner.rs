use crate::{
    cgroup::{ExecutionCgroup, read_cgroup_metrics},
    error::WorkerError,
    model::{ExecutionEvent, ExecutionMetrics, ExecutionReport, Output, TerminationReason},
    runtime::{
        EventPipeline, ForkBackend, ProcessLauncher, Stream, kill_process_group_id,
        reader::spawn_reader, std_backend::StdProcessBackend,
    },
};
use std::{
    io::Read,
    os::unix::process::ExitStatusExt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError, Sender, SyncSender},
    },
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

        info!("cgroup:start");
        let cgroup = ExecutionCgroup::create()?;
        cgroup.set_memory_limit(command.limits.memory_bytes)?;
        cgroup.set_pid_limit(command.limits.pids_max)?;
        info!("cgroup:end");

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

    pub fn run_with_events(
        &self,
        command: RuntimeCommand,
        event_tx: SyncSender<ExecutionEvent>,
        cancel_flag: Arc<AtomicBool>,
    ) -> Result<ExecutionReport, WorkerError> {
        let launcher = ProcessLauncher::new(ForkBackend);

        info!("cgroup:start");
        let cgroup = ExecutionCgroup::create()?;
        cgroup.set_memory_limit(command.limits.memory_bytes)?;
        cgroup.set_pid_limit(command.limits.pids_max)?;
        info!("cgroup:end");

        let mut process = launcher.launch(&command, &cgroup)?;

        let _ = event_tx.send(ExecutionEvent::Started);

        let stdout = process
            .stdout()
            .ok_or_else(|| std::io::Error::other("stdout unavailable"))?;

        let stderr = process
            .stderr()
            .ok_or_else(|| std::io::Error::other("stderr unavailable"))?;

        let (internal_tx, internal_rx) = mpsc::sync_channel::<ExecutionEvent>(128);

        let tx_out = internal_tx.clone();
        let event_tx_out = event_tx.clone();
        let stdout_handle = thread::spawn(move || {
            let mut buf = [0; 4096];
            let mut stdout_stream = stdout;
            while let Ok(n) = stdout_stream.read(&mut buf) {
                if n == 0 {
                    break;
                }
                let chunk = buf[..n].to_vec();
                // 1. Stream out to the client live
                let _ = event_tx_out.send(ExecutionEvent::Stdout(chunk.clone()));
                info!("Streamed output");
                // 2. Buffer for the final ExecutionReport
                let _ = tx_out.send(ExecutionEvent::Stdout(chunk));
            }
        });

        // Spawn Stderr reader
        let tx_err = internal_tx.clone();
        let event_tx_err = event_tx.clone(); // Last use, can consume directly
        let stderr_handle = thread::spawn(move || {
            let mut buf = [0; 4096];
            let mut stderr_stream = stderr;
            while let Ok(n) = stderr_stream.read(&mut buf) {
                if n == 0 {
                    break;
                }
                let chunk = buf[..n].to_vec();
                let _ = event_tx_err.send(ExecutionEvent::Stderr(chunk.clone()));
                let _ = tx_err.send(ExecutionEvent::Stderr(chunk));
            }
        });

        // Drop the original internal sender so the receiver stops waiting when threads finish
        drop(internal_tx);

        let mut stdout_data = Vec::new();
        let mut stderr_data = Vec::new();

        let start_time = Instant::now();
        let mut wall_timeout_occurred = false;
        let mut client_cancelled = false;

        let exit_status = loop {
            if let Some(status) = process.try_wait()? {
                break status;
            }

            if cancel_flag.load(Ordering::Relaxed) {
                warn!("Client disconnected; forcefully killing process group");
                client_cancelled = true;
                let _ = kill_process_group_id(process.pid() as u32);

                // Block briefly for the kernel to reap the killed process
                break process.wait()?;
            }

            if start_time.elapsed() >= command.limits.wall_time {
                warn!("Execution wall time limit reached; forcefully killing process group");
                wall_timeout_occurred = true;
                let _ = kill_process_group_id(process.pid() as u32);
                break process.wait()?;
            }

            match internal_rx.recv_timeout(POLL_INTERVAL) {
                Ok(ExecutionEvent::Stdout(chunk)) => stdout_data.extend_from_slice(&chunk),
                Ok(ExecutionEvent::Stderr(chunk)) => stderr_data.extend_from_slice(&chunk),
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => {
                    // Normal behavior; continue the poll loop
                }
                Err(RecvTimeoutError::Disconnected) => {
                    // Readers finished early, fallback to sleeping while we wait for process exit
                    thread::sleep(POLL_INTERVAL);
                }
            }
        };

        let total_wall_time = start_time.elapsed();

        while let Ok(event) = internal_rx.recv() {
            match event {
                ExecutionEvent::Stdout(chunk) => stdout_data.extend_from_slice(&chunk),
                ExecutionEvent::Stderr(chunk) => stderr_data.extend_from_slice(&chunk),
                _ => {}
            }
        }

        // Drain standard streams
        let _ = stdout_handle.join();
        let _ = stderr_handle.join();

        // Step 3: Collect cgroup metrics before destroying cgroup
        let (max_rss_bytes, peak_pids, cpu_time) = read_cgroup_metrics(&cgroup.path());
        let is_oom = is_cgroup_oom(&cgroup.path());

        // Step 1: Classify termination reason
        let termination = if client_cancelled {
            TerminationReason::Signal(libc::SIGKILL)
        } else if wall_timeout_occurred {
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
