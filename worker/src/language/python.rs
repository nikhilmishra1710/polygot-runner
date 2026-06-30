use std::path::PathBuf;

use crate::{
    model::ExecutionRequest,
    runtime::{Executable, RuntimeCommand},
    workspace::Workspace,
};

use super::LanguageRuntime;

pub struct PythonRuntime;

impl LanguageRuntime for PythonRuntime {
    fn build_command(&self, request: &ExecutionRequest, workspace: &Workspace) -> RuntimeCommand {
        RuntimeCommand {
            executable: Executable {
                path: PathBuf::from("python3"),
            },
            args: vec!["main.py".to_string()],
            working_directory: workspace.path().to_path_buf(),
            stdin: request.stdin.clone(),
        }
    }
}
