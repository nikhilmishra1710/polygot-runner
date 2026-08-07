use tracing::{error, info};

use crate::{
    error::WorkerError,
    language::RuntimeRegistry,
    model::{ExecutionReport, ExecutionRequest},
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

    pub fn execute(&self, request: &ExecutionRequest) -> Result<ExecutionReport, WorkerError> {
        let workspace = self.workspace_manager.create(request)?;

        let runtime = self.runtime_registry.get(request.language);

        let plan = runtime.prepare(request, &workspace).unwrap();

        if let Some(complie) = plan.compile {
            let result = self.process_runner.run(complie)?;
            if !result.termination.is_success() {
                error!("Some error occured");
                return Ok(result);
            }
            info!("compilation completed!!");
        }
        let result = self.process_runner.run(plan.execute)?;

        Ok(result)
    }
}
