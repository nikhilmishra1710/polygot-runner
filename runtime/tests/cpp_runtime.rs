use std::path::PathBuf;
mod common;
use runtime_worker::{
    language::{CppRuntime, LanguageRuntime},
    model::{ExecutionRequest, Language, ResourceLimits, SourceFile, TerminationReason},
    toolchain::{Tool, ToolchainRegistry},
    workspace::WorkspaceManager,
};

use crate::common::{default_limits, execute_cpp};

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
        limits: ResourceLimits::default(),
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
        limits: ResourceLimits::default(),
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

#[test]
fn run_cpp_file() {
    let source = r#"
#include <iostream>

int main() {
    std::cout<<"Hello World";
    return 0;
}
"#;

    let result = execute_cpp(source, default_limits());

    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let stdout = String::from_utf8_lossy(&result.output.stdout);

    assert_eq!(stdout.trim(), "Hello World");
}

#[test]
fn run_cpp_file_does_not_compile() {
    let source = r#"
#include <iostream>

int main() {
    std::cout<<"Hello World";
    return 0;

"#;

    let result = execute_cpp(source, default_limits());

    assert_eq!(result.termination, TerminationReason::ExitCode(1));
}
