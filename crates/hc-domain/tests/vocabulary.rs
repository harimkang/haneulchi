use hc_domain::{
    AppSnapshot, ClaimState, ProjectSummary, SessionFocusState, SessionRuntimeState,
    SessionSummary, TrackerStatus, WorkflowRuntimeStatus, WorkflowHealth,
};

#[test]
fn workflow_and_session_vocabulary_match_docs() {
    assert_eq!(
        SessionRuntimeState::all(),
        [
            "launching",
            "running",
            "waiting_input",
            "review_ready",
            "blocked",
            "done",
            "error",
            "exited",
        ]
    );
    assert_eq!(
        WorkflowHealth::all(),
        ["none", "ok", "invalid_kept_last_good", "reload_pending",]
    );
    assert_eq!(ClaimState::all(), ["none", "claimed", "released", "stale"]);
    assert_eq!(SessionFocusState::all(), ["focused", "background"]);
}

#[test]
fn snapshot_contract_types_expose_required_sprint_two_groups() {
    let workflow = WorkflowRuntimeStatus {
        state: WorkflowHealth::Ok,
        path: "/tmp/demo/WORKFLOW.md".to_string(),
        last_good_hash: Some("sha256:abc123".to_string()),
        last_reload_at: Some("2026-03-22T00:00:00Z".to_string()),
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
        1,
        0,
    )
    .with_task_count("Inbox", 1)
    .with_task_count("Ready", 2);
    let session = SessionSummary::new("ses_demo", "proj_demo", "Demo shell")
        .with_mode("generic")
        .with_runtime_state(SessionRuntimeState::Running)
        .with_claim_state(ClaimState::None)
        .with_focus_state(SessionFocusState::Focused)
        .with_cwd("/tmp/demo")
        .with_workspace_root("/tmp/demo")
        .with_base_root(".")
        .with_unread_count(0);

    let snapshot = AppSnapshot::new(workflow, tracker)
        .with_project(project)
        .with_session(session);

    assert_eq!(snapshot.projects.len(), 1);
    assert_eq!(snapshot.sessions.len(), 1);
    assert_eq!(snapshot.ops.workflow.state, WorkflowHealth::Ok);
    assert_eq!(snapshot.ops.tracker.health, "ok");
    assert_eq!(snapshot.projects[0].task_counts["Inbox"], 1);
    assert_eq!(snapshot.sessions[0].focus_state, SessionFocusState::Focused);
}
