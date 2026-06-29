# Development

## Prerequisites

* Rust (stable)
* Cargo
* Linux or WSL2

Future phases will additionally require:

* cgroups v2
* Linux namespaces
* seccomp support

---

## Build

```
cd worker
cargo build
```

---

## Tests

```
cargo test
```

---

## Formatting

```
cargo fmt
cargo fmt --check
```

---

## Linting

```
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
