use std::time::Duration;
mod common;
use runtime_worker::{
    engine::ExecutionEngine,
    model::{ExecutionRequest, ExecutionStatus, Language, ResourceLimits, SourceFile},
};

use crate::common::wall_time_limit;

#[test]
fn timeout_kills_process_group() {
    let engine = ExecutionEngine::new();

    let source = r#"
import subprocess
import os
import time

child = subprocess.Popen(["sleep", "30"])

print(child.pid, flush=True)

while True:
    time.sleep(1)
"#;

    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: source.as_bytes().to_vec(),
        }],
        stdin: Vec::new(),
        limits: wall_time_limit(Duration::from_secs(1)),
    };

    let result = engine.execute(&request).unwrap();
    println!("status: {:?}", result.status);
    println!("stdout:\n{}", String::from_utf8_lossy(&result.stdout));
    println!("stderr:\n{}", String::from_utf8_lossy(&result.stderr));
    println!("exit_code: {:?}", result.exit_code);
    assert_eq!(result.status, ExecutionStatus::TimeLimitExceeded);

    let pid = String::from_utf8(result.stdout)
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();

    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
}
