use std::fs;
use std::path::Path;
use std::time::Duration;

pub fn read_cgroup_metrics(cgroup_path: &Path) -> (u64, u64, Duration) {
    let peak_rss = read_cgroup_u64(cgroup_path, "memory.peak")
        .or_else(|| read_cgroup_u64(cgroup_path, "memory.current"))
        .unwrap_or(0);

    let peak_pids = read_cgroup_u64(cgroup_path, "pids.peak")
        .or_else(|| read_cgroup_u64(cgroup_path, "pids.current"))
        .unwrap_or(0);

    let cpu_time = read_cgroup_cpu_usage(cgroup_path).unwrap_or(Duration::ZERO);

    (peak_rss, peak_pids, cpu_time)
}

fn read_cgroup_u64(cgroup_path: &Path, file_name: &str) -> Option<u64> {
    let content = fs::read_to_string(cgroup_path.join(file_name)).ok()?;
    content.trim().parse::<u64>().ok()
}

fn read_cgroup_cpu_usage(cgroup_path: &Path) -> Option<Duration> {
    let content = fs::read_to_string(cgroup_path.join("cpu.stat")).ok()?;

    // Parses lines like "usage_usec 123456"
    for line in content.lines() {
        if line.starts_with("usage_usec") {
            let mut parts = line.split_whitespace();
            let _key = parts.next()?;
            let usec_str = parts.next()?;
            let usec = usec_str.parse::<u64>().ok()?;
            return Some(Duration::from_micros(usec));
        }
    }
    None
}
