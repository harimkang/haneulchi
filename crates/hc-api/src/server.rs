use std::fs;
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use httparse::Request;
use serde_json::Value;

use crate::automation::reconcile_now_json;
use crate::dispatch::dispatch_send_json;
use crate::envelope::{error_json, success_json};
use crate::sessions::{
    session_attach_task_json, session_detach_task_json, session_details_json, session_focus_json,
    session_release_takeover_json, session_takeover_json, sessions_list_json_filtered,
};
use crate::state::{current_snapshot, state_json_for};
use crate::tasks::{task_automation_mode_json, task_create_json, task_move_json, tasks_list_json};
use crate::workflow::{workflow_reload_json, workflow_validate_json};

pub struct ApiServer {
    socket_path: PathBuf,
    listener: UnixListener,
}

impl ApiServer {
    pub fn bind(path: impl AsRef<Path>) -> Result<Self, String> {
        let socket_path = path.as_ref().to_path_buf();
        if let Some(parent) = socket_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        if socket_path.exists() {
            match UnixStream::connect(&socket_path) {
                Ok(_) => {
                    return Err(format!("socket_already_owned:{}", socket_path.display()));
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        ErrorKind::ConnectionRefused
                            | ErrorKind::NotConnected
                            | ErrorKind::ConnectionReset
                            | ErrorKind::TimedOut
                            | ErrorKind::WouldBlock
                            | ErrorKind::InvalidInput
                            | ErrorKind::AddrNotAvailable
                            | ErrorKind::NotFound
                    ) =>
                {
                    fs::remove_file(&socket_path)
                        .map_err(|remove_error| remove_error.to_string())?;
                }
                Err(error) => return Err(error.to_string()),
            }
        }
        let listener = UnixListener::bind(&socket_path).map_err(|error| error.to_string())?;
        fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            socket_path,
            listener,
        })
    }

    pub fn serve_requests(self, limit: usize) -> Result<(), String> {
        for _ in 0..limit {
            let (stream, _) = self.listener.accept().map_err(|error| error.to_string())?;
            handle_stream(stream)?;
        }
        let _ = fs::remove_file(&self.socket_path);
        Ok(())
    }
}

fn handle_stream(mut stream: UnixStream) -> Result<(), String> {
    let bytes = match read_http_request(&mut stream) {
        Ok(bytes) => bytes,
        Err(error) => {
            let payload = error_json("invalid_request", &error, current_snapshot().ok().as_ref())?;
            write_response(&mut stream, 400, &payload)?;
            return Ok(());
        }
    };

    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut request = Request::new(&mut headers);
    let header_len = match request.parse(&bytes).map_err(|error| error.to_string())? {
        httparse::Status::Complete(len) => len,
        httparse::Status::Partial => {
            let payload = error_json(
                "invalid_request",
                "incomplete_request",
                current_snapshot().ok().as_ref(),
            )?;
            write_response(&mut stream, 400, &payload)?;
            return Ok(());
        }
    };

    let method = match request.method {
        Some(method) => method,
        None => {
            let payload = error_json(
                "invalid_request",
                "missing_method",
                current_snapshot().ok().as_ref(),
            )?;
            write_response(&mut stream, 400, &payload)?;
            return Ok(());
        }
    };
    let path = match request.path {
        Some(path) => path,
        None => {
            let payload = error_json(
                "invalid_request",
                "missing_path",
                current_snapshot().ok().as_ref(),
            )?;
            write_response(&mut stream, 400, &payload)?;
            return Ok(());
        }
    };
    let body = match std::str::from_utf8(&bytes[header_len..]) {
        Ok(body) => body.trim(),
        Err(error) => {
            let payload = error_json(
                "invalid_request",
                &error.to_string(),
                current_snapshot().ok().as_ref(),
            )?;
            write_response(&mut stream, 400, &payload)?;
            return Ok(());
        }
    };

    let (status, payload) = match resolve_route_response(route(method, path, body)) {
        Ok(response) => response,
        Err(error) => (
            500,
            error_json("internal_error", &error, current_snapshot().ok().as_ref())?,
        ),
    };
    write_response(&mut stream, status, &payload)?;
    Ok(())
}

fn route(method: &str, raw_path: &str, body: &str) -> Result<(u16, String), String> {
    let (path, query) = raw_path.split_once('?').unwrap_or((raw_path, ""));
    let segments = path.trim_start_matches('/').split('/').collect::<Vec<_>>();
    let query = parse_query(query);

    match (method, segments.as_slice()) {
        ("GET", ["v1", "state"]) => wrap_success(
            200,
            state_json_for(query.get("project_id").map(String::as_str))?,
        ),
        ("GET", ["v1", "sessions"]) => wrap_success(
            200,
            sessions_list_json_filtered(
                query.get("project_id").map(String::as_str),
                query.get("state").map(String::as_str),
                query.get("mode").map(String::as_str),
                query.get("task_id").map(String::as_str),
                query
                    .get("dispatchable")
                    .and_then(|value| value.parse::<bool>().ok()),
            )?,
        ),
        ("GET", ["v1", "sessions", session_id]) => {
            wrap_success(200, session_details_json(session_id)?)
        }
        ("GET", ["v1", "tasks"]) => wrap_success(200, tasks_list_json(None)?),
        ("POST", ["v1", "sessions", session_id, "focus"]) => {
            wrap_success(202, session_focus_json(session_id)?)
        }
        ("POST", ["v1", "sessions", session_id, "takeover"]) => {
            wrap_success(200, session_takeover_json(session_id)?)
        }
        ("POST", ["v1", "sessions", session_id, "release-takeover"]) => {
            wrap_success(200, session_release_takeover_json(session_id)?)
        }
        ("POST", ["v1", "sessions", session_id, "attach-task"]) => {
            let json = parse_json(body)?;
            let task_id = json
                .get("task_id")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing_task_id".to_string())?;
            wrap_success(200, session_attach_task_json(session_id, task_id)?)
        }
        ("POST", ["v1", "sessions", session_id, "detach-task"]) => {
            wrap_success(200, session_detach_task_json(session_id)?)
        }
        ("POST", ["v1", "tasks"]) => {
            let json = parse_json(body)?;
            let project_id = json
                .get("project_id")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing_project_id".to_string())?;
            let title = json
                .get("title")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing_title".to_string())?;
            let priority = json.get("priority").and_then(Value::as_str);
            wrap_success(200, task_create_json(project_id, title, priority)?)
        }
        ("POST", ["v1", "tasks", task_id, "move"]) => {
            let json = parse_json(body)?;
            let column = json
                .get("column")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing_column".to_string())?;
            wrap_success(200, task_move_json(task_id, column)?)
        }
        ("POST", ["v1", "tasks", task_id, "automation-mode"]) => {
            let json = parse_json(body)?;
            let mode = json
                .get("mode")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing_mode".to_string())?;
            wrap_success(200, task_automation_mode_json(task_id, mode)?)
        }
        ("POST", ["v1", "dispatch"]) => {
            let json = parse_json(body)?;
            let target_session_id = json
                .get("target_session_id")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing_target_session_id".to_string())?;
            let task_id = json.get("task_id").and_then(Value::as_str);
            let target_live = json
                .get("target_live")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let payload = json
                .get("payload")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing_payload".to_string())?;
            let response = dispatch_send_json(target_session_id, task_id, target_live, payload)?;
            let parsed: Value = serde_json::from_str(&response).map_err(|e| e.to_string())?;
            if parsed.get("ok").and_then(Value::as_bool) == Some(false) {
                let reason = parsed["events"]
                    .as_array()
                    .and_then(|events| events.last())
                    .and_then(|event| event["reason_code"].as_str())
                    .unwrap_or("dispatch_failed");
                wrap_error(409, reason, reason)
            } else {
                wrap_success(200, response)
            }
        }
        ("POST", ["v1", "workflow", "validate"]) => {
            let json = parse_json(body)?;
            let project_root = json
                .get("project_root")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing_project_root".to_string())?;
            wrap_success(200, workflow_validate_json(project_root)?)
        }
        ("POST", ["v1", "workflow", "reload"]) => {
            let json = parse_json(body)?;
            let project_root = json
                .get("project_root")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing_project_root".to_string())?;
            wrap_success(200, workflow_reload_json(project_root)?)
        }
        ("POST", ["v1", "reconcile"]) => {
            let json = parse_json(body)?;
            let project_id = json.get("project_id").and_then(Value::as_str);
            wrap_success(200, reconcile_now_json(project_id)?)
        }
        _ => wrap_error(404, "not_found", "route_not_found"),
    }
}

pub fn route_for_test(method: &str, path: &str, body: &str) -> Result<(u16, String), String> {
    resolve_route_response(route(method, path, body))
}

fn resolve_route_response(result: Result<(u16, String), String>) -> Result<(u16, String), String> {
    match result {
        Ok(response) => Ok(response),
        Err(error) => {
            let (status, code, message) = classify_route_error(&error);
            let payload = error_json(&code, &message, current_snapshot().ok().as_ref())?;
            Ok((status, payload))
        }
    }
}

fn classify_route_error(error: &str) -> (u16, String, String) {
    if error.starts_with("missing_") || error == "invalid_request" {
        return (400, error.to_string(), error.to_string());
    }

    match error {
        "session_not_found" | "task_not_found" => (404, error.to_string(), error.to_string()),
        "task_claim_conflict"
        | "takeover_conflict"
        | "stale_target_session"
        | "manual_takeover_active" => (409, error.to_string(), error.to_string()),
        "invalid_transition" | "dispatch_failed" => (422, error.to_string(), error.to_string()),
        _ if error.contains("front matter parse error")
            || error.contains("workflow")
            || error.contains("last-known-good")
            || error.contains("invalid_kept_last_good") =>
        {
            (422, "workflow_invalid".to_string(), error.to_string())
        }
        _ => (500, "internal_error".to_string(), error.to_string()),
    }
}

fn parse_json(body: &str) -> Result<Value, String> {
    if body.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(body).map_err(|error| error.to_string())
}

fn wrap_success(status: u16, payload_json: String) -> Result<(u16, String), String> {
    let snapshot = current_snapshot()?;
    let value: Value = serde_json::from_str(&payload_json).map_err(|error| error.to_string())?;
    Ok((status, success_json(value, &snapshot)?))
}

fn wrap_error(status: u16, code: &str, message: &str) -> Result<(u16, String), String> {
    let snapshot = current_snapshot().ok();
    Ok((status, error_json(code, message, snapshot.as_ref())?))
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        422 => "Unprocessable Entity",
        _ => "Internal Server Error",
    }
}

fn write_response(stream: &mut UnixStream, status: u16, payload: &str) -> Result<(), String> {
    let response = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status_text(status),
        payload.len(),
        payload
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| error.to_string())
}

fn read_http_request(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|error| error.to_string())?;
    let mut buffer = Vec::new();

    loop {
        let mut chunk = [0_u8; 4096];
        let bytes_read = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);

        if let Some(header_end) = find_header_end(&buffer) {
            let content_length = content_length(&buffer[..header_end])?;
            if buffer.len() >= header_end + content_length {
                return Ok(buffer);
            }
        }
    }

    Ok(buffer)
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

fn content_length(header_bytes: &[u8]) -> Result<usize, String> {
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut request = Request::new(&mut headers);
    let _ = request
        .parse(header_bytes)
        .map_err(|error| error.to_string())?;
    Ok(request
        .headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case("content-length"))
        .and_then(|header| std::str::from_utf8(header.value).ok())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0))
}

fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    query
        .split('&')
        .filter(|item| !item.is_empty())
        .filter_map(|item| item.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}
