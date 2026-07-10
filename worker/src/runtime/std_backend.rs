use std::{
    os::unix::process::CommandExt,
    process::{Command, Stdio},
};

use crate::{
    error::WorkerError,
    runtime::{RunningProcess, backend::ProcessBackend, unix::configure_child},
};

use super::RuntimeCommand;

pub struct StdProcessBackend;

impl ProcessBackend for StdProcessBackend {
    fn launch(&self, command: &RuntimeCommand) -> Result<RunningProcess, WorkerError> {
        let mut process = Command::new(&command.executable.path);

        process
            .args(&command.args)
            .current_dir(&command.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let limits = command.limits.clone();

        // SAFETY:
        // Runs after fork() and before exec().
        // Only async-signal-safe operations should happen here.
        unsafe {
            process.pre_exec(move || {
                configure_child(&limits)?;
                Ok(())
            });
        }

        let child = process.spawn()?;

        let mut process = RunningProcess::from_child(child);

        process.write_stdin(&command.stdin)?;

        Ok(process)
    }
}
