use crate::{
    model::Language,
    toolchain::{Tool, ToolchainRegistry},
};

use super::{CppRuntime, LanguageRuntime, PythonRuntime};

pub struct RuntimeRegistry {
    python: PythonRuntime,
    cpp: CppRuntime,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        let tools = ToolchainRegistry::from_environment().unwrap();

        let python = PythonRuntime::new(tools.tool(Tool::Python));

        let cpp_runtime = CppRuntime::new(tools.tool(Tool::CppCompiler));

        Self {
            python: python,
            cpp: cpp_runtime,
        }
    }

    pub fn get(&self, language: Language) -> &dyn LanguageRuntime {
        match language {
            Language::Python => &self.python,
            Language::Cpp => &self.cpp,
        }
    }
}
