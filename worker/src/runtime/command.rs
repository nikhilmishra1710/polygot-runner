use std::path::PathBuf;

use crate::toolchain::Executable;


#[derive(Debug)]
pub struct RuntimeCommand {
    pub executable: Executable,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
    pub stdin: Vec<u8>,
}