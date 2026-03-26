use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use hc_domain::{
    OrchestratorRuntime, ReviewItem, TaskAutomationMode, TrackerBinding, WorkflowReloadEvent,
};
use hc_storage::{NewTaskRecord, SqliteStore};

#[test]
fn orchestrator_runtime_reload_tracker_and_review_evidence_restore_across_reopen() {
    let path = temp_db_path("orchestrator");
    let expected_review = {
        let mut review = ReviewItem::new_pending(
            "review_01",
            "task_ops",
            Some("ses_ops".to_string()),
            "Ready for operator review",
            "2026-03-23T03:10:00Z",
        );
        review.diff_summary = Some("2 files changed".to_string());
        review.tests_summary = Some("cargo test -p hc-storage".to_string());
        review.command_summary = Some("cargo test --workspace".to_string());
        review.hook_summary = Some("after_run succeeded; before_run skipped".to_string());
        review.evidence_summary = Some("Captured diff, tests, and logs".to_string());
        review.checklist_summary = Some("2/2 review checks complete".to_string());
        review.evidence_manifest_path = Some("evidence/manifest.json".to_string());
        review.review_checklist_result = Some("pass".to_string());
        review.warnings = vec!["after_run_failed".to_string()];
        review
    };

    {
        let store = SqliteStore::open(&path).expect("sqlite store");
        seed_task(&store, "task_ops");

        let runtime = OrchestratorRuntime {
            singleton_key: "main".to_string(),
            cadence_ms: 15_000,
            last_tick_at: Some("2026-03-23T03:00:00Z".to_string()),
            last_reconcile_at: Some("2026-03-23T03:05:00Z".to_string()),
            max_slots: 4,
            running_slots: 2,
            workflow_state: "invalid_kept_last_good".to_string(),
            tracker_state: "degraded".to_string(),
        };
        let reload_event = WorkflowReloadEvent {
            id: "reload_01".to_string(),
            project_id: "proj_demo".to_string(),
            file_path: "/tmp/demo/WORKFLOW.md".to_string(),
            status: "invalid_kept_last_good".to_string(),
            loaded_hash: Some("sha256:new".to_string()),
            kept_last_good_hash: Some("sha256:last-good".to_string()),
            message: Some("front matter parse error".to_string()),
            created_at: "2026-03-23T03:06:00Z".to_string(),
        };
        let tracker_binding = TrackerBinding {
            id: "trk_01".to_string(),
            task_id: "task_ops".to_string(),
            provider: "github".to_string(),
            external_id: "101".to_string(),
            external_key: "AUTH-101".to_string(),
            sync_mode: "reference_only".to_string(),
            state: "bound".to_string(),
            last_sync_at: Some("2026-03-23T03:07:00Z".to_string()),
        };

        store
            .orchestrator()
            .save_runtime(runtime.clone())
            .expect("runtime row");
        store
            .orchestrator()
            .append_workflow_reload_event(reload_event.clone())
            .expect("reload row");
        store
            .tracker_bindings()
            .upsert(tracker_binding.clone())
            .expect("tracker binding");
        store
            .reviews()
            .save(expected_review.clone())
            .expect("review row");

        assert_eq!(
            store.orchestrator().load_runtime().expect("runtime lookup"),
            Some(runtime)
        );
        assert_eq!(
            store
                .orchestrator()
                .list_workflow_reload_events("proj_demo")
                .expect("reload rows"),
            vec![reload_event]
        );
        assert_eq!(
            store
                .tracker_bindings()
                .list_for_task("task_ops")
                .expect("tracker rows"),
            vec![tracker_binding]
        );
        assert_eq!(
            store
                .reviews()
                .latest_for_task("task_ops")
                .expect("latest review"),
            Some(expected_review.clone())
        );
    }

    let reopened = SqliteStore::open(&path).expect("reopened sqlite store");
    assert_eq!(
        reopened
            .orchestrator()
            .load_runtime()
            .expect("restored runtime")
            .expect("runtime row")
            .workflow_state,
        "invalid_kept_last_good"
    );
    assert_eq!(
        reopened
            .orchestrator()
            .list_workflow_reload_events("proj_demo")
            .expect("restored reload rows")[0]
            .kept_last_good_hash
            .as_deref(),
        Some("sha256:last-good")
    );
    assert_eq!(
        reopened
            .tracker_bindings()
            .list_for_task("task_ops")
            .expect("restored tracker rows")[0]
            .provider,
        "github"
    );
    assert_eq!(
        reopened
            .reviews()
            .latest_for_task("task_ops")
            .expect("restored latest review"),
        Some(expected_review)
    );

    fs::remove_file(path).ok();
}

#[test]
fn schema_exposes_authoritative_automation_views_and_indexes() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let mut statement = store
        .connection()
        .prepare(
            r#"
            SELECT name
            FROM sqlite_master
            WHERE name IN (
                'v_control_tower_ops_strip',
                'v_task_drawer_automation',
                'v_automation_health',
                'idx_tasks_automation_mode',
                'idx_retry_queue_due_state'
            )
            ORDER BY name ASC
            "#,
        )
        .expect("sqlite master query");

    let names = statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("sqlite master rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect names");

    assert_eq!(
        names,
        vec![
            "idx_retry_queue_due_state".to_string(),
            "idx_tasks_automation_mode".to_string(),
            "v_automation_health".to_string(),
            "v_control_tower_ops_strip".to_string(),
            "v_task_drawer_automation".to_string(),
        ]
    );
}

fn seed_task(store: &SqliteStore, task_id: &str) {
    store
        .tasks()
        .create(NewTaskRecord {
            id: task_id.to_string(),
            project_id: "proj_demo".to_string(),
            display_key: "TASK-OPS".to_string(),
            title: "Orchestrator runtime baseline".to_string(),
            description: "Persist orchestrator metadata".to_string(),
            priority: "p0".to_string(),
            automation_mode: TaskAutomationMode::Assisted,
            created_at: "2026-03-23T03:00:00Z".to_string(),
            updated_at: "2026-03-23T03:00:00Z".to_string(),
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
