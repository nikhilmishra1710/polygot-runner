use std::{fs, time::Duration};

use runtime_worker::{model::*, workspace::WorkspaceManager};

#[test]
fn creates_workspace_with_files() {
    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: b"print('Hello')".to_vec(),
        }],
        stdin: Vec::new(),
        limits: ResourceLimits::default(),
    };
    
    let manager = WorkspaceManager::new();
    let workspace = manager.create(&request).unwrap();

    let file = workspace.path().join("main.py");

    assert!(file.exists());

    let contents = fs::read_to_string(file).unwrap();

    assert_eq!(contents, "print('Hello')");
}
