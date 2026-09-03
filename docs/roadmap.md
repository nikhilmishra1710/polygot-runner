# Roadmap

## Phase 0

Project foundation.

* Repository setup
* Worker crate
* Module boundaries
* CI
* Documentation

**Status: COMPLETED**

---

## Phase 1

Execution engine.

* Workspace manager
* Process runner
* Execute local programs
* Language abstraction

**Status: COMPLETED**
- Workspace manager implemented in `runtime/src/workspace/`
- Process runner (NativeProcessRunner) implemented in `runtime/src/runtime/process_runner.rs`
- Language abstraction with Python and C++ runtimes in `runtime/src/language/`
- Execution engine coordinates the pipeline in `runtime/src/engine/`

---

## Phase 2

Linux namespaces.

* PID namespace
* Mount namespace
* UTS namespace
* IPC namespace
* Network namespace
* User namespace

**Status: COMPLETED**
- Implemented in `runtime/src/runtime/` with:
  * User namespace support
  * Mount namespace with pivot_root
  * PID namespace with double-fork pattern
  * UTS namespace (hostname)
  * IPC namespace
  * Network namespace
  * cgroups v2 for resource isolation

---

## Phase 3

Resource isolation.

* cgroups v2
* CPU limits
* Memory limits
* Process limits
* Timeouts

**Status: COMPLETED**
- Resource limits applied via cgroups v2 in `runtime/src/runtime/limits.rs`
- Wall time, CPU time, memory, pids, file size, and open file limits enforced
- Timeout handling in `runtime/src/runtime/process_runner.rs`

---

## Phase 4

Filesystem isolation.

* tmpfs
* bind mounts
* pivot_root
* ephemeral workspaces

**Status: COMPLETED**
- Filesystem isolation implemented in `runtime/src/runtime/`
  * Uses pivot_root to change root filesystem
  * Bind mounts for /proc and /dev
  * tmpfs for temporary storage
  * Ephemeral workspaces created per execution
  * Old root filesystem properly cleaned up

---

## Phase 5

Security hardening.

* seccomp
* capability dropping
* read-only filesystem
* no network

**Status: COMPLETED**
- seccomp filtering implemented in `runtime/src/runtime/seccomp.rs`
  * Whitelists essential syscalls
  * Blocks dangerous syscalls (execve at wrong times, ptrace, etc.)
- Capability dropping implemented via user namespace (running as non-root inside sandbox)
- Read-only filesystem considerations (can be enhanced)
- Network isolation via network namespace (no external access)

---

## Phase 6

Go API.

**Status: PLANNED**
- Authentication
* Job submission
* Job status
* WebSocket log streaming
* Queue management

---

## Phase 7

Job queue.

**Status: PLANNED**
- Implementation of job queuing system
- Prioritization and scheduling
- Persistence

---

## Phase 8

Distributed workers.

**Status: PLANNED**
- Horizontal worker scaling
- Load balancing
- Worker registration/discovery
- Fault tolerance

---

## Phase 9

Streaming logs.

**Status: PLANNED**
- Real-time log streaming from sandbox to client
- Log buffering and persistence
- Structured log format

---

## Phase 10

Production readiness.

* Metrics
* Monitoring
* Persistent storage
* Execution caching

**Status: PLANNED**
- Prometheus metrics endpoint
- Health checks and monitoring
- Persistent storage for artifacts
- Caching of frequently used dependencies/compiler installations
