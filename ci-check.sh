#!/bin/bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
if ! command -v flock > /dev/null 2>&1; then
    echo "error: required command 'flock' was not found" >&2
    exit 1
fi
mkdir -p "$PROJECT_ROOT/target/rs-ci"
exec 9> "$PROJECT_ROOT/target/rs-ci/ci-check.lock"
flock 9
exec env RS_CI_PROJECT_ROOT="$PROJECT_ROOT" "$PROJECT_ROOT/.rs-ci/ci-check.sh" "$@"
