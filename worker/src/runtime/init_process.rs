use crate::runtime::{ChildBootstrap, RuntimeCommand, unix::close};
use std::{
    os::fd::OwnedFd,
};

pub struct InitProcess;

impl InitProcess {
    pub fn run(
        command: &RuntimeCommand,
        stdin_read: OwnedFd,
        stdout_write: OwnedFd,
        stderr_write: OwnedFd,
    ) -> ! {
        let pid = unsafe { libc::fork() };

        match pid {
            -1 => {
                eprintln!("Fatal: Init process failed to fork payload");
                std::process::exit(1);
            }
            0 => {
                // We are PID 2 (The Payload)
                unsafe {
                    // Tell the kernel to kill us if Init (PID 1) dies
                    libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
                }

                ChildBootstrap::new(command, stdin_read, stdout_write, stderr_write).run()
            }
            payload_pid => {
                // We are PID 1 (The Init Process)

                // 1. Close pipes in Init!
                // Only the payload should hold these open, otherwise the Parent
                // will never receive an EOF when the payload exits.

                close(stdin_read);
                close(stdout_write);
                close(stderr_write);

                // 3. Install signal handlers to forward to the payload
                let signals = [
                    libc::SIGTERM,
                    libc::SIGINT,
                    libc::SIGQUIT,
                    libc::SIGUSR1,
                    libc::SIGUSR2,
                    libc::SIGHUP,
                ];

                for &sig in &signals {
                    unsafe {
                        libc::signal(sig, libc::SIG_IGN);
                    }
                }

                // 4. Zombie Reaping Loop
                loop {
                    let mut status = 0;

                    // waitpid(-1) waits for ANY child process to change state
                    let reaped_pid = unsafe { libc::waitpid(-1, &mut status, 0) };

                    if reaped_pid < 0 {
                        let err = std::io::Error::last_os_error();
                        // If waitpid was interrupted by our signal handler, just loop again
                        if err.raw_os_error() == Some(libc::EINTR) {
                            continue;
                        }
                        // If ECHILD is returned, there are absolutely no children left
                        if err.raw_os_error() == Some(libc::ECHILD) {
                            std::process::exit(0);
                        }
                        continue;
                    }

                    // If the process that just died was our main payload...
                    if reaped_pid == payload_pid {
                        if libc::WIFEXITED(status) {
                            std::process::exit(libc::WEXITSTATUS(status));
                        } else if libc::WIFSIGNALED(status) {
                            let sig = libc::WTERMSIG(status);
                            unsafe {
                                // Reset handler and kill self to propagate exact signal
                                libc::signal(sig, libc::SIG_DFL);
                                libc::kill(libc::getpid(), sig);
                            }
                            std::process::exit(128 + sig);
                        } else {
                            std::process::exit(1);
                        }
                    }

                    // If reaped_pid != payload_pid, we just successfully reaped a zombie
                    // created by the payload! The loop naturally continues.
                }
            }
        }
    }
}
