/// Integration tests for Sprint 5 system-readiness checks.
///
/// These tests verify that the host environment satisfies the minimum
/// requirements for haneulchi to operate: required binaries exist, shell
/// interpreters are available, and `detect_degraded_issues` correctly
/// classifies healthy vs. degraded contexts.
use hc_control_plane::{RecoveryContext, detect_degraded_issues};
use hc_domain::WorkflowHealth;

// ── binary readiness ──────────────────────────────────────────────────────────

#[test]
fn readiness_git_binary_is_present() {
    let status = std::process::Command::new("git")
        .arg("--version")
        .status()
        .expect("failed to spawn git");
    assert!(status.success(), "git --version should exit 0");
}

#[test]
fn readiness_shell_binary_is_present() {
    assert!(
        std::path::Path::new("/bin/sh").exists(),
        "/bin/sh must exist"
    );
    assert!(
        std::path::Path::new("/bin/zsh").exists(),
        "/bin/zsh must exist"
    );
}

// ── workflow health ───────────────────────────────────────────────────────────

#[test]
fn readiness_workflow_health_ok_has_no_issues() {
    let context = RecoveryContext {
        preset_ids: vec!["default".to_string()],
        workflow_health: WorkflowHealth::Ok,
        ..Default::default()
    };
    let issues = detect_degraded_issues(&context);
    let workflow_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.issue_code == "invalid_workflow_reload")
        .collect();
    assert!(
        workflow_issues.is_empty(),
        "WorkflowHealth::Ok must not emit workflow issues; got {workflow_issues:?}"
    );
}

// ── preset readiness ──────────────────────────────────────────────────────────

#[test]
fn readiness_empty_presets_produces_preset_missing_issue() {
    let context = RecoveryContext {
        preset_ids: vec![],
        ..Default::default()
    };
    let codes: Vec<_> = detect_degraded_issues(&context)
        .into_iter()
        .map(|i| i.issue_code)
        .collect();
    assert!(
        codes.contains(&"preset_missing".to_string()),
        "expected preset_missing in {codes:?}"
    );
}

// ── project path readiness ────────────────────────────────────────────────────

#[test]
fn readiness_all_paths_exist_produces_no_path_issues() {
    let dir = std::env::temp_dir();
    let context = RecoveryContext {
        preset_ids: vec!["default".to_string()],
        project_paths: vec![dir.to_string_lossy().to_string()],
        ..Default::default()
    };
    let path_issues: Vec<_> = detect_degraded_issues(&context)
        .into_iter()
        .filter(|i| i.issue_code == "missing_project_path")
        .collect();
    assert!(
        path_issues.is_empty(),
        "existing temp dir must not produce path issues; got {path_issues:?}"
    );
}

#[test]
fn readiness_missing_path_produces_path_issue() {
    let context = RecoveryContext {
        project_paths: vec!["/nonexistent/path/xyz_unique".to_string()],
        ..Default::default()
    };
    let codes: Vec<_> = detect_degraded_issues(&context)
        .into_iter()
        .map(|i| i.issue_code)
        .collect();
    assert!(
        codes.contains(&"missing_project_path".to_string()),
        "expected missing_project_path in {codes:?}"
    );
}

// ── keychain readiness ────────────────────────────────────────────────────────

#[test]
fn readiness_keychain_not_available_produces_keychain_issue() {
    // Callers pre-populate secret_ref_ids with IDs they have already
    // determined to be absent from the keychain.
    let context = RecoveryContext {
        secret_ref_ids: vec!["ref-missing".to_string()],
        ..Default::default()
    };
    let codes: Vec<_> = detect_degraded_issues(&context)
        .into_iter()
        .map(|i| i.issue_code)
        .collect();
    assert!(
        codes.contains(&"keychain_ref_missing".to_string()),
        "expected keychain_ref_missing in {codes:?}"
    );
}
