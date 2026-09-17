use serde::{Deserialize, Serialize};

use crate::model::ExecutionReport;

use super::JobId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobResult {
    pub id: JobId,
    pub report: ExecutionReport,
}
