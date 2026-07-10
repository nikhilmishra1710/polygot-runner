use rustix::pipe::{PipeFlags, pipe_with};
use std::{io, os::fd::OwnedFd};

use rustix::process::{Pid, Signal, getpgid, kill_process_group};

use crate::{model::ResourceLimits, runtime::apply_resource_limits};

pub struct Pipe {
    pub read: OwnedFd,
    pub write: OwnedFd,
}

pub fn kill_process_group_id(pid: u32) -> io::Result<()> {
    let pgid = getpgid(Pid::from_raw(pid as i32)).map_err(io::Error::from)?;

    kill_process_group(pgid, Signal::KILL).map_err(io::Error::from)
}

pub fn create_process_group() -> io::Result<()> {
    let rc = unsafe { libc::setpgid(0, 0) };

    if rc == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

pub fn configure_child(limits: &ResourceLimits) -> io::Result<()> {
    create_process_group()?;

    apply_resource_limits(limits)?;

    Ok(())
}

pub fn pipe() -> std::io::Result<Pipe> {
    let (read, write) = pipe_with(PipeFlags::CLOEXEC)?;

    Ok(Pipe { read, write })
}

pub fn close(fd: OwnedFd) {
    drop(fd);
}
