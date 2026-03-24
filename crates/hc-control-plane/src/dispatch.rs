use hc_domain::SessionSummary;
use hc_domain::time::now_iso8601;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchLifecycleState {
    Queued,
    Started,
    Sent,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DispatchEvent {
    pub state: DispatchLifecycleState,
    pub target_session_id: String,
    pub task_id: Option<String>,
    pub reason_code: Option<String>,
    pub created_at: String,
}

pub fn dispatch_to_session(
    session: &mut SessionSummary,
    target_live: bool,
    payload: &str,
) -> Vec<DispatchEvent> {
    let created_at = now_iso8601();
    let mut events = vec![
        DispatchEvent {
            state: DispatchLifecycleState::Queued,
            target_session_id: session.session_id.clone(),
            task_id: session.task_id.clone(),
            reason_code: None,
            created_at: created_at.clone(),
        },
        DispatchEvent {
            state: DispatchLifecycleState::Started,
            target_session_id: session.session_id.clone(),
            task_id: session.task_id.clone(),
            reason_code: None,
            created_at: created_at.clone(),
        },
    ];

    if session.manual_control == "takeover" {
        session.dispatch_state = "dispatch_failed".to_string();
        session.dispatch_reason = Some("manual_takeover_active".to_string());
        events.push(DispatchEvent {
            state: DispatchLifecycleState::Failed,
            target_session_id: session.session_id.clone(),
            task_id: session.task_id.clone(),
            reason_code: Some("manual_takeover_active".to_string()),
            created_at,
        });
        return events;
    }

    if !target_live {
        session.dispatch_state = "dispatch_failed".to_string();
        session.dispatch_reason = Some("stale_target_session".to_string());
        events.push(DispatchEvent {
            state: DispatchLifecycleState::Failed,
            target_session_id: session.session_id.clone(),
            task_id: session.task_id.clone(),
            reason_code: Some("stale_target_session".to_string()),
            created_at,
        });
        return events;
    }

    session.latest_summary = Some(format!("Dispatch sent: {payload}"));
    session.dispatch_reason = None;
    events.push(DispatchEvent {
        state: DispatchLifecycleState::Sent,
        target_session_id: session.session_id.clone(),
        task_id: session.task_id.clone(),
        reason_code: None,
        created_at,
    });
    events
}

pub fn dispatch_snapshot(
    snapshot: &mut hc_domain::AppSnapshot,
    target_session_id: &str,
    task_id: Option<&str>,
    target_live: bool,
    payload: &str,
) -> Vec<DispatchEvent> {
    let Some(session) = snapshot
        .sessions
        .iter_mut()
        .find(|session| session.session_id == target_session_id)
    else {
        return Vec::new();
    };

    if task_id.is_some() {
        session.task_id = task_id.map(ToOwned::to_owned);
    }

    let events = dispatch_to_session(session, target_live, payload);
    if let Some(failure) = events.last().filter(|event| event.state == DispatchLifecycleState::Failed)
    {
        snapshot.attention.push(hc_domain::AttentionSummary {
            attention_id: format!("attention-dispatch-{}", target_session_id),
            kind: "session_error".to_string(),
            project_id: session.project_id.clone(),
            session_id: Some(target_session_id.to_string()),
            task_id: task_id.map(ToOwned::to_owned),
            title: session.title.clone(),
            summary: failure
                .reason_code
                .clone()
                .unwrap_or_else(|| "dispatch_failed".to_string()),
            created_at: Some(failure.created_at.clone()),
            severity: "critical".to_string(),
            action_hint: Some("focus_session".to_string()),
        });
    }

    events
}
