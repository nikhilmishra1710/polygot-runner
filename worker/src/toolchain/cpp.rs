use super::Executable;

pub struct CppToolchain {
    compiler: Executable,
}

impl CppToolchain {
    pub fn new(
        compiler: Executable,
    ) -> Self {
        Self { compiler }
    }

    pub fn compiler(&self) -> &Executable {
        &self.compiler
    }
}