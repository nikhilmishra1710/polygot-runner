use crate::{model::ExecutionRequest, runtime::RuntimeCommand, workspace::Workspace};

pub trait LanguageRuntime: Send + Sync {
    fn build_command(&self, request: &ExecutionRequest, workspace: &Workspace) -> RuntimeCommand;
}
