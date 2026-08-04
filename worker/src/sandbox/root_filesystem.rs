use std::{
    ffi::CString,
    fs, io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

const SYSTEM_MOUNTS: &[&str] = &["/bin", "/usr", "/lib", "/lib64"];

pub struct RootFilesystem {
    root: PathBuf,
}

impl RootFilesystem {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn prepare(&self) -> io::Result<()> {
        let directories = [
            "",
            "bin",
            "usr",
            "lib",
            "lib64",
            "etc",
            "tmp",
            "proc",
            "dev",
            ".old_root",
        ];

        for dir in directories {
            fs::create_dir_all(self.root.join(dir))?;
        }

        Ok(())
    }

    pub fn bind_system(&self) -> io::Result<()> {
        for mount in SYSTEM_MOUNTS {
            if Path::new(mount).exists() {
                self.bind(mount)?;
            }
        }

        Ok(())
    }

    pub fn enter(&self) -> io::Result<()> {
        // Required before pivot_root()
        self.make_mountpoint()?;

        println!("uid={}", unsafe { libc::geteuid() });
        println!("gid={}", unsafe { libc::getegid() });
        println!(
            "{}",
            std::fs::read_to_string("/proc/self/uid_map").unwrap_or_default()
        );
        println!(
            "{}",
            std::fs::read_to_string("/proc/self/gid_map").unwrap_or_default()
        );
        println!("pivot");
        self.pivot_root()?;

        println!("before proc");
        match self.mount_proc() {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "mounting proc failed. This commonly occurs on WSL2 because unprivileged proc mounts are restricted.",
        ));
            }
            Err(e) => return Err(e),
        }
        println!("after proc");

        println!("dev");
        self.mount_dev()?;

        println!("detach");
        self.detach_old_root()?;

        println!("done");

        Ok(())
    }

    fn bind(&self, source: &str) -> io::Result<()> {
        let src = Path::new(source);
        let dst = self.root.join(source.trim_start_matches('/'));

        bind_mount(src, &dst)
    }

    fn make_mountpoint(&self) -> io::Result<()> {
        bind_mount(&self.root, &self.root)
    }

    pub fn mount_proc(&self) -> io::Result<()> {
        let target = CString::new("/proc")?;
        let source = CString::new("proc")?;
        let fstype = CString::new("proc")?;

        let rc = unsafe {
            libc::mount(
                source.as_ptr(),
                target.as_ptr(),
                fstype.as_ptr(),
                0,
                std::ptr::null(),
            )
        };

        if rc == -1 {
            return Err(io::Error::last_os_error());
        }
        println!("{}", std::fs::read_to_string("/proc/self/status")?);
        Ok(())
    }

    pub fn mount_dev(&self) -> io::Result<()> {
        bind_mount(Path::new("/dev"), Path::new("/dev"))
    }

    fn pivot_root(&self) -> io::Result<()> {
        let new_root = CString::new(self.root.as_os_str().as_bytes())?;

        let put_old = CString::new(self.root.join(".old_root").as_os_str().as_bytes())?;
        println!("root = {:?}", self.root);
        println!("old = {:?}", self.root.join(".old_root"));
        let rc =
            unsafe { libc::syscall(libc::SYS_pivot_root, new_root.as_ptr(), put_old.as_ptr()) };

        if rc == -1 {
            return Err(io::Error::last_os_error());
        }
        println!("pivot rc = {}", rc);
        println!("errno = {:?}", io::Error::last_os_error());

        Ok(())
    }

    fn detach_old_root(&self) -> io::Result<()> {
        let old_root = CString::new("/.old_root")?;

        let rc = unsafe { libc::umount2(old_root.as_ptr(), libc::MNT_DETACH) };

        if rc == -1 {
            return Err(io::Error::last_os_error());
        }

        fs::remove_dir("/.old_root")?;

        Ok(())
    }
}

fn bind_mount(source: &Path, destination: &Path) -> io::Result<()> {
    let src = CString::new(source.as_os_str().as_bytes())?;
    let dst = CString::new(destination.as_os_str().as_bytes())?;

    let rc = unsafe {
        libc::mount(
            src.as_ptr(),
            dst.as_ptr(),
            std::ptr::null(),
            libc::MS_BIND | libc::MS_REC,
            std::ptr::null(),
        )
    };

    if rc == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
