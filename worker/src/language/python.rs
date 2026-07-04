use crate::{
    error::WorkerError,
    model::{ExecutionPlan, ExecutionRequest},
    runtime::RuntimeCommand,
    toolchain::Executable,
    workspace::Workspace,
};

use super::LanguageRuntime;

pub struct PythonRuntime {
    python: Executable,
}

impl LanguageRuntime for PythonRuntime {
    fn new(python: &Executable) -> Self {
        PythonRuntime {
            python: python.clone(),
        }
    }

    fn prepare(
        &self,
        request: &ExecutionRequest,
        workspace: &Workspace,
    ) -> Result<ExecutionPlan, WorkerError> {
        Ok(ExecutionPlan {
            compile: None,
            execute: RuntimeCommand {
                executable: self.python.clone(),
                args: vec!["main.py".to_string()],
                working_directory: workspace.path().to_path_buf(),
                stdin: request.stdin.clone(),
                wall_time: request.limits.wall_time
            },
        })
    }
}
