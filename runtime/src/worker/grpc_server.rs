// src/worker/grpc_server.rs

use std::sync::{Arc, mpsc};
use tokio::sync::mpsc as tokio_mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

// 1. Include the generated gRPC code
pub mod pb {
    tonic::include_proto!("execution.v1");
}

use pb::execution_service_server::ExecutionService;
use pb::{ExecuteRequest, ExecutionEvent, execution_event::EventType};

use crate::model::TerminationReason as InternalTermination;
use pb::termination_reason::Reason as ProtoReason;
use std::time::Duration;

use crate::job::{ExecutionJob, JobId};
use crate::model::ExecutionEvent as InternalEvent;
use crate::model::{ExecutionRequest, Language, ResourceLimits, SourceFile, TerminationReason};
use crate::worker::worker::Worker;

// 2. Define the Server Struct
pub struct GrpcWorkerServer {
    worker: Arc<Worker>,
}

impl GrpcWorkerServer {
    pub fn new(worker: Worker) -> Self {
        Self {
            worker: Arc::new(worker),
        }
    }
}

// 3. Implement the gRPC Service Trait
#[tonic::async_trait]
impl ExecutionService for GrpcWorkerServer {
    type ExecuteStream = ReceiverStream<Result<ExecutionEvent, Status>>;

    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<Self::ExecuteStream>, Status> {
        let req = request.into_inner();

        // Convert Proto Request -> Internal Request
        let internal_req = map_proto_to_internal(req).map_err(|e| Status::invalid_argument(e))?;

        let job_id = uuid::Uuid::new_v4().to_string();
        let job = ExecutionJob {
            id: JobId(job_id.clone()),
            request: internal_req,
        };

        // Create channels for bridging Sync Rust to Async gRPC
        let (grpc_tx, grpc_rx) = tokio_mpsc::channel(128);
        let (sync_tx, sync_rx) = mpsc::sync_channel::<InternalEvent>(128);

        // We clone the worker so we can move it into the blocking thread
        let worker = Arc::clone(&self.worker);

        // Spawn a blocking thread to run the execution engine
        tokio::task::spawn_blocking(move || {
            let _ = worker.execute_with_events(job, sync_tx.clone());
            // When this function finishes, sync_tx is dropped,
            // which cleanly ends the bridging loop below.
        });

        // Spawn an async task to forward events to the gRPC stream
        tokio::spawn(async move {
            // Read from the sync engine
            for event in sync_rx {
                let proto_event = match event {
                    InternalEvent::Started => ExecutionEvent {
                        r#type: EventType::Started.into(),
                        data: vec![],
                        final_result: None,
                    },
                    InternalEvent::Stdout(data) => ExecutionEvent {
                        r#type: EventType::Stdout.into(),
                        data,
                        final_result: None,
                    },
                    InternalEvent::Stderr(data) => ExecutionEvent {
                        r#type: EventType::Stderr.into(),
                        data,
                        final_result: None,
                    },

                    InternalEvent::Finished { result } => ExecutionEvent {
                        r#type: EventType::Finished.into(),
                        data: vec![],
                        final_result: Some(map_job_result_to_proto(result)),
                    }, // Note: If you added a Finished variant to your InternalEvent, map it here!
                };

                // Send to the gRPC stream
                if grpc_tx.send(Ok(proto_event)).await.is_err() {
                    break; // Client disconnected
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(grpc_rx)))
    }
}

// Helper mappings (Implement these based on your specific struct fields)
fn map_proto_to_internal(req: ExecuteRequest) -> Result<ExecutionRequest, String> {
    let language = match req.language.as_str() {
        "python" => Language::Python,
        "cpp" => Language::Cpp,
        _ => return Err("Unsupported language".to_string()),
    };

    let files = req
        .files
        .into_iter()
        .map(|f| SourceFile {
            path: f.path.into(),
            contents: f.contents,
        })
        .collect();

    Ok(ExecutionRequest {
        language,
        files,
        stdin: req.stdin,
        limits: ResourceLimits::default(), // Or map from proto if added later
    })
}

// Helper to convert std::time::Duration to prost-types Duration
fn to_proto_duration(d: Duration) -> prost_types::Duration {
    prost_types::Duration {
        seconds: d.as_secs() as i64,
        nanos: d.subsec_nanos() as i32,
    }
}

fn map_job_result_to_proto(result: crate::job::JobResult) -> pb::JobResult {
    let termination_reason = match result.report.termination {
        InternalTermination::ExitCode(c) => ProtoReason::ExitCode(c),
        InternalTermination::Signal(s) => ProtoReason::Signal(s),
        InternalTermination::WallTimeout => ProtoReason::WallTimeout(true),
        InternalTermination::CpuLimit => ProtoReason::CpuLimit(true),
        InternalTermination::MemoryLimit => ProtoReason::MemoryLimit(true),
        InternalTermination::OomKilled => ProtoReason::OomKilled(true),
        InternalTermination::SeccompViolation => ProtoReason::SeccompViolation(true),
    };

    pb::JobResult {
        id: result.id.0,
        report: Some(pb::ExecutionReport {
            output: Some(pb::Output {
                stdout: result.report.output.stdout,
                stderr: result.report.output.stderr,
            }),
            termination: Some(pb::TerminationReason {
                reason: Some(termination_reason),
            }),
            metrics: Some(pb::ExecutionMetrics {
                wall_time: Some(to_proto_duration(result.report.metrics.wall_time)),
                cpu_time: Some(to_proto_duration(result.report.metrics.cpu_time)),
                max_rss_bytes: result.report.metrics.max_rss_bytes,
                peak_pids: result.report.metrics.peak_pids,
            }),
        }),
    }
}
