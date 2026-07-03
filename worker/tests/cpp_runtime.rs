use std::path::PathBuf;

use runtime_worker::{
    language::{CppRuntime, LanguageRuntime},
    model::{ExecutionRequest, Language, SourceFile},
    workspace::WorkspaceManager,
};

#[test]
fn builds_python_command() {
    let request = ExecutionRequest {
        language: Language::Cpp,
        files: vec![SourceFile {
            path: "main.cpp".into(),
            contents: b"#include<iostream>
            int main() {
                cout<<'Hello world';
                return 0;
            }"
            .to_vec(),
        }],
        stdin: Vec::new(),
    };

    let manager = WorkspaceManager::new();
    let workspace = manager.create(&request).unwrap();

    let runtime = CppRuntime;
    let plan = runtime.prepare(&request, &workspace).unwrap();

    let compile = plan.compile.unwrap();
    assert_eq!(compile.executable.path, PathBuf::from("g++"));
    assert_eq!(plan.execute.executable.path, PathBuf::from("./a.out"));
    assert!(plan.execute.args.is_empty());
}

#[test]
fn builds_python_command_check_compile_not_none() {
    let request = ExecutionRequest {
        language: Language::Cpp,
        files: vec![SourceFile {
            path: "main.cpp".into(),
            contents: b"#include<iostream>
            int main() {
                cout<<'Hello world';
                return 0;
            }"
            .to_vec(),
        }],
        stdin: Vec::new(),
    };

    let manager = WorkspaceManager::new();
    let workspace = manager.create(&request).unwrap();

    let runtime = CppRuntime;
    let plan = runtime.prepare(&request, &workspace).unwrap();

    assert_eq!(plan.execute.executable.path, PathBuf::from("./a.out"));
    assert!(!plan.compile.is_none());
    assert_eq!(plan.execute.stdin, request.stdin);
    assert_eq!(plan.execute.working_directory, workspace.path());
}
