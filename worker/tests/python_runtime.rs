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
    let command = runtime.build_command(&request, &workspace);

    assert_eq!(command.executable.path, PathBuf::from("python3"));
    assert_eq!(command.args, vec!["main.py"]);
}
