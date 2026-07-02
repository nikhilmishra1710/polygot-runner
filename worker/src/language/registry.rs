use crate::model::Language;

use super::{LanguageRuntime, PythonRuntime};

pub struct RuntimeRegistry {
    python: PythonRuntime,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self {
            python: PythonRuntime,
        }
    }

    pub fn get(&self, language: Language) -> &dyn LanguageRuntime {
        match language {
            Language::Python => &self.python,
        }
    }
}
