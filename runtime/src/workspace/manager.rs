use std::{fs, io};

use crate::model::ExecutionRequest;

use super::Workspace;

pub struct WorkspaceManager;

impl WorkspaceManager {
    pub fn new() -> Self {
        Self
    }

    pub fn create(&self, request: &ExecutionRequest) -> io::Result<Workspace> {
        let workspace = Workspace::new()?;

        for file in &request.files {
            let path = workspace.path().join(&file.path);

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(path, &file.contents)?;
        }

        Ok(workspace)
    }
}
