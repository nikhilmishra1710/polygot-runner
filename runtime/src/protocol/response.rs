use serde::{Deserialize, Serialize};

use crate::{job::JobResult, model::ExecutionEvent};

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkerResponse {
    Event(ExecutionEvent),
    Result(JobResult),
    Error(String),
}