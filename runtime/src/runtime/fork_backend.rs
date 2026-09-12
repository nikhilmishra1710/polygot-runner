use crate::{
    cgroup::ExecutionCgroup,
    error::WorkerError,
    runtime::{
        ChildCoordinator, InitProcess, ParentCoordinator, RunningProcess, RuntimeCommand,
        backend::ProcessBackend,
        unix::{close, pipe},
    },
    sandbox::{MountNamespace, NamespaceManager},
};
use std::io;
use tracing::{debug, error};

pub struct ForkBackend;

impl ProcessBackend for ForkBackend {
    fn launch(
        &self,
        command: &RuntimeCommand,
        cgroup: &ExecutionCgroup,
    ) -> Result<RunningProcess, WorkerError> {
        debug!("Launching process with fork_backend");
        // fork() returns a rustix::io::Result<Option<Pid>>
        // Note: The `?` operator requires WorkerError to implement From<rustix::io::Errno>
        let stdin_pipe = pipe()
            .map_err(|e| {
                error!("Failed to create stdin pipe: {}", e);
                WorkerError::from(e)
            })?;
        debug!("Stdin pipe created");
        let stdout_pipe = pipe()
            .map_err(|e| {
                error!("Failed to create stdout pipe: {}", e);
                WorkerError::from(e)
            })?;
        debug!("Stdout pipe created");
        let stderr_pipe = pipe()
            .map_err(|e| {
                error!("Failed to create stderr pipe: {}", e);
                WorkerError::from(e)
            })?;
        debug!("Stderr pipe created");
        let parent_pipe = pipe()
            .map_err(|e| {
                error!("Failed to create parent pipe: {}", e);
                WorkerError::from(e)
            })?;
        debug!("Parent pipe created");
        let child_pipe = pipe()
            .map_err(|e| {
                error!("Failed to create child pipe: {}", e);
                WorkerError::from(e)
            })?;
        debug!("Child pipe created");

        let parent_coordinator = ParentCoordinator::new(parent_pipe.read, child_pipe.write)
            .map_err(|e| {
                error!("Failed to create parent coordinator: {}", e);
                WorkerError::from(e)
            })?;
        debug!("Parent coordinator created");
        let child_coodinator = ChildCoordinator::new(child_pipe.read, parent_pipe.write)
            .map_err(|e| {
                error!("Failed to create child coordinator: {}", e);
                WorkerError::from(e)
            })?;
        debug!("Child coordinator created");

        debug!("Calling libc::fork()");
        let pid = unsafe { libc::fork() };
        match pid {
            -1 => {
                error!("Fork failed: {}", io::Error::last_os_error());
                return Err(WorkerError::from(io::Error::last_os_error()));
            }

            0 => {
                debug!("In first child process (PID: {})", unsafe { libc::getpid() });
                if let Err(e) = cgroup.attach(0) {
                    error!("Child failed to attach to cgroup: {}", e);
                    std::process::exit(1);
                }
                debug!("Attached to cgroup");

                unsafe {
                    if libc::setpgid(0, 0) != 0 {
                        error!(
                            "Failed to set process group: {}",
                            io::Error::last_os_error()
                        );
                        return Err(WorkerError::from(io::Error::last_os_error()));
                    }
                }
                debug!("Set process group");

                close(stdin_pipe.write);
                close(stdout_pipe.read);
                close(stderr_pipe.read);
                debug!("Closed unnecessary pipe ends in child");

                if let Err(e) = NamespaceManager::enter_user_namespace() {
                    error!("Failed to enter user namespace: {}", e);
                    return Err(WorkerError::from(e));
                }
                debug!("Entered user namespace");

                if let Err(e) = child_coodinator.namespace_created() {
                    error!("Failed to signal namespace created: {}", e);
                    return Err(WorkerError::from(e));
                }
                debug!("Signaled namespace created");

                if let Err(e) = child_coodinator.wait_for_parent() {
                    error!("Failed to wait for parent: {}", e);
                    return Err(WorkerError::from(e));
                }
                debug!("Waited for parent");

                if let Err(e) = MountNamespace::enter() {
                    error!("Failed to enter mount namespace: {}", e);
                    return Err(WorkerError::from(e));
                }
                debug!("Entered mount namespace");

                if let Err(e) = MountNamespace::make_private() {
                    error!("Failed to make mount namespace private: {}", e);
                    return Err(WorkerError::from(e));
                }
                debug!("Made mount namespace private");

                unsafe {
                    if libc::unshare(libc::CLONE_NEWPID) != 0 {
                        error!(
                            "Failed to unshare PID namespace: {}",
                            io::Error::last_os_error()
                        );
                        return Err(WorkerError::from(io::Error::last_os_error()));
                    }
                }
                debug!("Unshared PID namespace");

                // 2. FORK AGAIN. This grandchild will be placed INTO the new PID namespace.
                debug!("Calling second libc::fork()");
                let inner_pid = unsafe { libc::fork() };
                match inner_pid {
                    -1 => {
                        error!("Second fork failed: {}", io::Error::last_os_error());
                        return Err(WorkerError::from(io::Error::last_os_error()));
                    }
                    0 => {
                        // We are now PID 1 inside the new PID namespace!
                        debug!("In second grandchild process (PID: {})", unsafe { libc::getpid() });
                        // mount_proc() will now succeed.
                        unsafe {
                            // Tell the Linux kernel: "If my parent (the Middle Child) dies, kill me immediately with SIGKILL."
                            // This ensures no orphaned sandboxes can survive a timeout.
                            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
                        }
                        debug!("Set parent death signal");

                        debug!("Running init process");
                            InitProcess::run(
                                command,
                                stdin_pipe.read,
                                stdout_pipe.write,
                                stderr_pipe.write,
                            );
                    }
                    pid => {
                        debug!(
                            "In middle child process (PID: {}) waiting for inner child {}",
                            unsafe { libc::getpid() },
                            pid
                        );
                        close(stdin_pipe.read);
                        close(stdout_pipe.write);
                        close(stderr_pipe.write);
                        // The middle process waits for the actual payload to finish executing
                        let mut status = 0;
                        if unsafe { libc::waitpid(pid, &mut status, 0) } == -1 {
                            error!("Waitpid failed: {}", io::Error::last_os_error());
                        }

                        let exit_code = if libc::WIFEXITED(status) {
                            libc::WEXITSTATUS(status)
                        } else if libc::WIFSIGNALED(status) {
                            // Standard Linux convention: 128 + signal number
                            128 + libc::WTERMSIG(status)
                        } else {
                            1 // fallback error
                        };

                        debug!("Inner child exited with code: {}", exit_code);
                        unsafe { libc::_exit(exit_code) };
                    }
                }
            }

            pid => {
                debug!("In parent process (PID: {}) waiting for child {}", unsafe { libc::getpid() }, pid);
                close(stdin_pipe.read);
                close(stdout_pipe.write);
                close(stderr_pipe.write);
                debug!("Closed unnecessary pipe ends in parent");

                if let Err(e) = parent_coordinator.wait_for_namespace() {
                    error!("Parent coordinator wait_for_namespace failed: {}", e);
                    return Err(WorkerError::from(e));
                }
                debug!("Waited for namespace");

                if let Err(e) = NamespaceManager::finish_user_namespace(pid) {
                    error!("Failed to finish user namespace for child {}: {}", pid, e);
                    return Err(WorkerError::from(e));
                }
                debug!("Finished user namespace");

                if let Err(e) = parent_coordinator.continue_child() {
                    error!("Parent coordinator continue_child failed: {}", e);
                    return Err(WorkerError::from(e));
                }
                debug!("Continued child");

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