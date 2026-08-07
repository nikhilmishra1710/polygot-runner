use super::ExecutionStatus;

#[derive(Debug)]
pub struct ExecutionResult {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: Option<i32>,
    pub status: ExecutionStatus,
}
