#!/usr/bin/env bash
# NDK clang driver for static aarch64-unknown-linux-musl test binaries.
#
# The NDK ships a bare clang, not a musl-targeting wrapper, so the target
# triple has to be injected before the linker flags. cargo's `linker` option
# runs this script; `link-arg` values arrive as arguments.
: "${ANDROID_NDK_HOME:?ANDROID_NDK_HOME must point at an NDK installation}"
exec "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/clang" \
    --target=aarch64-unknown-linux-musl \
    "$@"
