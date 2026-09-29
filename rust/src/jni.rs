//! JNI surface.
//!
//! Kotlin talks to this and nothing else. The JNI types come from the `jni`
//! crate, which is a build dependency only on Android.
//!
//! Handles are opaque pointers passed as `jlong`. Each one is a [`Session`],
//! which owns a [`Calculator`] plus the memory registers and history tape.

#![cfg(target_os = "android")]

use jni::objects::JClass;
use jni::sys::{jlong, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;

use crate::engine::Calculator;
use crate::expr::parser::AngleMode;

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

// --- conversions ---------------------------------------------------------

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
