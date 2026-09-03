# Compilation and Execution Separation

## Status
Accepted

## Context
For compiled languages (C++, Rust, etc.), we need to compile source code before execution. The question is whether to perform compilation and execution in the same sandboxed environment or to separate them.

## Decision
We will separate compilation and execution into distinct sandboxed invocations. Compilation happens first in a sandbox, and if successful, execution happens in a second (potentially fresh) sandbox.

## Consequences

### Positive
- Compilation errors cannot affect the execution environment
- Ability to apply different security policies to compilation vs execution (e.g., allow network access for dependency download during compilation but not execution)
- Cleaner separation: execution environment only contains compiled binaries, not source code or compilers
- Compilers themselves run in isolation, reducing risk from compiler vulnerabilities

### Negative
- Slightly more complex execution flow
- Need to transfer compiled artifacts between sandbox instances
- Two sandbox setups/teardowns instead of one
- Potential performance overhead from double sandbox initialization

## Implementation Details
- See `runtime/src/engine/execution_engine.rs` for the separation logic
- The `ExecutionPlan` contains optional `compile` and required `execute` fields
- If compilation is specified, it runs first; only if successful does execution proceed
- Both compilation and execution use the same sandboxing mechanisms (namespaces, cgroups, seccomp)
- The workspace persists between compilation and execution phases for artifact sharing