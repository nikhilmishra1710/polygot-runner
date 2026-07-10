use std::{
    fs::File,
    io::Write,
    os::{fd::OwnedFd, unix::process::ExitStatusExt},
    process::{Child, ExitStatus},
};

pub struct RunningProcess {
    pid: libc::pid_t,

    stdin: Option<OwnedFd>,
    stdout: Option<OwnedFd>,
    stderr: Option<OwnedFd>,
}

impl RunningProcess {
    pub fn from_fork(
        pid: libc::pid_t,
        stdin: Option<OwnedFd>,
        stdout: Option<OwnedFd>,
        stderr: Option<OwnedFd>,
    ) -> Self {
        Self {
            pid,
            stdin,
            stdout,
            stderr,
        }
    }

    pub fn from_child(mut child: Child) -> Self {
        Self {
            pid: child.id() as libc::pid_t,

            stdin: child.stdin.take().map(OwnedFd::from),
            stdout: child.stdout.take().map(OwnedFd::from),
            stderr: child.stderr.take().map(OwnedFd::from),
        }
    }

    pub fn pid(&self) -> libc::pid_t {
        self.pid
    }

    pub fn stdin(&mut self) -> Option<OwnedFd> {
        self.stdin.take()
    }

    pub fn stdout(&mut self) -> Option<File> {
        self.stdout.take().map(File::from)
    }

    pub fn stderr(&mut self) -> Option<File> {
        self.stderr.take().map(File::from)
    }

    pub fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        let mut status = 0;

        let rc = unsafe { libc::waitpid(self.pid, &mut status, libc::WNOHANG) };

        if rc == -1 {
            return Err(std::io::Error::last_os_error());
        }

        if rc == 0 {
            return Ok(None);
        }

        Ok(Some(ExitStatus::from_raw(status)))
    }

    pub fn wait(&mut self) -> std::io::Result<ExitStatus> {
        let mut status = 0;

        let rc = unsafe { libc::waitpid(self.pid, &mut status, 0) };

        if rc == -1 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(ExitStatus::from_raw(status))
    }

    pub fn write_stdin(&mut self, input: &[u8]) -> std::io::Result<()> {
        if let Some(fd) = self.stdin.take() {
            let mut file = File::from(fd);
            file.write_all(input)?;
        }

        Ok(())
    }
}
