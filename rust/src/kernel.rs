//! FFI bindings to the AArch64 assembly kernel.
//!
//! The kernel is written in assembly and exported with C linkage. There is no
//! bindgen and no C shim between Rust and it: the symbols are declared directly
//! here, and the struct layout is mirrored from `calc_state.h`, whose field
//! offsets the C shim asserts at compile time.

/// Mirror of `calc_state` in `app/src/main/asm/calc_state.h`.
///
/// The kernel writes these fields by absolute offset, so the layout is part of
/// the contract. The host-side tests check the offsets against the constants
/// used by the assembly.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CalcState {
    pub acc: u64,
    pub entry: u64,
    pub rhs: u64,
    pub acc_sign: u8,
    pub entry_sign: u8,
    pub rhs_sign: u8,
    pub dot: u8,
    pub int_digits: u8,
    pub frac_digits: u8,
    pub op: u8,
    pub last_op: u8,
    pub typing: u8,
    pub error: u8,
    pub reserved: [u8; 6],
}

impl Default for CalcState {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl CalcState {
    pub const fn zeroed() -> Self {
        CalcState {
            acc: 0,
            entry: 0,
            rhs: 0,
            acc_sign: 0,
            entry_sign: 0,
            rhs_sign: 0,
            dot: 0,
            int_digits: 0,
            frac_digits: 0,
            op: 0,
            last_op: 0,
            typing: 0,
            error: 0,
            reserved: [0; 6],
        }
    }
}

/// The buffer size the kernel is guaranteed to write within, mirroring
/// `CALC_FORMAT_BUFFER_SIZE`.
pub const FORMAT_BUFFER_SIZE: usize = 64;

// Keycodes, mirroring `app/src/main/asm/calc_keys.inc`.
pub const KEY_0: i32 = 0;
pub const KEY_1: i32 = 1;
pub const KEY_2: i32 = 2;
pub const KEY_3: i32 = 3;
pub const KEY_4: i32 = 4;
pub const KEY_5: i32 = 5;
pub const KEY_6: i32 = 6;
pub const KEY_7: i32 = 7;
pub const KEY_8: i32 = 8;
pub const KEY_9: i32 = 9;
pub const KEY_DOT: i32 = 10;
pub const KEY_EQUALS: i32 = 11;
pub const KEY_ADD: i32 = 12;
pub const KEY_SUB: i32 = 13;
pub const KEY_MUL: i32 = 14;
pub const KEY_DIV: i32 = 15;
pub const KEY_CLEAR: i32 = 16;
pub const KEY_SIGN: i32 = 17;

extern "C" {
    pub fn calc_reset(state: *mut CalcState);
    pub fn calc_key(state: *mut CalcState, keycode: i32);
    pub fn calc_format(state: *mut CalcState, out: *mut u8) -> i32;
}
