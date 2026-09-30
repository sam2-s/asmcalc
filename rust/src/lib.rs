//! Rust feature layer for the AArch64 assembly calculator.
//!
//! The arithmetic kernel is assembly and lives in `app/src/main/asm`. Rust owns
//! everything layered on top of it: expression parsing, transcendentals, unit
//! conversion, number bases, arbitrary precision, matrices and complex numbers.

pub mod algebra;
pub mod engine;
pub mod expr;
pub mod fixed;
#[cfg(target_os = "android")]
pub mod jni;

// Link the AArch64 assembly kernel into the cdylib.
//
// `cc` emits a static library, and cargo places native libraries *after* its
// own rlibs on the link line. Nothing in the Rust rlibs references the kernel
// directly, so the linker is free to discard the archive entirely and the
// assembly never reaches the .so. Forcing the dependency here makes the kernel
// part of the crate graph, which both fixes the ordering and documents the
// relationship: the Rust feature layer sits on top of the assembly kernel.
//
// Only AArch64 builds have the archive, and only those are the real thing: on
// the host `kernel_engine` supplies a Rust transcription instead, and the
// kernel parity tests are the ones that check the two agree.
#[cfg(target_arch = "aarch64")]
#[link(name = "asmcalc_kernel", kind = "static")]
extern "C" {}
pub mod kernel;
pub mod kernel_engine;
pub mod programmer;
pub mod units;
