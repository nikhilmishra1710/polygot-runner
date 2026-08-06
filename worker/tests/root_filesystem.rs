mod common;
use runtime_worker::model::{ExecutionStatus, TerminationReason};

use crate::common::{default_limits, execute_python, open_file_limit};

#[test]
fn new_filesytem_is_seen() {
    let source = r#"
import os

print(os.getcwd())
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.termination, TerminationReason::ExitCode(0));
    let stdout = String::from_utf8_lossy(&result.output.stdout);

    assert_eq!(stdout.trim(), "/");
}

#[test]
fn root_is_sandbox_root() {
    let source = r#"
import os
print(os.path.realpath("/"), flush=True)
"#;

    let result = execute_python(source, Default::default());

    let stdout = String::from_utf8_lossy(&result.output.stdout);

    assert_eq!(stdout.trim(), "/");
}

#[test]
fn workspace_is_current_directory() {
    let source = r#"
import os
print(os.getcwd(), flush=True)
"#;

    let result = execute_python(source, Default::default());

    let stdout = String::from_utf8_lossy(&result.output.stdout);

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

    let stdout = String::from_utf8_lossy(&result.output.stdout);
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

    let stdout = String::from_utf8_lossy(&result.output.stdout);

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

    let stdout = String::from_utf8_lossy(&result.output.stdout);

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

    let stdout = String::from_utf8_lossy(&result.output.stdout);

    assert_eq!(stdout.trim(), "/");
}

#[test]
fn test_bin_is_readonly() {
    let source = r#"
from pathlib import Path
try:
    Path("/bin/test_file").write_text("should fail")
    print("bad")
except OSError:
    print("ok")
"#;

    let result = execute_python(source, default_limits());
    assert_eq!(result.termination, TerminationReason::ExitCode(0));
    assert_eq!(String::from_utf8_lossy(&result.output.stdout).trim(), "ok");
}

#[test]
fn test_etc_is_readonly() {
    let source = r#"
from pathlib import Path
try:
    Path("/etc/test_file").write_text("should fail")
    print("bad")
except OSError:
    print("ok")
"#;

    let result = execute_python(source, default_limits());
    assert_eq!(result.termination, TerminationReason::ExitCode(0));
    assert_eq!(String::from_utf8_lossy(&result.output.stdout).trim(), "ok");
}

#[test]
fn test_workspace_is_writable() {
    let source = r#"
from pathlib import Path
Path("hello.txt").write_text("abc")
print(Path("hello.txt").read_text())
"#;

    let result = execute_python(source, default_limits());
    assert_eq!(result.termination, TerminationReason::ExitCode(0));
    assert_eq!(String::from_utf8_lossy(&result.output.stdout).trim(), "abc");
}

#[test]
fn test_tmpfs_is_ephemeral() {
    // 1. Write file to /tmp in first sandbox execution
    let source1 = r#"
from pathlib import Path
Path("/tmp/persistent_check.txt").write_text("123")
print(Path("/tmp/persistent_check.txt").exists())
"#;

    let result1 = execute_python(source1, default_limits());
    assert_eq!(result1.termination, TerminationReason::ExitCode(0));
    assert_eq!(
        String::from_utf8_lossy(&result1.output.stdout).trim(),
        "True"
    );

    // 2. Check if file exists in a brand-new sandbox execution
    let source2 = r#"
from pathlib import Path
print(Path("/tmp/persistent_check.txt").exists())
"#;

    let result2 = execute_python(source2, default_limits());
    assert_eq!(result2.termination, TerminationReason::ExitCode(0));
    assert_eq!(
        String::from_utf8_lossy(&result2.output.stdout).trim(),
        "False"
    );
}

#[test]
fn test_dev_null_access() {
    let source = r#"
with open("/dev/null", "w") as f:
    f.write("hello")
print("ok")
"#;

    let result = execute_python(source, default_limits());
    assert_eq!(result.termination, TerminationReason::ExitCode(0));
    assert_eq!(String::from_utf8_lossy(&result.output.stdout).trim(), "ok");
}

#[test]
fn test_proc_self_exists() {
    let source = r#"
import os
print(os.path.exists("/proc/self"))
"#;

    let result = execute_python(source, default_limits());
    assert_eq!(result.termination, TerminationReason::ExitCode(0));
    assert_eq!(
        String::from_utf8_lossy(&result.output.stdout).trim(),
        "True"
    );
}
