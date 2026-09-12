use std::sync::mpsc::{Receiver, Sender};

use crate::{
    engine::ExecutionEngine, error::WorkerError, job::{ExecutionJob, JobResult},
};
use tracing::{debug, info};

use super::WorkerCommand;

pub struct Worker {
    engine: ExecutionEngine,
}

impl Worker {
    pub fn new() -> Self {
        debug!("Creating new worker");
        Self {
            engine: ExecutionEngine::new(),
        }
    }

    pub fn execute(
        &self,
        job: ExecutionJob,
    ) -> Result<JobResult, WorkerError> {
        let id = job.id.clone();
        info!("Executing job with id: {:?}", id);
        let report = self.engine.execute(&job.request)?;
        debug!("Job executed successfully");
        Ok(JobResult { id, report })
    }

    pub fn run(
        self,
        receiver: Receiver<WorkerCommand>,
        sender: Sender<JobResult>,
    ) -> Result<(), WorkerError> {
        info!("Worker started, waiting for commands");
        for command in receiver {
            debug!("Received command: {:?}", command);
            match command {
                WorkerCommand::Execute(job) => {
                    info!("Executing job with id: {:?}", job.id);
                    let result = self.execute(job)?;
                    debug!("Job executed, sending result");
                    if let Err(e) = sender.send(result) {
                        return Err(WorkerError::Io(std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            format!("worker result receiver dropped: {}", e),
                        )));
                    }
                    debug!("Result sent successfully");
                }

                WorkerCommand::Shutdown => {
                    info!("Shutdown command received, stopping worker");
                    break;
                }
            }
        }
        info!("Worker stopped");
        Ok(())
    }
}