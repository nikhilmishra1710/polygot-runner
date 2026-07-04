use clap::{Parser, ValueEnum};
use runtime_worker::engine::ExecutionEngine;
use runtime_worker::model::{ExecutionRequest, Language, ResourceLimits, SourceFile};
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "polyrunner")]
#[command(version = "1.0")]
#[command(about = "Runs code", long_about = None)]
struct Cli {
    mode: Mode,

    /// Sets a custom config file path (Usage: -c file.json or --config file.json)
    #[arg(short, long, value_name = "FILE")]
    file_path: Option<String>,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum Mode {
    Execute,
}

fn main() {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Execute => {
            // Check if the file is NOT empty (i.e., Some)
            if let Some(file_path) = cli.file_path {
                println!("Success! Executing file: {}", file_path);
                let mut file = File::open(&file_path).unwrap();
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents);
                if contents.len() <= 0 {
                    eprintln!("Error: The file is empty");
                    process::exit(0); // Exit with an error code
                }
                println!("Content: {}", contents);
                let temp = SourceFile {
                    path: "main.py".into(),
                    contents: contents.into_bytes().to_vec(),
                };
                let file_vector = vec![temp];
                let request = ExecutionRequest {
                    language: Language::Python,
                    files: file_vector,
                    stdin: Vec::new(),
                    limits: ResourceLimits {
                        wall_time: Duration::from_secs(2),
                    },
                };
                let engine = ExecutionEngine::new();

                let result = engine.execute(&request).unwrap();
                println!("Output: {}", String::from_utf8(result.stdout).unwrap());
                process::exit(result.exit_code.unwrap_or(1));
            } else {
                eprintln!("Error: A file is required when using 'execute' mode.");
                process::exit(1); // Exit with an error code
            }
        }
    }
}
