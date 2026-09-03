use std::{
    io,
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
};

use tracing::debug;

use crate::{
    protocol::{self, WorkerRequest, WorkerResponse},
    worker::Worker,
};

pub struct WorkerServer {
    worker: Worker,
}

impl WorkerServer {
    pub fn new(worker: Worker) -> Self {
        Self { worker }
    }

    pub fn bind(&self, path: &Path) -> io::Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }

        let listener = UnixListener::bind(path)?;

        for stream in listener.incoming() {
            debug!("handle_connection started");
            let stream = stream?;

            let should_shutdown = self.handle_connection(stream)?;
            debug!("handle_connection returned");

            if should_shutdown {
                debug!("Shutting down server loop");
                break;
            }
        }

        if path.exists() {
            let _ = std::fs::remove_file(path);
            debug!("Cleaned up socket file");
        }

        Ok(())
    }

    fn handle_connection(&self, mut stream: UnixStream) -> io::Result<bool> {
        loop {
            let request: WorkerRequest = match protocol::receive(&mut stream) {
                Ok(request) => request,

                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                    break;
                }

                Err(error) => {
                    let response = WorkerResponse::Error(error.to_string());

                    protocol::send(&mut stream, &response)?;

                    break;
                }
            };

            match request {
                WorkerRequest::Execute(job) => {
                    let response = match self.worker.execute(job) {
                        Ok(result) => WorkerResponse::Result(result),

                        Err(error) => WorkerResponse::Error(error.to_string()),
                    };

                    protocol::send(&mut stream, &response)?;
                }

                WorkerRequest::Shutdown => {
                    debug!("Worker receive shutdown");
                    break;
                }
            }
        }

        Ok(true)
    }
}
