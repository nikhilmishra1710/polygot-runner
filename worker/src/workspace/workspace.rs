use std::path::Path;

use crate::sandbox::Sandbox;

pub struct Workspace {
    sandbox: Sandbox,
}

impl Workspace {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Workspace {
            sandbox: Sandbox::new().unwrap(),
        })
    }

    pub fn path(&self) -> &Path {
        self.sandbox.workspace()
    }

    pub fn sandbox(&self) -> &Sandbox {
        &self.sandbox
    }
}
