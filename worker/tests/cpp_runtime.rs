use std::{path::PathBuf, time::Duration};

use runtime_worker::{
    language::{CppRuntime, LanguageRuntime}, model::{ExecutionRequest, Language, ResourceLimits, SourceFile}, toolchain::{Tool, ToolchainRegistry}, workspace::WorkspaceManager,
};

#[test]
fn builds_cpp_command() {
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
        limits: ResourceLimits {
            wall_time: Duration::from_secs(2),
        },
    };

    let manager = WorkspaceManager::new();
    let workspace = manager.create(&request).unwrap();
    let tools = ToolchainRegistry::from_environment().unwrap();
    let runtime = CppRuntime::new(tools.tool(Tool::CppCompiler));
    let plan = runtime.prepare(&request, &workspace).unwrap();

    let compile = plan.compile.unwrap();
    assert_eq!(
        compile.executable.path,
        tools.tool(Tool::CppCompiler).clone().path
    );
    assert_eq!(plan.execute.executable.path, PathBuf::from("./a.out"));
    assert!(plan.execute.args.is_empty());
}

#[test]
fn builds_cpp_command_check_compile_not_none() {
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
        limits: ResourceLimits {
            wall_time: Duration::from_secs(2),
        },
    };

    let manager = WorkspaceManager::new();
    let workspace = manager.create(&request).unwrap();

    let tools = ToolchainRegistry::from_environment().unwrap();
    let runtime = CppRuntime::new(tools.tool(Tool::CppCompiler));
    let plan = runtime.prepare(&request, &workspace).unwrap();

    let compile = plan.compile.unwrap();
    assert_eq!(
        compile.executable.path,
        tools.tool(Tool::CppCompiler).clone().path
    );
    assert_eq!(plan.execute.executable.path, PathBuf::from("./a.out"));
    assert_eq!(plan.execute.stdin, request.stdin);
    assert_eq!(plan.execute.working_directory, workspace.path());
}
