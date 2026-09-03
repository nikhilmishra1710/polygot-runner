use std::path::Path;

use runtime_worker::worker::{Worker, WorkerServer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let worker = Worker::new();

    let server = WorkerServer::new(worker);

    server.bind(Path::new("/tmp/runtime-worker.sock"))?;

    Ok(())
}
