use serde_json::json;

use crate::state::current_snapshot;

pub fn sessions_list_json() -> Result<String, String> {
    sessions_list_json_filtered(None, None, None, None, None)
}

pub fn sessions_list_json_filtered(
    project_id: Option<&str>,
    state: Option<&str>,
    mode: Option<&str>,
    task_id: Option<&str>,
    dispatchable: Option<bool>,
) -> Result<String, String> {
    let snapshot = current_snapshot()?;
    let sessions = snapshot
        .sessions
        .into_iter()
        .filter(|session| project_id.is_none_or(|project_id| session.project_id == project_id))
        .filter(|session| state.is_none_or(|state| session.runtime_state.as_str() == state))
        .filter(|session| mode.is_none_or(|mode| session.mode == mode))
        .filter(|session| task_id.is_none_or(|task_id| session.task_id.as_deref() == Some(task_id)))
        .filter(|session| {
            dispatchable.is_none_or(|dispatchable| {
                (session.dispatch_state == "dispatchable") == dispatchable
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&sessions).map_err(|error| error.to_string())
}

pub fn session_details_json(session_id: &str) -> Result<String, String> {
    let snapshot = current_snapshot()?;
    let session = snapshot
        .sessions
        .iter()
        .find(|session| session.session_id == session_id)
        .cloned()
        .ok_or_else(|| "session_not_found".to_string())?;
    let recent_events = snapshot
        .attention
        .iter()
        .filter(|item| {
            item.session_id.as_deref() == Some(session_id)
                || (session.task_id.is_some() && item.task_id == session.task_id)
        })
        .take(10)
        .map(|item| {
            json!({
                "kind": item.kind,
                "title": item.title,
                "summary": item.summary,
                "created_at": item.created_at,
                "action_hint": item.action_hint,
            })
        })
        .collect::<Vec<_>>();
    let review_binding = if session.runtime_state == hc_domain::SessionRuntimeState::ReviewReady {
        session.task_id.as_ref().map(|task_id| {
            json!({
                "task_id": task_id,
                "status": "review_ready",
            })
        })
    } else {
        None
    };
    let payload = json!({
        "session_id": session.session_id,
        "project_id": session.project_id,
        "task_id": session.task_id,
        "mode": session.mode,
        "runtime_state": session.runtime_state,
        "manual_control": session.manual_control,
        "dispatch_state": session.dispatch_state,
        "claim_state": session.claim_state,
        "adapter_kind": session.adapter_kind,
        "title": session.title,
        "cwd": session.cwd,
        "workspace_root": session.workspace_root,
        "base_root": session.base_root,
        "branch": session.branch,
        "latest_summary": session.latest_summary,
        "unread_count": session.unread_count,
        "last_activity_at": session.last_activity_at,
        "focus_state": session.focus_state,
        "can_focus": session.can_focus,
        "can_takeover": session.can_takeover,
        "can_release_takeover": session.can_release_takeover,
        "last_dispatch": {
            "state": session.dispatch_state,
            "reason_code": session.dispatch_reason
        },
        "workflow_binding": snapshot.ops.workflow,
        "task_binding": session.task_id,
        "recent_events": recent_events,
        "review_binding": review_binding,
        "terminal_geometry": serde_json::Value::Null,
        "started_at": serde_json::Value::Null,
        "ended_at": serde_json::Value::Null,
        "exit_code": serde_json::Value::Null
    });
    serde_json::to_string(&payload).map_err(|error| error.to_string())
}

pub fn session_focus_json(session_id: &str) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    control_plane
        .focus_session(session_id)
        .map_err(map_control_plane_error)?;
    serde_json::to_string(&json!({
        "session_id": session_id,
        "accepted_at": hc_domain::time::now_iso8601(),
        "ui_action": "focus_requested"
    }))
    .map_err(|error| error.to_string())
}

pub fn session_takeover_json(session_id: &str) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    control_plane
        .takeover_session(session_id)
        .map_err(map_control_plane_error)?;
    serde_json::to_string(&json!({ "session_id": session_id, "manual_control": "takeover" }))
        .map_err(|error| error.to_string())
}

pub fn session_release_takeover_json(session_id: &str) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    control_plane
        .release_takeover_session(session_id)
        .map_err(map_control_plane_error)?;
    serde_json::to_string(&json!({ "session_id": session_id, "manual_control": "none" }))
        .map_err(|error| error.to_string())
}

pub fn session_attach_task_json(session_id: &str, task_id: &str) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    control_plane
        .attach_task(session_id, task_id)
        .map_err(map_control_plane_error)?;
    serde_json::to_string(&json!({ "session_id": session_id, "task_id": task_id }))
        .map_err(|error| error.to_string())
}

pub fn session_detach_task_json(session_id: &str) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    control_plane
        .detach_task(session_id)
        .map_err(map_control_plane_error)?;
    serde_json::to_string(&json!({ "session_id": session_id, "task_id": serde_json::Value::Null }))
        .map_err(|error| error.to_string())
}

fn map_control_plane_error(error: hc_control_plane::ControlPlaneError) -> String {
    match error {
        hc_control_plane::ControlPlaneError::SessionNotFound(_) => "session_not_found".to_string(),
        hc_control_plane::ControlPlaneError::TaskNotFound(_) => "task_not_found".to_string(),
        hc_control_plane::ControlPlaneError::TaskClaimConflict(_) => {
            "task_claim_conflict".to_string()
        }
        hc_control_plane::ControlPlaneError::TaskProjectMismatch { .. } => {
            "invalid_transition".to_string()
        }
        hc_control_plane::ControlPlaneError::AttentionNotFound(_) => {
            "invalid_transition".to_string()
        }
        hc_control_plane::ControlPlaneError::Storage(_) => "storage_error".to_string(),
        hc_control_plane::ControlPlaneError::Worktree(_) => "worktree_error".to_string(),
    }
}
