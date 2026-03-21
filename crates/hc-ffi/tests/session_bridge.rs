use std::ffi::{CStr, CString};
use std::time::{Duration, Instant};

use hc_ffi::{
    hc_bytes_free, hc_string_free, hc_terminal_session_drain, hc_terminal_session_resize,
    hc_terminal_session_snapshot_json, hc_terminal_session_spawn_json,
    hc_terminal_session_terminate, hc_terminal_session_write,
};
use serde_json::Value;

#[test]
fn c_abi_can_spawn_write_resize_drain_and_terminate_session() {
    let payload = CString::new(
        r#"{
            "program": "/bin/sh",
            "args": ["-lc", "cat"],
            "geometry": { "cols": 80, "rows": 24 }
        }"#,
    )
    .unwrap();

    let response = hc_terminal_session_spawn_json(payload.as_ptr());
    let response_json = unsafe { CStr::from_ptr(response.ptr) }
        .to_str()
        .unwrap()
        .to_string();
    hc_string_free(response);

    let response_value: Value = serde_json::from_str(&response_json).unwrap();
    let session_id = response_value["session_id"].as_str().unwrap().to_string();

    let session_id_c = CString::new(session_id.clone()).unwrap();
    assert_eq!(
        hc_terminal_session_write(session_id_c.as_ptr(), b"ffi\n".as_ptr(), 4),
        0
    );

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut captured = Vec::new();

    while Instant::now() < deadline {
        let bytes = hc_terminal_session_drain(session_id_c.as_ptr());
        if !bytes.ptr.is_null() {
            let drained = unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) }.to_vec();
            captured.extend_from_slice(&drained);
            hc_bytes_free(bytes);
        }

        if String::from_utf8_lossy(&captured).contains("ffi") {
            break;
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    assert!(String::from_utf8_lossy(&captured).contains("ffi"));
    assert_eq!(hc_terminal_session_resize(session_id_c.as_ptr(), 100, 40), 0);

    let snapshot = hc_terminal_session_snapshot_json(session_id_c.as_ptr());
    let snapshot_json = unsafe { CStr::from_ptr(snapshot.ptr) }
        .to_str()
        .unwrap()
        .to_string();
    hc_string_free(snapshot);

    let snapshot_value: Value = serde_json::from_str(&snapshot_json).unwrap();
    assert_eq!(snapshot_value["geometry"]["cols"], 100);
    assert_eq!(snapshot_value["geometry"]["rows"], 40);

    assert_eq!(hc_terminal_session_terminate(session_id_c.as_ptr()), 0);
}
