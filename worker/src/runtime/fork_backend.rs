use crate::{
    error::WorkerError,
    runtime::{
        ChildBootstrap, RunningProcess, RuntimeCommand,
        backend::ProcessBackend,
        unix::{close, configure_child, pipe},
    },
};
use std::{
    ffi::CString,
    io,
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::ffi::OsStrExt,
    },
    path::Path,
};

pub struct ForkBackend;

impl ProcessBackend for ForkBackend {
    fn launch(&self, command: &RuntimeCommand) -> Result<RunningProcess, WorkerError> {
        // fork() returns a rustix::io::Result<Option<Pid>>
        // Note: The `?` operator requires WorkerError to implement From<rustix::io::Errno>
        let stdin_pipe = pipe()?;
        let stdout_pipe = pipe()?;
        let stderr_pipe = pipe()?;
        let pid = unsafe { libc::fork() };

        match pid {
            -1 => {
                return Err(io::Error::last_os_error().into());
            }

            0 => {
                close(stdin_pipe.write);
                close(stdout_pipe.read);
                close(stderr_pipe.read);

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

fn exec(executable: &Path, args: &[String]) -> io::Result<()> {
    let exe = CString::new(executable.as_os_str().as_bytes())?;

    let mut argv = Vec::with_capacity(args.len() + 1);

    argv.push(exe.clone());

    for arg in args {
        argv.push(CString::new(arg.as_bytes())?);
    }

    let argv_ptrs: Vec<_> = argv
        .iter()
        .map(|s| s.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();

    unsafe {
        libc::execv(exe.as_ptr(), argv_ptrs.as_ptr());
    }

    Err(io::Error::last_os_error())
}

fn redirect(fd: &OwnedFd, target: libc::c_int) -> io::Result<()> {
    let rc = unsafe { libc::dup2(fd.as_raw_fd(), target) };

    if rc == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
