use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("tool not found: {0}")]
    ToolNotFound(#[from] which::Error),
}
