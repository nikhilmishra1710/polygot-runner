#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEvent {
    pub stream: Stream,
    pub bytes: Vec<u8>,
}

impl ExecutionEvent {
    pub fn stdout(bytes: Vec<u8>) -> Self {
        Self {
            stream: Stream::Stdout,
            bytes,
        }
    }

    pub fn stderr(bytes: Vec<u8>) -> Self {
        Self {
            stream: Stream::Stderr,
            bytes,
        }
    }
}
