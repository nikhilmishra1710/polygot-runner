use std::{
    io::{self, Write},
    process::{Command, Stdio},
};

use crate::model::{ExecutionResult, ExecutionStatus};

use super::RuntimeCommand;

pub struct NativeProcessRunner;

impl NativeProcessRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, command: RuntimeCommand) -> io::Result<ExecutionResult> {
        let mut child = Command::new(&command.executable.path)
            .args(&command.args)
            .current_dir(&command.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(&command.stdin)?;
        }

        let output = child.wait_with_output()?;

        Ok(ExecutionResult {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.status.code(),
            status: if output.status.success() {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::RuntimeError
            },
        })
    }
}
