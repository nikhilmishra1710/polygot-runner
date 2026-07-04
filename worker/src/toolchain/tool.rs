#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Python,
    CppCompiler,
    RustCompiler,
    Cargo,
    Go,
    JavaCompiler,
    JavaRuntime,
}