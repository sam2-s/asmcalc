#!/usr/bin/env bash
# Linker driver for aarch64-linux-android.
#
# This script also exists to neutralise a machine wide ~/.cargo/config.toml that
# adds `-C link-arg=-static`. That is right for a static executable and wrong
# for a cdylib: a .so needs a dynamic segment, and passing it here as a *linker*
# argument is ignored by the NDK driver, so the flag has to be removed at the
# source instead. Cargo has no way to unset it from a project config, so the
# project instead sets RUSTFLAGS explicitly in the Gradle build, which replaces
# the machine config for the whole invocation.
#
# Either way this driver also has to:
#   * pick the NDK wrapper for the requested API level, rather than hardcoding
#     a path from one machine
#   * force lld, because a CI image with GNU ld first on PATH fails with
#     "cannot find 'ld'" when linking aarch64 objects
set -euo pipefail

: "${ANDROID_NDK_HOME:?ANDROID_NDK_HOME must point at an NDK installation}"

BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
API="${ANDROID_API_LEVEL:-21}"

# Drop any -static/-static-libgcc the caller inherited, then link with lld.
args=()
for arg in "$@"; do
    case "$arg" in
        -static|-static-libgcc|-static-pie) continue ;;
    esac
    args+=("$arg")
done

exec "$BIN/aarch64-linux-android${API}-clang" -fuse-ld=lld "${args[@]}"
