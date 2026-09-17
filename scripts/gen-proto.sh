#!/usr/bin/env bash
# Regenerate Go code and trigger Rust generation from proto/execution.proto
# Requires: protoc, protoc-gen-go, protoc-gen-go-grpc, cargo
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API_DIR="$ROOT/api"
RUST_DIR="$ROOT/runtime"
export PATH="$PATH:$(go env GOPATH)/bin"

# 1. Generate Go Code
echo "Generating Go protobufs..."
protoc \
-I "$ROOT/proto" \
--go_out="$API_DIR" --go_opt=module=runtime-platform/api \
--go-grpc_out="$API_DIR" --go-grpc_opt=module=runtime-platform/api \
"$ROOT/proto/execution.proto"

echo "Generated api/gen/execution/v1/"

# 2. Synchronize Rust Code
echo "Synchronizing Rust protobufs..."
cd "$RUST_DIR"

# Touching the proto file forces Cargo to recognize it as "new"
# so it guarantees build.rs will regenerate the Rust code immediately.
touch "$ROOT/proto/execution.proto"

# Running cargo check compiles the proto file without building the whole binary
cargo check

echo "Rust generation complete and in sync!"