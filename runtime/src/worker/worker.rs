use std::sync::mpsc::{Receiver, Sender};

use crate::{
    engine::ExecutionEngine, error::WorkerError, job::{ExecutionJob, JobResult},
};

use super::WorkerCommand;

pub struct Worker {
    engine: ExecutionEngine,
}

impl Worker {
    pub fn new() -> Self {
        Self {
            engine: ExecutionEngine::new(),
        }
    }

    pub fn execute(
        &self,
        job: ExecutionJob,
    ) -> Result<JobResult, WorkerError> {
        let id = job.id.clone();

        let report = self.engine.execute(&job.request)?;

        Ok(JobResult { id, report })
    }

    pub fn run(
        self,
        receiver: Receiver<WorkerCommand>,
        sender: Sender<JobResult>,
    ) -> Result<(), WorkerError> {
        for command in receiver {
            match command {
                WorkerCommand::Execute(job) => {
                    let result = self.execute(job)?;

                    sender
                        .send(result)
                        .map_err(|_| {
                            WorkerError::Io(std::io::Error::new(
                                std::io::ErrorKind::BrokenPipe,
                                "worker result receiver dropped",
                            ))
                        })?;
                }

                WorkerCommand::Shutdown => {
                    break;
                }
            }
        }

        Ok(())
    }
}