use crate::state::current_snapshot;

pub fn automation_json() -> Result<String, String> {
    let snapshot = current_snapshot()?;
    serde_json::to_string(&serde_json::json!({
        "automation": snapshot.ops.automation,
        "workflow": snapshot.ops.workflow,
        "tracker": snapshot.ops.tracker,
    }))
    .map_err(|error| error.to_string())
}

pub fn reconcile_now_json(project_id: Option<&str>) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    let snapshot = control_plane.snapshot().clone();
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
        snapshot.ops.automation.running_slots,
        snapshot.ops.automation.max_slots,
        &live_session_ids,
    )
    .map_err(|error| error.to_string())?;
    let reconcile = hc_control_plane::reconcile_snapshot(control_plane.snapshot_mut());
    let updated_snapshot = control_plane.snapshot().clone();
    serde_json::to_string(&serde_json::json!({
        "ok": true,
        "action": "reconcile_requested",
        "project_id": project_id,
        "result": result,
        "reconcile": reconcile,
        "snapshot": updated_snapshot
    }))
    .map_err(|error| error.to_string())
}
