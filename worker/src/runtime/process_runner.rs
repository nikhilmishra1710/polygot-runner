use std::{
    io::Write,
    os::unix::process::CommandExt,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::{
    error::WorkerError,
    model::{ExecutionResult, ExecutionStatus},
    runtime::{
        EventPipeline, Stream, create_process_group, kill_process_group_id, reader::spawn_reader,
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
        let mut child = Command::new(&command.executable.path)
            .args(&command.args)
            .current_dir(&command.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
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

        let pipeline = EventPipeline::new();

        let stdout_handle = spawn_reader(stdout, Stream::Stdout, pipeline.sender());

        let stderr_handle = spawn_reader(stderr, Stream::Stderr, pipeline.sender());

        let start = Instant::now();

        let exit_status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }

            if start.elapsed() >= command.wall_time {
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
