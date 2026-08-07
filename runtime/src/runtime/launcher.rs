use crate::{
    cgroup::ExecutionCgroup,
    error::WorkerError,
    runtime::{RunningProcess, backend::ProcessBackend},
};

use super::RuntimeCommand;

pub struct ProcessLauncher<B> {
    backend: B,
}

impl<B: ProcessBackend> ProcessLauncher<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn launch(
        &self,
        command: &RuntimeCommand,
        cgroup: &ExecutionCgroup,
    ) -> Result<RunningProcess, WorkerError> {
        self.backend.launch(command, cgroup)
    }
}
