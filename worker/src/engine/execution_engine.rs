use std::io::Error;

use crate::{
    error::WorkerError,
    language::RuntimeRegistry,
    model::{ExecutionRequest, ExecutionResult},
    runtime::NativeProcessRunner,
    workspace::WorkspaceManager,
};

pub struct ExecutionEngine {
    workspace_manager: WorkspaceManager,
    process_runner: NativeProcessRunner,
    runtime_registry: RuntimeRegistry,
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self {
            workspace_manager: WorkspaceManager::new(),
            process_runner: NativeProcessRunner::new(),
            runtime_registry: RuntimeRegistry::new(),
        }
    }

    pub fn execute(&self, request: &ExecutionRequest) -> Result<ExecutionResult, WorkerError> {
        let workspace = self.workspace_manager.create(request)?;

        let runtime = self.runtime_registry.get(request.language);

        let plan = runtime.prepare(request, &workspace).unwrap();

        if let Some(complie) = plan.compile {
            let result = self.process_runner.run(complie);
            let Ok(_) = result else {
                eprintln!("Some error occured");
                return Err(WorkerError::Io(Error::new(
                    std::io::ErrorKind::Other,
                    "Some error occured in compilation",
                )));
            };
            println!("compilation completed!!");
        }
        let result = self.process_runner.run(plan.execute)?;

        Ok(result)
    }
}
