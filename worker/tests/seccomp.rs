mod common;

use crate::common::{default_limits, execute_python};
use runtime_worker::model::TerminationReason;

#[test]
fn test_seccomp_is_active() {
    let source = r#"
import os
with open("/proc/self/status") as f:
    for line in f:
        if line.startswith("Seccomp:"):
            print(line.strip())
"#;

    let result = execute_python(source, default_limits());
    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let stdout = String::from_utf8_lossy(&result.output.stdout);
    assert_eq!(stdout.trim(), "Seccomp:\t2");
}

#[test]
fn test_allowed_syscalls() {
    let source = r#"
with open("test.txt", "w") as f:
    f.write("hello")
print("success")
"#;

    let result = execute_python(source, default_limits());
    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let stdout = String::from_utf8_lossy(&result.output.stdout);
    assert_eq!(stdout.trim(), "success");
}

#[test]
fn test_seccomp_violation() {
    let source = r#"
import ctypes

# Attempt to call the reboot(2) syscall directly
libc = ctypes.CDLL(None)

# 0x01234567 is the LINUX_REBOOT_MAGIC1 constant. 
# It doesn't matter if it fails gracefully in a normal environment, 
# seccomp will intercept it and kill the process instantly.
libc.reboot(0x01234567)
"#;

    let result = execute_python(source, default_limits());

    // The kernel should catch reboot(2) and instantly SIGSYS the process
    assert_eq!(result.termination, TerminationReason::SeccompViolation);
}
