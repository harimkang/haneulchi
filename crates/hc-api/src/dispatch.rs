pub fn dispatch_send_json(
    target_session_id: &str,
    task_id: Option<&str>,
    target_live: bool,
    payload: &str,
) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    let events = hc_control_plane::dispatch_snapshot(
        control_plane.snapshot_mut(),
        target_session_id,
        task_id,
        target_live,
        payload,
    );
    if events.is_empty() {
        return Err("session_not_found".to_string());
    }
    let ok = !events
        .last()
        .is_some_and(|event| event.state == hc_control_plane::DispatchLifecycleState::Failed);
    serde_json::to_string(&serde_json::json!({
        "ok": ok,
        "events": events
    }))
    .map_err(|error| error.to_string())
}
