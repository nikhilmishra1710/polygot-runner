use std::time::Duration;
mod common;
use runtime_worker::{
    engine::ExecutionEngine,
    model::{ExecutionRequest, Language, SourceFile, TerminationReason},
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
    assert_eq!(result.termination, TerminationReason::WallTimeout);
}
