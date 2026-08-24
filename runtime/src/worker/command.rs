use crate::job::ExecutionJob;

pub enum WorkerCommand {
    Execute(ExecutionJob),
    Shutdown,
}
