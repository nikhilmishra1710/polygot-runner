use crate::{
    error::WorkerError,
    model::{ExecutionPlan, ExecutionRequest},
    toolchain::Executable,
    workspace::Workspace,
};

pub trait LanguageRuntime: Send + Sync {
    fn new(tool: &Executable) -> Self
    where
        Self: Sized;
    fn prepare(
        &self,
        request: &ExecutionRequest,
        workspace: &Workspace,
    ) -> Result<ExecutionPlan, WorkerError>;
}
