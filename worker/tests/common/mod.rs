use runtime_worker::{
    engine::ExecutionEngine,
    model::{ExecutionRequest, ExecutionResult, Language, ResourceLimits, SourceFile},
};
use std::time::Duration;

pub fn execute_python(source: &str, limits: ResourceLimits) -> ExecutionResult {
    execute(Language::Python, "main.py", source, limits)
}

pub fn execute(
    language: Language,
    filename: &str,
    source: &str,
    limits: ResourceLimits,
) -> ExecutionResult {
    let engine = ExecutionEngine::new();

    let request = ExecutionRequest {
        language,
        files: vec![SourceFile {
            path: filename.into(),
            contents: source.as_bytes().to_vec(),
        }],
        stdin: Vec::new(),
        limits,
    };

    engine.execute(&request).unwrap()
}

fn default_limits() -> ResourceLimits {
    ResourceLimits::default()
}

pub fn cpu_limit(seconds: u64) -> ResourceLimits {
    let mut limits = default_limits();
    limits.cpu_time = Duration::from_secs(seconds);
    limits
}

pub fn open_file_limit(max: u64) -> ResourceLimits {
    let mut limits = default_limits();
    limits.max_open_files = max;
    limits
}

pub fn file_size_limit(bytes: u64) -> ResourceLimits {
    let mut limits = default_limits();
    limits.max_file_size = bytes;
    limits
}

pub fn wall_time_limit(seconds: Duration) -> ResourceLimits {
    let mut limits = default_limits();
    limits.wall_time = seconds;
    limits
}
