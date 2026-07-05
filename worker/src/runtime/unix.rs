use std::io;

use rustix::process::{Pid, Signal, getpgid, kill_process_group};

pub fn kill_process_group_id(pid: u32) -> io::Result<()> {
    let pgid = getpgid(Pid::from_raw(pid as i32)).map_err(io::Error::from)?;

    kill_process_group(pgid, Signal::KILL).map_err(io::Error::from)
}
