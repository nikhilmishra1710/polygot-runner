use std::path::{Path, PathBuf};

use tempfile::TempDir;

pub struct Workspace {
    root: TempDir,
}

impl Workspace {

    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Workspace {
            root: tempfile::tempdir()?,
        })
    }
    
    pub fn path(&self) -> &Path {
        self.root.path()
    }

    pub fn into_path(self) -> PathBuf {
        self.root.keep()
    }
}
