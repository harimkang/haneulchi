use std::collections::BTreeMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;
use std::sync::{Mutex, OnceLock};

use hc_runtime::terminal::geometry::TerminalGeometry;
use hc_runtime::terminal::runtime::TerminalRuntime;
use hc_runtime::terminal::session::TerminalLaunchConfig;
use serde::Deserialize;

use crate::{HcBytes, HcString};

#[derive(Debug, Deserialize)]
struct SpawnRequest {
    program: String,
    args: Vec<String>,
    #[serde(default)]
    current_directory: Option<PathBuf>,
    geometry: SpawnGeometry,
    #[serde(default)]
    environment: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct SpawnGeometry {
    cols: u16,
    rows: u16,
}

pub(crate) fn runtime() -> &'static Mutex<TerminalRuntime> {
    static RUNTIME: OnceLock<Mutex<TerminalRuntime>> = OnceLock::new();
    RUNTIME.get_or_init(|| Mutex::new(TerminalRuntime::default()))
}

pub(crate) fn lock_runtime() -> Result<std::sync::MutexGuard<'static, TerminalRuntime>, String> {
    runtime()
        .lock()
        .map_err(|_| "terminal runtime lock poisoned".to_string())
}

pub fn reset_runtime_for_tests() {
    if let Ok(mut runtime) = lock_runtime() {
        *runtime = TerminalRuntime::default();
    }
}

fn read_c_string(value: *const c_char) -> Result<String, String> {
    if value.is_null() {
        return Err("null pointer".to_string());
    }

    let text = unsafe { CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| error.to_string())?;

    Ok(text.to_string())
}

fn string_to_hcstring(value: Result<String, String>) -> HcString {
    let payload = match value {
        Ok(value) => value,
        Err(error) => serde_json::json!({ "error": error }).to_string(),
    };
    let string = CString::new(payload).expect("json payload is nul-free");

    HcString {
        ptr: string.into_raw(),
    }
}

fn bytes_to_hcbytes(value: Result<Vec<u8>, String>) -> HcBytes {
    match value {
        Ok(mut bytes) if !bytes.is_empty() => {
            let len = bytes.len();
            let ptr = bytes.as_mut_ptr();
            std::mem::forget(bytes);
            HcBytes { ptr, len }
        }
        Ok(_) | Err(_) => HcBytes {
            ptr: ptr::null_mut(),
            len: 0,
        },
    }
}

pub fn terminal_session_spawn_json(config_json: &str) -> Result<String, String> {
    let request: SpawnRequest =
        serde_json::from_str(config_json).map_err(|error| error.to_string())?;
    let launch = TerminalLaunchConfig {
        program: request.program,
        args: request.args,
        current_directory: request.current_directory,
        environment: request.environment,
    };
    let geometry = TerminalGeometry::new(request.geometry.cols, request.geometry.rows);
    let session_id = lock_runtime().and_then(|mut runtime| {
        runtime
            .spawn(launch, geometry)
            .map_err(|error| error.to_string())
    })?;

    serde_json::to_string(&serde_json::json!({ "session_id": session_id }))
        .map_err(|error| error.to_string())
}

pub fn terminal_session_drain(session_id: &str) -> Result<Vec<u8>, String> {
    lock_runtime().and_then(|mut runtime| {
        runtime
            .drain_output(session_id)
            .map_err(|error| error.to_string())
    })
}

pub fn terminal_session_write(session_id: &str, data: &[u8]) -> Result<(), String> {
    lock_runtime().and_then(|mut runtime| {
        runtime
            .write_input(session_id, data)
            .map_err(|error| error.to_string())
    })
}

pub fn terminal_session_resize(session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
    lock_runtime().and_then(|mut runtime| {
        runtime
            .resize(session_id, TerminalGeometry::new(cols, rows))
            .map_err(|error| error.to_string())
    })
}

pub fn terminal_session_terminate(session_id: &str) -> Result<(), String> {
    lock_runtime().and_then(|mut runtime| {
        runtime
            .terminate(session_id)
            .map_err(|error| error.to_string())
    })
}

pub fn terminal_session_snapshot_json(session_id: &str) -> Result<String, String> {
    let snapshot = lock_runtime().and_then(|runtime| {
        runtime
            .snapshot(session_id)
            .map_err(|error| error.to_string())
    })?;

    serde_json::to_string(&snapshot).map_err(|error| error.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_terminal_session_spawn_json(config_json: *const c_char) -> HcString {
    string_to_hcstring(
        read_c_string(config_json).and_then(|text| terminal_session_spawn_json(&text)),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_terminal_session_drain(session_id: *const c_char) -> HcBytes {
    bytes_to_hcbytes(read_c_string(session_id).and_then(|text| terminal_session_drain(&text)))
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `session_id` must point to a valid NUL-terminated C string, and `(ptr, len)`
/// must describe a readable buffer for the duration of this call.
pub unsafe extern "C" fn hc_terminal_session_write(
    session_id: *const c_char,
    ptr: *const u8,
    len: usize,
) -> i32 {
    if ptr.is_null() {
        return 1;
    }

    let result = read_c_string(session_id).and_then(|text| {
        // SAFETY: the FFI caller owns the `(ptr, len)` buffer and we reject null.
        let data = unsafe { std::slice::from_raw_parts(ptr, len) };
        terminal_session_write(&text, data)
    });

    if result.is_ok() { 0 } else { 1 }
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_terminal_session_resize(
    session_id: *const c_char,
    cols: u16,
    rows: u16,
) -> i32 {
    let result =
        read_c_string(session_id).and_then(|text| terminal_session_resize(&text, cols, rows));

    if result.is_ok() { 0 } else { 1 }
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_terminal_session_terminate(session_id: *const c_char) -> i32 {
    let result = read_c_string(session_id).and_then(|text| terminal_session_terminate(&text));

    if result.is_ok() { 0 } else { 1 }
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_terminal_session_snapshot_json(session_id: *const c_char) -> HcString {
    string_to_hcstring(
        read_c_string(session_id).and_then(|text| terminal_session_snapshot_json(&text)),
    )
}
