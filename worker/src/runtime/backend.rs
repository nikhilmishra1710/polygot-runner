use crate::{cgroup::ExecutionCgroup, error::WorkerError, runtime::RuntimeCommand};

use super::process::RunningProcess;

pub trait ProcessBackend {
    fn launch(&self, command: &RuntimeCommand, cgroup: &ExecutionCgroup) -> Result<RunningProcess, WorkerError>;
}
