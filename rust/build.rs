use std::path::{Path, PathBuf};

/// The AArch64 assembly kernel lives in the app module, not in this crate, so
/// the feature layer and the arithmetic kernel stay in one repository but two
/// languages.
const KERNEL: &str = "../app/src/main/asm/calc_core.S";
const KERNEL_INCLUDE: &str = "../app/src/main/asm/calc_keys.inc";

fn ndk_bin(name: &str) -> PathBuf {
    let ndk = std::env::var("ANDROID_NDK_HOME")
        .or_else(|_| std::env::var("ANDROID_NDK_ROOT"))
        .expect("ANDROID_NDK_HOME must point at an NDK installation");
    Path::new(&ndk).join("toolchains/llvm/prebuilt/linux-x86_64/bin").join(name)
}

fn main() {
    println!("cargo:rerun-if-changed={KERNEL}");
    println!("cargo:rerun-if-changed={KERNEL_INCLUDE}");
    println!("cargo:rerun-if-changed=build.rs");

    let target = std::env::var("TARGET").unwrap_or_default();

    // The kernel is AArch64 assembly, so it only exists for AArch64 targets.
    //
    // aarch64-unknown-linux-musl is supported alongside the Android target so
    // the FFI parity tests can be cross compiled into a *static* AArch64
    // binary and executed under qemu on the host. Android executables need
    // /system/bin/linker64, which qemu cannot supply, so musl is the AArch64
    // target that can actually be run here.
    let mut build = cc::Build::new();
    build.file(KERNEL).include("../app/src/main/asm").warnings(false);

    match target.as_str() {
        "aarch64-linux-android" => {
            build
                .compiler(ndk_bin("aarch64-linux-android21-clang"))
                .archiver(ndk_bin("llvm-ar"));
        }
        "aarch64-unknown-linux-musl" => {
            // The NDK clang is a bare multi-target driver, so the target triple
            // has to be passed explicitly for the aarch64 flags to apply.
            build
                .compiler(ndk_bin("clang"))
                .archiver(ndk_bin("llvm-ar"))
                .flag("--target=aarch64-unknown-linux-musl")
                .flag("-ffreestanding");
        }
        _ => {
            println!(
                "cargo:warning=the AArch64 kernel is not built for target {target}; \
                 the pure Rust layers still build and test, the FFI kernel does not"
            );
            return;
        }
    }

    build.compile("asmcalc_kernel");
}
