use std::time::Duration;

use runtime_worker::{
    engine::ExecutionEngine,
    model::{ExecutionRequest, ExecutionStatus, Language, ResourceLimits, SourceFile},
};

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
        limits: ResourceLimits {
            wall_time: Duration::from_secs(1),
        },
    };

    let result = engine.execute(&request).unwrap();

    assert_eq!(result.status, ExecutionStatus::TimeLimitExceeded);

    println!("result: {}", String::from_utf8(result.stdout.clone()).unwrap());
    let pid = String::from_utf8(result.stdout)
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();

    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
}
