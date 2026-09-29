#!/usr/bin/env bash
# NDK linker driver for aarch64-linux-android.
#
# The NDK ships aarch64-linux-android21-clang wrappers, but the exact API level
# is baked into their names, so the driver is selected here instead of being
# hardcoded in a machine specific config file.
: "${ANDROID_NDK_HOME:?ANDROID_NDK_HOME must point at an NDK installation}"
API="${ANDROID_API_LEVEL:-21}"
exec "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android${API}-clang" "$@"
