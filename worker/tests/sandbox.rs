use runtime_worker::sandbox::Sandbox;
use std::path::PathBuf;

#[test]
fn creates_workspace_directory() {
    let sandbox = Sandbox::new().unwrap();

    assert!(sandbox.root().exists());
    assert!(sandbox.workspace().exists());
}

#[test]
fn removes_sandbox_when_dropped() {
    let root: PathBuf;

    {
        let sandbox = Sandbox::new().unwrap();
        root = sandbox.root().to_path_buf();

        assert!(root.exists());
    }

    assert!(!root.exists());
}
