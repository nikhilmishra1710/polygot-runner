use clap::{Parser, ValueEnum};
use runtime_worker::engine::ExecutionEngine;
use runtime_worker::model::{ExecutionRequest, Language, ResourceLimits, SourceFile};
use std::fs::File;
use std::io::prelude::*;
use std::process;
use tracing_subscriber::EnvFilter;
use tracing::{error, info};

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
    tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("runtime_worker=info"))
            .unwrap(),
    )
    .init();

    let cli = Cli::parse();

    match cli.mode {
        Mode::Execute => {
            // Check if the file is NOT empty (i.e., Some)
            if let Some(file_path) = cli.file_path {
                info!("Success! Executing file: {}", file_path);
                let mut file = File::open(&file_path).unwrap();
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents);
                if contents.len() <= 0 {
                    error!("Error: The file is empty");
                    process::exit(0); // Exit with an error code
                }
                info!("Content: {}", contents);
                let temp = SourceFile {
                    path: "main.cpp".into(),
                    contents: contents.into_bytes().to_vec(),
                };
                let file_vector = vec![temp];
                let request = ExecutionRequest {
                    language: Language::Cpp,
                    files: file_vector,
                    stdin: Vec::new(),
                    limits: ResourceLimits::default(),
                };
                let engine = ExecutionEngine::new();

                let result = engine.execute(&request).unwrap();
                info!(
                    "Output: {}",
                    String::from_utf8(result.output.stdout).unwrap()
                );
                info!("Err: {}", String::from_utf8(result.output.stderr).unwrap());
                info!("Status: {:?}", result.termination);
                process::exit(0);
            } else {
                error!("Error: A file is required when using 'execute' mode.");
                process::exit(1); // Exit with an error code
            }
        }
    }
}
