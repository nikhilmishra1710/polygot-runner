use crate::job::ExecutionJob;

#[derive(Clone, Debug)]
pub enum WorkerCommand {
    Execute(ExecutionJob),
    Shutdown,
}
