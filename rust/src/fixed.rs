//! Safe wrapper around the AArch64 assembly kernel.
//!
//! `FixedEngine` replays a key sequence through the real assembly code and
//! formats the result with `calc_format`. This is the same code path the app
//! uses, so anything it computes is computed by assembly.

use crate::kernel::{self, CalcState, FORMAT_BUFFER_SIZE};

/// A calculator whose arithmetic is performed entirely in AArch64 assembly.
pub struct FixedEngine {
    state: CalcState,
    buffer: [u8; FORMAT_BUFFER_SIZE],
}

impl Default for FixedEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FixedEngine {
    pub fn new() -> Self {
        let mut state = CalcState::zeroed();
        // SAFETY: `calc_reset` only writes the 40 bytes of `state` through the
        // pointer it is given; `state` is a live, correctly sized local.
        unsafe { kernel::calc_reset(&mut state) };
        FixedEngine {
            state,
            buffer: [0u8; FORMAT_BUFFER_SIZE],
        }
    }

    /// Send one keycode to the kernel.
    pub fn press(&mut self, keycode: i32) {
        // SAFETY: `calc_key` reads and writes only the state it is given, and
        // `self.state` outlives the call.
        unsafe { kernel::calc_key(&mut self.state, keycode) };
    }

    /// Press a key described by a character, matching the harness convention:
    /// digits, `.`, `=`, `+`, `-`, `*`, `/`, `c` for clear and `~` for sign.
    pub fn press_char(&mut self, ch: char) {
        if let Some(key) = keycode_for(ch) {
            self.press(key);
        }
    }

    /// Replay a whole key sequence.
    pub fn press_str(&mut self, keys: &str) {
        for ch in keys.chars() {
            self.press_char(ch);
        }
    }

    /// Format the current display value using the assembly formatter.
    pub fn display(&mut self) -> String {
        let length = {
            // SAFETY: the kernel writes at most `FORMAT_BUFFER_SIZE` bytes into
            // the buffer, and `self.buffer` is exactly that size and live.
            let length = unsafe { kernel::calc_format(&mut self.state, self.buffer.as_mut_ptr()) };
            length.clamp(0, FORMAT_BUFFER_SIZE as i32) as usize
        };
        let bytes = &self.buffer[..length];
        String::from_utf8_lossy(bytes).into_owned()
    }

    /// Whether the kernel has latched an error.
    pub fn is_error(&self) -> bool {
        self.state.error != 0
    }
}

/// Map a character to a keycode, using the same table as the assembly test
/// harness so both agree on what a key sequence means.
pub fn keycode_for(ch: char) -> Option<i32> {
    Some(match ch {
        '0'..='9' => ch as i32 - '0' as i32,
        '.' => kernel::KEY_DOT,
        '=' => kernel::KEY_EQUALS,
        '+' => kernel::KEY_ADD,
        '-' => kernel::KEY_SUB,
        '*' => kernel::KEY_MUL,
        '/' => kernel::KEY_DIV,
        'c' => kernel::KEY_CLEAR,
        '~' => kernel::KEY_SIGN,
        _ => return None,
    })
}
