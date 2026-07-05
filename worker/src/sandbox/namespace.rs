use std::io;

use rustix::thread::unshare_unsafe;
use rustix::thread::UnshareFlags;

pub struct NamespaceManager;

impl NamespaceManager {
    pub fn setup() -> io::Result<()> {
        unsafe {
            unshare_unsafe(UnshareFlags::NEWUSER)?;
        }

        Ok(())
    }
}