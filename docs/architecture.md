# Architecture

## Components

The platform consists of four primary components.

### Worker

Language: Rust

Responsible for:

* Workspace creation
* Compilation
* Execution
* Sandboxing
* Resource isolation
* Log collection

The worker is the only component allowed to execute user code.

Currently implemented in the `runtime/` directory as `runtime-worker`.

---

### API

Language: Go (planned)

Responsibilities:

* Authentication
* Job submission
* Job status
* WebSocket log streaming
* Queue management

The API never invokes compilers or interpreters directly.

---

### SDK

Language: Python (planned)

Provides a convenient interface for interacting with the API.

---

### Web IDE

Language: TypeScript (planned)

Provides an interactive browser-based development experience.

---

## Execution Pipeline

```
Request

↓

Workspace

↓

Language Runtime

↓

Sandbox

↓

Execution

↓

Result
```

Each stage has a single responsibility.

### Detailed Component Interaction

```
              ┌─────────────────┐
              │   Web IDE/API   │ ◄────┐
              └─────────────────┘      │
                       │               │
                       ▼               │
              ┌─────────────────┐      │
              │   Job Queue     │      │
              └─────────────────┘      │
                       │               │
                       ▼               │
              ┌─────────────────┐      │
              │   Worker Pool   │      │
              └─────────────────┘      │
                       │               │
         ┌─────────────▼─────────────┐ │
         │     Execution Request     │ │
         └─────────────┬─────────────┘ │
                       │               │
         ┌─────────────▼─────────────┐ │
         │   Workspace Manager       │ │
         │  (creates isolated env)   │ │
         └─────────────┬─────────────┘ │
                       │               │
         ┌─────────────▼─────────────┐ │
         │    Language Runtime       │ │
         │ (Python/C++ preparation)  │ │
         └─────────────┬─────────────┘ │
                       │               │
         ┌─────────────▼─────────────┐ │
         │   Sandbox Setup           │ │
         │ (namespaces, cgroups,     │ │
         │  seccomp, filesystem)     │ │
         └─────────────┬─────────────┘ │
                       │               │
         ┌─────────────▼─────────────┐ │
         │   Compile (if needed)     │ │
         └─────────────┬─────────────┘ │
                       │               │
         ┌─────────────▼─────────────┐ │
         │     Execute User Code     │ │
         └─────────────┬─────────────┘ │
                       │               │
         ┌─────────────▼─────────────┐ │
         │   Result Collection       │ │
         │  (output, exit code,      │ │
         │   resource usage)         │ │
         └─────────────┬─────────────┘ │
                       │               │
         ┌─────────────▼─────────────┐ │
         │  Cleanup & Teardown       │ │
         └─────────────┬─────────────┘ │
                       │               │
              ┌─────────────────┐      │
              │   Execution     │      │
              │   Report/Result │      │
              └─────────────────┘      │
                       │               │
                       ▼               │
              ┌─────────────────┐      │
              │   Job Completion  │◄───┘
              └─────────────────┘
```

### Sandbox Isolation Layers

```
User Process
     │
     ├──── Network Namespace (isolated network)
     │
     ├──── PID Namespace (isolated process tree)
     │
     ├──── UTS Namespace (isolated hostname)
     │
     ├──── IPC Namespace (isolated IPC)
     │
     ├──── Mount Namespace (isolated filesystem)
     │     ├──── pivot_root() to new rootfs
     │     ├──── /proc bind mount
     │     ├──── /dev bind mount
     │     └──── tmpfs for /tmp
     │
     ├──── User Namespace (isolated user/group IDs)
     │     ├──── Maps to unprivileged user on host
     │     └──── Prevents privilege escalation
     │
     ├──── cgroups v2 (resource limits)
     │     ├──── memory.max (RAM limit)
     │     ├──── pids.max (process limit)
     │     ├──── cpu.max (CPU time limit)
     │     └──── io.max (I/O limit)
     │
     └──── seccomp (syscall filtering)
          ├──── Whitelisted syscalls (read, write, exit, etc.)
          └──── Blocked syscalls (ptrace, clone, mount, etc.)
```

Current implementation includes:
- Workspace management (in `runtime/src/workspace/`)
- Language runtimes for Python and C++ (in `runtime/src/language/`)
- Sandbox using Linux namespaces, cgroups, and seccomp (in `runtime/src/runtime/`)
- Execution engine coordinating the pipeline (in `runtime/src/engine/`)

---

## Design Principles

* Separate orchestration from execution.
* Separate language logic from runtime logic.
* Every execution uses a temporary workspace.
* Every execution is isolated.
* The execution pipeline should remain language agnostic.

---

## Future Sandbox

The runtime will eventually use:

* Linux namespaces
* cgroups v2
* seccomp
* mount namespaces
* tmpfs
* pivot_root
* capability dropping
* read-only filesystem
* network isolation

These are intentionally deferred until the execution engine is functional.
