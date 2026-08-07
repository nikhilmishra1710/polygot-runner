use crate::runtime::{ChildBootstrap, RuntimeCommand, unix::close};
use std::os::fd::OwnedFd;

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

                // 1. Close write/read file descriptors in PID 1 so EOF propagates
                //    to the parent reader when PID 2 closes them.
                close(stdin_read);
                close(stdout_write);
                close(stderr_write);

                // 2. Ignore common termination signals so PID 1 isn't killed
                //    before it finishes reaping PID 2.
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

                // 3. Zombie Reaping Loop
                loop {
                    let mut status = 0;

                    // waitpid(-1) waits for ANY child process to change state
                    let reaped_pid = unsafe { libc::waitpid(-1, &mut status, 0) };

                    if reaped_pid < 0 {
                        let err = std::io::Error::last_os_error();
                        if err.raw_os_error() == Some(libc::EINTR) {
                            continue;
                        }
                        if err.raw_os_error() == Some(libc::ECHILD) {
                            // No children left
                            std::process::exit(0);
                        }
                        continue;
                    }

                    // If the reaped process was our main payload, exit PID 1 immediately!
                    if reaped_pid == payload_pid {
                        if libc::WIFEXITED(status) {
                            std::process::exit(libc::WEXITSTATUS(status));
                        } else if libc::WIFSIGNALED(status) {
                            let sig = libc::WTERMSIG(status);
                            // Exit cleanly with standard Unix convention (128 + signal)
                            std::process::exit(128 + sig);
                        } else {
                            std::process::exit(1);
                        }
                    }

                    // If reaped_pid != payload_pid, we reaped a background child process.
                    // Loop continues.
                }
            }
        }
    }
}
