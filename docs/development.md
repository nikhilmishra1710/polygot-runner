# Development

## Prerequisites

* Rust (stable)
* Cargo
* Linux or WSL2

Note: The current implementation uses Linux namespaces, cgroups, and seccomp, so a Linux kernel with these features is required.

---

## Build

The execution engine is located in the `runtime/` directory.

```
cd runtime
cargo build
```

---

## Tests

Run the test suite for the runtime worker:

```
cd runtime
cargo test
```

---

## Formatting

```
cd runtime
cargo fmt
cargo fmt --check
```

---

## Linting

```
cd runtime
cargo clippy -- -D warnings
```

---

## Development Workflow

Each phase should end with:

* Working code
* Passing tests
* Clean formatting
* No Clippy warnings
* Updated documentation

Large refactors should be avoided. Functionality should evolve incrementally.

## Current Status

The project is in Phase 0 (Project setup and architecture) with the following completed:

* Repository structure
* Rust worker crate (`runtime/`)
* Library + CLI layout
* CI (GitHub Actions)
* Module boundaries

The execution engine, language runtimes for Python and C++, and basic sandboxing (namespaces, cgroups, seccomp) are implemented.

See the [roadmap](./roadmap.md) for upcoming phases.
