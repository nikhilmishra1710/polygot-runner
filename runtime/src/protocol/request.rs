use serde::{Deserialize, Serialize};

use crate::job::ExecutionJob;

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkerRequest {
    Execute(ExecutionJob),
    Shutdown,
}