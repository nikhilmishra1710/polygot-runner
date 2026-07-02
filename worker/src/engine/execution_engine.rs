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

    pub fn execute(
        &self,
        request: &ExecutionRequest,
    ) -> Result<ExecutionResult, WorkerError> {
        let workspace = self.workspace_manager.create(request)?;

        let runtime = self.runtime_registry.get(request.language);

        let command = runtime.build_command(request, &workspace);

        let result = self.process_runner.run(command)?;

        Ok(result)
    }
}