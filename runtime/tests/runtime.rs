mod common;
use std::{path::PathBuf, time::Duration};

use crate::common::{default_limits, execute_python, wall_time_limit};
use runtime_worker::{
    engine::ExecutionEngine,
    model::{ExecutionRequest, Language, SourceFile, TerminationReason},
    runtime::{NativeProcessRunner, RuntimeCommand},
    toolchain::Executable,
};
use tempfile::tempdir;
use tracing_subscriber::EnvFilter;

#[test]
fn runs_python_program() {
    let dir = tempdir().unwrap();

    std::fs::write(dir.path().join("main.py"), r#"print("Hello World")"#).unwrap();

    let executable = Executable {
        path: PathBuf::from("/usr/bin/python3"),
    };
    let runtime = NativeProcessRunner::new();
    let result = runtime
        .run(RuntimeCommand {
            executable: executable,
            args: vec!["main.py".to_string()],
            working_directory: dir.path().to_path_buf(),
            stdin: Vec::new(),

            limits: wall_time_limit(Duration::from_secs(2)),
        })
        .unwrap();

    assert_eq!(
        String::from_utf8(result.output.stdout).unwrap(),
        "Hello World\n"
    );
}

#[test]
fn test_multi_file_python() {
    let file1_source = r#"
import file1
def hello():
    print("hello from file1", flush=True)
"#;
    let source = r#"
from file1 import hello
import os
with open("/proc/self/status") as f:
    for line in f:
        if line.startswith("Seccomp:"):
            print(line.strip())

hello()
"#;

    let engine = ExecutionEngine::new();

    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![
            SourceFile {
                path: "main.py".into(),
                contents: source.as_bytes().to_vec(),
            },
            SourceFile {
                path: "file1.py".into(),
                contents: file1_source.as_bytes().to_vec(),
            },
        ],
        stdin: Vec::new(),
        limits: default_limits(),
    };

    let result = engine.execute(&request).unwrap();

    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let stdout = String::from_utf8_lossy(&result.output.stdout);
    assert_eq!(stdout.trim(), "Seccomp:\t2\nhello from file1");
}

#[test]
fn test_multi_file_cpp() {
    let file1_source = r#"
#include <iostream>

void hello() {
    std::cout << "Hello from file 1";
}
"#;
    let source = r#"
#include <iostream>
#include "file1.cpp"

int main() {
    std::cout << "Hello from main.cpp" << std::endl;
    hello();

    return 0;
}
"#;

    let engine = ExecutionEngine::new();

    let request = ExecutionRequest {
        language: Language::Cpp,
        files: vec![
            SourceFile {
                path: "main.cpp".into(),
                contents: source.as_bytes().to_vec(),
            },
            SourceFile {
                path: "file1.cpp".into(),
                contents: file1_source.as_bytes().to_vec(),
            },
        ],
        stdin: Vec::new(),
        limits: default_limits(),
    };

    let result = engine.execute(&request).unwrap();

    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let stdout = String::from_utf8_lossy(&result.output.stdout);
    assert_eq!(stdout.trim(), "Hello from main.cpp\nHello from file 1");
}
