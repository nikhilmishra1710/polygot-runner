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

---

### API

Language: Go

Responsibilities:

* Authentication
* Job submission
* Job status
* WebSocket log streaming
* Queue management

The API never invokes compilers or interpreters directly.

---

### SDK

Language: Python

Provides a convenient interface for interacting with the API.

---

### Web IDE

Language: TypeScript

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
