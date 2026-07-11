use std::io;

fn write_setgroups(pid: libc::pid_t) -> io::Result<()> {
    let path = format!("/proc/{pid}/setgroups");

    std::fs::write(path, "deny")?;

    Ok(())
}

fn write_uid_map(pid: libc::pid_t) -> io::Result<()> {
    let uid = unsafe { libc::geteuid() };

    let path = format!("/proc/{pid}/uid_map");
    let content = format!("0 {uid} 1");
    std::fs::write(path, content)?;
    Ok(())
}

fn write_gid_map(pid: libc::pid_t) -> io::Result<()> {
    let gid = unsafe { libc::getegid() };

    let path = format!("/proc/{pid}/gid_map");
    let content = format!("0 {gid} 1");
    std::fs::write(path, content)?;
    Ok(())
}

pub struct NamespaceManager;

impl NamespaceManager {
    pub fn enter_user_namespace() -> io::Result<()> {
        let rc = unsafe { libc::unshare(libc::CLONE_NEWUSER) };

        if rc == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn finish_user_namespace(pid: libc::pid_t) -> io::Result<()> {
        println!("setgroups");
        write_setgroups(pid)?;

        println!("uidmap");
        write_uid_map(pid)?;

        println!("gidmap");
        write_gid_map(pid)?;

        Ok(())
    }
}

pub struct MountNamespace;

impl MountNamespace {
    pub fn enter() -> io::Result<()> {
        let rc = unsafe { libc::unshare(libc::CLONE_NEWNS) };

        if rc == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn make_private() -> io::Result<()> {
        unsafe {
            libc::mount(
                std::ptr::null(),
                c"/".as_ptr(),
                std::ptr::null(),
                libc::MS_REC | libc::MS_PRIVATE,
                std::ptr::null(),
            );
        };

        Ok(())
    }
}
