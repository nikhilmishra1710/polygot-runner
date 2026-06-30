use crate::{model::ExecutionRequest, runtime::RuntimeCommand, workspace::Workspace};

pub trait LanguageRuntime {
    fn build_command(&self, request: &ExecutionRequest, workspace: &Workspace) -> RuntimeCommand;
}
