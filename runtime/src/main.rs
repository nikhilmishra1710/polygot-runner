use clap::{Parser, ValueEnum};
use runtime_worker::engine::ExecutionEngine;
use runtime_worker::job::{ExecutionJob, JobId};
use runtime_worker::model::{
    ExecutionRequest, Language, ResourceLimits, SourceFile, TerminationReason,
};
use runtime_worker::protocol::{WorkerRequest, WorkerResponse, receive, send};
use runtime_worker::worker::{Worker, WorkerServer};
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;
use std::{process, thread};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

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
    Default,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("runtime_worker=debug"))
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
        Mode::Default => {
            fn socket_path() -> PathBuf {
                std::env::temp_dir()
                    .join(format!("runtime-worker-test-{}.sock", std::process::id()))
            }

            let socket = socket_path();

            let server_socket = socket.clone();

            let server_thread = thread::spawn(move || {
                let worker = Worker::new();
                let server = WorkerServer::new(worker);

                server.bind(&server_socket).expect("worker server failed");
            });

            // Wait for the server to bind.
            let mut stream = loop {
                match UnixStream::connect(&socket) {
                    Ok(stream) => break stream,
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            };

            // ------------------------------------------------------------
            // Job 1: intentionally fail
            // ------------------------------------------------------------

            let failing_request = ExecutionRequest {
                language: Language::Python,
                files: vec![SourceFile {
                    path: "main.py".into(),
                    contents: b"import sys\nsys.exit(1)".to_vec(),
                }],
                stdin: Vec::new(),
                limits: ResourceLimits::default(),
            };

            let failing_job = ExecutionJob {
                id: JobId("failing-job".to_string()),
                request: failing_request,
            };

            send(&mut stream, &WorkerRequest::Execute(failing_job))
                .expect("failed to send failing execution request");

            let response: WorkerResponse =
                receive(&mut stream).expect("failed to receive failing job response");

            match response {
                WorkerResponse::Result(result) => {
                    assert_eq!(result.id, JobId("failing-job".to_string()));

                    assert_eq!(result.report.termination, TerminationReason::ExitCode(1));
                }

                WorkerResponse::Error(error) => {
                    panic!("worker returned protocol error: {error}");
                }
            }

            // ------------------------------------------------------------
            // Job 2: valid job after failure
            // ------------------------------------------------------------

            let successful_request = ExecutionRequest {
                language: Language::Python,
                files: vec![SourceFile {
                    path: "main.py".into(),
                    contents: b"print('Worker still alive')".to_vec(),
                }],
                stdin: Vec::new(),
                limits: ResourceLimits::default(),
            };

            let successful_job = ExecutionJob {
                id: JobId("successful-job".to_string()),
                request: successful_request,
            };

            send(&mut stream, &WorkerRequest::Execute(successful_job))
                .expect("failed to send successful execution request");

            let response: WorkerResponse =
                receive(&mut stream).expect("failed to receive successful job response");

            match response {
                WorkerResponse::Result(result) => {
                    assert_eq!(result.id, JobId("successful-job".to_string()));

                    assert_eq!(result.report.termination, TerminationReason::ExitCode(0));

                    assert_eq!(
                        String::from_utf8_lossy(&result.report.output.stdout),
                        "Worker still alive\n"
                    );
                }

                WorkerResponse::Error(error) => {
                    panic!("worker returned protocol error: {error}");
                }
            }

            // ------------------------------------------------------------
            // Shutdown
            // ------------------------------------------------------------

            send(&mut stream, &WorkerRequest::Shutdown).expect("failed to send shutdown");

            drop(stream);

            server_thread.join().expect("worker server thread panicked");

            assert!(!socket.exists(), "worker socket was not cleaned up");
        }
    }
}
