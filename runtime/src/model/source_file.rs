use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    // Relative path inside the workspace
    pub path: PathBuf,
    // File contents.
    pub contents: Vec<u8>,
}
