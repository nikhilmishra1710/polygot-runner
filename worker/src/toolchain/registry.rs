use std::collections::HashMap;

use crate::{
    error::WorkerError,
    toolchain::{Executable, Tool},
};

pub struct ToolchainRegistry {
    tools: HashMap<Tool, Executable>,
}

impl ToolchainRegistry {
    pub fn from_environment() -> Result<Self, WorkerError> {
        let mut tools = HashMap::new();

        tools.insert(Tool::Python, Executable::new(which::which("python3")?));

        tools.insert(Tool::CppCompiler, Executable::new(which::which("g++")?));

        Ok(Self { tools })
    }

    pub fn tool(&self, tool: Tool) -> &Executable {
        self.tools.get(&tool).expect("tool should exist")
    }
}
