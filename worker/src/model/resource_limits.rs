use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLimits {
    pub wall_time: Duration,
    pub cpu_time: Duration,
    pub max_open_files: u64,
    pub max_file_size: u64,
    pub memory_bytes: u64,
    pub pids_max: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            wall_time: Duration::from_secs(10),
            cpu_time: Duration::from_secs(5),
            max_open_files: 128,
            max_file_size: 10 * 1024 * 1024,
            memory_bytes: 256 * 1024 * 1024, // 256 MB
            pids_max: 64,
        }
    }
}
