use runtime_worker::worker::grpc_server::pb::termination_reason::Reason;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Server};

use runtime_worker::worker::grpc_server::GrpcWorkerServer;
use runtime_worker::worker::grpc_server::pb::{
    ExecuteRequest, SourceFile, execution_event::EventType,
    execution_service_client::ExecutionServiceClient,
    execution_service_server::ExecutionServiceServer,
};
use runtime_worker::worker::worker::Worker;

/// Helper function to spin up a gRPC server on a random open port and return a connected client.
async fn setup_grpc_test_client() -> ExecutionServiceClient<Channel> {
    // Bind to port 0 to let the OS assign a random open port, preventing test collisions
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let worker = Worker::new();
    let grpc_server = GrpcWorkerServer::new(worker);

    // Spawn the server in the background
    tokio::spawn(async move {
        Server::builder()
            .add_service(ExecutionServiceServer::new(grpc_server))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("gRPC server failed to run");
    });

    // Brief yield to ensure the server is accepting connections
    tokio::time::sleep(Duration::from_millis(50)).await;

    let url = format!("http://{}", addr);
    ExecutionServiceClient::connect(url)
        .await
        .expect("Failed to connect to test gRPC server")
}

#[tokio::test]
async fn grpc_executes_job_end_to_end() {
    let mut client = setup_grpc_test_client().await;

    let request = ExecuteRequest {
        language: "python".to_string(),
        files: vec![SourceFile {
            path: "main.py".to_string(),
            contents: b"print('Hello')".to_vec(),
        }],
        stdin: vec![],
    };

    let mut stream = client
        .execute(request)
        .await
        .expect("failed to send execute request")
        .into_inner();

    let mut final_result = None;
    let mut combined_stdout = Vec::new();

    // Read the stream
    while let Some(event) = stream.message().await.expect("Failed to read from stream") {
        match event.r#type() {
            t if t == EventType::Stdout => {
                combined_stdout.extend_from_slice(&event.data);
            }
            t if t == EventType::Finished => {
                final_result = event.final_result;
                break;
            }
            _ => {}
        }
    }

    let result = final_result.expect("Did not receive a final result");
    let report = result.report.expect("Did not receive a report");
    // Assert 0 exit code
    assert_eq!(
        report.termination.unwrap().reason.unwrap(),
        Reason::ExitCode(0)
    );

    // Assert stdout correctly populated
    assert_eq!(String::from_utf8_lossy(&combined_stdout), "Hello\n");

    // Fallback assert that the final report matches
    assert_eq!(
        String::from_utf8_lossy(&report.output.unwrap().stdout),
        "Hello\n"
    );
}

#[tokio::test]
async fn grpc_returns_execution_failure_without_killing_server() {
    let mut client = setup_grpc_test_client().await;

    // ------------------------------------------------------------
    // Job 1: intentionally fail
    // ------------------------------------------------------------
    let failing_request = ExecuteRequest {
        language: "python".to_string(),
        files: vec![SourceFile {
            path: "main.py".to_string(),
            contents: b"import sys\nsys.exit(1)".to_vec(),
        }],
        stdin: vec![],
    };

    let mut stream1 = client.execute(failing_request).await.unwrap().into_inner();
    let mut result1 = None;

    while let Some(event) = stream1.message().await.unwrap() {
        if event.r#type() == EventType::Finished {
            result1 = event.final_result;
            break;
        }
    }

    assert_eq!(
        result1
            .unwrap()
            .report
            .unwrap()
            .termination
            .unwrap()
            .reason
            .unwrap(),
        Reason::ExitCode(1),
        "Expected failing job to return exit code 1"
    );

    // ------------------------------------------------------------
    // Job 2: valid job after failure
    // ------------------------------------------------------------
    let successful_request = ExecuteRequest {
        language: "python".to_string(),
        files: vec![SourceFile {
            path: "main.py".to_string(),
            contents: b"print('Worker still alive')".to_vec(),
        }],
        stdin: vec![],
    };

    let mut stream2 = client
        .execute(successful_request)
        .await
        .unwrap()
        .into_inner();
    let mut result2 = None;
    let mut stdout2 = Vec::new();

    while let Some(event) = stream2.message().await.unwrap() {
        match event.r#type() {
            t if t == EventType::Stdout => stdout2.extend_from_slice(&event.data),
            t if t == EventType::Finished => {
                result2 = event.final_result;
                break;
            }
            _ => {}
        }
    }

    assert_eq!(
        result2
            .unwrap()
            .report
            .unwrap()
            .termination
            .unwrap()
            .reason
            .unwrap(),
        Reason::ExitCode(0)
    );
    assert_eq!(String::from_utf8_lossy(&stdout2), "Worker still alive\n");
}

#[tokio::test]
async fn grpc_streams_execution_events() {
    let mut client = setup_grpc_test_client().await;

    let script = r#"
import sys
import time

sys.stdout.write('hello\n')
sys.stdout.flush()

time.sleep(0.05)

sys.stdout.write('world\n')
sys.stdout.flush()
"#;

    let request = ExecuteRequest {
        language: "python".to_string(),
        files: vec![SourceFile {
            path: "main.py".to_string(),
            contents: script.as_bytes().to_vec(),
        }],
        stdin: vec![],
    };

    let mut stream = client.execute(request).await.unwrap().into_inner();

    let mut received_started = false;
    let mut combined_stdout = Vec::new();
    let mut final_result = None;

    while let Some(event) = stream.message().await.unwrap() {
        match event.r#type() {
            t if t == EventType::Started => received_started = true,
            t if t == EventType::Stdout => combined_stdout.extend_from_slice(&event.data),
            t if t == EventType::Finished => {
                final_result = event.final_result;
                break;
            }
            _ => {}
        }
    }

    assert!(received_started, "Did not receive Started event");
    assert_eq!(String::from_utf8_lossy(&combined_stdout), "hello\nworld\n");

    let result = final_result.unwrap();
    let report = result.report.unwrap();
    assert_eq!(
        report.termination.unwrap().reason.unwrap(),
        Reason::ExitCode(0)
    );
    assert_eq!(
        String::from_utf8_lossy(&report.output.unwrap().stdout),
        "hello\nworld\n"
    );
}

use std::time::Instant;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn grpc_executes_multiple_jobs_concurrently_without_mixing() {
    let mut client1 = setup_grpc_test_client().await;
    // We clone the client channel to simulate multiple independent concurrent users
    let mut client2 = client1.clone();
    let mut client3 = client1.clone();

    // A script that takes ~1.5 seconds, printing its identity along the way.
    // If run sequentially, 3 of these would take 4.5+ seconds.
    let make_script = |identity: &str| {
        format!(
            r#"
import time
import sys

print("START {id}")
sys.stdout.flush()
time.sleep(0.5)

print("MID {id}")
sys.stdout.flush()
time.sleep(0.5)

print("END {id}", file=sys.stderr)
sys.stderr.flush()
"#,
            id = identity
        )
    };

    let start_time = Instant::now();

    // Spawn 3 concurrent tasks simulating 3 distinct clients
    let task1 = tokio::spawn(async move {
        let req = ExecuteRequest {
            language: "python".to_string(),
            files: vec![SourceFile {
                path: "main.py".to_string(),
                contents: make_script("A").into_bytes(),
            }],
            stdin: vec![],
        };
        let mut stream = client1.execute(req).await.unwrap().into_inner();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        while let Some(event) = stream.message().await.unwrap() {
            match event.r#type() {
                t if t == EventType::Stdout => stdout.extend_from_slice(&event.data),
                t if t == EventType::Stderr => stderr.extend_from_slice(&event.data),
                _ => {}
            }
        }
        (
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    });

    let task2 = tokio::spawn(async move {
        let req = ExecuteRequest {
            language: "python".to_string(),
            files: vec![SourceFile {
                path: "main.py".to_string(),
                contents: make_script("B").into_bytes(),
            }],
            stdin: vec![],
        };
        let mut stream = client2.execute(req).await.unwrap().into_inner();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        while let Some(event) = stream.message().await.unwrap() {
            match event.r#type() {
                t if t == EventType::Stdout => stdout.extend_from_slice(&event.data),
                t if t == EventType::Stderr => stderr.extend_from_slice(&event.data),
                _ => {}
            }
        }
        (
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    });

    let task3 = tokio::spawn(async move {
        let req = ExecuteRequest {
            language: "python".to_string(),
            files: vec![SourceFile {
                path: "main.py".to_string(),
                contents: make_script("C").into_bytes(),
            }],
            stdin: vec![],
        };
        let mut stream = client3.execute(req).await.unwrap().into_inner();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        while let Some(event) = stream.message().await.unwrap() {
            match event.r#type() {
                t if t == EventType::Stdout => stdout.extend_from_slice(&event.data),
                t if t == EventType::Stderr => stderr.extend_from_slice(&event.data),
                _ => {}
            }
        }
        (
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    });

    // Await all 3 jobs simultaneously
    let (res1, res2, res3) = tokio::join!(task1, task2, task3);
    let (out1, err1) = res1.unwrap();
    let (out2, err2) = res2.unwrap();
    let (out3, err3) = res3.unwrap();

    let elapsed = start_time.elapsed();

    // 1. Assert no contamination between streams
    assert_eq!(out1, "START A\nMID A\n");
    assert_eq!(err1, "END A\n");

    assert_eq!(out2, "START B\nMID B\n");
    assert_eq!(err2, "END B\n");

    assert_eq!(out3, "START C\nMID C\n");
    assert_eq!(err3, "END C\n");

    // 2. Assert they ran concurrently.
    // 3 sequential jobs would take ~3.0s. If it takes < 2.5s, they ran in parallel.
    assert!(
        elapsed.as_secs_f32() < 2.5,
        "Jobs did not run concurrently! Elapsed: {:?}",
        elapsed
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn grpc_cancellation_kills_process_and_frees_threads() {
    let mut client = setup_grpc_test_client().await;
    let mut join_handles = vec![];

    // 1. Exhaust all 4 worker threads with a 10-second sleep job.
    for _ in 0..4 {
        let mut client_clone = client.clone();

        let handle = tokio::spawn(async move {
            let req = ExecuteRequest {
                language: "python".to_string(),
                files: vec![SourceFile {
                    path: "main.py".to_string(),
                    contents: b"import time; time.sleep(10)".to_vec(),
                }],
                stdin: vec![],
            };

            // Start the execution stream
            let mut stream = client_clone.execute(req).await.unwrap().into_inner();

            // Wait for the Started event to ensure the blocking thread is actively held
            if let Some(event) = stream.message().await.unwrap() {
                assert_eq!(event.r#type(), EventType::Started);
            }

            // Deliberately DROP the stream here. This mimics a client disconnecting.
            // When `stream` is dropped, tonic sends an HTTP/2 cancel frame,
            // the `grpc_tx` channel drops, and our `cancel_flag` flips to true.
            drop(stream);
        });
        join_handles.push(handle);
    }

    // Wait for all 4 clients to connect, start the job, and then disconnect
    for handle in join_handles {
        let _ = handle.await;
    }

    // Give Rust a brief moment to process the HTTP/2 RST_STREAM and kill the sandboxes
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 2. Send a 5th quick job with a strict 2-second timeout
    // If cancellation failed, the 4 threads are still sleeping for 10s, and this will timeout.
    let req5 = ExecuteRequest {
        language: "python".to_string(),
        files: vec![SourceFile {
            path: "main.py".to_string(),
            contents: b"print('I am free!')".to_vec(),
        }],
        stdin: vec![],
    };

    let future = async {
        let mut stream5 = client.execute(req5).await.unwrap().into_inner();
        let mut stdout = Vec::new();

        while let Some(event) = stream5.message().await.unwrap() {
            if event.r#type() == EventType::Stdout {
                stdout.extend_from_slice(&event.data);
            }
        }
        stdout
    };

    // Require the 5th job to finish in under 2 seconds.
    // It can only do this if a thread was freed.
    let result = timeout(Duration::from_secs(2), future).await;

    assert!(
        result.is_ok(),
        "5th job timed out! Cancellation failed to free a blocking thread."
    );

    let stdout = result.unwrap();
    assert_eq!(String::from_utf8(stdout).unwrap(), "I am free!\n");
}
