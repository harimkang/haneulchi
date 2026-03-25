use hc_domain::{
    AppSnapshot, AttentionSummary, RetryQueueEntry, SessionRuntimeState, SessionSummary,
    WorkflowHealth, WorkflowRuntimeStatus,
};

pub fn derive_attention(
    workflow: &WorkflowRuntimeStatus,
    sessions: &[SessionSummary],
    retry_queue: &[RetryQueueEntry],
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

        if session.runtime_state == SessionRuntimeState::ReviewReady {
            items.push(AttentionSummary {
                attention_id: format!("attention-review-{}", session.session_id),
                kind: "review_ready".to_string(),
                project_id: session.project_id.clone(),
                session_id: Some(session.session_id.clone()),
                task_id: session.task_id.clone(),
                title: session.title.clone(),
                summary: session
                    .latest_summary
                    .clone()
                    .unwrap_or_else(|| "Review pack ready.".to_string()),
                created_at: session.last_activity_at.clone(),
                severity: "info".to_string(),
                action_hint: Some("open_review".to_string()),
            });
        }
    }

    for retry in retry_queue {
        items.push(AttentionSummary {
            attention_id: format!("attention-retry-{}", retry.task_id),
            kind: "retry_due".to_string(),
            project_id: retry.project_id.clone(),
            session_id: None,
            task_id: Some(retry.task_id.clone()),
            title: format!("Retry due for {}", retry.task_id),
            summary: format!(
                "{} attempt {} is ready to retry.",
                retry.reason_code, retry.attempt
            ),
            created_at: retry.due_at.clone(),
            severity: "warn".to_string(),
            action_hint: Some("open_retry_queue".to_string()),
        });
    }

    items
}

pub fn resolve_attention(snapshot: &mut AppSnapshot, attention_id: &str) -> bool {
    remove_attention(snapshot, attention_id)
}

pub fn dismiss_attention(snapshot: &mut AppSnapshot, attention_id: &str) -> bool {
    remove_attention(snapshot, attention_id)
}

pub fn snooze_attention(snapshot: &mut AppSnapshot, attention_id: &str) -> bool {
    remove_attention(snapshot, attention_id)
}

fn remove_attention(snapshot: &mut AppSnapshot, attention_id: &str) -> bool {
    let before = snapshot.attention.len();
    snapshot
        .attention
        .retain(|item| item.attention_id != attention_id);
    snapshot.attention.len() != before
}
