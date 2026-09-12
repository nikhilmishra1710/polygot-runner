# Development

## Prerequisites

* Rust (stable)
* Cargo
* Go (for the `api/` service)
* python3 and g++ in PATH (the Rust ToolchainRegistry discovers them)
* Linux or WSL2

Note: The current implementation uses Linux namespaces, cgroups, and seccomp, so a Linux kernel with these features is required.

---

## Layout

* `runtime/` — Rust sandbox + workerd (binary `workerd`, Unix socket at `/tmp/runtime-worker.sock`)
* `api/` — Go gRPC API service (`ExecutionService`)
* `proto/execution.proto` — public API contract. Regenerate Go stubs with `scripts/gen-proto.sh`.
* `api/tests/` — end-to-end integration tests (gRPC client → Go API → unix socket → workerd → sandbox)

## API wire protocol (internal)

Go API → workerd over a Unix socket: 4-byte big-endian length prefix + JSON payload
(`{"Execute": {job}}` / `{"Result": {job_result}}` | `{"Error": "..."}`; `Vec<u8>` is a JSON
integer array). `runtime/tests/protocol_json.rs` pins the exact wire shapes.

---

## Build

```
cd runtime && cargo build
cd api && go build ./...
```

---

## Tests

Root-free tests:

```
cd runtime && cargo test --test frame --test protocol_json
cd api && go test ./...
```

Sandbox tests (workerd + full execution) require real root — namespaces do not
suffice because cgroup writes need it. Run locally with sudo:

```
sudo -E env PATH="$PATH" bash -c 'cd api && go test ./tests -v -count=1'
cd runtime && sudo -E env PATH="$PATH" cargo test
```

Or run everything in a privileged Docker container (no host root needed):

```
scripts/test-e2e-docker.sh
```

(requires Docker Desktop with WSL integration enabled, or native Docker on Linux).

---

## Formatting / Linting

```
cd runtime && cargo fmt && cargo fmt --check && cargo clippy -- -D warnings
cd api && gofmt -l . && go vet ./...
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
