use std::{
    io,
    os::fd::{AsRawFd, OwnedFd},
};

pub struct ParentCoordinator {
    ready: OwnedFd,
    continue_fd: OwnedFd,
}

pub struct ChildCoordinator {
    ready: OwnedFd,
    continue_fd: OwnedFd,
}

impl ChildCoordinator {
    pub fn new(ready: OwnedFd, continue_fd: OwnedFd) -> io::Result<Self> {
        Ok(Self { ready, continue_fd })
    }

    pub fn namespace_created(&self) -> io::Result<()> {
        let byte = [1u8];

        let rc = unsafe { libc::write(self.continue_fd.as_raw_fd(), byte.as_ptr().cast(), 1) };

        if rc == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn wait_for_parent(&self) -> io::Result<()> {
        let mut byte = [0u8; 1];

        let rc = unsafe { libc::read(self.ready.as_raw_fd(), byte.as_mut_ptr().cast(), 1) };

        if rc == -1 {
            return Err(io::Error::last_os_error());
        }

        if rc == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "parent closed sync pipe",
            ));
        }

        Ok(())
    }
}

impl ParentCoordinator {
    pub fn new(ready: OwnedFd, continue_fd: OwnedFd) -> io::Result<Self> {
        Ok(Self { ready, continue_fd })
    }

    pub fn wait_for_namespace(&self) -> io::Result<()> {
        let mut byte = [0u8; 1];

        let rc = unsafe { libc::read(self.ready.as_raw_fd(), byte.as_mut_ptr().cast(), 1) };

        if rc == -1 {
            return Err(io::Error::last_os_error());
        }

        if rc == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "parent closed sync pipe",
            ));
        }

        Ok(())
    }

    pub fn continue_child(&self) -> io::Result<()> {
        let byte = [1u8];

        let rc = unsafe { libc::write(self.continue_fd.as_raw_fd(), byte.as_ptr().cast(), 1) };

        if rc == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}
