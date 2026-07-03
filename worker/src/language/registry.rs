use crate::model::Language;

use super::{LanguageRuntime, PythonRuntime, CppRuntime};

pub struct RuntimeRegistry {
    python: PythonRuntime,
    cpp: CppRuntime
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self {
            python: PythonRuntime,
            cpp: CppRuntime
        }
    }

    pub fn get(&self, language: Language) -> &dyn LanguageRuntime {
        match language {
            Language::Python => &self.python,
            Language::Cpp => &self.cpp
        }
    }
}
