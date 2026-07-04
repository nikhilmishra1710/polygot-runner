use std::{path::PathBuf, time::Duration};

use tempfile::tempdir;

use runtime_worker::{
    runtime::{NativeProcessRunner, RuntimeCommand},
    toolchain::Executable,
};

#[test]
fn runs_python_program() {
    let dir = tempdir().unwrap();

    std::fs::write(dir.path().join("main.py"), r#"print("Hello World")"#).unwrap();

    let executable = Executable {
        path: PathBuf::from("python3"),
    };
    let runtime = NativeProcessRunner::new();
    let result = runtime
        .run(RuntimeCommand {
            executable: executable,
            args: vec!["main.py".to_string()],
            working_directory: dir.path().to_path_buf(),
            stdin: Vec::new(),

            wall_time: Duration::from_secs(2),
        })
        .unwrap();

    assert_eq!(String::from_utf8(result.stdout).unwrap(), "Hello World\n");
}
