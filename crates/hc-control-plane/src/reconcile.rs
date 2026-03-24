use hc_domain::{AppSnapshot, ClaimState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReconcileReport {
    pub released_session_ids: Vec<String>,
    pub skipped_manual_takeover_ids: Vec<String>,
}

pub fn reconcile_snapshot(snapshot: &mut AppSnapshot) -> ReconcileReport {
    let mut report = ReconcileReport::default();
    let initial_revision = snapshot.meta.projection_rev;

    for session in &mut snapshot.sessions {
        if session.claim_state != ClaimState::Stale {
            continue;
        }

        if session.manual_control == "takeover" {
            report
                .skipped_manual_takeover_ids
                .push(session.session_id.clone());
            continue;
        }

        session.claim_state = ClaimState::Released;
        session.dispatch_state = "not_dispatchable".to_string();
        session.dispatch_reason = Some("reconciled_stale_claim".to_string());
        report.released_session_ids.push(session.session_id.clone());
    }

    if !report.released_session_ids.is_empty() {
        snapshot.ops.automation.last_reconcile_at = Some("2026-03-23T11:05:00Z".to_string());
    }

    let has_retry_due = snapshot
        .retry_queue
        .iter()
        .any(|entry| entry.retry_state == hc_domain::RetryState::Due);
    if has_retry_due
        && !snapshot
            .attention
            .iter()
            .any(|item| item.kind == "retry_due")
    {
        snapshot.attention.push(hc_domain::AttentionSummary {
            attention_id: "attention-retry-due".to_string(),
            kind: "retry_due".to_string(),
            project_id: snapshot
                .retry_queue
                .first()
                .map(|entry| entry.project_id.clone())
                .unwrap_or_default(),
            session_id: None,
            task_id: snapshot.retry_queue.first().map(|entry| entry.task_id.clone()),
            title: "Retry due".to_string(),
            summary: "Retry queue has due work.".to_string(),
            created_at: Some("2026-03-23T11:05:00Z".to_string()),
            severity: "warn".to_string(),
            action_hint: Some("open_retry_queue".to_string()),
        });
    }

    if report.released_session_ids.is_empty() && !has_retry_due {
        return report;
    }

    snapshot.meta.projection_rev = initial_revision.saturating_add(1);

    report
}
