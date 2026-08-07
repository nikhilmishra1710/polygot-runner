use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SourceFile {
    // Relative path inside the workspace
    pub path: PathBuf,
    // File contents.
    pub contents: Vec<u8>,
}
