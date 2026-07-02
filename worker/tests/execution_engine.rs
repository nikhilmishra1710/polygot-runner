use runtime_worker::{
    engine::ExecutionEngine,
    model::{ExecutionRequest, ExecutionStatus, Language, SourceFile},
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
    };

    let result = engine.execute(&request).unwrap();

    assert_eq!(result.status, ExecutionStatus::Success);
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "Hello from engine\n"
    );
    assert!(result.stderr.is_empty());
}
