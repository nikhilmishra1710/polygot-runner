use runtime_worker::worker::grpc_server::pb::TerminationReason;
use runtime_worker::worker::grpc_server::pb::termination_reason::Reason;
use std::time::Duration;
use tokio::net::TcpListener;
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
