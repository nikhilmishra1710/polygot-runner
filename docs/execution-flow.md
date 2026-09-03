# Execution Flow

This document describes the step-by-step execution flow of user code through the polyglot execution platform.

## Request Lifecycle

```
┌─────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│   User Request  │    │   Job Queue      │    │  Worker Pool     │
│ (via API/Web IDE)│───►│ (Redis/RabbitMQ) │───►│ (Available)      │
└─────────────────┘    └──────────────────┘    └──────────────────┘
          │                         │                        │
          ▼                         ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│  Execution      │    │  Worker Assigns  │    │  Worker Picks    │
│  Request        │    │  Job to Worker   │    │  Job from Queue  │
└─────────────────┘    └──────────────────┘    └──────────────────┘
          │                         │                        │
          ▼                         ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Workspace       │    │ Language         │    │ Sandbox Setup    │
│ Creation        │    │ Runtime Selection│    │ (Namespaces,     │
│ (temp dir)      │    │ (Python/C++/etc) │    │  Cgroups, Seccomp)│
└─────────────────┘    └──────────────────┘    └──────────────────┘
          │                         │                        │
          ▼                         ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Compile (if     │    │ Execute User     │    │ Result Collection│
│ needed)         │    │ Code             │    │ (stdout, stderr, │
│                 │    │                  │    │  exit code, etc) │
└─────────────────┘    └──────────────────┘    └──────────────────┘
          │                         │                        │
          ▼                         ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Cleanup &       │    │ Send Result      │    │ Update Job Status│
│ Teardown        │    │ to API/Queue     │    │ & Notify User    │
└─────────────────┘    └──────────────────┘    └──────────────────┘
```

## Detailed Step-by-Step Process

### 1. Request Submission
- User submits code via Web IDE or API
- Request includes: language, source code, stdin, resource limits
- API validates request and places it in job queue

### 2. Worker Assignment
- Available worker pulls job from queue
- Worker receives: ExecutionRequest with language, files, limits

### 3. Workspace Creation
- WorkspaceManager creates isolated temporary directory
- Unique workspace ID generated (UUID)
- Source files written to workspace

### 4. Language Runtime Selection
- ExecutionEngine selects appropriate LanguageRuntime based on language
- Examples: PythonRuntime, CppRuntime

### 5. Sandbox Setup
- LanguageRuntime prepares ExecutionPlan
- Sandbox isolation applied:
  - User namespace (privilege separation)
  - Mount namespace (filesystem isolation via pivot_root)
  - PID namespace (process tree isolation)
  - Network namespace (network isolation)
  - IPC namespace (IPC isolation)
  - Cgroups v2 (resource limits)
  - Seccomp (syscall filtering)

### 6. Compilation (If Needed)
- For compiled languages (C++, Rust, etc.)
- Compiler runs in same sandboxed environment
- Compilation errors captured and returned immediately

### 7. Execution
- User code executed in fully isolated sandbox
- Stdout, stderr captured
- Resource usage monitored (CPU, memory, pids, etc.)
- Execution terminates on:
  - Normal exit
  - Resource limit exceeded (OOM, timeout, etc.)
  - Runtime error (segfault, exception)

### 8. Result Collection
- ExecutionReport generated with:
  - Exit code/termination reason
  - Captured stdout/stderr
  - Resource usage statistics
  - Execution timestamp

### 9. Cleanup
- Sandbox torn down (namespaces removed, cgroups deleted)
- Workspace directory deleted
- Worker returns to available pool

### 10. Response
- ExecutionReport sent back via job queue
- API retrieves result and sends to user
- Web IDE displays output to user

## Error Handling Paths

```
┌─────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Invalid Request │    │ Compilation      │    │ Sandbox Setup    │
│ (API Validation)│───►│ Failure          │───►│ Failure          │
└─────────────────┘    └──────────────────┘    └──────────────────┘
          │                         │                        │
          ▼                         ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Return Error    │    │ Return Error     │    │ Return Error     │
│ to User         │    │ to User          │    │ to User          │
└─────────────────┘    └──────────────────┘    └──────────────────┘
```

```
┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Execution        │    │ Resource Limit   │    │ Internal Worker  │
│ Failure          │    │ Exceeded         │    │ Failure          │
└──────────────────┘    └──────────────────┘    └──────────────────┘
          │                         │                        │
          ▼                         ▼                        ▼
┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Return Error     │    │ Return Error     │    │ Return Error     │
│ to User          │    │ to User          │    │ to User          │
└──────────────────┘    └──────────────────┘    └──────────────────┘
```