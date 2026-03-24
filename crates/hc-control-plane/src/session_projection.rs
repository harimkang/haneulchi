use hc_domain::{AppSnapshot, SessionFocusState};

use crate::commands::ControlPlaneError;

pub fn focus_session(snapshot: &mut AppSnapshot, session_id: &str) -> Result<(), ControlPlaneError> {
    let mut found = false;

    for session in &mut snapshot.sessions {
        if session.session_id == session_id {
            session.focus_state = SessionFocusState::Focused;
            found = true;
        } else {
            session.focus_state = SessionFocusState::Background;
        }
    }

    if !found {
        return Err(ControlPlaneError::SessionNotFound(session_id.to_string()));
    }

    snapshot.ops.app.focused_session_id = Some(session_id.to_string());
    Ok(())
}

pub fn takeover_session(
    snapshot: &mut AppSnapshot,
    session_id: &str,
) -> Result<(), ControlPlaneError> {
    let session = snapshot
        .sessions
        .iter_mut()
        .find(|session| session.session_id == session_id)
        .ok_or_else(|| ControlPlaneError::SessionNotFound(session_id.to_string()))?;

    session.manual_control = "takeover".to_string();
    session.can_takeover = false;
    session.can_release_takeover = true;
    Ok(())
}

pub fn release_takeover_session(
    snapshot: &mut AppSnapshot,
    session_id: &str,
) -> Result<(), ControlPlaneError> {
    let session = snapshot
        .sessions
        .iter_mut()
        .find(|session| session.session_id == session_id)
        .ok_or_else(|| ControlPlaneError::SessionNotFound(session_id.to_string()))?;

    session.manual_control = "none".to_string();
    session.can_takeover = true;
    session.can_release_takeover = false;
    Ok(())
}
