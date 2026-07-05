use std::{
    io::{self},
    os::unix::process::CommandExt,
    process::{Command, Stdio},
};

use crate::{
    error::WorkerError,
    model::ResourceLimits,
    runtime::{RunningProcess, backend::ProcessBackend},
    sandbox::NamespaceManager,
};

use super::{RuntimeCommand, apply_resource_limits};

pub struct ProcessLauncher;

fn configure_child(limits: &ResourceLimits) -> io::Result<()> {
    NamespaceManager::setup().map_err(io::Error::other)?;

    apply_resource_limits(limits)?;

    Ok(())
}

pub struct StdProcessBackend;

impl ProcessBackend for StdProcessBackend {
    fn launch(&self, command: &RuntimeCommand) -> Result<RunningProcess, WorkerError> {
        let mut process = Command::new(&command.executable.path);

        process
            .args(&command.args)
            .current_dir(&command.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);

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

        let mut process = RunningProcess::new(child);

        process.write_stdin(&command.stdin)?;

        Ok(process)
    }
}
