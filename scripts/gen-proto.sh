#!/usr/bin/env bash
# Regenerate Go code from proto/execution.proto into api/gen/execution/v1/.
# Requires: protoc, protoc-gen-go, protoc-gen-go-grpc (on PATH or GOPATH/bin).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API_DIR="$ROOT/api"
export PATH="$PATH:$(go env GOPATH)/bin"

# module=... strips the module prefix so output lands at
# api/gen/execution/v1/ matching the go_package option.
protoc \
  -I "$ROOT/proto" \
  --go_out="$API_DIR" --go_opt=module=runtime-platform/api \
  --go-grpc_out="$API_DIR" --go-grpc_opt=module=runtime-platform/api \
  "$ROOT/proto/execution.proto"

echo "Generated api/gen/execution/v1/"
