use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct ExecutionCgroup {
    path: PathBuf,
}

impl ExecutionCgroup {
    pub fn create() -> io::Result<Self> {
        // Ensure cgroups v2 is available here if needed
        let id = Uuid::new_v4().to_string();

        // Ensure this maps to your absolute cgroup mount path
        let path = PathBuf::from(format!("/sys/fs/cgroup/{}", id));
        fs::create_dir_all(&path)?;

        Ok(Self { path })
    }

    // Reconstructs the wrapper from a path so the child can use it
    pub fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn attach(&self, pid: i32) -> io::Result<()> {
        std::fs::write(self.procs_file(), pid.to_string())
    }

    pub fn set_memory_limit(&self, bytes: u64) -> io::Result<()> {
        // memory.max sets the hard limit for memory usage in bytes.
        // If the process exceeds this, the kernel's OOM killer will terminate it.
        std::fs::write(self.path.join("memory.max"), bytes.to_string())
    }

    pub fn set_pid_limit(&self, limit: u32) -> io::Result<()> {
        // pids.max prevents fork bombs by limiting the total number of
        // processes/threads allowed inside this cgroup.
        std::fs::write(self.path.join("pids.max"), limit.to_string())
    }

    pub fn set_cpu_limit(&self, quota_us: u64, period_us: u64) -> io::Result<()> {
        // cpu.max takes two space-separated values: "$MAX $PERIOD"
        // For example, "100000 100000" means 100% of 1 CPU core.
        // "50000 100000" means 50% of 1 CPU core.
        std::fs::write(
            self.path.join("cpu.max"),
            format!("{} {}", quota_us, period_us),
        )
    }

    fn procs_file(&self) -> PathBuf {
        self.path.join("cgroup.procs")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ExecutionCgroup {
    fn drop(&mut self) {
        // Clean up the cgroup when the runner finishes.
        // Ignore errors to avoid panicking during teardown.
        if let Err(err) = std::fs::remove_dir(&self.path) {
            eprintln!("failed to remove cgroup: {err}");
        }
    }
}
