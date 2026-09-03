# Resource Limits Implementation

## Status
Accepted

## Context
To prevent denial-of-service attacks and ensure fair resource usage, we need to limit the resources that user code can consume (CPU, memory, processes, etc.).

## Decision
We will use cgroups v2 to enforce resource limits on sandboxed processes. Specific limits include:
- memory.max: RAM limit
- pids.max: Number of processes
- cpu.max: CPU time allocation
- io.max: I/O bandwidth
- Additional limits via rlimit (RLIMIT_FSIZE, RLIMIT_NOFILE, etc.)
- Wall time monitoring via parent process

## Consequences

### Positive
- Granular control over resource usage
- Hierarchical and inheritable limits
- Modern cgroups v2 interface
- Ability to limit multiple resource types
- Container-like isolation without container runtime

### Negative
- Requires Linux kernel with cgroups v2 support
- More complex than simple rlimit approaches
- Need to manage cgroup lifecycle (creation, attachment, deletion)
- Potential for cgroup fragmentation if not cleaned properly

## Implementation Details
- See `runtime/src/runtime/limits.rs` for cgroups v2 setup and configuration
- See `runtime/src/runtime/process_runner.rs` for attaching processes to cgroups
- Wall time limits implemented via parent process monitoring in `runtime/src/runtime/process_runner.rs`
- Traditional rlimits (file size, open files) applied via `prlimit` before exec