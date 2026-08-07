use crate::{
    cgroup::ExecutionCgroup, error::WorkerError, model::ExecutionPlan, runtime::RuntimeCommand,
};

use super::process::RunningProcess;

pub trait ProcessBackend {
    fn launch(
        &self,
        plan: &RuntimeCommand,
        cgroup: &ExecutionCgroup,
    ) -> Result<RunningProcess, WorkerError>;
}
