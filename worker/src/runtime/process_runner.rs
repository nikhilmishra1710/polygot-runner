use std::{
    io::{BufReader, Read, Write},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::{
    error::WorkerError, model::{ExecutionResult, ExecutionStatus}, runtime::{ProcessRunner, reader::{Stream, spawn_reader}},
};

use super::RuntimeCommand;

pub struct NativeProcessRunner;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

impl NativeProcessRunner {
    pub fn new() -> Self {
        Self
    }

    fn spawn(&self, command: &RuntimeCommand) -> Result<Child, WorkerError> {
        let mut child = Command::new(&command.executable.path)
            .args(&command.args)
            .current_dir(&command.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

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

        let stdout_handle = spawn_reader(stdout, Stream::Stdout);
        let stderr_handle = spawn_reader(stderr, Stream::Stderr);

        let start = Instant::now();

        let exit_status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }

            if start.elapsed() >= command.wall_time {
                child.kill()?;
                child.wait()?;

                return Ok(ExecutionResult {
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                    exit_code: None,
                    status: ExecutionStatus::TimeLimitExceeded,
                });
            }

            thread::sleep(POLL_INTERVAL);
        };

        let stdout_consumer = stdout_handle
            .join()
            .map_err(|_| std::io::Error::other("stdout reader panicked"))??;

        let stderr_consumer = stderr_handle
            .join()
            .map_err(|_| std::io::Error::other("stderr reader panicked"))??;

        let (stdout, _) = stdout_consumer.into_output();
        let (_, stderr) = stderr_consumer.into_output();

        Ok(ExecutionResult {
            stdout: stdout,
            stderr: stderr,
            exit_code: exit_status.code(),
            status: if exit_status.success() {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::RuntimeError
            },
        })
    }
}
