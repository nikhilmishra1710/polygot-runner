use std::path::PathBuf;

use crate::{
    error::WorkerError,
    model::{ExecutionPlan, ExecutionRequest},
    runtime::{RuntimeCommand},
    toolchain::Executable,
    workspace::Workspace,
};

use super::LanguageRuntime;

pub struct CppRuntime;

impl LanguageRuntime for CppRuntime {
    fn prepare(
        &self,
        request: &ExecutionRequest,
        workspace: &Workspace,
    ) -> Result<ExecutionPlan, WorkerError> {
        Ok(ExecutionPlan {
            compile: Some(RuntimeCommand {
                executable: Executable {
                    path: PathBuf::from("g++"),
                },
                args: vec!["main.cpp".to_string()],
                working_directory: workspace.path().to_path_buf(),
                stdin: Vec::new(),
            }),
            execute: RuntimeCommand {
                executable: Executable {
                    path: PathBuf::from("./a.out"),
                },
                args: vec![],
                working_directory: workspace.path().to_path_buf(),
                stdin: request.stdin.clone(),
            },
        })
    }
}
