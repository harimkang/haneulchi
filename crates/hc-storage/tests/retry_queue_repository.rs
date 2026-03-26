use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use hc_domain::{
    AppSnapshot, AppState, AutomationOpsSummary, ClaimState, OrchestratorRuntime,
    RetryQueueEntry, RetryState, TrackerStatus, WorkflowHealth, WorkflowReloadEvent,
    WorkflowRuntimeStatus,
};
use hc_storage::{
    NewRetryQueueEntry, NewTaskRecord, RetryFailureClass, SqliteStore, advance_retry_state,
    schedule_retry_entry,
};

#[test]
fn state_snapshot_serializes_nested_ops_contract_and_retry_queue_claim_state() {
    let workflow = WorkflowRuntimeStatus {
        state: WorkflowHealth::Ok,
        path: "/tmp/demo/WORKFLOW.md".to_string(),
        last_good_hash: Some("sha256:ops".to_string()),
        last_reload_at: Some("2026-03-23T02:00:00Z".to_string()),
        last_error: None,
    };
    let tracker = TrackerStatus {
        state: "local_only".to_string(),
        last_sync_at: Some("2026-03-23T02:01:00Z".to_string()),
        health: "ok".to_string(),
    };

    let snapshot = AppSnapshot::new(workflow, tracker)
        .with_automation(AutomationOpsSummary {
            status: "running".to_string(),
            cadence_ms: 15_000,
            last_tick_at: Some("2026-03-23T02:05:00Z".to_string()),
            last_reconcile_at: Some("2026-03-23T02:06:00Z".to_string()),
            running_slots: 2,
            max_slots: 4,
            retry_due_count: 1,
            queued_claim_count: 3,
            paused: false,
        })
        .with_app_state(AppState {
            active_route: "control_tower".to_string(),
            focused_session_id: Some("ses_dispatch".to_string()),
            degraded_flags: vec!["tracker_lag".to_string()],
        })
        .with_retry_entry(RetryQueueEntry {
            task_id: "task_retry".to_string(),
            project_id: "proj_demo".to_string(),
            attempt: 2,
            reason_code: "adapter_timeout".to_string(),
            due_at: Some("2026-03-23T02:07:00Z".to_string()),
            backoff_ms: 30_000,
            claim_state: ClaimState::Claimed,
            retry_state: RetryState::Due,
        });

    let json = serde_json::to_value(&snapshot).expect("snapshot json");

    assert!(json.get("workflow").is_none());
    assert!(json.get("tracker").is_none());
    assert!(json.get("app").is_none());
    assert_eq!(json["ops"]["automation"]["status"], "running");
    assert_eq!(json["ops"]["workflow"]["state"], "ok");
    assert_eq!(json["ops"]["tracker"]["health"], "ok");
    assert_eq!(json["ops"]["app"]["active_route"], "control_tower");
    assert_eq!(json["retry_queue"][0]["claim_state"], "claimed");
}

#[test]
fn retry_queue_repository_restores_claim_state_projection_across_reopen() {
    let path = temp_db_path("retry-queue");
    {
        let store = SqliteStore::open(&path).expect("sqlite store");
        seed_task(&store, "task_retry");

        let saved = store
            .retry_queue()
            .save(NewRetryQueueEntry {
                id: "retry_01".to_string(),
                task_id: "task_retry".to_string(),
                project_id: "proj_demo".to_string(),
                attempt: 3,
                reason_code: "workflow_invalid".to_string(),
                due_at: Some("2026-03-23T03:00:00Z".to_string()),
                backoff_ms: 120_000,
                claim_state: ClaimState::Stale,
                retry_state: RetryState::BackingOff,
            })
            .expect("retry row");

        assert_eq!(saved.claim_state, ClaimState::Stale);
        assert_eq!(
            store.retry_queue().list().expect("retry rows"),
            vec![saved.clone()]
        );
    }

    let reopened = SqliteStore::open(&path).expect("reopened sqlite store");
    let restored = reopened.retry_queue().list().expect("restored retry rows");
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].project_id, "proj_demo");
    assert_eq!(restored[0].reason_code, "workflow_invalid");
    assert_eq!(restored[0].claim_state, ClaimState::Stale);
    assert_eq!(restored[0].retry_state, RetryState::BackingOff);

    fs::remove_file(path).ok();
}

#[test]
fn retry_queue_backoff_and_stall_semantics_are_deterministic() {
    let scheduled = schedule_retry_entry(
        "task_retry",
        "proj_demo",
        2,
        1000,
        RetryFailureClass::Retryable,
        ClaimState::Claimed,
    )
    .expect("retry entry");

    assert_eq!(scheduled.backoff_ms, 60_000);
    assert_eq!(scheduled.due_at.as_deref(), Some("61000"));
    assert_eq!(scheduled.retry_state, RetryState::BackingOff);

    let due = advance_retry_state(&scheduled, 61_000, false);
    assert_eq!(due.retry_state, RetryState::Due);

    let stalled = advance_retry_state(&scheduled, 61_000, true);
    assert_eq!(stalled.retry_state, RetryState::Exhausted);

    let non_retryable = schedule_retry_entry(
        "task_retry",
        "proj_demo",
        1,
        1000,
        RetryFailureClass::NonRetryable,
        ClaimState::Released,
    );
    assert_eq!(non_retryable, None);
}

#[test]
fn automation_health_view_rolls_up_runtime_and_retry_queue_state() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    seed_task(&store, "task_retry_health");

    store
        .orchestrator()
        .save_runtime(OrchestratorRuntime {
            singleton_key: "main".to_string(),
            cadence_ms: 30_000,
            last_tick_at: Some("2026-03-26T01:00:00Z".to_string()),
            last_reconcile_at: Some("2026-03-26T01:01:00Z".to_string()),
            max_slots: 6,
            running_slots: 4,
            workflow_state: "invalid_kept_last_good".to_string(),
            tracker_state: "degraded".to_string(),
        })
        .expect("runtime row");
    store
        .orchestrator()
        .append_workflow_reload_event(WorkflowReloadEvent {
            id: "reload_health".to_string(),
            project_id: "proj_demo".to_string(),
            file_path: "/tmp/demo/WORKFLOW.md".to_string(),
            status: "invalid_kept_last_good".to_string(),
            loaded_hash: Some("sha256:new".to_string()),
            kept_last_good_hash: Some("sha256:last-good".to_string()),
            message: Some("front matter parse error".to_string()),
            created_at: "2026-03-26T01:02:00Z".to_string(),
        })
        .expect("reload event");
    store
        .retry_queue()
        .save(NewRetryQueueEntry {
            id: "retry_health".to_string(),
            task_id: "task_retry_health".to_string(),
            project_id: "proj_demo".to_string(),
            attempt: 2,
            reason_code: "adapter_timeout".to_string(),
            due_at: Some("2026-03-26T01:03:00Z".to_string()),
            backoff_ms: 60_000,
            claim_state: ClaimState::Claimed,
            retry_state: RetryState::Due,
        })
        .expect("retry row");

    let row = store
        .connection()
        .query_row(
            r#"
            SELECT
                workflow_state,
                running_slots,
                max_slots,
                retry_due_count
            FROM v_automation_health
            "#,
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .expect("automation health row");

    assert_eq!(
        row,
        ("invalid_kept_last_good".to_string(), 4_i64, 6_i64, 1_i64)
    );
}

fn seed_task(store: &SqliteStore, task_id: &str) {
    store
        .tasks()
        .create(NewTaskRecord {
            id: task_id.to_string(),
            project_id: "proj_demo".to_string(),
            display_key: "TASK-RETRY".to_string(),
            title: "Retry queue baseline".to_string(),
            description: "Persist retry queue rows".to_string(),
            priority: "p1".to_string(),
            automation_mode: hc_domain::TaskAutomationMode::AutoEligible,
            created_at: "2026-03-23T02:00:00Z".to_string(),
            updated_at: "2026-03-23T02:00:00Z".to_string(),
        })
        .expect("seed task");
}

fn temp_db_path(suffix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix epoch")
        .as_nanos();
    env::temp_dir().join(format!("hc-storage-{suffix}-{unique}.sqlite"))
}
