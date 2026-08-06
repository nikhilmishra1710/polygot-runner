use std::{
    ffi::CString,
    io,
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::ffi::OsStrExt,
    },
};

use crate::{
    runtime::{RuntimeCommand, unix::configure_child},
    sandbox::RootFilesystem,
    seccomp::SeccompFilter,
};

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
        let rootfs = RootFilesystem::new(&self.command.working_directory);

        rootfs.setup(&self.command.working_directory, 64 * 1024 * 1024)?;
        rootfs.enter()?;
        configure_child(&self.command.limits)?;
        std::env::set_current_dir("/")?;

        unsafe {
            // This tells the kernel: "This process and its children can NEVER gain new privileges"
            // (e.g., via setuid binaries). Seccomp requires this.
            if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                eprintln!("Fatal: Failed to set NO_NEW_PRIVS");
                std::process::exit(1);
            }
        }

        // 4. Install Seccomp Filter
        if let Err(e) = SeccompFilter::install() {
            eprintln!("Fatal: Failed to install seccomp filter: {}", e);
            std::process::exit(1);
        }

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
