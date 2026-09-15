use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    thread,
    time::Duration,
};

use runtime_worker::{
    job::{ExecutionJob, JobId}, model::{
        ExecutionEvent, ExecutionRequest, ExecutionStatus, Language, ResourceLimits, SourceFile, TerminationReason,
    }, protocol::{WorkerRequest, WorkerResponse, receive, send}, worker::{Worker, WorkerServer},
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

        WorkerResponse::Event(..) => {}
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

        WorkerResponse::Event(..) => {}
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

        WorkerResponse::Event(..) => {}
    }

    // ------------------------------------------------------------
    // Shutdown
    // ------------------------------------------------------------

    send(&mut stream, &WorkerRequest::Shutdown).expect("failed to send shutdown");

    drop(stream);

    server_thread.join().expect("worker server thread panicked");

    assert!(!socket.exists(), "worker socket was not cleaned up");
}

#[test]
fn unix_socket_streams_execution_events() {
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

    // Python script that explicitly flushes to guarantee multiple stdout chunks
    let script = r#"
import sys
import time

sys.stdout.write('hello\n')
sys.stdout.flush()

time.sleep(0.05)

sys.stdout.write('world\n')
sys.stdout.flush()
"#;

    let request = ExecutionRequest {
        language: Language::Python,
        files: vec![SourceFile {
            path: "main.py".into(),
            contents: script.as_bytes().to_vec(),
        }],
        stdin: Vec::new(),
        limits: ResourceLimits::default(),
    };

    let job = ExecutionJob {
        id: JobId("streaming-job".to_string()),
        request,
    };

    send(&mut stream, &WorkerRequest::Execute(job)).expect("failed to send execution request");

    let mut received_started = false;
    let mut combined_stdout = Vec::new();
    let mut final_result = None;
    let mut count = 0;
    // Loop to collect all events until we hit the final Result or Error
    loop {
        let response: WorkerResponse =
            receive(&mut stream).expect("failed to receive worker response");

        match response {
            WorkerResponse::Event(event) => match event {
                ExecutionEvent::Started => {
                    received_started = true;
                }
                ExecutionEvent::Stdout(chunk) => {
                    combined_stdout.extend_from_slice(&chunk);
                }
                ExecutionEvent::Stderr(_) => {},
                ExecutionEvent::Finished { .. } => {}
            },
            WorkerResponse::Result(result) => {
                final_result = Some(result);
                break; // Break the loop once we get the final result
            }
            WorkerResponse::Error(error) => {
                panic!("worker returned protocol error: {error}");
            }
        }
    }

    // 1. Verify we got the Started event
    assert!(received_started, "Did not receive Started event");

    // 2. Verify all streamed stdout chunks combined correctly
    assert_eq!(String::from_utf8_lossy(&combined_stdout), "hello\nworld\n");

    // 3. Verify the final result report still contains the full output and correct status
    let result = final_result.unwrap();
    assert_eq!(result.id, JobId("streaming-job".to_string()));
    assert_eq!(result.report.termination, TerminationReason::ExitCode(0));
    assert_eq!(
        String::from_utf8_lossy(&result.report.output.stdout),
        "hello\nworld\n"
    );

    // Shutdown cleanly
    send(&mut stream, &WorkerRequest::Shutdown).expect("failed to send shutdown");
    drop(stream);
    server_thread.join().expect("worker server thread panicked");
}
