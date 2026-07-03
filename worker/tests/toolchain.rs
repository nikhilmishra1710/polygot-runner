use runtime_worker::toolchain::PythonToolchain;

#[test]
fn discovers_python() {
    let toolchain =
        PythonToolchain::from_environment().unwrap();

    assert!(
        toolchain
            .interpreter()
            .path
            .exists()
    );
}