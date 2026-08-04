mod common;
use runtime_worker::model::ExecutionStatus;

use crate::common::{execute_python, open_file_limit};

#[test]
fn namespace_test() {
    let source = r#"
import os

print(os.getuid(), os.geteuid())
"#;

    let result = execute_python(source, open_file_limit(8));

    assert_eq!(result.status, ExecutionStatus::Success);

    let stdout = String::from_utf8_lossy(&result.stdout);

    assert_eq!(stdout, "0 0\n");
}
