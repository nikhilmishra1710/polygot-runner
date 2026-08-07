mod common;

use crate::common::{cpu_limit, execute_python, file_size_limit, memory_limit, open_file_limit};
use runtime_worker::model::TerminationReason;

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

    assert_eq!(result.termination, TerminationReason::ExitCode(1));

    let stderr = String::from_utf8_lossy(&result.output.stderr);

    assert!(stderr.contains("Too many open files") || stderr.contains("EMFILE"));
}

#[test]
fn cpu_limit_exceeded() {
    let source = r#"
while True:
    pass
"#;

    let result = execute_python(source, cpu_limit(5));

    assert_eq!(result.termination, TerminationReason::CpuLimit,);
}

#[test]
fn memory_limit_exceeded() {
    let source = r#"
a = []

while True:
    a.append(bytearray(256 * 1024 * 1024))
"#;

    let result = execute_python(source, memory_limit(2 * 1024 * 1024));

    assert_eq!(result.termination, TerminationReason::OomKilled);
}

#[test]
fn file_size_limit_exceeded() {
    let source = r#"
with open("large.bin", "wb") as f:
    while True:
        f.write(b"x" * 8192)
"#;

    let result = execute_python(source, file_size_limit(8192));

    assert_eq!(result.termination, TerminationReason::ExitCode(1));

    let stderr = String::from_utf8_lossy(&result.output.stderr);

    assert!(stderr.contains("File too large"));
}
