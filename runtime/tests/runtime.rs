mod common;
use std::{path::PathBuf, time::Duration};

use crate::common::wall_time_limit;
use runtime_worker::{
    runtime::{NativeProcessRunner, RuntimeCommand},
    toolchain::Executable,
};
use tempfile::tempdir;

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
