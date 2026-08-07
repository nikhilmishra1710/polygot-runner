use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::error::WorkerError;

pub struct Sandbox {
    root: TempDir,
    workspace: PathBuf,
}

impl Sandbox {
    pub fn new() -> Result<Self, WorkerError> {
        let root = tempfile::tempdir()?;

        let workspace = root.path().join("workspace");

        std::fs::create_dir(&workspace)?;

        Ok(Self { root, workspace })
    }

    pub fn root(&self) -> &Path {
        self.root.path()
    }

    pub fn workspace(&self) -> &Path {
        &self.workspace
    }
}
