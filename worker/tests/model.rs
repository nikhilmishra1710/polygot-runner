use std::time::Duration;

use runtime_worker::model::*;

#[test]
fn create_execution_request() {
    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: b"print('Hello')".to_vec(),
        }],
        stdin: Vec::new(),
        limits: ResourceLimits::default(),
    };

    assert_eq!(request.language, Language::Python);
    assert_eq!(request.files.len(), 1);
}

#[test]
fn execution_status_equality() {
    assert_eq!(
        TerminationReason::ExitCode(0),
        TerminationReason::ExitCode(0)
    );
}