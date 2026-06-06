#!/bin/bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
RS_CI_BUILD_TOOLCHAIN="${RS_CI_BUILD_TOOLCHAIN:-1.94.0}"

configure_llvm_tools() {
    local sysroot
    local host
    local llvm_dir

    if [ -n "${LLVM_COV:-}" ] && [ -x "$LLVM_COV" ] \
        && [ -n "${LLVM_PROFDATA:-}" ] && [ -x "$LLVM_PROFDATA" ]; then
        return
    fi

    sysroot=$(rustc +"$RS_CI_BUILD_TOOLCHAIN" --print sysroot 2> /dev/null || true)
    host=$(rustc +"$RS_CI_BUILD_TOOLCHAIN" -vV 2> /dev/null \
        | sed -n 's/^host: //p')
    if [ -z "$sysroot" ] || [ -z "$host" ]; then
        return
    fi

    llvm_dir="$sysroot/lib/rustlib/$host/bin"
    if { [ -z "${LLVM_COV:-}" ] || [ ! -x "$LLVM_COV" ]; } \
        && [ -x "$llvm_dir/llvm-cov" ]; then
        export LLVM_COV="$llvm_dir/llvm-cov"
    fi
    if { [ -z "${LLVM_PROFDATA:-}" ] || [ ! -x "$LLVM_PROFDATA" ]; } \
        && [ -x "$llvm_dir/llvm-profdata" ]; then
        export LLVM_PROFDATA="$llvm_dir/llvm-profdata"
    fi
}

configure_llvm_tools
exec env RS_CI_PROJECT_ROOT="$PROJECT_ROOT" "$PROJECT_ROOT/.rs-ci/coverage.sh" "$@"
