use std::{
    os::unix::process::ExitStatusExt,
    thread,
    time::{Duration, Instant},
};

use crate::{
    error::WorkerError, model::{ExecutionResult, ExecutionStatus, ResourceLimit::Cpu}, runtime::{
        EventPipeline, ForkBackend, ProcessLauncher, Stream, kill_process_group_id, reader::spawn_reader, std_backend::StdProcessBackend,
    },
};

use super::RuntimeCommand;

pub struct NativeProcessRunner;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

impl NativeProcessRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, command: RuntimeCommand) -> Result<ExecutionResult, WorkerError> {
        let launcher = ProcessLauncher::new(ForkBackend);

        let mut process = launcher.launch(&command)?;

        let stdout = process
            .stdout()
            .ok_or_else(|| std::io::Error::other("stdout unavailable"))?;

        let stderr = process
            .stderr()
            .ok_or_else(|| std::io::Error::other("stderr unavailable"))?;

        let pipeline = EventPipeline::new();

        let stdout_handle = spawn_reader(stdout, Stream::Stdout, pipeline.sender());

        let stderr_handle = spawn_reader(stderr, Stream::Stderr, pipeline.sender());

        let start = Instant::now();

        let exit_status = loop {
            if let Some(status) = process.try_wait()? {
                break status;
            }

            if start.elapsed() >= command.limits.wall_time {
                kill_process_group_id(process.pid() as u32)?;
                process.wait()?;

                stdout_handle
                    .join()
                    .map_err(|_| std::io::Error::other("stdout reader panicked"))??;

                stderr_handle
                    .join()
                    .map_err(|_| std::io::Error::other("stderr reader panicked"))??;

                let (stdout, stderr) = pipeline.finish()?.into_output();

                return Ok(ExecutionResult {
                    stdout: stdout,
                    stderr: stderr,
                    exit_code: None,
                    status: ExecutionStatus::TimeLimitExceeded,
                });
            }

            thread::sleep(POLL_INTERVAL);
        };

        stdout_handle
            .join()
            .map_err(|_| std::io::Error::other("stdout reader panicked"))??;

        stderr_handle
            .join()
            .map_err(|_| std::io::Error::other("stderr reader panicked"))??;

        let (stdout, stderr) = pipeline.finish()?.into_output();

        let status = if exit_status.success() {
            ExecutionStatus::Success
        } else if let Some(signal) = exit_status.signal() {
            match signal {
                // Process exceeded its CPU time limit
                libc::SIGXCPU => {
                    println!("Process terminated by SIGXCPU (CPU time limit exceeded)");
                    // Note: Assuming `Cpu` is imported/available in this scope based on your snippet
                    ExecutionStatus::ResourceLimitExceeded(Cpu)
                }

                // Process was forcefully killed (OOM killer, manual kill, etc.)
                libc::SIGKILL => {
                    println!("Process terminated by SIGKILL (Forcefully killed)");
                    ExecutionStatus::RuntimeError
                }

                // Catch any other signal and log its integer value
                sig => {
                    println!("Process terminated by unhandled Unix signal: {}", sig);
                    ExecutionStatus::RuntimeError
                }
            }
        } else {
            // Standard non-zero exit code
            println!(
                "Process exited with standard non-zero code: {:?} {:?} {:?}",
                exit_status.code(),
                stderr,
                stdout
            );
            ExecutionStatus::RuntimeError
        };

        Ok(ExecutionResult {
            stdout: stdout,
            stderr: stderr,
            exit_code: exit_status.code(),
            status: status,
        })
    }
}
