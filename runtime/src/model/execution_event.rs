use serde::{Deserialize, Serialize};

use crate::{
    job::JobResult,
    model::{ExecutionReport, TerminationReason},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    Started,
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Finished { result: JobResult },
}
