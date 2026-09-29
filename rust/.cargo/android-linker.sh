#!/usr/bin/env bash
# Linker driver for aarch64-linux-android.
#
# Two things this handles that a plain path to the NDK wrapper would not:
#
#   * the NDK names its wrappers per API level, so the level is selected here
#     instead of being baked into a machine specific config file
#   * the NDK wrapper defaults to the system linker, and a CI image that has GNU
#     ld ahead of lld in PATH fails with "cannot find 'ld'". Passing
#     -fuse-ld=lld makes the choice explicit rather than inherited.
set -euo pipefail

: "${ANDROID_NDK_HOME:?ANDROID_NDK_HOME must point at an NDK installation}"

BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
API="${ANDROID_API_LEVEL:-21}"

exec "$BIN/aarch64-linux-android${API}-clang" -fuse-ld=lld "$@"
