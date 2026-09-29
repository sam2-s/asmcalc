#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

resolve_ndk() {
    local candidate
    for candidate in "${ANDROID_NDK_HOME:-}" "${ANDROID_NDK_ROOT:-}"; do
        if [[ -n "$candidate" && -d "$candidate" ]]; then
            echo "$candidate"
            return
        fi
    done
    for candidate in "${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}/ndk"/* "$HOME/Android/Sdk/ndk"/*; do
        if [[ -d "$candidate" ]]; then
            echo "$candidate"
        fi
    done | sort -V | tail -n 1
}

NDK="$(resolve_ndk)"
TOOLCHAIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64"
CC="$TOOLCHAIN/bin/aarch64-linux-android21-clang"
LD="$TOOLCHAIN/bin/ld.lld"
OUT="$ROOT/build/hosttest"

if [[ -n "${QEMU_AARCH64:-}" ]]; then
    QEMU="$QEMU_AARCH64"
elif command -v qemu-aarch64 >/dev/null 2>&1; then
    QEMU=qemu-aarch64
elif command -v qemu-aarch64-static >/dev/null 2>&1; then
    QEMU=qemu-aarch64-static
elif [[ -x "$HOME/.local/opt/qemu/usr/bin/qemu-aarch64-static" ]]; then
    QEMU="$HOME/.local/opt/qemu/usr/bin/qemu-aarch64-static"
else
    QEMU=qemu-aarch64
fi

if [[ ! -x "$CC" || ! -x "$LD" ]]; then
    echo "error: NDK toolchain not found at $TOOLCHAIN" >&2
    echo "       set ANDROID_NDK_HOME to your NDK r27 installation" >&2
    exit 1
fi

mkdir -p "$OUT"

sources=()
while IFS= read -r source; do
    sources+=("$source")
done < <(find "$ROOT/hosttest" "$ROOT/app/src/main/asm" "$ROOT/app/src/test/asm" -name '*.S' 2>/dev/null | sort)

if [[ ${#sources[@]} -eq 0 ]]; then
    echo "error: no assembly sources found" >&2
    exit 1
fi

objects=()
for source in "${sources[@]}"; do
    object="$OUT/$(basename "${source%.S}").o"
    "$CC" -c -O2 -Wall -o "$object" "$source"
    objects+=("$object")
done

"$LD" -static -e _start -Ttext=0x400000 -Tdata=0x410000 \
    -o "$OUT/calc_tests.elf" "${objects[@]}"
"$TOOLCHAIN/bin/llvm-strip" "$OUT/calc_tests.elf"

if [[ "${SKIP_RUN:-0}" == "1" ]]; then
    echo "linked: $OUT/calc_tests.elf"
    exit 0
fi

if ! command -v "$QEMU" >/dev/null 2>&1; then
    echo "error: $QEMU not found; the test binary is AArch64 and the host is x86_64" >&2
    echo "       arch: sudo pacman -S qemu-user-static   (or set QEMU_AARCH64)" >&2
    echo "       built: $OUT/calc_tests.elf" >&2
    exit 2
fi

"$QEMU" "$OUT/calc_tests.elf"
