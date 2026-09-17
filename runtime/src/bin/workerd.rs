// src/bin/workerd.rs

use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use runtime_worker::worker::grpc_server::{
    GrpcWorkerServer, pb::execution_service_server::ExecutionServiceServer,
};
use runtime_worker::worker::worker::Worker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Configure the gRPC address (0.0.0.0 allows external connections for scaling)
    let addr: SocketAddr = "0.0.0.0:50051".parse()?;

    info!("Starting Worker gRPC Server on {}", addr);

    let worker = Worker::new(); // Ensure your Worker struct implements Clone
    let grpc_server = GrpcWorkerServer::new(worker);

    Server::builder()
        .add_service(ExecutionServiceServer::new(grpc_server))
        .serve(addr)
        .await?;

    Ok(())
}
