# Language Selection for Worker

## Status
Accepted

## Context
The execution worker needs to be a performant, safe, low-level system that can interact with Linux primitives (namespaces, cgroups, seccomp) and execute user code securely.

## Decision
We selected Rust as the implementation language for the runtime worker.

## Consequences

### Positive
- Memory safety without garbage collection (important for sandbox security)
- Zero-cost abstractions for performance
- Excellent FFI capabilities for Linux system calls
- Strong type system helps prevent logical errors
- Growing ecosystem for systems programming
- Cargo provides excellent dependency management
- No runtime needed beyond standard library

### Negative
- Steeper learning curve compared to languages like Go or Python
- Longer compilation times
- Smaller ecosystem for some high-level abstractions
- More verbose error handling

## Why Not Alternatives

### C/C++
- Would provide similar performance and low-level access
- Lack of memory safety guarantees increases risk of sandbox escapes
- More prone to buffer overflows, use-after-free, etc.

### Go
- Excellent standard library and simplicity
- Garbage collection introduces unpredictable pauses
- Larger binary size due to runtime
- Less control over memory layout and system interactions

### Python/Ruby/etc.
- Interpreted languages would be too slow for execution engine
- Difficult to securely sandbox an interpreter within another sandbox
- High memory overhead

## Implementation Details
- Edition 2024 used for latest features
- Dependencies carefully selected for minimal footprint and security
- clap for command-line argument parsing
- tracing for structured logging
- libseccomp and rustix for Linux syscall interfaces