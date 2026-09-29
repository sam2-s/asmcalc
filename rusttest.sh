#!/usr/bin/env bash
# Run the Rust feature layer against the real AArch64 assembly kernel.
#
# The kernel is AArch64 assembly, so the tests are cross compiled to a static
# aarch64-unknown-linux-musl binary and executed under qemu. musl is used
# because it links statically: an Android binary would need
# /system/bin/linker64, which qemu cannot provide.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT/rust"

TARGET=aarch64-unknown-linux-musl
OUT="target/$TARGET/debug/deps"

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

# RUSTFLAGS is set to a single space so it overrides the machine wide
# ~/.cargo/config.toml, which forces -static and would break the cdylib.
RUSTFLAGS=" " cargo test --offline --target "$TARGET" --no-run

if ! command -v "$QEMU" >/dev/null 2>&1 && [[ ! -x "$QEMU" ]]; then
    echo "error: $QEMU not found; these tests are AArch64 binaries" >&2
    echo "       arch: sudo pacman -S qemu-user-static" >&2
    exit 2
fi

status=0
for binary in "$OUT"/*; do
    case "$binary" in
        *.d|*.o|*.rlib) continue ;;
    esac
    [[ -x "$binary" && -f "$binary" ]] || continue
    "$QEMU" "$binary" --test-threads=1 || status=1
done

exit $status
