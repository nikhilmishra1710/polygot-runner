use runtime_worker::toolchain::{Tool, ToolchainRegistry};

#[test]
fn discovers_python() {
    let tools = ToolchainRegistry::from_environment().unwrap();

    let python = tools.tool(Tool::Python);
    assert!(python.path.exists());

    let cpp = tools.tool(Tool::CppCompiler);
    assert!(cpp.path.exists());
}
