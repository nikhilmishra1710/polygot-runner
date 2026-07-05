use std::{
    io::Write,
    os::unix::process::{CommandExt, ExitStatusExt},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::{
    error::WorkerError,
    model::{ExecutionResult, ExecutionStatus, ResourceLimit::Cpu},
    runtime::{
        EventPipeline, Stream, apply_resource_limits, kill_process_group_id, reader::spawn_reader,
    },
};

use super::RuntimeCommand;

pub struct NativeProcessRunner;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

impl NativeProcessRunner {
    pub fn new() -> Self {
        Self
    }

    fn spawn(&self, command: &RuntimeCommand) -> Result<Child, WorkerError> {
        let mut process = Command::new(&command.executable.path);
        process
            .args(&command.args)
            .current_dir(&command.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);

        let limits = command.limits.clone();
        unsafe {
            process.pre_exec(move || {
                let _ = apply_resource_limits(&limits);
                Ok(())
            });
        }
        let mut child = process.spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&command.stdin)?;
        }

        Ok(child)
    }

    pub fn run(&self, command: RuntimeCommand) -> Result<ExecutionResult, WorkerError> {
        let mut child = self.spawn(&command).unwrap();

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| std::io::Error::other("stdout unavailable"))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| std::io::Error::other("stderr unavailable"))?;

        let pipeline = EventPipeline::new();

        let stdout_handle = spawn_reader(stdout, Stream::Stdout, pipeline.sender());

        let stderr_handle = spawn_reader(stderr, Stream::Stderr, pipeline.sender());

        let start = Instant::now();

        let exit_status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }

            if start.elapsed() >= command.limits.wall_time {
                kill_process_group_id(child.id())?;
                child.wait()?;

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
                "Process exited with standard non-zero code: {:?}",
                exit_status.code()
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
