use hc_domain::{TrackerStatus, WorkflowHealth, WorkflowRuntimeStatus};

pub fn sample_workflow_status() -> WorkflowRuntimeStatus {
    WorkflowRuntimeStatus {
        state: WorkflowHealth::Ok,
        path: "/tmp/demo/WORKFLOW.md".to_string(),
        last_good_hash: Some("sha256:demo".to_string()),
        last_reload_at: Some("2026-03-22T00:00:00Z".to_string()),
        last_error: None,
    }
}

pub fn sample_tracker_status() -> TrackerStatus {
    TrackerStatus {
        state: "local_only".to_string(),
        last_sync_at: None,
        health: "ok".to_string(),
    }
}
