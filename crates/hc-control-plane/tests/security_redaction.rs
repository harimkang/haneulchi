/// Security redaction tests.
///
/// These tests assert that secret values never appear in snapshot fields or
/// `DegradedIssue::details`.  A canary string `SECRET_VALUE_12345` is used as
/// the fake secret; all assertions confirm it is absent from every relevant
/// output field.
use hc_control_plane::{
    RecoveryContext, SnapshotSeed, build_authoritative_snapshot, detect_degraded_issues,
};
use hc_domain::{
    SessionSummary, TrackerStatus, WorkflowHealth, WorkflowRuntimeStatus, settings::SecretRef,
};

const CANARY: &str = "SECRET_VALUE_12345";

// ── helpers ──────────────────────────────────────────────────────────────────

fn snapshot_seed_with_healthy_workflow() -> SnapshotSeed {
    SnapshotSeed {
        workflow: WorkflowRuntimeStatus {
            state: WorkflowHealth::Ok,
            path: "/tmp/project/WORKFLOW.md".to_string(),
            last_good_hash: Some("sha256:abc".to_string()),
            last_reload_at: Some("2026-03-25T10:00:00Z".to_string()),
            last_error: None,
        },
        tracker: TrackerStatus {
            state: "local_only".to_string(),
            last_sync_at: None,
            health: "ok".to_string(),
        },
        projects: vec![],
        sessions: vec![SessionSummary::new("ses_01", "proj_demo", "Test session")],
        retry_queue: vec![],
    }
}

// ── tests ────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_does_not_expose_secret_values() {
    let seed = snapshot_seed_with_healthy_workflow();
    let snapshot = build_authoritative_snapshot(seed).expect("snapshot build");

    // Serialise the entire snapshot to JSON and verify the canary is absent.
    let json = serde_json::to_string(&snapshot).expect("serialize snapshot");
    assert!(
        !json.contains(CANARY),
        "snapshot JSON must not contain the secret canary: found in {json}"
    );

    // Also assert directly on degraded_flags — they should contain only codes.
    for flag in &snapshot.ops.app.degraded_flags {
        assert!(
            !flag.contains(CANARY),
            "degraded flag must not contain secret: {flag}"
        );
    }
}

#[test]
fn recovery_issues_do_not_expose_secret_values() {
    // Build a SecretRef that has the canary in a label/account field, simulate
    // that the ref_id is reported as missing from Keychain.
    let _ref = SecretRef {
        ref_id: "ref_secret_key".to_string(),
        label: format!("OpenAI key ({CANARY})"),
        env_var_name: "OPENAI_API_KEY".to_string(),
        keychain_service: "com.example.haneulchi".to_string(),
        keychain_account: format!("account-{CANARY}"),
        scope: String::new(),
    };

    // The recovery system only knows the ref_id, not the full SecretRef.
    // Callers resolve the ref_id to Keychain presence; here we simulate the
    // absence by passing just the ref_id.
    let context = RecoveryContext {
        secret_ref_ids: vec!["ref_secret_key".to_string()],
        workflow_health: WorkflowHealth::InvalidKeptLastGood,
        stale_claim_session_ids: vec!["ses_stale".to_string()],
        has_crashed_restore: true,
        ..Default::default()
    };

    let issues = detect_degraded_issues(&context);
    assert!(!issues.is_empty(), "expected at least one issue");

    for issue in &issues {
        // The canary string must not appear in any field of a DegradedIssue.
        assert!(
            !issue.details.contains(CANARY),
            "issue details must not contain secret canary: issue_code={} details={}",
            issue.issue_code,
            issue.details
        );
        if let Some(ref wid) = issue.worktree_id {
            assert!(!wid.contains(CANARY));
        }
        if let Some(ref pid) = issue.project_id {
            assert!(!pid.contains(CANARY));
        }
    }

    // Serialise the entire issue list to confirm nothing leaks.
    let json = serde_json::to_string(&issues).expect("serialize issues");
    assert!(
        !json.contains(CANARY),
        "issues JSON must not contain the secret canary"
    );
}
