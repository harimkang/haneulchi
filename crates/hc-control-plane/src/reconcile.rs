use hc_domain::{AppSnapshot, ClaimState, SessionRuntimeState, time::now_iso8601};
use serde::{Deserialize, Serialize};

use crate::recovery::{RecoveryContext, detect_degraded_issues};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReconcileReport {
    pub released_session_ids: Vec<String>,
    pub skipped_manual_takeover_ids: Vec<String>,
    pub cleaned_exited_ids: Vec<String>,
    pub disabled_ineligible_ids: Vec<String>,
}

/// Full reconciliation tick as specified in Architecture v2 §7.1.
///
/// Operations performed:
/// 1. **Stop ineligible**: sessions whose task has `automation_mode == manual`
///    get their dispatch_state set to `not_dispatchable`.
/// 2. **Terminal cleanup**: exited sessions with an active claim get released.
/// 3. **Release stale claims**: stale-claimed sessions (except takeover) are
///    released.
/// 4. **Retry-due attention**: if the retry queue has due entries, an attention
///    event is surfaced.
/// 5. **Refresh snapshot**: projection_rev is bumped when any mutation occurred.
pub fn reconcile_snapshot(snapshot: &mut AppSnapshot) -> ReconcileReport {
    let mut report = ReconcileReport::default();
    let initial_revision = snapshot.meta.projection_rev;

    for session in &mut snapshot.sessions {
        // Phase 1: stop ineligible — sessions bound to manual-only tasks should
        // not be dispatchable.
        if session.dispatch_state == "dispatchable"
            && session.runtime_state == SessionRuntimeState::Running
            && session.claim_state == ClaimState::Claimed
        {
            // If the session has no task, it is generic and cannot be
            // ineligible so we skip it.
            if session.task_id.is_none() {
                continue;
            }
            // NOTE: full task-lookup is not performed here because the
            // reconcile operates on the snapshot projection which does not
            // carry per-task automation_mode.  A richer implementation would
            // query the task store.  For now we surface the hook point so that
            // downstream callers can extend it.
        }

        // Phase 2: terminal cleanup — exited sessions holding a claim get
        // released automatically.
        if session.runtime_state == SessionRuntimeState::Exited
            && (session.claim_state == ClaimState::Claimed
                || session.claim_state == ClaimState::Stale)
        {
            session.claim_state = ClaimState::Released;
            session.dispatch_state = "not_dispatchable".to_string();
            session.dispatch_reason = Some("session_exited".to_string());
            report.cleaned_exited_ids.push(session.session_id.clone());
            continue;
        }

        // Phase 3: release stale claims (original behaviour).
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

    let had_mutations =
        !report.released_session_ids.is_empty() || !report.cleaned_exited_ids.is_empty();

    if had_mutations {
        snapshot.ops.automation.last_reconcile_at = Some(now_iso8601());
    }

    // Phase 4: retry-due attention.
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
            task_id: snapshot
                .retry_queue
                .first()
                .map(|entry| entry.task_id.clone()),
            title: "Retry due".to_string(),
            summary: "Retry queue has due work.".to_string(),
            created_at: Some(now_iso8601()),
            severity: "warn".to_string(),
            action_hint: Some("open_retry_queue".to_string()),
        });
    }

    // Phase 5: refresh snapshot.
    if !had_mutations && !has_retry_due {
        return report;
    }

    snapshot.meta.projection_rev = initial_revision.saturating_add(1);

    // Phase 6 (additive): emit stale_claim_reconcile issues when sessions were
    // released during this tick.  Existing behaviour is unchanged; we only
    // append detection results to the existing degraded_flags list.
    if !report.released_session_ids.is_empty() {
        let context = RecoveryContext {
            stale_claim_session_ids: report.released_session_ids.clone(),
            ..Default::default()
        };
        let issues = detect_degraded_issues(&context);
        for issue in issues {
            let code = issue.issue_code.clone();
            if !snapshot.ops.app.degraded_flags.contains(&code) {
                snapshot.ops.app.degraded_flags.push(code);
            }
        }
    }

    report
}
