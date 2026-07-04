use std::time::Duration;

use runtime_worker::{
    engine::ExecutionEngine, model::{ExecutionRequest, ExecutionStatus, Language, ResourceLimits, SourceFile},
};

#[test]
fn executes_python_program() {
    let engine = ExecutionEngine::new();

    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: b"print('Hello from engine')".to_vec(),
        }],
        stdin: Vec::new(),
        limits: ResourceLimits {
            wall_time: Duration::from_secs(2),
        },
    };

    let result = engine.execute(&request).unwrap();

    assert_eq!(result.status, ExecutionStatus::Success);
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "Hello from engine\n"
    );
    assert!(result.stderr.is_empty());
}

#[test]
fn executes_python_program_infinite_loop() {
    let engine = ExecutionEngine::new();

    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: b"while True:
                pass".to_vec(),
        }],
        stdin: Vec::new(),
        limits: ResourceLimits {
            wall_time: Duration::from_secs(1),
        },
    };

    let result = engine.execute(&request).unwrap();

    assert_eq!(result.status, ExecutionStatus::TimeLimitExceeded);
}
