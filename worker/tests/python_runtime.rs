use std::path::PathBuf;

use runtime_worker::{
    language::{LanguageRuntime, PythonRuntime},
    model::{ExecutionRequest, Language, SourceFile},
    workspace::WorkspaceManager,
};

#[test]
fn builds_python_command() {
    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: b"print('Hello')".to_vec(),
        }],
        stdin: Vec::new(),
    };

    let manager = WorkspaceManager::new();
    let workspace = manager.create(&request).unwrap();

    let runtime = PythonRuntime;
    let plan = runtime.prepare(&request, &workspace).unwrap();

    assert_eq!(plan.execute.executable.path, PathBuf::from("python3"));
    assert_eq!(plan.execute.args, vec!["main.py"]);
}

#[test]
fn builds_python_command_check_compile_none() {
    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: b"print('Hello')".to_vec(),
        }],
        stdin: Vec::new(),
    };

    let manager = WorkspaceManager::new();
    let workspace = manager.create(&request).unwrap();

    let runtime = PythonRuntime;
    let plan = runtime.prepare(&request, &workspace).unwrap();

    assert_eq!(plan.execute.executable.path, PathBuf::from("python3"));
    assert_eq!(plan.execute.args, vec!["main.py"]);
    assert!(plan.compile.is_none());
    assert_eq!(plan.execute.stdin, request.stdin);
    assert_eq!(plan.execute.working_directory, workspace.path());
}
