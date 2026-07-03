use which::which;

use crate::error::WorkerError;

use super::Executable;

pub struct PythonToolchain {
    interpreter: Executable,
}

impl PythonToolchain {
    pub fn from_environment() -> Result<Self, WorkerError> {
        let interpreter = Executable::new(which("python3")?);

        Ok(Self { interpreter })
    }

    pub fn interpreter(&self) -> &Executable {
        &self.interpreter
    }
}