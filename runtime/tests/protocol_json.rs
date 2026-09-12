//! Verifies the worker protocol wire format is JSON with the exact shape
//! the Go API client (api/internal/worker) speaks. If this test fails after
//! changing serde attributes or payload types, update the Go client to match.

mod common;

use runtime_worker::{
    job::{ExecutionJob, JobId},
    protocol::{receive, send, WorkerRequest, WorkerResponse},
};

// The exact JSON the Go UnixWorkerClient sends for an Execute request.
const GO_EXECUTE_JSON: &str = r#"{
  "Execute": {
    "id": "go-client-exec",
    "request": {
      "language": "Python",
      "files": [{"path": "main.py", "contents": [112, 114, 105, 110, 116]}],
      "stdin": [],
      "limits": {
        "wall_time": {"secs": 10, "nanos": 0},
        "cpu_time": {"secs": 10, "nanos": 0},
        "max_open_files": 128,
        "max_file_size": 10485760,
        "memory_bytes": 268435456,
        "pids_max": 64
      }
    }
  }
}"#;

#[test]
fn decodes_go_client_execute_request() {
    let request: WorkerRequest =
        serde_json::from_str(GO_EXECUTE_JSON).expect("Go request JSON must deserialize");

    match request {
        WorkerRequest::Execute(job) => {
            assert_eq!(job.id, JobId("go-client-exec".to_string()));
            assert_eq!(job.request.files.len(), 1);
            assert_eq!(job.request.files[0].path.to_str().unwrap(), "main.py");
            assert_eq!(job.request.files[0].contents, b"print");
            assert_eq!(job.request.limits.max_open_files, 128);
        }
        WorkerRequest::Shutdown => panic!("expected Execute"),
    }
}

#[test]
fn request_round_trip_through_framed_codec() {
    let job = common::python_job("rt-1", "print(1)");

    let mut buffer = Vec::new();
    send(&mut buffer, &WorkerRequest::Execute(job.clone())).unwrap();

    let decoded: WorkerRequest = receive(&mut buffer.as_slice()).unwrap();

    match decoded {
        WorkerRequest::Execute(decoded_job) => {
            assert_eq!(decoded_job.id, job.id);
            assert_eq!(
                decoded_job.request.files[0].contents,
                job.request.files[0].contents
            );
        }
        WorkerRequest::Shutdown => panic!("expected Execute"),
    }
}

#[test]
fn error_response_shape_is_worker_error_variant() {
    let response = WorkerResponse::Error("workspace failed".to_string());

    let mut buffer = Vec::new();
    send(&mut buffer, &response).unwrap();
    let frame = read_payload(&buffer);

    // Go client looks for {"Error": "..."} externally-tagged shape.
    let v: serde_json::Value = serde_json::from_slice(&frame).unwrap();
    assert_eq!(v, serde_json::json!({"Error": "workspace failed"}));
}

#[test]
fn shutdown_request_shape() {
    let mut buffer = Vec::new();
    send(&mut buffer, &WorkerRequest::Shutdown).unwrap();
    let frame = read_payload(&buffer);

    // Externally-tagged unit variant serializes as a bare string in serde.
    let v: serde_json::Value = serde_json::from_slice(&frame).unwrap();
    assert_eq!(v, serde_json::json!("Shutdown"));
}

fn read_payload(buffer: &[u8]) -> Vec<u8> {
    // Skip 4-byte big-endian length prefix.
    buffer[4..].to_vec()
}

// Silence unused warning for helpers used by other test targets.
#[allow(dead_code)]
fn _unused(_: &ExecutionJob) {}
