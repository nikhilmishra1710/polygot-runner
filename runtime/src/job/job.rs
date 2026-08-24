use crate::{job::JobId, model::ExecutionRequest};

#[derive(Debug, Clone)]
pub struct ExecutionJob {
    pub id: JobId,
    pub request: ExecutionRequest,
}
