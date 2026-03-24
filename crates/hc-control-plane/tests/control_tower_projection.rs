use hc_control_plane::{ReviewQueueItem, build_control_tower_projection};
use hc_domain::{
    AppSnapshot, ClaimState, ProjectSummary, SessionFocusState, SessionRuntimeState,
    SessionSummary, TrackerStatus, WorkflowHealth, WorkflowRuntimeStatus,
};

fn snapshot() -> AppSnapshot {
    let workflow = WorkflowRuntimeStatus {
        state: WorkflowHealth::Ok,
        path: "/tmp/demo/WORKFLOW.md".to_string(),
        last_good_hash: Some("sha256:card".to_string()),
        last_reload_at: Some("2026-03-23T10:00:00Z".to_string()),
        last_error: None,
    };
    let tracker = TrackerStatus {
        state: "local_only".to_string(),
        last_sync_at: None,
        health: "ok".to_string(),
    };
    let project = ProjectSummary::new(
        "proj_demo",
        "demo",
        "/tmp/demo",
        WorkflowHealth::Ok,
        2,
        1,
    );

    let mut waiting = SessionSummary::new("ses_waiting", "proj_demo", "Needs input");
    waiting.runtime_state = SessionRuntimeState::WaitingInput;
    waiting.claim_state = ClaimState::Claimed;
    waiting.focus_state = SessionFocusState::Background;
    waiting.latest_summary = Some("Awaiting operator answer".to_string());
    waiting.latest_commentary = Some("Need a schema confirmation.".to_string());
    waiting.last_activity_at = Some("2026-03-23T10:05:00Z".to_string());

    let mut running = SessionSummary::new("ses_running", "proj_demo", "Implementing");
    running.runtime_state = SessionRuntimeState::Running;
    running.claim_state = ClaimState::Claimed;
    running.focus_state = SessionFocusState::Focused;
    running.latest_summary = Some("Applying the latest migration".to_string());
    running.last_activity_at = Some("2026-03-23T10:03:00Z".to_string());

    AppSnapshot::new(workflow, tracker)
        .with_project(project)
        .with_session(waiting)
        .with_session(running)
}

#[test]
fn control_tower_projection_builds_project_card_heat_strip_and_recent_artifacts() {
    let projection = build_control_tower_projection(
        &snapshot(),
        &[ReviewQueueItem {
            task_id: "task_review".to_string(),
            project_id: "proj_demo".to_string(),
            title: "Review auth flow".to_string(),
            summary: "Review ready".to_string(),
            touched_files: vec!["src/lib.rs".to_string()],
            diff_summary: Some("+42 -3".to_string()),
            tests_summary: Some("12 passing".to_string()),
            command_summary: Some("cargo test".to_string()),
            hook_summary: Some("after_run ok".to_string()),
            evidence_summary: Some("Diff and tests captured".to_string()),
            checklist_summary: Some("2/2 checks complete".to_string()),
            warnings: vec![],
            evidence_manifest_path: Some("evidence/manifest.json".to_string()),
            ci_run_url: None,
            pr_url: None,
            timeline: vec![],
        }],
    );

    assert_eq!(projection.project_cards.len(), 1);
    assert_eq!(projection.recent_artifacts.len(), 1);

    let card = &projection.project_cards[0];
    assert_eq!(card.project_id, "proj_demo");
    assert_eq!(card.state, "attention");
    assert_eq!(card.heat_strip.running, 1);
    assert_eq!(card.heat_strip.waiting_input, 1);
    assert_eq!(card.latest_summary.as_deref(), Some("Awaiting operator answer"));
    assert_eq!(
        card.latest_commentary.as_deref(),
        Some("Need a schema confirmation.")
    );

    let artifact = &projection.recent_artifacts[0];
    assert_eq!(artifact.project_id, "proj_demo");
    assert_eq!(artifact.jump_target, "review_queue");
    assert_eq!(artifact.task_id, "task_review");
}
