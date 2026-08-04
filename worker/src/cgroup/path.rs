use std::path::{Path, PathBuf};

const CGROUP_ROOT: &str = "/sys/fs/cgroup";
const RUNTIME_DIR: &str = "runtime-worker";

pub struct CgroupPath {
    path: PathBuf,
}

impl CgroupPath {
    pub fn new(id: &str) -> Self {
        Self {
            path: Path::new(CGROUP_ROOT)
                .join(RUNTIME_DIR)
                .join(id),
        }
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }
}