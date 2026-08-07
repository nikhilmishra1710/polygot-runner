use crate::{
    cgroup::ExecutionCgroup,
    error::WorkerError,
    runtime::{
        ChildCoordinator, InitProcess, ParentCoordinator, RunningProcess,
        RuntimeCommand,
        backend::ProcessBackend,
        unix::{close, pipe},
    },
    sandbox::{MountNamespace, NamespaceManager},
};
use std::io;

pub struct ForkBackend;

impl ProcessBackend for ForkBackend {
    fn launch(
        &self,
        command: &RuntimeCommand,
        cgroup: &ExecutionCgroup,
    ) -> Result<RunningProcess, WorkerError> {
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
                if let Err(e) = cgroup.attach(0) {
                    eprintln!("Fatal: Child failed to attach to cgroup: {}", e);
                    std::process::exit(1);
                }

                unsafe {
                    if libc::setpgid(0, 0) != 0 {
                        return Err(io::Error::last_os_error().into());
                    }
                }

                close(stdin_pipe.write);
                close(stdout_pipe.read);
                close(stderr_pipe.read);
                println!("NamespaceManager start");
                NamespaceManager::enter_user_namespace()?;
                println!("NamespaceManager end");
                child_coodinator.namespace_created()?;
                child_coodinator.wait_for_parent()?;
                MountNamespace::enter()?;
                MountNamespace::make_private()?;

                unsafe {
                    if libc::unshare(libc::CLONE_NEWPID) != 0 {
                        return Err(io::Error::last_os_error().into());
                    }
                }

                // 2. FORK AGAIN. This grandchild will be placed INTO the new PID namespace.
                let inner_pid = unsafe { libc::fork() };
                match inner_pid {
                    -1 => {
                        return Err(io::Error::last_os_error().into());
                    }
                    0 => {
                        // We are now PID 1 inside the new PID namespace!
                        // mount_proc() will now succeed.
                        unsafe {
                            // Tell the Linux kernel: "If my parent (the Middle Child) dies, kill me immediately with SIGKILL."
                            // This ensures no orphaned sandboxes can survive a timeout.
                            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
                        }
                        InitProcess::run(
                            command,
                            stdin_pipe.read,
                            stdout_pipe.write,
                            stderr_pipe.write,
                        )
                    }
                    pid => {
                        close(stdin_pipe.read);
                        close(stdout_pipe.write);
                        close(stderr_pipe.write);
                        // The middle process waits for the actual payload to finish executing
                        let mut status = 0;
                        unsafe { libc::waitpid(pid, &mut status, 0) };

                        let exit_code = if libc::WIFEXITED(status) {
                            println!("killed wiith status: {status}");
                            libc::WEXITSTATUS(status)
                        } else if libc::WIFSIGNALED(status) {
                            // Standard Linux convention: 128 + signal number
                            println!("killed wiith status: 128 + {status}");
                            128 + libc::WTERMSIG(status)
                        } else {
                            println!("killed wiith status: {status}");
                            1 // fallback error
                        };

                        unsafe { libc::_exit(exit_code) };
                    }
                }
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
