use std::{
    io,
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::{Arc, atomic::AtomicBool, mpsc},
    thread,
};

use tracing::{debug, error, info};

use crate::{
    model::ExecutionEvent,
    protocol::{self, WorkerRequest, WorkerResponse},
    worker::Worker,
};

pub struct WorkerServer {
    worker: Worker,
}

impl WorkerServer {
    pub fn new(worker: Worker) -> Self {
        debug!("Creating new worker server");
        Self { worker }
    }

    pub fn bind(&self, path: &Path) -> io::Result<()> {
        info!("Binding worker socket to {}", path.display());
        if path.exists() {
            debug!("Removing existing socket file");
            std::fs::remove_file(path)?;
        }

        let listener = UnixListener::bind(path)?;
        info!("Worker socket bound to {}", path.display());

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    info!("Accepted new connection");
                    debug!("handle_connection started");
                    let should_shutdown = self.handle_connection(stream)?;
                    debug!("handle_connection returned");

                    if should_shutdown {
                        info!("Shutdown requested, breaking server loop");
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    // Continue to next connection
                }
            }
        }

        if path.exists() {
            debug!("Cleaning up socket file");
            let _ = std::fs::remove_file(path);
            debug!("Cleaned up socket file");
        }

        info!("Worker server stopped");
        Ok(())
    }

    fn handle_connection(&self, mut stream: UnixStream) -> io::Result<bool> {
        debug!("Handling new connection");

        loop {
            let request: WorkerRequest = match protocol::receive(&mut stream) {
                Ok(request) => {
                    debug!("Received request: {:?}", request);
                    request
                }
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                    debug!("Client disconnected unexpectedly (EOF)");
                    break;
                }
                Err(error) => {
                    error!("Failed to receive request: {}", error);
                    let response = WorkerResponse::Error(error.to_string());
                    if let Err(e) = protocol::send(&mut stream, &response) {
                        error!("Failed to send error response: {}", e);
                    }
                    break;
                }
            };

            match request {
                WorkerRequest::Execute(job) => {
                    info!("Executing job with id: {:?}", job.id);

                    // Create the channel INSIDE the loop so each job gets a fresh event stream
                    let (event_tx, event_rx) = mpsc::sync_channel::<ExecutionEvent>(128);
                    let cancel_flag = Arc::new(AtomicBool::new(false));
                    let cancel_clone = Arc::clone(&cancel_flag);
                    // Use thread::scope so we can safely borrow `self` in the background thread
                    let response = thread::scope(|s| {
                        // 1. Spawn the worker execution in a scoped background thread
                        let worker_handle = s.spawn(|| {
                            match self.worker.execute_with_events(job, event_tx, cancel_flag) {
                                Ok(result) => {
                                    debug!("Job executed successfully");
                                    WorkerResponse::Result(result)
                                }
                                Err(error) => {
                                    error!("Job execution failed: {}", error);
                                    WorkerResponse::Error(error.to_string())
                                }
                            }
                        });

                        // 2. Actively drain the event channel on the main thread!
                        // This prevents the sync_channel from filling up and deadlocking the worker.
                        for event in event_rx {
                            debug!("Live event from worker: {:?}", event);
                            let _ = protocol::send(&mut stream, &WorkerResponse::Event(event));
                        }

                        // 3. Wait for the final response (the channel closes when the worker finishes, so the `for` loop will exit)
                        worker_handle.join().unwrap()
                    });

                    // 4. Send the final response to the stream
                    if let Err(e) = protocol::send(&mut stream, &response) {
                        error!("Failed to send response: {}", e);
                        break;
                    }
                    debug!("Response sent successfully");
                }

                WorkerRequest::Shutdown => {
                    info!("Shutdown command received");
                    debug!("Worker receive shutdown");
                    break;
                }
            }
        }
        debug!("Connection handling finished");
        Ok(true)
    }
}
