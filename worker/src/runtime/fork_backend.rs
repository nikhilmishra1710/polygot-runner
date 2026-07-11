use crate::{
    error::WorkerError,
    runtime::{
        ChildBootstrap, ChildCoordinator, ParentCoordinator, RunningProcess, RuntimeCommand,
        backend::ProcessBackend,
        unix::{close, pipe},
    },
    sandbox::{MountNamespace, NamespaceManager},
};
use std::io;

pub struct ForkBackend;

impl ProcessBackend for ForkBackend {
    fn launch(&self, command: &RuntimeCommand) -> Result<RunningProcess, WorkerError> {
        // fork() returns a rustix::io::Result<Option<Pid>>
        // Note: The `?` operator requires WorkerError to implement From<rustix::io::Errno>
        let stdin_pipe = pipe()?;
        let stdout_pipe = pipe()?;
        let stderr_pipe = pipe()?;
        let parent_pipe = pipe()?;
        let child_pipe = pipe()?;
        let parent_coordinator = ParentCoordinator::new(parent_pipe.read, child_pipe.write)?;
        let child_coodinator = ChildCoordinator::new(child_pipe.read, parent_pipe.write)?;
        let pid = unsafe { libc::fork() };

        match pid {
            -1 => {
                return Err(io::Error::last_os_error().into());
            }

            0 => {
                close(stdin_pipe.write);
                close(stdout_pipe.read);
                close(stderr_pipe.read);
                println!("NamespaceManager start");
                NamespaceManager::enter_user_namespace()?;
                println!("NamespaceManager end");
                child_coodinator.namespace_created()?;
                MountNamespace::enter()?;
                MountNamespace::make_private()?;
                child_coodinator.wait_for_parent()?;

                ChildBootstrap::new(
                    command,
                    stdin_pipe.read,
                    stdout_pipe.write,
                    stderr_pipe.write,
                )
                .run()
            }

            pid => {
                close(stdin_pipe.read);
                close(stdout_pipe.write);
                close(stderr_pipe.write);

                parent_coordinator.wait_for_namespace()?;
                NamespaceManager::finish_user_namespace(pid)?;
                parent_coordinator.continue_child()?;

                Ok(RunningProcess::from_fork(
                    pid,
                    Some(stdin_pipe.write),
                    Some(stdout_pipe.read),
                    Some(stderr_pipe.read),
                ))
            }
        }
    }
}
