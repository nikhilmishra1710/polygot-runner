use rustix::pipe::{PipeFlags, pipe_with};
use std::{io, os::fd::OwnedFd, thread::sleep, time::Duration};

use rustix::process::{Pid, Signal, getpgid, kill_process_group};

use crate::{model::ResourceLimits, runtime::apply_resource_limits};

pub struct Pipe {
    pub read: OwnedFd,
    pub write: OwnedFd,
}

pub fn kill_process_group_id(pid: u32) -> io::Result<()> {
    let raw_pid = Pid::from_raw(pid as i32).ok_or_else(|| io::Error::other("Invalid PID"))?;

    let pgid = getpgid(Some(raw_pid)).map_err(io::Error::from)?;

    // 2. Send SIGTERM (Graceful shutdown request)
    // We ignore errors here in case the process exits naturally the microsecond before we signal it.
    let _ = kill_process_group(pgid, Signal::TERM);

    // 3. The Grace Period
    // Give InitProcess and the Payload a fraction of a second to flush stdout/stderr and exit cleanly.
    sleep(Duration::from_millis(250));

    // 4. Send SIGKILL (The Hammer)
    // This is unblockable. If the process is still alive, the kernel annihilates it instantly.
    let _ = kill_process_group(pgid, Signal::KILL);
    Ok(())
}

pub fn create_process_group() -> io::Result<()> {
    let rc = unsafe { libc::setpgid(0, 0) };

    if rc == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

pub fn configure_child(limits: &ResourceLimits) -> io::Result<()> {
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
