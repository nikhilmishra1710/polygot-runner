mod common;
use runtime_worker::model::ExecutionStatus;

use crate::common::{execute_python, open_file_limit};

#[test]
fn new_filesytem_is_seen() {
    let source = r#"
import os

print(os.getcwd())
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.status, ExecutionStatus::Success);
    let stdout = String::from_utf8_lossy(&result.stdout);

    assert_eq!(stdout.trim(), "/");
}

#[test]
fn root_is_sandbox_root() {
    let source = r#"
import os
print(os.path.realpath("/"), flush=True)
"#;

    let result = execute_python(source, Default::default());

    let stdout = String::from_utf8_lossy(&result.stdout);

    assert_eq!(stdout.trim(), "/");
}

#[test]
fn workspace_is_current_directory() {
    let source = r#"
import os
print(os.getcwd(), flush=True)
"#;

    let result = execute_python(source, Default::default());

    let stdout = String::from_utf8_lossy(&result.stdout);

    assert_eq!(stdout.trim(), "/");
}

#[test]
fn proc_filesystem_is_mounted() {
    let source = r#"
import os

print(os.path.isdir("/proc"), flush=True)
print(os.path.exists("/proc/self"), flush=True)
"#;

    let result = execute_python(source, Default::default());

    let stdout = String::from_utf8_lossy(&result.stdout);
    let mut lines = stdout.lines();

    assert_eq!(lines.next(), Some("True"));
    assert_eq!(lines.next(), Some("True"));
}

#[test]
fn dev_directory_exists() {
    let source = r#"
import os

print(os.path.isdir("/dev"), flush=True)
"#;

    let result = execute_python(source, Default::default());

    let stdout = String::from_utf8_lossy(&result.stdout);

    assert_eq!(stdout.trim(), "True");
}

#[test]
fn tmp_is_isolated_from_host() {
    let source = r#"
import os

path = "/tmp/runtime_test"

with open(path, "w") as f:
    f.write("sandbox")

print(os.path.exists(path), flush=True)
"#;

    let result = execute_python(source, Default::default());

    let stdout = String::from_utf8_lossy(&result.stdout);

    assert_eq!(stdout.trim(), "True");

    assert!(!std::path::Path::new("/tmp/runtime_test").exists());
}

#[test]
fn cannot_escape_root_with_dotdot() {
    let source = r#"
import os

os.chdir("/")
os.chdir("../../../../../../../../")

print(os.getcwd(), flush=True)
"#;

    let result = execute_python(source, Default::default());

    let stdout = String::from_utf8_lossy(&result.stdout);

    assert_eq!(stdout.trim(), "/");
}

#[test]
fn host_files_are_not_visible() {
    let source = r#"
import os

print(os.path.exists("/home"), flush=True)
print(os.path.exists("/etc/shadow"), flush=True)
"#;

    let result = execute_python(source, Default::default());

    let stdout = String::from_utf8_lossy(&result.stdout);

    let mut lines = stdout.lines();

    assert_eq!(lines.next(), Some("False"));
    assert_eq!(lines.next(), Some("False"));
}