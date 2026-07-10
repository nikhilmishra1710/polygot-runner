use std::{
    ffi::CString,
    io,
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::ffi::OsStrExt,
    },
};

use crate::runtime::{RuntimeCommand, unix::configure_child};

pub struct ChildBootstrap<'a> {
    command: &'a RuntimeCommand,

    stdin: OwnedFd,
    stdout: OwnedFd,
    stderr: OwnedFd,
}

impl<'a> ChildBootstrap<'a> {
    pub fn new(
        command: &'a RuntimeCommand,
        stdin: OwnedFd,
        stdout: OwnedFd,
        stderr: OwnedFd,
    ) -> Self {
        Self {
            command,
            stdin,
            stdout,
            stderr,
        }
    }
}

impl<'a> ChildBootstrap<'a> {
    pub fn run(self) -> ! {
        if let Err(err) = self.run_inner() {
            eprintln!("{err}");
        }

        unsafe {
            libc::_exit(127);
        }
    }

    fn run_inner(self) -> io::Result<()> {
        self.redirect_stdio()?;
        self.configure()?;
        self.exec()?;

        unreachable!();
    }

    fn redirect_stdio(&self) -> io::Result<()> {
        redirect(&self.stdin, libc::STDIN_FILENO)?;
        redirect(&self.stdout, libc::STDOUT_FILENO)?;
        redirect(&self.stderr, libc::STDERR_FILENO)?;

        Ok(())
    }

    fn configure(&self) -> io::Result<()> {
        configure_child(&self.command.limits)?;

        std::env::set_current_dir(&self.command.working_directory)?;

        Ok(())
    }

    fn exec(self) -> io::Result<()> {
        let exe = CString::new(self.command.executable.path.as_os_str().as_bytes())?;

        let mut argv = Vec::new();

        argv.push(exe.clone());

        for arg in &self.command.args {
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
}

fn redirect(fd: &OwnedFd, target: libc::c_int) -> io::Result<()> {
    let rc = unsafe { libc::dup2(fd.as_raw_fd(), target) };

    if rc == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
