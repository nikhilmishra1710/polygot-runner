use std::path::PathBuf;

use crate::{
    error::WorkerError,
    model::{ExecutionPlan, ExecutionRequest},
    runtime::{Executable, RuntimeCommand},
    workspace::Workspace,
};

use super::LanguageRuntime;

pub struct PythonRuntime;

impl LanguageRuntime for PythonRuntime {
    fn prepare(
        &self,
        request: &ExecutionRequest,
        workspace: &Workspace,
    ) -> Result<ExecutionPlan, WorkerError> {
        Ok(ExecutionPlan {
            compile: None,
            execute: RuntimeCommand {
                executable: Executable {
                    path: PathBuf::from("python3"),
                },
                args: vec!["main.py".to_string()],
                working_directory: workspace.path().to_path_buf(),
                stdin: request.stdin.clone(),
            },
        })
    }
}
