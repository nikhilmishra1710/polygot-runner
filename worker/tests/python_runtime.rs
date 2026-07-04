use std::time::Duration;

use runtime_worker::{
    language::{LanguageRuntime, PythonRuntime},
    model::{ExecutionRequest, Language, ResourceLimits, SourceFile},
    toolchain::{Tool, ToolchainRegistry},
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
        limits: ResourceLimits {
            wall_time: Duration::from_secs(2),
        },
    };

    let manager = WorkspaceManager::new();
    let workspace = manager.create(&request).unwrap();

    let tools = ToolchainRegistry::from_environment().unwrap();
    let runtime = PythonRuntime::new(tools.tool(Tool::Python));
    let plan = runtime.prepare(&request, &workspace).unwrap();

    assert_eq!(
        plan.execute.executable.path,
        tools.tool(Tool::Python).clone().path
    );
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
        limits: ResourceLimits {
            wall_time: Duration::from_secs(2),
        },
    };

    let manager = WorkspaceManager::new();
    let workspace = manager.create(&request).unwrap();

    let tools = ToolchainRegistry::from_environment().unwrap();
    let runtime = PythonRuntime::new(tools.tool(Tool::Python));
    let plan = runtime.prepare(&request, &workspace).unwrap();

    assert_eq!(
        plan.execute.executable.path,
        tools.tool(Tool::Python).clone().path
    );
    assert_eq!(plan.execute.args, vec!["main.py"]);
    assert!(plan.compile.is_none());
    assert_eq!(plan.execute.stdin, request.stdin);
    assert_eq!(plan.execute.working_directory, workspace.path());
}
