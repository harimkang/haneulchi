pub fn reconcile_now_json() -> Result<String, String> {
    let snapshot = {
        let control_plane = hc_control_plane::lock_shared_control_plane()?;
        control_plane.snapshot().clone()
    };
    let live_session_ids = snapshot
        .sessions
        .iter()
        .filter(|session| {
            matches!(
                session.runtime_state,
                hc_domain::SessionRuntimeState::Launching
                    | hc_domain::SessionRuntimeState::Running
                    | hc_domain::SessionRuntimeState::WaitingInput
                    | hc_domain::SessionRuntimeState::ReviewReady
                    | hc_domain::SessionRuntimeState::Blocked
            )
        })
        .map(|session| session.session_id.clone())
        .collect::<Vec<_>>();

    let result = hc_control_plane::shared_scheduler_tick(
        snapshot.ops.running_slots,
        snapshot.ops.max_slots,
        &live_session_ids,
    )
        .map_err(|error| error.to_string())?;
    serde_json::to_string(&serde_json::json!({
        "ok": true,
        "action": "reconcile_requested",
        "result": result
    }))
    .map_err(|error| error.to_string())
}
