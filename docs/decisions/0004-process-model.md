# Process Model for Sandbox Isolation

## Status
Accepted

## Context
To properly isolate user code execution, we need to isolate various system resources including PID tree, network, mounts, IPC, and UTS. The challenge is setting up these namespaces correctly while maintaining control over the sandbox initialization process.

## Decision
We use a double-fork pattern combined with unshare() calls to create a fully isolated sandbox environment:

1. First fork: Creates child process that will set up namespaces
2. In first child: 
   - unshare(CLONE_NEWUSER) to create user namespace
   - Set up uid/gid mappings to map to unprivileged user on host
   - Signal parent that user namespace is ready
3. Second fork (in first child): 
   - Creates grandchild that will get full namespace isolation
   - In grandchild:
     - unshare(CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWNET | CLONE_NEWIPC | CLONE_NEWUTS)
     - Set up mount namespace with pivot_root
     - Set up /proc and /dev bind mounts
     - Apply resource limits via cgroups
     - Apply seccomp filters
     - Exec user code
4. The intermediate child waits for the grandchild to complete and reports results

## Consequences

### Positive
- Proper PID isolation: sandbox init gets PID 1, user code gets PID 2
- Clean separation: namespace setup process doesn't run user code
- Ability to run sandbox init as root (to set up namespaces) then drop privileges
- Well-understood pattern for container-like isolation
- Signal handling and zombie reaping can be managed properly

### Negative
- More complex process hierarchy
- Requires careful signal forwarding between processes
- Need to handle zombie reaping of both child processes
- Slightly more overhead due to extra fork

## Implementation Details
- See `runtime/src/runtime/init_process.rs` for the sandbox init process
- See `runtime/src/runtime/process_runner.rs` for the double-fork implementation
- Namespace setup occurs in the correct order: user namespace first, then others