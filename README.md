# Polyglot Execution Platform

A polyglot code execution platform built from first principles using Linux primitives.

The project aims to execute untrusted code securely across multiple programming languages while exploring the technologies that power modern online IDEs and code execution services.

Unlike platforms that rely solely on Docker, the execution runtime in this project will be built directly on Linux namespaces, cgroups, seccomp, mount isolation, and process management.

---

## Vision

Build a secure, scalable execution platform capable of running code written in multiple languages inside isolated environments.

Supported languages (planned):

* C++
* Rust
* Python
* Go
* Java
* JavaScript

Long-term goals include:

* Secure sandboxing
* Resource isolation
* Job queues
* Streaming logs
* Web IDE
* REST API
* SDKs
* Horizontal worker scaling

---

## High-Level Architecture

```
           Web IDE
               │
               ▼
          Go API Server
               │
               ▼
           Job Queue
               │
               ▼
         Rust Worker(s)
               │
               ▼
 Linux Sandbox Runtime
               │
               ▼
     Compile & Execute
```

The Rust worker owns the execution lifecycle.

The API layer never directly executes user code.

---

## Repository Structure

```
api/            Go API server
worker/         Rust execution engine
sdk/            Client SDKs
web/            Web IDE
deployment/     Docker & Kubernetes manifests
docs/           Design documents
proto/          Shared contracts
scripts/        Development scripts
```

---

## Development Status

Current Phase:

> Phase 0 — Project setup and architecture

Completed:

* Repository structure
* Rust worker crate
* Library + CLI layout
* CI
* Module boundaries

Upcoming:

* Local execution engine
* Workspace management
* Process runner
* Language runtime abstraction

---

## Guiding Principles

* Linux-first design
* Library-first architecture
* Small, reviewable iterations
* Security by design
* Language-agnostic execution pipeline
* Minimal dependencies

---

## Building

Worker:

```bash
cd worker
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

---

## Roadmap

The roadmap for each implementation phase is available in:

```
docs/roadmap.md
```

Architecture decisions are documented in:

```
docs/architecture.md
```
