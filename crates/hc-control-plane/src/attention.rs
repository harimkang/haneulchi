use hc_domain::{AttentionSummary, SessionRuntimeState, WorkflowHealth, WorkflowRuntimeStatus};
use hc_domain::SessionSummary;

pub fn derive_attention(
    workflow: &WorkflowRuntimeStatus,
    sessions: &[SessionSummary],
) -> Vec<AttentionSummary> {
    let mut items = Vec::new();

    if workflow.state == WorkflowHealth::InvalidKeptLastGood {
        items.push(AttentionSummary {
            attention_id: "attention-workflow-invalid".to_string(),
            kind: "workflow_invalid".to_string(),
            project_id: sessions
                .first()
                .map(|session| session.project_id.clone())
                .unwrap_or_default(),
            session_id: None,
            task_id: None,
            title: "Workflow invalid reload kept last known good".to_string(),
            summary: workflow
                .last_error
                .clone()
                .unwrap_or_else(|| "Reload failed; last known good remains active.".to_string()),
            created_at: workflow.last_reload_at.clone(),
            severity: "warn".to_string(),
            action_hint: Some("reload_workflow".to_string()),
        });
    }

    for session in sessions {
        if session.runtime_state == SessionRuntimeState::WaitingInput {
            items.push(AttentionSummary {
                attention_id: format!("attention-{}", session.session_id),
                kind: "waiting_input".to_string(),
                project_id: session.project_id.clone(),
                session_id: Some(session.session_id.clone()),
                task_id: session.task_id.clone(),
                title: session.title.clone(),
                summary: session
                    .latest_summary
                    .clone()
                    .unwrap_or_else(|| "Operator input required.".to_string()),
                created_at: session.last_activity_at.clone(),
                severity: "warn".to_string(),
                action_hint: Some("focus_session".to_string()),
            });
        }
    }

    items
}
