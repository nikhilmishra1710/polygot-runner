# Runtime Worker Documentation

This document describes the documentation structure for the runtime worker project. Each file focuses on one subsystem so that implementation details remain isolated and easy to maintain.

---

# architecture.md

## Purpose

High-level overview of the runtime and how requests move through the system.

## Topics

* Goals
* Overall architecture
* Component responsibilities
* Directory layout
* Design principles

## Diagram

```
ExecutionRequest
        │
        ▼
ExecutionEngine
        │
        ▼
LanguageRuntime
        │
        ▼
ExecutionPlan
        │
        ▼
NativeProcessRunner
        │
        ▼
ForkBackend
        │
        ▼
Sandbox
```

## Components

### ExecutionEngine

Coordinates execution.

Responsible for

* creating workspaces
* selecting language runtime
* preparing execution plan
* executing stages
* collecting results

---

### LanguageRuntime

Language-specific preparation.

Examples

* Python
* C++
* Rust
* Java

---

### NativeProcessRunner

Responsible for executing one RuntimeCommand.

It knows nothing about languages.

---

### Sandbox

Responsible for isolation.

Contains

* namespaces
* root filesystem
* cgroups
* seccomp

---

# execution-flow.md

## Request lifecycle

```
ExecutionRequest

↓

WorkspaceManager

↓

LanguageRuntime

↓

ExecutionPlan

↓

Compile (optional)

↓

Execute

↓

ExecutionReport
```

## Python

```
Request

↓

Workspace

↓

python main.py

↓

Report
```

## C++

```
Request

↓

Workspace

↓

g++

↓

Binary

↓

Execute

↓

Report
```

Explain why compilation and execution are independent sandbox invocations.

---

# sandbox.md

Describe every isolation mechanism.

## User Namespace

Purpose

* root inside sandbox
* unprivileged on host

Sequence

```
Parent

fork()

↓

Child

unshare(CLONE_NEWUSER)

↓

Signal parent

↓

Parent writes

uid_map
gid_map
setgroups

↓

Child continues
```

---

## Mount Namespace

Purpose

Private mount table.

Sequence

```
unshare(CLONE_NEWNS)

↓

mount --make-rprivate /

↓

pivot_root()

↓

mount proc

↓

mount dev
```

---

## PID Namespace

Purpose

Separate process tree.

Describe why a second fork is required.

```
Parent

↓

Middle child

↓

unshare(CLONE_NEWPID)

↓

fork()

↓

Sandbox init (PID 1)

↓

Program
```

---

## Root Filesystem

Describe

* bind mounts
* pivot_root
* old root removal

---

## cgroups

Purpose

* memory limits
* pid limits

Explain lifecycle

```
Create

↓

Configure

↓

Attach process

↓

Run

↓

Destroy
```

---

## seccomp

Describe

* syscall filtering
* whitelist
* default action

---

# root-filesystem.md

Explain construction of sandbox filesystem.

Directory layout

```
rootfs/

bin/

usr/

lib/

lib64/

etc/

tmp/

proc/

dev/
```

Explain

* bind mounts
* pivot_root
* proc mount
* dev bind mount

Explain cleanup.

---

# process-model.md

Describe process hierarchy.

```
Runtime

↓

fork

↓

Namespace child

↓

fork

↓

Sandbox init (PID 1)

↓

User program
```

Explain why the second fork exists.

Explain signal forwarding.

Explain zombie reaping.

Explain process groups.

---

# resource-limits.md

Describe every limit.

Current limits

* wall time
* CPU time
* memory
* pids
* file size
* open files

Describe enforcement.

Wall time

Parent monitors elapsed time.

CPU

RLIMIT_CPU.

Memory

cgroup memory.max.

PIDs

cgroup pids.max.

File size

RLIMIT_FSIZE.

Open files

RLIMIT_NOFILE.

---

# language-runtime.md

Explain architecture.

```
ExecutionRequest

↓

LanguageRuntime

↓

ExecutionPlan
```

Describe

Python

Compile: none

Execute:

```
python3 main.py
```

C++

Compile

```
g++
```

Execute

```
./main
```

Rust

Compile

```
rustc
```

Execute

```
./main
```

Explain how to add a new language.

---

# testing.md

Explain testing strategy.

## Unit tests

* workspace
* runtime registry
* execution plan

## Integration tests

Namespaces

Root filesystem

PID namespace

Resource limits

Execution

Cleanup

## Stress tests

Large stdout

Fork bomb

Large file output

Recursive directories

Signal forwarding

Zombie reaping

---

# roadmap.md

Completed

* Workspace management
* Runtime registry
* Python runtime
* C++ runtime
* User namespace
* Mount namespace
* PID namespace
* Root filesystem
* cgroups
* Resource limits
* Execution engine
* seccomp
* basic metrics

Future

* network namespace
* capability dropping
* overlay filesystem
* OCI compatibility
* remote worker
* distributed scheduler
* metrics (more mature)
* streaming execution
* REST API
* CLI
