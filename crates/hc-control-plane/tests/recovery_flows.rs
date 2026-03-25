/// Integration tests for Sprint 5 degraded-recovery detection flows.
///
/// Each test constructs a `RecoveryContext`, calls `detect_degraded_issues`,
/// and asserts that the expected issue code is present.
use hc_control_plane::{RecoveryContext, detect_degraded_issues, recovery_action_for_issue};
use hc_domain::{WorkflowHealth, settings::RecoveryAction};

// ── helpers ──────────────────────────────────────────────────────────────────

fn codes(context: &RecoveryContext) -> Vec<String> {
    detect_degraded_issues(context)
        .into_iter()
        .map(|i| i.issue_code)
        .collect()
}

// ── tests ────────────────────────────────────────────────────────────────────

#[test]
fn preset_missing_emits_recovery_issue() {
    let context = RecoveryContext {
        preset_ids: vec![],
        ..Default::default()
    };
    let issue_codes = codes(&context);
    assert!(
        issue_codes.contains(&"preset_missing".to_string()),
        "expected preset_missing in {issue_codes:?}"
    );

    let issues = detect_degraded_issues(&context);
    let preset_issue = issues
        .iter()
        .find(|i| i.issue_code == "preset_missing")
        .expect("preset_missing issue");
    assert_eq!(
        recovery_action_for_issue(preset_issue),
        RecoveryAction::IgnoreAndContinue
    );
}

#[test]
fn missing_project_path_emits_recovery_issue() {
    // Use a path that is guaranteed not to exist.
    let context = RecoveryContext {
        project_paths: vec!["/nonexistent/haneulchi/path/that/does/not/exist".to_string()],
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = issues
        .iter()
        .find(|i| i.issue_code == "missing_project_path")
        .expect("missing_project_path issue");

    // details must reference the path but not contain any secret value
    assert!(issue.details.contains("/nonexistent/haneulchi/path/that/does/not/exist"));
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::ResetPath
    );
}

#[test]
fn keychain_ref_missing_emits_recovery_issue() {
    let context = RecoveryContext {
        secret_ref_ids: vec!["ref_openai_key".to_string()],
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = issues
        .iter()
        .find(|i| i.issue_code == "keychain_ref_missing")
        .expect("keychain_ref_missing issue");

    // details must contain only the ref ID, not any secret value
    assert!(issue.details.contains("ref_openai_key"));
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::RefreshKeychainRef
    );
}

#[test]
fn invalid_workflow_reload_emits_recovery_issue() {
    let context = RecoveryContext {
        workflow_health: WorkflowHealth::InvalidKeptLastGood,
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = issues
        .iter()
        .find(|i| i.issue_code == "invalid_workflow_reload")
        .expect("invalid_workflow_reload issue");

    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::ReloadWorkflow
    );
}

#[test]
fn deleted_repo_emits_recovery_issue() {
    let context = RecoveryContext {
        deleted_repo_paths: vec!["/some/path".to_string()],
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = issues
        .iter()
        .find(|i| i.issue_code == "deleted_repo")
        .expect("deleted_repo issue");

    assert!(issue.details.contains("/some/path"));
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::MarkArchived
    );
}

#[test]
fn before_run_hook_failure_emits_recovery_issue() {
    let context = RecoveryContext {
        has_before_run_hook_failure: true,
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = issues
        .iter()
        .find(|i| i.issue_code == "before_run_hook_failure")
        .expect("before_run_hook_failure issue");

    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::IgnoreAndContinue
    );
}

#[test]
fn stale_claim_reconcile_emits_recovery_issue() {
    let context = RecoveryContext {
        stale_claim_session_ids: vec!["ses_stale_01".to_string(), "ses_stale_02".to_string()],
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let stale_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.issue_code == "stale_claim_reconcile")
        .collect();

    assert_eq!(stale_issues.len(), 2, "expected one issue per stale session");
    for issue in &stale_issues {
        assert_eq!(
            recovery_action_for_issue(issue),
            RecoveryAction::IgnoreAndContinue
        );
    }
}
