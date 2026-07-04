use std::io;

use rustix::process::{Pid, Signal, getpgid, kill_process_group, setpgid};

pub fn create_process_group() -> io::Result<()> {
    setpgid(Pid::from_raw(0), Pid::from_raw(0)).map_err(io::Error::from)
}

pub fn kill_process_group_id(pid: u32) -> io::Result<()> {
    let pgid = getpgid(Pid::from_raw(pid as i32)).map_err(io::Error::from)?;

    kill_process_group(pgid, Signal::KILL).map_err(io::Error::from)
}
