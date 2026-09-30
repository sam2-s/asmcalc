//! JNI surface.
//!
//! Kotlin talks to this and nothing else. The JNI types come from the `jni`
//! crate, which is a build dependency only on Android.
//!
//! Handles are opaque pointers passed as `jlong`. Each one is a [`Session`],
//! which owns a [`Calculator`] plus the memory registers and history tape.

#![cfg(target_os = "android")]

use jni::objects::JClass;
use jni::sys::{jboolean, jlong, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;

use crate::engine::Calculator;
use crate::expr::parser::AngleMode;
use crate::programmer::{Base, Bitwise, WordSize};

/// State behind one native handle.
pub struct Session {
    calculator: Calculator,
    /// Flattened history, so Kotlin can read it without calling per row.
    history: Vec<(String, String)>,
}

impl Session {
    fn new() -> Self {
        Session {
            calculator: Calculator::new(),
            history: Vec::new(),
        }
    }

    fn sync_history(&mut self) {
        self.history = self
            .calculator
            .history()
            .iter()
            .map(|entry| (entry.expression.clone(), entry.result.clone()))
            .collect();
    }
}

fn with_session<R>(handle: jlong, fallback: R, f: impl FnOnce(&mut Session) -> R) -> R {
    if handle == 0 {
        return fallback;
    }
    let session = unsafe { &mut *(handle as usize as *mut Session) };
    f(session)
}

macro_rules! session {
    ($env:expr, $handle:expr, $body:expr) => {
        with_session($handle, Default::default(), $body)
    };
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeCreate(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let session = Box::into_raw(Box::new(Session::new()));
    session as jlong
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        unsafe {
            drop(Box::from_raw(handle as usize as *mut Session));
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativePress(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    keycode: jni::sys::jint,
) {
    session!(_env, handle, |session| {
        session.calculator.press(keycode);
        session.sync_history();
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeDisplay(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let text = session!(&env, handle, |session| session.calculator.display().to_string());
    match env.new_string(text) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeExpression(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let text = session!(&env, handle, |session| {
        session.calculator.expression().to_string()
    });
    match env.new_string(text) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeSetEngine(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    engine: jni::sys::jint,
) {
    session!(_env, handle, |session| {
        session.calculator.set_engine(engine_from_jni(engine));
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeGetEngine(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jni::sys::jint {
    session!(_env, handle, |session| {
        engine_to_jni(session.calculator.engine())
    })
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeSetAngleMode(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    mode: jni::sys::jint,
) {
    session!(_env, handle, |session| {
        session.calculator.set_angles(angle_from_jni(mode));
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeAngleMode(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let text = session!(&env, handle, |session| {
        session.calculator.angles().short_name().to_string()
    });
    match env.new_string(text) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeSubmit(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    expression: jstring,
) -> jstring {
    let mut env = env;
    let input = match read_string(&mut env, expression) {
        Some(text) => text,
        None => return std::ptr::null_mut(),
    };
    let result = session!(&env, handle, |session| {
        let result = session.calculator.submit(&input);
        session.sync_history();
        result
    });
    match env.new_string(result) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeSubmitExact(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    expression: jstring,
) -> jstring {
    let mut env = env;
    let input = match read_string(&mut env, expression) {
        Some(text) => text,
        None => return std::ptr::null_mut(),
    };
    let result = session!(&env, handle, |session| {
        let result = session.calculator.submit_exact(&input);
        session.sync_history();
        result
    });
    match env.new_string(result) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeHistorySize(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jni::sys::jint {
    session!(_env, handle, |session| session.history.len() as jni::sys::jint)
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeHistoryEntry(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jni::sys::jint,
) -> jstring {
    let entry = session!(&env, handle, |session| {
        session
            .history
            .get(index.max(0) as usize)
            .map(|(expression, result)| format!("{expression}={result}"))
            .unwrap_or_default()
    });
    match env.new_string(entry) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeRecall(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jni::sys::jint,
) {
    session!(_env, handle, |session| {
        session.calculator.recall(index.max(0) as usize);
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeMemory(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    slot: jni::sys::jint,
    operation: jni::sys::jint,
) -> f64 {
    session!(_env, handle, |session| {
        let slot = slot.max(0) as usize;
        match operation {
            0 => session.calculator.memory_add(slot),     // M+
            1 => session.calculator.memory_subtract(slot), // M-
            2 => session.calculator.memory_recall(slot),   // MR
            3 => {
                // MR as a key: put it into the display.
                let value = session.calculator.memory_recall(slot);
                let _ = session.calculator.submit(&crate::expr::parser::format_number(value));
                value
            }
            4 => {
                session.calculator.memory_store(slot);
                0.0
            }
            _ => {
                session.calculator.memory_clear(slot);
                0.0
            }
        }
    })
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeSetBase(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    base: jni::sys::jint,
) {
    session!(_env, handle, |session| {
        session.calculator.set_base(Base::from_jni(base));
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeGetBase(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jni::sys::jint {
    session!(_env, handle, |session| {
        session.calculator.base().to_jni()
    })
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeSetWordSize(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    word: jni::sys::jint,
) {
    session!(_env, handle, |session| {
        session.calculator.set_word_size(WordSize::from_jni(word));
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeProgrammerPush(
    mut _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    ch: jstring,
) -> jboolean {
    let text = match read_string(&mut _env, ch) {
        Some(text) => text,
        None => return JNI_FALSE,
    };
    let mut accepted = false;
    session!(_env, handle, |session| {
        if let Some(first) = text.chars().next() {
            accepted = session.calculator.programmer_push(first);
        }
    });
    u8::from(accepted)
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeProgrammerClear(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    session!(_env, handle, |session| {
        session.calculator.programmer_clear();
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeProgrammerBackspace(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    session!(_env, handle, |session| {
        session.calculator.programmer_backspace();
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeProgrammerApply(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    operation: jni::sys::jint,
) {
    session!(_env, handle, |session| {
        // The operand is whatever is on the display right now, which is how a
        // programmer calculator behaves: type a value, press a bitwise key.
        let operand: u64 = session
            .calculator
            .display()
            .split_whitespace()
            .collect::<String>()
            .parse()
            .unwrap_or(0);
        session.calculator.programmer_apply(Bitwise::from_jni(operation), operand);
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeProgrammerRepeat(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    operation: jni::sys::jint,
) {
    session!(_env, handle, |session| {
        session.calculator.programmer_repeat(Bitwise::from_jni(operation));
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeProgrammerBases(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let text = session!(&env, handle, |session| {
        session
            .calculator
            .programmer_bases()
            .iter()
            .map(|(base, value)| format!("{} {}", base.short_name(), value))
            .collect::<Vec<_>>()
            .join("\n")
    });
    match env.new_string(text) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_nga_asmcalc_CalcNative_nativeConvert(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    text: jstring,
) -> jstring {
    let input = match read_string(&mut env, text) {
        Some(text) => text,
        None => return std::ptr::null_mut(),
    };
    let output = session!(&env, handle, |session| match session.calculator.convert(&input) {
        Ok((category, value)) => format!(
            "{}: {} {}",
            category.name(),
            crate::units::format_result(value),
            input.rsplit(char::is_whitespace).next().unwrap_or("")
        ),
        Err(error) => error,
    });
    match env.new_string(output) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// --- conversions ---------------------------------------------------------

/// Read a Java string, returning `None` for a null or undecodable value.
///
/// Takes the environment by value in a nested scope so callers keep theirs.
fn read_string(env: &mut JNIEnv, value: jstring) -> Option<String> {
    use jni::objects::JString;
    if value.is_null() {
        return None;
    }
    let java_string = unsafe { JString::from_raw(value) };
    env.get_string(&java_string)
        .ok()
        .map(|text| text.into())
}

fn engine_from_jni(value: jni::sys::jint) -> crate::engine::Engine {
    match value {
        1 => crate::engine::Engine::Scientific,
        2 => crate::engine::Engine::Programmer,
        _ => crate::engine::Engine::Fixed,
    }
}

fn engine_to_jni(value: crate::engine::Engine) -> jni::sys::jint {
    match value {
        crate::engine::Engine::Fixed => 0,
        crate::engine::Engine::Scientific => 1,
        crate::engine::Engine::Programmer => 2,
    }
}

fn angle_from_jni(value: jni::sys::jint) -> AngleMode {
    match value {
        1 => AngleMode::Radians,
        2 => AngleMode::Gradians,
        _ => AngleMode::Degrees,
    }
}

/// Keeps the JNI_TRUE constant referenced on every toolchain.
#[allow(dead_code)]
const _: jni::sys::jboolean = if JNI_TRUE == JNI_FALSE { 0 } else { 1 };
