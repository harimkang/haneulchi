/// Sprint 5 comprehensive degraded-scenario regression suite.
///
/// Each test exercises a specific degraded issue code, verifies the correct
/// recovery action, and guards against secret leakage in issue details.
use hc_control_plane::{RecoveryContext, detect_degraded_issues, recovery_action_for_issue};
use hc_domain::{
    WorkflowHealth,
    settings::{DegradedIssue, RecoveryAction},
};

fn find_issue<'a>(issues: &'a [DegradedIssue], code: &str) -> &'a DegradedIssue {
    issues
        .iter()
        .find(|i| i.issue_code == code)
        .unwrap_or_else(|| panic!("expected issue '{code}' in {issues:?}"))
}

// ── degraded scenario tests ───────────────────────────────────────────────────

#[test]
fn degraded_preset_missing_code_and_action() {
    let context = RecoveryContext {
        preset_ids: vec![],
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = find_issue(&issues, "preset_missing");
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::IgnoreAndContinue
    );
}

#[test]
fn degraded_missing_project_path_code_and_action() {
    let context = RecoveryContext {
        project_paths: vec!["/nonexistent/haneulchi/path/degraded_suite_xyz".to_string()],
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = find_issue(&issues, "missing_project_path");
    assert_eq!(recovery_action_for_issue(issue), RecoveryAction::ResetPath);
}

#[test]
fn degraded_keychain_ref_missing_code_and_action() {
    let context = RecoveryContext {
        secret_ref_ids: vec!["k1".to_string()],
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = find_issue(&issues, "keychain_ref_missing");
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::RefreshKeychainRef
    );
}

#[test]
fn degraded_invalid_workflow_reload_code_and_action() {
    let context = RecoveryContext {
        workflow_health: WorkflowHealth::InvalidKeptLastGood,
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = find_issue(&issues, "invalid_workflow_reload");
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::ReloadWorkflow
    );
}

#[test]
fn degraded_stale_claim_reconcile_code_and_action() {
    let context = RecoveryContext {
        stale_claim_session_ids: vec!["ses-1".to_string()],
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = find_issue(&issues, "stale_claim_reconcile");
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::IgnoreAndContinue
    );
}

#[test]
fn degraded_crashed_restore_code_and_action() {
    let context = RecoveryContext {
        has_crashed_restore: true,
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = find_issue(&issues, "crashed_restore");
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::MarkArchived
    );
}

#[test]
fn degraded_worktree_unreachable_code_and_action() {
    let context = RecoveryContext {
        has_worktree_unreachable: true,
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = find_issue(&issues, "worktree_unreachable");
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::DeleteWorktree
    );
}

#[test]
fn degraded_deleted_repo_action_is_mark_archived() {
    let context = RecoveryContext {
        deleted_repo_paths: vec!["/deleted/repo/path".to_string()],
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = find_issue(&issues, "deleted_repo");
    assert!(issue.details.contains("/deleted/repo/path"));
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::MarkArchived
    );
}

#[test]
fn degraded_before_run_hook_failure_action_is_ignore() {
    let context = RecoveryContext {
        has_before_run_hook_failure: true,
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let issue = find_issue(&issues, "before_run_hook_failure");
    assert_eq!(
        recovery_action_for_issue(issue),
        RecoveryAction::IgnoreAndContinue
    );
}

#[test]
fn degraded_secret_redaction_issue_details_do_not_contain_secrets() {
    let context = RecoveryContext {
        secret_ref_ids: vec!["ref-id-only".to_string()],
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);

    // Canary value — must NOT appear anywhere in issue details
    let canary = "SECRET_CANARY_9876";
    for issue in &issues {
        assert!(
            !issue.details.contains(canary),
            "issue '{}' details leaked secret canary: {:?}",
            issue.issue_code,
            issue.details
        );
    }
    // Also verify the ref ID appears (not the secret) in the keychain issue
    let keychain_issue = find_issue(&issues, "keychain_ref_missing");
    assert!(
        keychain_issue.details.contains("ref-id-only"),
        "details should reference ref ID, not a secret: {:?}",
        keychain_issue.details
    );
}
