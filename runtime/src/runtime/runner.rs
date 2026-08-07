use crate::{error::WorkerError, model::ExecutionResult, runtime::RuntimeCommand};

pub trait ProcessRunner {
    fn new() -> Self
    where
        Self: Sized;
    fn run(&self, command: RuntimeCommand) -> Result<ExecutionResult, WorkerError>;
}
