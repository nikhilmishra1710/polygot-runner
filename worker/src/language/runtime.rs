use crate::{
    error::WorkerError,
    model::{ExecutionPlan, ExecutionRequest},
    workspace::Workspace,
};

pub trait LanguageRuntime: Send + Sync {
    fn prepare(
        &self,
        request: &ExecutionRequest,
        workspace: &Workspace,
    ) -> Result<ExecutionPlan, WorkerError>;
}
