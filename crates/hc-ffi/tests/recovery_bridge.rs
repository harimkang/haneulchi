use hc_ffi::{degraded_issues_json, recovery_action_for_issue_json};
use serde_json::Value;

#[test]
fn degraded_issues_json_returns_valid_json() {
    // Provide a context with a crashed_restore flag set.
    let context_json = r#"{
        "preset_ids": ["preset_01"],
        "project_paths": [],
        "secret_ref_ids": [],
        "workflow_health": "ok",
        "stale_claim_session_ids": [],
        "has_crashed_restore": true,
        "has_worktree_unreachable": false
    }"#;

    let result = degraded_issues_json(context_json);
    assert!(result.is_ok(), "degraded_issues_json should succeed: {:?}", result);

    let json = result.unwrap();
    let value: Value = serde_json::from_str(&json).expect("degraded_issues_json must return valid JSON");
    assert!(value.is_array(), "degraded_issues_json should return a JSON array");

    let issues = value.as_array().unwrap();
    assert!(
        issues.iter().any(|issue| issue["issue_code"] == "crashed_restore"),
        "expected crashed_restore issue in {:?}",
        issues
    );
}

#[test]
fn recovery_action_json_returns_valid_action() {
    let known_codes = [
        "preset_missing",
        "missing_project_path",
        "keychain_ref_missing",
        "invalid_workflow_reload",
        "worktree_unreachable",
        "crashed_restore",
        "stale_claim_reconcile",
    ];

    for code in known_codes {
        let result = recovery_action_for_issue_json(code);
        assert!(result.is_ok(), "recovery_action_for_issue_json({code}) should succeed");
        let action = result.unwrap();
        assert!(!action.is_empty(), "action string should not be empty for code: {code}");
    }

    // Unknown codes should still return a safe default.
    let default_result = recovery_action_for_issue_json("unknown_code");
    assert!(default_result.is_ok());
    assert_eq!(default_result.unwrap(), "ignore_and_continue");
}
