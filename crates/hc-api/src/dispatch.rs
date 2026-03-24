pub fn dispatch_send_json(
    target_session_id: &str,
    task_id: Option<&str>,
    target_live: bool,
    payload: &str,
) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    let events =
        hc_control_plane::dispatch_snapshot(control_plane.snapshot_mut(), target_session_id, task_id, target_live, payload);
    if events.is_empty() {
        return Err("session_not_found".to_string());
    }
    if let Some(failure) = events.last().filter(|event| event.state == hc_control_plane::DispatchLifecycleState::Failed) {
        return Err(
            failure
                .reason_code
                .clone()
                .unwrap_or_else(|| "dispatch_failed".to_string()),
        );
    }
    serde_json::to_string(&serde_json::json!({
        "ok": true,
        "events": events
    }))
    .map_err(|error| error.to_string())
}
