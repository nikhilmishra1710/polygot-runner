use crate::model::ExecutionReport;

use super::JobId;

#[derive(Debug)]
pub struct JobResult {
    pub id: JobId,
    pub report: ExecutionReport,
}
