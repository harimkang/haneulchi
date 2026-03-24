use hc_control_plane::{
    ControlPlaneError, ControlPlaneState, SnapshotBuildError, SnapshotSeed,
    build_authoritative_snapshot, project_snapshot, reset_task_board_for_tests,
    shared_attach_session, shared_create_task, shared_scheduler_tick, shared_set_automation_mode,
    shared_task_move,
};
use hc_domain::{
    ClaimState, ProjectSummary, RetryQueueEntry, RetryState, SessionFocusState, SessionRuntimeState,
    SessionSummary, TaskAutomationMode, TaskColumn, TrackerStatus, WorkflowHealth,
    WorkflowRuntimeStatus,
};

fn workflow_status(state: WorkflowHealth) -> WorkflowRuntimeStatus {
    WorkflowRuntimeStatus {
        state,
        path: "/tmp/demo/WORKFLOW.md".to_string(),
        last_good_hash: Some("sha256:abc123".to_string()),
        last_reload_at: Some("2026-03-22T00:00:00Z".to_string()),
        last_error: None,
    }
}

fn tracker_status() -> TrackerStatus {
    TrackerStatus {
        state: "local_only".to_string(),
        last_sync_at: None,
        health: "ok".to_string(),
    }
}

#[test]
fn session_projection_includes_workflow_review_ready_and_retry_due_attention() {
    let snapshot = project_snapshot(SnapshotSeed {
        workflow: workflow_status(WorkflowHealth::InvalidKeptLastGood),
        tracker: tracker_status(),
        projects: vec![ProjectSummary::new(
            "proj_demo",
            "demo",
            "/tmp/demo",
            WorkflowHealth::InvalidKeptLastGood,
            1,
            0,
        )],
        sessions: vec![
            SessionSummary::new("ses_waiting", "proj_demo", "Needs input")
                .with_runtime_state(SessionRuntimeState::WaitingInput)
                .with_claim_state(ClaimState::Claimed)
                .with_focus_state(SessionFocusState::Background)
                .with_cwd("/tmp/demo"),
            SessionSummary::new("ses_review", "proj_demo", "Review ready")
                .with_runtime_state(SessionRuntimeState::ReviewReady)
                .with_claim_state(ClaimState::Released)
                .with_focus_state(SessionFocusState::Background)
                .with_cwd("/tmp/demo"),
        ],
        retry_queue: vec![RetryQueueEntry {
            task_id: "task_retry".to_string(),
            project_id: "proj_demo".to_string(),
            attempt: 2,
            reason_code: "adapter_timeout".to_string(),
            due_at: Some("2026-03-23T10:00:00Z".to_string()),
            backoff_ms: 30_000,
            claim_state: ClaimState::Stale,
            retry_state: RetryState::Due,
        }],
    });

    assert_eq!(snapshot.ops.workflow.state, WorkflowHealth::InvalidKeptLastGood);
    assert_eq!(snapshot.sessions.len(), 2);
    assert_eq!(snapshot.attention.len(), 4);
    assert!(snapshot
        .attention
        .iter()
        .any(|item| item.kind == "waiting_input" && item.action_hint.as_deref() == Some("focus_session")));
    assert!(snapshot
        .attention
        .iter()
        .any(|item| item.kind == "review_ready" && item.action_hint.as_deref() == Some("open_review")));
    assert!(snapshot
        .attention
        .iter()
        .any(|item| item.kind == "retry_due" && item.action_hint.as_deref() == Some("open_retry_queue")));
    assert!(snapshot
        .attention
        .iter()
        .any(|item| item.kind == "workflow_invalid" && item.action_hint.as_deref() == Some("reload_workflow")));
}

#[test]
fn shared_commands_update_projection_state_consistently() {
    reset_task_board_for_tests();
    let snapshot = project_snapshot(SnapshotSeed {
        workflow: workflow_status(WorkflowHealth::Ok),
        tracker: tracker_status(),
        projects: vec![ProjectSummary::new(
            "proj_demo",
            "demo",
            "/tmp/demo",
            WorkflowHealth::Ok,
            2,
            0,
        )],
        sessions: vec![
            SessionSummary::new("ses_01", "proj_demo", "One")
                .with_runtime_state(SessionRuntimeState::Running)
                .with_focus_state(SessionFocusState::Focused),
            SessionSummary::new("ses_02", "proj_demo", "Two")
                .with_runtime_state(SessionRuntimeState::Running)
                .with_focus_state(SessionFocusState::Background),
        ],
        retry_queue: vec![],
    });
    let mut state = ControlPlaneState::from_snapshot(snapshot);

    state.focus_session("ses_02").expect("focus succeeds");
    state.takeover_session("ses_02").expect("takeover succeeds");
    state.attach_task("ses_02", "task_ready").expect("attach succeeds");
    assert_eq!(
        state.attach_task("ses_01", "task_ready"),
        Err(ControlPlaneError::TaskClaimConflict("task_ready".to_string()))
    );
    state.detach_task("ses_02").expect("detach succeeds");
    state.release_takeover_session("ses_02").expect("release succeeds");

    let snapshot = state.snapshot();
    assert_eq!(snapshot.ops.app.focused_session_id.as_deref(), Some("ses_02"));
    assert_eq!(
        snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "ses_02")
            .expect("focused session")
            .focus_state,
        SessionFocusState::Focused
    );
    assert_eq!(
        snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "ses_02")
            .expect("focused session")
            .task_id,
        None
    );
    assert_eq!(
        snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "ses_02")
            .expect("focused session")
            .claim_state,
        ClaimState::None
    );
    assert_eq!(
        snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "ses_02")
            .expect("focused session")
            .manual_control,
        "none"
    );
}

#[test]
fn snapshot_projection_carries_meta_and_ops_parity_fields() {
    let snapshot = project_snapshot(SnapshotSeed {
        workflow: workflow_status(WorkflowHealth::Ok),
        tracker: tracker_status(),
        projects: vec![],
        sessions: vec![],
        retry_queue: vec![],
    });

    assert_eq!(snapshot.meta.snapshot_rev, 1);
    assert_eq!(snapshot.meta.runtime_rev, 1);
    assert_eq!(snapshot.meta.projection_rev, 1);
    assert_eq!(snapshot.ops.automation.cadence_ms, 15_000);
    assert_eq!(snapshot.ops.automation.queued_claim_count, 0);
    assert_eq!(snapshot.ops.tracker.health, "ok");
}

#[test]
fn snapshot_builder_reports_snapshot_unavailable_instead_of_silent_empty_projection() {
    let result = build_authoritative_snapshot(SnapshotSeed {
        workflow: workflow_status(WorkflowHealth::Ok),
        tracker: TrackerStatus {
            state: "local_only".to_string(),
            last_sync_at: None,
            health: "snapshot_unavailable".to_string(),
        },
        projects: vec![],
        sessions: vec![],
        retry_queue: vec![],
    });

    assert_eq!(result, Err(SnapshotBuildError::SnapshotUnavailable));
}

#[test]
fn scheduler_respects_slot_capacity_and_reports_stale_targets() {
    reset_task_board_for_tests();
    let extra = shared_create_task("proj_demo", "Overflow candidate", None).expect("extra task");
    let extra_two = shared_create_task("proj_demo", "Second overflow", None).expect("second extra task");
    shared_task_move(&extra.id, TaskColumn::Ready, "test_seed").expect("move extra task");
    shared_task_move(&extra_two.id, TaskColumn::Ready, "test_seed").expect("move second extra");
    shared_set_automation_mode("task_ready", TaskAutomationMode::AutoEligible)
        .expect("task_ready automation");
    shared_set_automation_mode(&extra.id, TaskAutomationMode::AutoEligible)
        .expect("extra automation");
    shared_set_automation_mode(&extra_two.id, TaskAutomationMode::AutoEligible)
        .expect("second extra automation");
    shared_attach_session("task_ready", "stale-session").expect("stale attachment");

    let result = shared_scheduler_tick(0, 1, &[]).expect("scheduler result");

    assert_eq!(result.launched_task_ids, vec![extra.id]);
    assert_eq!(result.queued.len(), 1);
    assert_eq!(result.queued[0].reason_code, "slot_capacity_exhausted");
    assert_eq!(result.queued[0].task_id, extra_two.id);
    assert_eq!(result.failures.len(), 1);
    assert_eq!(result.failures[0].reason_code, "stale_target_session");
}
