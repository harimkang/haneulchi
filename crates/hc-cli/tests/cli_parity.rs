use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hc_cli::run;
use hc_control_plane::{reset_shared_control_plane_snapshot_for_tests, reset_task_board_for_tests};
use hc_domain::{
    AppSnapshot, AppState, OpsSummary, SessionFocusState, SessionRuntimeState, SessionSummary,
    TrackerStatus, WorkflowHealth, WorkflowRuntimeStatus,
};

fn seed_snapshot() {
    reset_task_board_for_tests();
    let workflow = WorkflowRuntimeStatus {
        state: WorkflowHealth::Ok,
        path: "/tmp/demo/WORKFLOW.md".to_string(),
        last_good_hash: Some("sha256:cli".to_string()),
        last_reload_at: Some("2026-03-23T19:00:00Z".to_string()),
        last_error: None,
    };
    let tracker = TrackerStatus {
        state: "local_only".to_string(),
        last_sync_at: None,
        health: "ok".to_string(),
    };
    let mut snapshot = AppSnapshot::new(workflow, tracker)
        .with_automation(OpsSummary {
            status: "running".to_string(),
            cadence_ms: 15_000,
            last_tick_at: Some("2026-03-23T19:00:00Z".to_string()),
            last_reconcile_at: None,
            running_slots: 1,
            max_slots: 2,
            retry_due_count: 0,
            queued_claim_count: 0,
            paused: false,
        })
        .with_app_state(AppState {
            active_route: "project_focus".to_string(),
            focused_session_id: Some("ses_cli".to_string()),
            degraded_flags: vec![],
        });
    snapshot.sessions = vec![
        SessionSummary::new("ses_cli", "proj_demo", "CLI session")
            .with_runtime_state(SessionRuntimeState::Running)
            .with_focus_state(SessionFocusState::Focused)
            .with_cwd("/tmp/demo")
            .with_workspace_root("/tmp/demo")
            .with_base_root("."),
    ];
    reset_shared_control_plane_snapshot_for_tests(snapshot);
}

fn temp_socket_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    std::env::temp_dir().join(format!("hc-cli-parity-{unique}.sock"))
}

#[test]
fn cli_parity_covers_state_session_task_workflow_reconcile_and_dispatch_commands() {
    seed_snapshot();
    let socket_path = temp_socket_path();
    let server = hc_api::server::ApiServer::bind(&socket_path).expect("bind server");
    let handle = thread::spawn(move || {
        server
            .serve_requests(22)
            .unwrap_or_else(|error| panic!("server error: {error}"))
    });
    thread::sleep(Duration::from_millis(50));
    unsafe {
        std::env::set_var("HC_CONTROL_SOCKET", &socket_path);
    }

    let state = run(&["state".into(), "--json".into()]).expect("state json");
    assert!(state.contains("\"ok\":true"));
    assert!(state.contains("\"request_id\""));
    let state_human = run(&["state".into()]).expect("state human");
    assert!(state_human.contains("snapshot_rev="));
    assert!(state_human.contains("slots="));

    let compact_state = run(&[
        "state".into(),
        "--json".into(),
        "--compact".into(),
        "--project".into(),
        "proj_demo".into(),
    ])
    .expect("compact state json");
    assert!(compact_state.contains("\"project_id\":\"proj_demo\""));
    assert!(!compact_state.contains("\"project_id\":\"proj_other\""));

    let session_list =
        run(&["session".into(), "list".into(), "--json".into()]).expect("session list");
    assert!(session_list.contains("\"ses_cli\""));
    let session_list_human = run(&["session".into(), "list".into()]).expect("session list human");
    assert!(session_list_human.contains("SESSION"));
    assert!(session_list_human.contains("ses_cli"));

    let filtered_session_list = run(&[
        "session".into(),
        "list".into(),
        "--json".into(),
        "--project".into(),
        "proj_demo".into(),
        "--state".into(),
        "running".into(),
    ])
    .expect("filtered session list");
    assert!(filtered_session_list.contains("\"ses_cli\""));

    let session_get = run(&[
        "session".into(),
        "get".into(),
        "ses_cli".into(),
        "--json".into(),
    ])
    .expect("session get");
    assert!(session_get.contains("\"session_id\":\"ses_cli\""));

    let focus = run(&["session".into(), "focus".into(), "ses_cli".into()]).expect("session focus");
    assert!(focus.contains("Focus requested"));
    let takeover = run(&["session".into(), "takeover".into(), "ses_cli".into()]).expect("takeover");
    assert!(takeover.contains("Takeover enabled"));
    let release = run(&[
        "session".into(),
        "release-takeover".into(),
        "ses_cli".into(),
    ])
    .expect("release");
    assert!(release.contains("Takeover released"));
    let attach = run(&[
        "session".into(),
        "attach-task".into(),
        "ses_cli".into(),
        "--task".into(),
        "task_ready".into(),
    ])
    .expect("attach");
    assert!(attach.contains("Attached task"));
    let detach = run(&["session".into(), "detach-task".into(), "ses_cli".into()]).expect("detach");
    assert!(detach.contains("Detached task"));

    let created = run(&[
        "task".into(),
        "create".into(),
        "--project".into(),
        "proj_demo".into(),
        "--title".into(),
        "CLI created".into(),
        "--priority".into(),
        "high".into(),
    ])
    .expect("task create");
    assert!(created.contains("Created task"));
    assert!(created.contains("high"));

    let moved = run(&[
        "task".into(),
        "move".into(),
        "task_ready".into(),
        "--column".into(),
        "Review".into(),
    ])
    .expect("move");
    assert!(moved.contains("Moved task"));
    let assigned = run(&[
        "task".into(),
        "assign".into(),
        "task_ready".into(),
        "--session".into(),
        "ses_cli".into(),
    ])
    .expect("assign");
    assert!(assigned.contains("Assigned task"));
    let automation_mode = run(&[
        "task".into(),
        "automation-mode".into(),
        "task_ready".into(),
        "--mode".into(),
        "assisted".into(),
    ])
    .expect("automation mode");
    assert!(automation_mode.contains("Automation mode"));

    let workflow_root = std::env::temp_dir().join("hc-cli-workflow-contract");
    std::fs::create_dir_all(&workflow_root).expect("workflow root");
    std::fs::write(
        workflow_root.join("WORKFLOW.md"),
        "---\nworkflow:\n  name: CLI Workflow\n---\n{{task.title}}\n",
    )
    .expect("workflow file");
    let validate = run(&[
        "workflow".into(),
        "validate".into(),
        "--project".into(),
        workflow_root.display().to_string(),
    ])
    .expect("workflow validate");
    assert!(validate.contains("Workflow state: ok"));
    let reload = run(&[
        "workflow".into(),
        "reload".into(),
        "--project".into(),
        workflow_root.display().to_string(),
    ])
    .expect("workflow reload");
    assert!(reload.contains("Workflow state:"));

    let reconcile = run(&[
        "reconcile".into(),
        "now".into(),
        "--project".into(),
        "proj_demo".into(),
    ])
    .expect("reconcile");
    assert!(reconcile.contains("Reconcile requested"));
    assert!(reconcile.contains("proj_demo"));
    let dispatch = run(&[
        "dispatch".into(),
        "send".into(),
        "--task".into(),
        "task_ready".into(),
        "--target".into(),
        "ses_cli".into(),
        "--message".into(),
        "run tests".into(),
    ])
    .expect("dispatch");
    assert!(dispatch.contains("Dispatch sent"));

    let missing_focus = run(&["session".into(), "focus".into(), "ses_missing".into()])
        .expect_err("missing session should fail");
    assert!(missing_focus.contains("session_not_found"));

    let missing_focus_json = run(&[
        "session".into(),
        "focus".into(),
        "ses_missing".into(),
        "--json".into(),
    ])
    .expect_err("missing session json should fail");
    assert!(missing_focus_json.contains("\"code\":\"session_not_found\""));
    assert!(missing_focus_json.contains("\"retryable\":false"));
    assert!(missing_focus_json.contains("\"details\""));
    handle.join().expect("server join");
}
