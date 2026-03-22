use hc_control_plane::{project_snapshot, reset_task_board_for_tests, ControlPlaneError, ControlPlaneState, SnapshotSeed};
use hc_domain::{
    ClaimState, ProjectSummary, SessionFocusState, SessionRuntimeState, SessionSummary,
    TrackerStatus, WorkflowHealth, WorkflowRuntimeStatus,
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
fn session_projection_includes_workflow_and_waiting_input_attention() {
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
        ],
    });

    assert_eq!(snapshot.workflow.state, WorkflowHealth::InvalidKeptLastGood);
    assert_eq!(snapshot.sessions.len(), 1);
    assert_eq!(snapshot.attention.len(), 2);
    assert!(snapshot
        .attention
        .iter()
        .any(|item| item.kind == "waiting_input" && item.action_hint.as_deref() == Some("focus_session")));
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
    assert_eq!(snapshot.app.focused_session_id.as_deref(), Some("ses_02"));
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
