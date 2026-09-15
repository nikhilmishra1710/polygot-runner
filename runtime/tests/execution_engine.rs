mod common;
use std::time::Duration;

use runtime_worker::{
    engine::ExecutionEngine,
    job::{ExecutionJob, JobId},
    model::{
        ExecutionEvent, ExecutionRequest, Language, ResourceLimits, SourceFile, TerminationReason,
    },
    worker::Worker,
};

use crate::common::wall_time_limit;

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
        limits: ResourceLimits::default(),
    };

    let result = engine.execute(&request).unwrap();

    assert_eq!(result.termination, TerminationReason::ExitCode(0));
    assert_eq!(
        String::from_utf8(result.output.stdout).unwrap(),
        "Hello from engine\n"
    );
    assert!(result.output.stderr.is_empty());
}

#[test]
fn executes_python_program_infinite_loop() {
    let engine = ExecutionEngine::new();

    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: b"while True:
                pass"
                .to_vec(),
        }],
        stdin: Vec::new(),
        limits: wall_time_limit(Duration::from_secs(2)),
    };

    let result = engine.execute(&request).unwrap();

    assert_eq!(result.termination, TerminationReason::WallTimeout);
}

#[test]
fn execution_streams_stdout() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer() // Writes to stdout/stderr in a way that respects cargo test
        .with_max_level(tracing::Level::DEBUG) // Change to INFO/TRACE as needed
        .try_init();
    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: br#"
import time
print("hello")
time.sleep(1)
print("world")
"#
            .to_vec(),
        }],
        stdin: Vec::new(),
        limits: ResourceLimits::default(),
    };

    let job = ExecutionJob {
        id: JobId("stream-test".into()),
        request,
    };

    let (tx, rx) = std::sync::mpsc::sync_channel(128);

    let worker = Worker::new();

    let result = worker
        .execute_with_events(job, tx)
        .expect("execution failed");

    let events: Vec<ExecutionEvent> = rx.iter().collect();

    assert!(matches!(events.first(), Some(ExecutionEvent::Started)));

    assert!(events.iter().any(|event| {
        matches!(
            event,
            ExecutionEvent::Stdout(data)
                if data == b"hello\nworld\n"
        )
    }));

    assert!(matches!(
        events.last(),
        Some(ExecutionEvent::Finished { .. })
    ));

    assert_eq!(
        String::from_utf8_lossy(&result.report.output.stdout),
        "hello\nworld\n"
    );
}
