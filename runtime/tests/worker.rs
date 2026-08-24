use runtime_worker::{
    job::JobId,
    worker::{Worker, WorkerCommand},
};

mod common;

use crate::common::python_job;

#[test]
fn worker_processes_multiple_jobs() {
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let (result_tx, result_rx) = std::sync::mpsc::channel();

    let worker = Worker::new();

    let handle = std::thread::spawn(move || {
        worker.run(command_rx, result_tx).unwrap();
    });

    command_tx
        .send(WorkerCommand::Execute(python_job(
            "job-1",
            "print('job 1')",
        )))
        .unwrap();

    command_tx
        .send(WorkerCommand::Execute(python_job(
            "job-2",
            "print('job 2')",
        )))
        .unwrap();

    command_tx.send(WorkerCommand::Shutdown).unwrap();

    let result1 = result_rx.recv().unwrap();
    let result2 = result_rx.recv().unwrap();

    assert_eq!(result1.id, JobId("job-1".into()));
    assert_eq!(result2.id, JobId("job-2".into()));

    handle.join().unwrap();
}

#[test]
fn failed_job_does_not_stop_worker() {
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let (result_tx, result_rx) = std::sync::mpsc::channel();

    let worker = Worker::new();

    let handle = std::thread::spawn(move || {
        worker.run(command_rx, result_tx).unwrap();
    });

    command_tx
        .send(WorkerCommand::Execute(python_job(
            "failed",
            "raise Exception('failure')",
        )))
        .unwrap();

    command_tx
        .send(WorkerCommand::Execute(python_job(
            "success",
            "print('hello')",
        )))
        .unwrap();

    command_tx.send(WorkerCommand::Shutdown).unwrap();

    let failed = result_rx.recv().unwrap();
    let success = result_rx.recv().unwrap();

    assert_eq!(failed.id, JobId("failed".into()));
    assert!(!failed.report.termination.is_success());

    assert_eq!(success.id, JobId("success".into()));
    assert!(success.report.termination.is_success());

    handle.join().unwrap();
}

#[test]
fn worker_shuts_down() {
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let (result_tx, _result_rx) = std::sync::mpsc::channel();

    let worker = Worker::new();

    let handle = std::thread::spawn(move || {
        worker.run(command_rx, result_tx).unwrap();
    });

    command_tx.send(WorkerCommand::Shutdown).unwrap();

    assert!(handle.join().is_ok());
}

#[test]
fn worker_processes_jobs_sequentially() {
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let (result_tx, result_rx) = std::sync::mpsc::channel();

    let worker = Worker::new();

    let handle = std::thread::spawn(move || {
        worker.run(command_rx, result_tx).unwrap();
    });

    command_tx
        .send(WorkerCommand::Execute(python_job("job-1", "print('one')")))
        .unwrap();

    command_tx
        .send(WorkerCommand::Execute(python_job("job-2", "print('two')")))
        .unwrap();

    let result1 = result_rx.recv().unwrap();
    let result2 = result_rx.recv().unwrap();

    assert_eq!(result1.id, JobId("job-1".into()));
    assert_eq!(result2.id, JobId("job-2".into()));

    assert!(result1.report.termination.is_success());
    assert!(result2.report.termination.is_success());

    command_tx.send(WorkerCommand::Shutdown).unwrap();

    handle.join().unwrap();
}
