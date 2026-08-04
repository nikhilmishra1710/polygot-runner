use std::{
    fs,
    io,
    path::Path,
};

pub struct ControllerInfo {
    pub unified: bool,
}

impl ControllerInfo {
    pub fn detect() -> io::Result<Self> {
        let controllers =
            fs::read_to_string("/sys/fs/cgroup/cgroup.controllers")?;

        Ok(Self {
            unified: !controllers.trim().is_empty(),
        })
    }

    pub fn ensure_v2(&self) -> io::Result<()> {
        if self.unified {
            Ok(())
        } else {
            Err(io::Error::other(
                "cgroup v2 is not available",
            ))
        }
    }

    pub fn exists() -> bool {
        Path::new("/sys/fs/cgroup").exists()
    }
}