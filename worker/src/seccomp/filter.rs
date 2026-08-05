use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall};
use std::io;

pub struct SeccompFilter;

impl SeccompFilter {
    pub fn install() -> io::Result<()> {
        // Create the filter context with a default action to KILL the process on violation
        let mut ctx = ScmpFilterContext::new_filter(ScmpAction::KillProcess)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        // A robust initial allowlist for Python.
        let allowed_syscalls = [
            // ==========================================
            // 1. Process & Thread Management
            // ==========================================
            "clone", "fork", "vfork", "execve", "exit", "exit_group", "wait4",
            "getpid", "getppid", "gettid", "getuid", "getgid", "geteuid", "getegid",
            "getresuid", "getresgid", "set_tid_address", "set_robust_list",
            "prctl", "arch_prctl", "rseq", "sched_yield", "sched_getaffinity",
            "sched_setaffinity", "futex",

            // ==========================================
            // 2. Memory Management
            // ==========================================
            "mmap", "mprotect", "munmap", "brk", "mremap", "madvise", "mlock", "munlock",

            // ==========================================
            // 3. File System & Metadata
            // ==========================================
            "open", "openat", "read", "write", "close", "close_range",
            "stat", "lstat", "fstat", "newfstatat", "fstatat64", "statx", "statfs", "fstatfs",
            "lseek", "pread64", "pwrite64", "readv", "writev", "access", "faccessat",
            "dup", "dup2", "dup3", "fcntl", "ioctl", "fsync", "fdatasync", "sync",
            "getcwd", "chdir", "rename", "renameat", "mkdir", "mkdirat", "rmdir",
            "unlink", "unlinkat", "readlink", "readlinkat", "chmod", "fchmod", "fchmodat",
            "chown", "fchown", "fchownat", "getdents64", 

            // ==========================================
            // 4. Signals & Time
            // ==========================================
            "rt_sigaction", "rt_sigprocmask", "rt_sigreturn", "sigaltstack",
            "nanosleep", "gettimeofday", "clock_gettime", "clock_nanosleep", "alarm",

            // ==========================================
            // 5. Polling, Events & IPC (Pipes)
            // ==========================================
            "pipe", "pipe2", "select", "pselect6", "poll", "ppoll",
            "epoll_create", "epoll_create1", "epoll_ctl", "epoll_wait", "epoll_pwait",
            "eventfd2",

            // ==========================================
            // 6. System Info & Limits
            // ==========================================
            "uname", "getrlimit", "prlimit64", "getrusage", "sysinfo", "getrandom",

            // ==========================================
            // 7. Networking (Local & Basic Sockets)
            // Python often opens sockets for internal DNS routing or IPC
            // ==========================================
            "socket", "connect", "bind", "listen", "accept", "accept4",
            "sendto", "recvfrom", "sendmsg", "recvmsg", "getsockopt", "setsockopt",
            "getsockname", "getpeername", "socketpair",

            // ==========================================
            // 8. Namespace & Isolation (Required for your specific tests)
            // ==========================================
            "mount", "umount2", "pivot_root", "chroot", "unshare", "setns"
        ];

        for syscall_name in allowed_syscalls.iter() {
            if let Ok(syscall) = ScmpSyscall::from_name(syscall_name) {
                // Add the rule allowing this specific syscall
                ctx.add_rule(ScmpAction::Allow, syscall).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to add {}: {}", syscall_name, e),
                    )
                })?;
            }
        }

        // Load the filter into the kernel
        ctx.load()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(())
    }
}
