use std::path::PathBuf;

#[derive(Debug)]
pub struct Executable {
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct RuntimeCommand {
    pub executable: Executable,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
    pub stdin: Vec<u8>,
}