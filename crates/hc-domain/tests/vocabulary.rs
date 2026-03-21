use hc_domain::{SessionRuntimeState, WorkflowHealth};

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
}
