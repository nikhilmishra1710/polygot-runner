#!/usr/bin/env bash
# Run the full test suite (including root-gated sandbox/api integration
# tests) inside a privileged Docker container.
#
# Usage:
#   scripts/test-e2e-docker.sh            # full suite in container
#   scripts/test-e2e-docker.sh -- <args>  # forward extra args into container CMD
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE=ide-runtime-e2e

# Delete the old image if it exists (ignore errors if it doesn't)
docker rmi "$IMAGE" 2>/dev/null || true

docker build -f "$ROOT/deployment/docker/Dockerfile.e2e" -t "$IMAGE" "$ROOT"

# --privileged: sandbox needs user/net/mount namespaces, cgroup writes,
# ptrace/raw mounts. Do NOT use --cgroupns=private: in Docker Desktop/WSL2
# the private cgroup namespace has no delegated subtree, so cgroup creation
# fails with EACCES.
# -t allocates a TTY only when interactive (CI has none).
INTERACTIVE=""
if [ -t 0 ]; then INTERACTIVE="-t"; fi

exec docker run --rm $INTERACTIVE \
--privileged \
--cgroupns=host \
-v /sys/fs/cgroup:/sys/fs/cgroup:rw \
"$IMAGE" "$@"
