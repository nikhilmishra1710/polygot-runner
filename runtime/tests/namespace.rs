mod common;
use runtime_worker::model::TerminationReason;

use crate::common::{execute_python, open_file_limit};

#[test]
fn namespace_test() {
    let source = r#"
import os

print(os.getuid(), os.geteuid())
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let stdout = String::from_utf8_lossy(&result.output.stdout);

    assert_eq!(stdout, "0 0\n");
}

#[test]
fn pid_ppid_test() {
    let source = r#"
import os

print(os.getpid(), os.getppid())
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let stdout = String::from_utf8_lossy(&result.output.stdout);

    assert_eq!(stdout, "2 1\n");
}

#[test]
fn orphan_are_reaped() {
    let source = r#"
import os

if os.fork() == 0:
    os._exit(0)
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.termination, TerminationReason::ExitCode(0));
}

#[test]
fn orphans_are_adopted() {
    let source = r#"
import os
import time

pid = os.fork()

if pid == 0:
    if os.fork() == 0:
        time.sleep(0.5)
        print(os.getppid(), flush=True)
        os._exit(0)

    os._exit(0)

time.sleep(2)
"#;

    let result = execute_python(source, open_file_limit(8));
    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let stdout = String::from_utf8_lossy(&result.output.stdout);
    assert_eq!(stdout.trim(), "1");
}

#[test]
fn zombies_are_reaped() {
    let source = r#"
import os
import time

# Create several ORPHANS
for _ in range(10):
    pid = os.fork()

    if pid == 0:
        # We are the Child. Fork a Grandchild.
        grandchild_pid = os.fork()
        if grandchild_pid == 0:
            # We are the Grandchild. Exit immediately to become a zombie.
            os._exit(0)
            
        # The Child exits immediately. 
        # This abandons the Grandchild, making it an Orphan!
        # The kernel will re-parent the Grandchild to PID 1.
        os._exit(0)
    else:
        # We are PID 2. We MUST reap our direct Child so it doesn't 
        # stay a zombie belonging to us.
        os.waitpid(pid, 0)

# Give PID 1 time to reap the newly orphaned grandchildren
time.sleep(2)

zombies = 0

for pid in os.listdir("/proc"):
    if not pid.isdigit():
        continue

    try:
        with open(f"/proc/{pid}/status") as f:
            status = f.read()

        if "\nState:\tZ" in status:
            zombies += 1
    except FileNotFoundError:
        pass

print(zombies)
"#;

    let result = execute_python(source, open_file_limit(8));
    assert_eq!(result.termination, TerminationReason::ExitCode(0));

    let stdout = String::from_utf8_lossy(&result.output.stdout);
    assert_eq!(stdout.trim(), "0");
}

#[test]
fn signal_forwarding() {
    let source = r#"
import signal
import time

signal.signal(signal.SIGTERM, lambda *_: print("term", flush=True))

while True:
    time.sleep(1)
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.termination, TerminationReason::WallTimeout);

    let stdout = String::from_utf8_lossy(&result.output.stdout);

    assert_eq!(stdout, "term\n");
}
