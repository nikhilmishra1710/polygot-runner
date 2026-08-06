use std::{
    ffi::CString,
    fs, io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

const READONLY_SYSTEM_MOUNTS: &[&str] = &["/bin", "/usr", "/lib", "/lib64", "/etc"];

pub struct RootFilesystem {
    root: PathBuf,
}

impl RootFilesystem {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Creates base directory structure inside the staging root.
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
            "workspace",
            ".old_root",
        ];

        for dir in directories {
            fs::create_dir_all(self.root.join(dir))?;
        }

        Ok(())
    }

    /// Mounts system directories as READ-ONLY into the staging root.
    pub fn bind_system(&self) -> io::Result<()> {
        for mount in READONLY_SYSTEM_MOUNTS {
            let host_path = Path::new(mount);
            if host_path.exists() {
                let dst = self.root.join(mount.trim_start_matches('/'));
                bind_mount_readonly(host_path, &dst)?;
            }
        }

        Ok(())
    }

    /// Mounts an ephemeral, memory-backed tmpfs onto /tmp.
    pub fn bind_tmpfs(&self, size_bytes: u64) -> io::Result<()> {
        let dst = self.root.join("tmp");
        mount_tmpfs(&dst, size_bytes)
    }

    /// Binds the host's target workspace directory as WRITABLE.
    pub fn bind_workspace(&self, host_workspace_path: &Path) -> io::Result<()> {
        let dst = self.root.join("workspace");
        bind_mount_writable(host_workspace_path, &dst)
    }

    pub fn setup(&self, host_workspace_path: &Path, tmpfs_size_bytes: u64) -> io::Result<()> {
        // 1. Create directory structure inside the staging root
        self.prepare()?;

        // 2. Bind host system directories (/bin, /usr, /lib, /lib64, /etc) as READ-ONLY
        self.bind_system()?;

        // 3. Mount ephemeral memory-backed tmpfs on /tmp
        self.bind_tmpfs(tmpfs_size_bytes)?;

        // 4. Bind the host workspace directory as WRITABLE
        self.bind_workspace(host_workspace_path)?;

        Ok(())
    }

    /// Configures the pivot_root and populates /proc and minimal /dev inside the container.
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

    fn make_mountpoint(&self) -> io::Result<()> {
        bind_mount_writable(&self.root, &self.root)
    }

    pub fn mount_proc(&self) -> io::Result<()> {
        let target = CString::new("/proc")?;
        let source = CString::new("proc")?;
        let fstype = CString::new("proc")?;

        let flags = libc::MS_NOSUID | libc::MS_NODEV | libc::MS_NOEXEC;

        let rc = unsafe {
            libc::mount(
                source.as_ptr(),
                target.as_ptr(),
                fstype.as_ptr(),
                flags,
                std::ptr::null(),
            )
        };

        if rc == -1 {
            return Err(io::Error::last_os_error());
        }
        println!("{}", std::fs::read_to_string("/proc/self/status")?);
        Ok(())
    }

    /// Populates /dev with selective read/write character device binds and standard symlinks.
    pub fn mount_dev(&self) -> io::Result<()> {
        let dev_files = ["null", "zero", "random", "urandom", "tty"];

        for dev_file in &dev_files {
            let host_dev = Path::new("/dev").join(dev_file);
            let container_dev = Path::new("/dev").join(dev_file);

            if host_dev.exists() {
                if !container_dev.exists() {
                    // Touch empty file as mount target anchor
                    fs::File::create(&container_dev)?;
                }
                bind_mount_writable(&host_dev, &container_dev)?;
            }
        }

        // Standard device symlinks
        let _ = std::os::unix::fs::symlink("/proc/self/fd", "/dev/fd");
        let _ = std::os::unix::fs::symlink("/proc/self/fd/0", "/dev/stdin");
        let _ = std::os::unix::fs::symlink("/proc/self/fd/1", "/dev/stdout");
        let _ = std::os::unix::fs::symlink("/proc/self/fd/2", "/dev/stderr");

        Ok(())
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

// ==========================================
// Low-level Mount Helper Functions
// ==========================================

/// Binds a source path to target as READ-ONLY using a two-pass remount.
fn bind_mount_readonly(source: &Path, destination: &Path) -> io::Result<()> {
    let src = CString::new(source.as_os_str().as_bytes())?;
    let dst = CString::new(destination.as_os_str().as_bytes())?;

    unsafe {
        // Step 1: Initial recursive bind mount
        let rc = libc::mount(
            src.as_ptr(),
            dst.as_ptr(),
            std::ptr::null(),
            libc::MS_BIND | libc::MS_REC,
            std::ptr::null(),
        );
        if rc == -1 {
            return Err(io::Error::last_os_error());
        }

        // Step 2: Remount target as Read-Only with security flags
        let readonly_flags = libc::MS_BIND
            | libc::MS_REMOUNT
            | libc::MS_RDONLY
            | libc::MS_NOSUID
            | libc::MS_NODEV
            | libc::MS_REC;

        let rc = libc::mount(
            std::ptr::null(),
            dst.as_ptr(),
            std::ptr::null(),
            readonly_flags,
            std::ptr::null(),
        );
        if rc == -1 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
}

/// Binds a source path to target as WRITABLE.
fn bind_mount_writable(source: &Path, destination: &Path) -> io::Result<()> {
    let src = CString::new(source.as_os_str().as_bytes())?;
    let dst = CString::new(destination.as_os_str().as_bytes())?;

    let flags = libc::MS_BIND | libc::MS_REC | libc::MS_NOSUID | libc::MS_NODEV;

    let rc = unsafe {
        libc::mount(
            src.as_ptr(),
            dst.as_ptr(),
            std::ptr::null(),
            flags,
            std::ptr::null(),
        )
    };

    if rc == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

/// Mounts an in-memory tmpfs filesystem.
fn mount_tmpfs(destination: &Path, size_bytes: u64) -> io::Result<()> {
    let dst = CString::new(destination.as_os_str().as_bytes())?;
    let fstype = CString::new("tmpfs")?;

    let options_str = format!("size={},mode=1777", size_bytes);
    let options = CString::new(options_str)?;

    let flags = libc::MS_NOSUID | libc::MS_NODEV;

    let rc = unsafe {
        libc::mount(
            fstype.as_ptr(),
            dst.as_ptr(),
            fstype.as_ptr(),
            flags,
            options.as_ptr() as *const libc::c_void,
        )
    };

    if rc == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
