use serde::{Deserialize, Serialize};

use crate::job::JobResult;

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkerResponse {
    Result(JobResult),
    Error(String),
}