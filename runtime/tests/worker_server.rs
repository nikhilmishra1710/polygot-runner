use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    thread,
    time::Duration,
};

use runtime_worker::{
    job::{ExecutionJob, JobId},
    model::{
        ExecutionRequest, ExecutionStatus, Language, ResourceLimits, SourceFile, TerminationReason,
    },
    protocol::{WorkerRequest, WorkerResponse, receive, send},
    worker::{Worker, WorkerServer},
};

fn socket_path() -> PathBuf {
    // Using UUID guarantees this run won't collide with old zombie sockets
    let id = uuid::Uuid::new_v4();
    std::env::temp_dir().join(format!("runtime-worker-test-{id}.sock"))
}

#[test]
fn unix_socket_executes_job_end_to_end() {
    let socket = socket_path();

    let server_socket = socket.clone();

    let server_thread = thread::spawn(move || {
        let worker = Worker::new();
        let server = WorkerServer::new(worker);

        server.bind(&server_socket).expect("worker server failed");
    });

    // Give the server a moment to bind.
    //
    // If your server already exposes a readiness mechanism, use that
    // instead of sleeping.
    let mut stream = loop {
        match UnixStream::connect(&socket) {
            Ok(stream) => break stream,
            Err(_) => thread::sleep(Duration::from_millis(10)),
        }
    };

    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: b"print('Hello')".to_vec(),
        }],
        stdin: Vec::new(),
        limits: ResourceLimits::default(),
    };

    let job = ExecutionJob {
        id: JobId("integration-test-job".to_string()),
        request,
    };

    send(&mut stream, &WorkerRequest::Execute(job)).expect("failed to send execution request");

    let response: WorkerResponse = receive(&mut stream).expect("failed to receive worker response");

    match response {
        WorkerResponse::Result(result) => {
            assert_eq!(result.id, JobId("integration-test-job".to_string()));

            assert_eq!(result.report.termination, TerminationReason::ExitCode(0));

            assert_eq!(
                String::from_utf8_lossy(&result.report.output.stdout),
                "Hello\n"
            );
        }

        WorkerResponse::Error(error) => {
            panic!("worker returned protocol error: {error}");
        }
    }

    // Shut the server down cleanly.
    send(&mut stream, &WorkerRequest::Shutdown).expect("failed to send shutdown");

    drop(stream);

    server_thread.join().expect("worker server thread panicked");

    assert!(!socket.exists(), "worker socket was not cleaned up");
}

#[test]
fn unix_socket_returns_execution_failure_without_killing_server() {
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
