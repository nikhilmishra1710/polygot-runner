use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Executable {
    pub path: PathBuf,
}

impl Executable {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}