mod common;
use runtime_worker::model::{ExecutionStatus};

use common::{cpu_limit, execute_python, file_size_limit, open_file_limit};

#[test]
fn open_file_limit_exceeded() {
    let source = r#"
import resource

print(resource.getrlimit(resource.RLIMIT_NOFILE), flush=True)
files = []

while True:
    files.append(open("file.txt", "a"))
"#;

    // max_open_files = 32

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.status, ExecutionStatus::RuntimeError);

    let stderr = String::from_utf8_lossy(&result.stderr);

    assert!(stderr.contains("Too many open files") || stderr.contains("EMFILE"));
}

#[test]
fn cpu_limit_exceeded() {
    let source = r#"
while True:
    pass
"#;

    let result = execute_python(source, cpu_limit(5));

    assert_eq!(result.status, ExecutionStatus::RuntimeError,);
}

#[test]
fn file_size_limit_exceeded() {
    let source = r#"
with open("large.bin", "wb") as f:
    while True:
        f.write(b"x" * 4096)
"#;

    let result = execute_python(source, file_size_limit(8192));

    assert_eq!(result.status, ExecutionStatus::RuntimeError);

    let stderr = String::from_utf8_lossy(&result.stderr);

    assert!(stderr.contains("File too large"));
}
