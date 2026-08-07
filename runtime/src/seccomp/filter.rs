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
            "clone",
            "clone3",
            "fork",
            "vfork",
            "execve",
            "execveat",
            "exit",
            "exit_group",
            "wait4",
            "waitid",
            "getpid",
            "getppid",
            "gettid",
            "getuid",
            "getgid",
            "geteuid",
            "getegid",
            "getresuid",
            "getresgid",
            "set_tid_address",
            "set_robust_list",
            "get_robust_list",
            "prctl",
            "arch_prctl",
            "rseq",
            "sched_yield",
            "sched_getaffinity",
            "sched_setaffinity",
            "sched_getscheduler",
            "sched_setscheduler",
            "futex",
            // ==========================================
            // 2. Memory Management (GCC/Clang rely heavily on mmap/mprotect)
            // ==========================================
            "mmap",
            "mprotect",
            "munmap",
            "brk",
            "mremap",
            "madvise",
            "mlock",
            "munlock",
            "msync",
            "mincore",
            "pkey_mprotect",
            "pkey_alloc",
            "pkey_free",
            "umask",
            // ==========================================
            // 3. File System, Metadata & Extended Lookups (Critical for GCC / cc1 / ld)
            // ==========================================
            "open",
            "openat",
            "openat2",
            "read",
            "write",
            "close",
            "close_range",
            "stat",
            "lstat",
            "fstat",
            "newfstatat",
            "fstatat64",
            "statx",
            "statfs",
            "fstatfs",
            "lseek",
            "pread64",
            "pwrite64",
            "readv",
            "writev",
            "preadv",
            "pwritev",
            "preadv2",
            "pwritev2",
            "access",
            "faccessat",
            "faccessat2", // <--- Syscall 439 (Fixes GCC cc1plus crashes)
            "dup",
            "dup2",
            "dup3",
            "fcntl",
            "ioctl",
            "fsync",
            "fdatasync",
            "sync",
            "getcwd",
            "chdir",
            "fchdir",
            "rename",
            "renameat",
            "renameat2",
            "mkdir",
            "mkdirat",
            "rmdir",
            "unlink",
            "unlinkat",
            "readlink",
            "readlinkat",
            "chmod",
            "fchmod",
            "fchmodat",
            "chown",
            "fchown",
            "fchownat",
            "getdents",
            "getdents64",
            "symlink",
            "symlinkat",
            "link",
            "linkat",
            "copy_file_range",
            "utimensat",
            "futimesat",
            // ==========================================
            // 4. Signals, Timers & Profiling
            // ==========================================
            "rt_sigaction",
            "rt_sigprocmask",
            "rt_sigreturn",
            "rt_sigpending",
            "rt_sigtimedwait",
            "sigaltstack",
            "nanosleep",
            "gettimeofday",
            "clock_gettime",
            "clock_getres",
            "clock_nanosleep",
            "alarm",
            "timer_create",
            "timer_settime",
            "timer_delete",
            // ==========================================
            // 5. Polling, Events, Inter-Process Communication (Pipes & Subprocesses)
            // ==========================================
            "pipe",
            "pipe2",
            "select",
            "pselect6",
            "poll",
            "ppoll",
            "epoll_create",
            "epoll_create1",
            "epoll_ctl",
            "epoll_wait",
            "epoll_pwait",
            "epoll_pwait2",
            "eventfd",
            "eventfd2",
            "memfd_create",
            // ==========================================
            // 6. System Info, Resource Limits & Entropy
            // ==========================================
            "uname",
            "getrlimit",
            "setrlimit",
            "prlimit64",
            "getrusage",
            "sysinfo",
            "getrandom",
            // ==========================================
            // 7. Networking (Local / Basic Sockets)
            // ==========================================
            "socket",
            "connect",
            "bind",
            "listen",
            "accept",
            "accept4",
            "sendto",
            "recvfrom",
            "sendmsg",
            "recvmsg",
            "getsockopt",
            "setsockopt",
            "getsockname",
            "getpeername",
            "socketpair",
            "shutdown",
            // ==========================================
            // 8. Namespace & Isolation
            // ==========================================
            "mount",
            "umount2",
            "pivot_root",
            "chroot",
            "unshare",
            "setns",
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
