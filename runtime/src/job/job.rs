use crate::{job::JobId, model::ExecutionRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionJob {
    pub id: JobId,
    pub request: ExecutionRequest,
}
