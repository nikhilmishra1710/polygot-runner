use std::{
    io::{BufReader, Read, Write},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::{
    error::WorkerError,
    model::{ExecutionResult, ExecutionStatus},
    runtime::ProcessRunner,
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
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "stdout unavailable"))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "stderr unavailable"))?;

        let stdout_handle = thread::spawn(move || -> std::io::Result<Vec<u8>> {
            let mut buffer = Vec::new();
            BufReader::new(stdout).read_to_end(&mut buffer)?;
            Ok(buffer)
        });

        let stderr_handle = thread::spawn(move || -> std::io::Result<Vec<u8>> {
            let mut buffer = Vec::new();
            BufReader::new(stderr).read_to_end(&mut buffer)?;
            Ok(buffer)
        });

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

        let stdout = stdout_handle
            .join()
            .map_err(|_| std::io::Error::other("stdout reader thread panicked"))??;

        let stderr = stderr_handle
            .join()
            .map_err(|_| std::io::Error::other("stderr reader thread panicked"))??;

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
