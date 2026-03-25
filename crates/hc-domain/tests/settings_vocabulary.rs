use hc_domain::settings::{
    AppStateRecord, DegradedIssue, LayoutRecord, NotificationRule, ProjectRecord, RecoveryAction,
    SecretRef, SessionMetadataRecord, TerminalSettings, WorktreePolicy,
};

#[test]
fn recovery_action_all_string_values() {
    assert_eq!(
        RecoveryAction::all(),
        [
            "delete_worktree",
            "reset_path",
            "reload_workflow",
            "refresh_keychain_ref",
            "mark_archived",
            "ignore_and_continue",
        ]
    );
}

#[test]
fn terminal_settings_field_construction() {
    let ts = TerminalSettings {
        shell: "/bin/zsh".to_string(),
        default_cols: 220,
        default_rows: 50,
        scrollback_lines: 10000,
        font_name: "Menlo".to_string(),
        theme: "dark".to_string(),
        cursor_style: "block".to_string(),
    };
    assert_eq!(ts.shell, "/bin/zsh");
    assert_eq!(ts.default_cols, 220);
    assert_eq!(ts.default_rows, 50);
    assert_eq!(ts.scrollback_lines, 10000);
    assert_eq!(ts.font_name, "Menlo");
    assert_eq!(ts.theme, "dark");
    assert_eq!(ts.cursor_style, "block");
}

#[test]
fn terminal_settings_default_has_empty_new_fields() {
    let ts = TerminalSettings::default();
    assert_eq!(ts.font_name, "");
    assert_eq!(ts.theme, "");
    assert_eq!(ts.cursor_style, "");
}

#[test]
fn secret_ref_field_construction() {
    let sr = SecretRef {
        ref_id: "ref_001".to_string(),
        label: "GitHub Token".to_string(),
        env_var_name: "GITHUB_TOKEN".to_string(),
        keychain_service: "haneulchi".to_string(),
        keychain_account: "github".to_string(),
        scope: String::new(),
    };
    assert_eq!(sr.ref_id, "ref_001");
    assert_eq!(sr.env_var_name, "GITHUB_TOKEN");
    assert_eq!(sr.scope, "");
}

#[test]
fn worktree_policy_field_construction() {
    let wp = WorktreePolicy {
        policy_id: "pol_001".to_string(),
        max_age_days: Some(30),
        max_count: Some(10),
        auto_prune: true,
    };
    assert_eq!(wp.policy_id, "pol_001");
    assert_eq!(wp.max_age_days, Some(30));
    assert_eq!(wp.max_count, Some(10));
    assert!(wp.auto_prune);
}

#[test]
fn notification_rule_field_construction() {
    let nr = NotificationRule {
        rule_id: "rule_001".to_string(),
        event_kind: "session_error".to_string(),
        enabled: true,
    };
    assert_eq!(nr.rule_id, "rule_001");
    assert_eq!(nr.event_kind, "session_error");
    assert!(nr.enabled);
}

#[test]
fn project_record_field_construction() {
    let pr = ProjectRecord {
        project_id: "proj_001".to_string(),
        name: "my-project".to_string(),
        root_path: "/tmp/my-project".to_string(),
        last_focused_at: Some("2026-03-24T00:00:00Z".to_string()),
    };
    assert_eq!(pr.project_id, "proj_001");
    assert_eq!(pr.root_path, "/tmp/my-project");
}

#[test]
fn layout_record_field_construction() {
    let lr = LayoutRecord {
        layout_id: "layout_001".to_string(),
        project_id: "proj_001".to_string(),
        data_json: "{}".to_string(),
        saved_at: None,
    };
    assert_eq!(lr.layout_id, "layout_001");
    assert_eq!(lr.data_json, "{}");
}

#[test]
fn session_metadata_record_field_construction() {
    let smr = SessionMetadataRecord {
        session_id: "ses_001".to_string(),
        project_id: "proj_001".to_string(),
        title: "My Session".to_string(),
        cwd: "/tmp".to_string(),
        branch: Some("main".to_string()),
        last_active_at: None,
        is_recoverable: true,
    };
    assert_eq!(smr.session_id, "ses_001");
    assert!(smr.is_recoverable);
}

#[test]
fn app_state_record_field_construction() {
    let asr = AppStateRecord {
        active_route: "project_focus".to_string(),
        last_project_id: Some("proj_001".to_string()),
        last_session_id: None,
        saved_at: Some("2026-03-24T00:00:00Z".to_string()),
    };
    assert_eq!(asr.active_route, "project_focus");
    assert_eq!(asr.last_project_id, Some("proj_001".to_string()));
}

#[test]
fn degraded_issue_field_construction() {
    let di = DegradedIssue {
        issue_code: "missing_worktree".to_string(),
        worktree_id: Some("wt_001".to_string()),
        project_id: None,
        details: "Worktree path does not exist on disk".to_string(),
    };
    assert_eq!(di.issue_code, "missing_worktree");
    assert_eq!(di.worktree_id, Some("wt_001".to_string()));
    assert_eq!(di.project_id, None);
}
