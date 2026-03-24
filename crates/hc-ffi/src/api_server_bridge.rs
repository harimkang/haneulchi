use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::HcString;

struct ApiServerHandle {
    socket_path: PathBuf,
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

fn read_optional_c_string(value: *const c_char) -> Result<Option<String>, String> {
    if value.is_null() {
        return Ok(None);
    }

    let text = unsafe { CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| error.to_string())?
        .trim()
        .to_string();

    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

pub fn api_server_start_json(socket_path: Option<&str>) -> Result<String, String> {
    let handle = server_handle();
    let mut guard = handle
        .lock()
        .map_err(|_| "api_server_lock_poisoned".to_string())?;

    if let Some(existing) = guard.as_ref() {
        return serde_json::to_string(&serde_json::json!({
            "ok": true,
            "status": "running",
            "socket_path": existing.socket_path,
        }))
        .map_err(|error| error.to_string());
    }

    let socket_path = resolve_api_server_socket_path(socket_path);
    let server = hc_api::server::ApiServer::bind(&socket_path)?;
    std::thread::spawn(move || {
        let _ = server.serve_requests(usize::MAX);
    });

    *guard = Some(ApiServerHandle {
        socket_path: socket_path.clone(),
    });

    serde_json::to_string(&serde_json::json!({
        "ok": true,
        "status": "running",
        "socket_path": socket_path,
    }))
    .map_err(|error| error.to_string())
}

fn server_handle() -> &'static Mutex<Option<ApiServerHandle>> {
    static HANDLE: OnceLock<Mutex<Option<ApiServerHandle>>> = OnceLock::new();
    HANDLE.get_or_init(|| Mutex::new(None))
}

pub fn resolve_api_server_socket_path(socket_path: Option<&str>) -> PathBuf {
    socket_path
        .map(PathBuf::from)
        .or_else(|| std::env::var("HC_CONTROL_SOCKET").ok().map(PathBuf::from))
        .unwrap_or_else(default_socket_path)
}

fn default_socket_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Haneulchi")
        .join("run")
        .join("control.sock")
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_api_server_start_json(socket_path: *const c_char) -> HcString {
    string_to_hcstring(
        read_optional_c_string(socket_path)
            .and_then(|socket_path| api_server_start_json(socket_path.as_deref())),
    )
}
