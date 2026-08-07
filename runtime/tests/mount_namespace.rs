mod common;
use runtime_worker::model::TerminationReason;

use crate::common::{execute_python, open_file_limit};

#[test]
fn check_mount_created() {
    let source = r#"
import os

print(os.readlink("/proc/self/ns/mnt"))
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.termination, TerminationReason::ExitCode(0));
    let host_ns = std::fs::read_link("/proc/self/ns/mnt").unwrap();
    let stdout = String::from_utf8_lossy(&result.output.stdout);

    assert_ne!(stdout.trim(), host_ns.to_string_lossy().trim());
}

#[test]
fn child_can_create_a_mount() {
    let source = r#"
import subprocess

subprocess.run(
    ["mount", "-t", "tmpfs", "tmpfs", "/tmp"],
    check=True,
)

print("mounted")
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    assert_eq!(
        String::from_utf8(result.output.stdout).unwrap(),
        "mounted\n"
    );
}

#[test]
fn child_mount_not_visible_to_parent() {
    let source = r#"
import subprocess

subprocess.run(
    ["mount", "-t", "tmpfs", "tmpfs", "/tmp"],
    check=True,
)

print("done")
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let mounts = std::fs::read_to_string("/proc/self/mounts").unwrap();

    assert!(!mounts.contains("tmpfs /tmp"));
}

#[test]
fn child_mount_sees_all_mounts() {
    let source = r#"
with open("/proc/self/mounts") as f:
    print(len(f.readlines()))
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let stdout = String::from_utf8_lossy(&result.output.stdout);
    let count: usize = stdout.trim().parse().unwrap();

    assert!(count > 0);
}
