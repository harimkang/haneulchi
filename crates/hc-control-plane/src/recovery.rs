use hc_domain::{WorkflowHealth, settings::{DegradedIssue, RecoveryAction}};

// ── context ──────────────────────────────────────────────────────────────────

/// All inputs that drive degraded-issue detection for a single reconciliation
/// cycle.  Callers populate fields relevant to their context; unused fields
/// default to empty/false.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct RecoveryContext {
    /// Available preset IDs.  An empty list triggers `preset_missing`.
    pub preset_ids: Vec<String>,
    /// Filesystem paths that must exist.  Missing entries trigger
    /// `missing_project_path`.
    pub project_paths: Vec<String>,
    /// Secret-ref IDs that should be present in the Keychain.  Missing ones
    /// trigger `keychain_ref_missing`.
    pub secret_ref_ids: Vec<String>,
    /// Current workflow health from the snapshot.
    pub workflow_health: WorkflowHealth,
    /// Session IDs whose claims were found stale during reconcile.
    pub stale_claim_session_ids: Vec<String>,
    /// Set to `true` when a crash-restore condition is detected.
    pub has_crashed_restore: bool,
    /// Set to `true` when the active worktree cannot be reached.
    pub has_worktree_unreachable: bool,
    /// Repo paths (project roots) that have been detected as missing their .git directory.
    /// Each triggers `deleted_repo`.
    #[serde(default)]
    pub deleted_repo_paths: Vec<String>,
    /// Set to `true` when a before_run hook has failed during workflow bootstrap.
    #[serde(default)]
    pub has_before_run_hook_failure: bool,
}

// ── detection ────────────────────────────────────────────────────────────────

/// Detect degraded issues from the provided context.
///
/// Details fields are restricted to IDs, paths, and codes — never secret
/// values.
pub fn detect_degraded_issues(context: &RecoveryContext) -> Vec<DegradedIssue> {
    let mut issues = Vec::new();

    // preset_missing — no presets available
    if context.preset_ids.is_empty() {
        issues.push(DegradedIssue {
            issue_code: "preset_missing".to_string(),
            worktree_id: None,
            project_id: None,
            details: "no presets available".to_string(),
        });
    }

    // missing_project_path — paths that do not exist on disk
    for path in &context.project_paths {
        if !std::path::Path::new(path).exists() {
            issues.push(DegradedIssue {
                issue_code: "missing_project_path".to_string(),
                worktree_id: None,
                project_id: None,
                // details carries only the path reference — never a secret
                details: format!("path not found: {path}"),
            });
        }
    }

    // keychain_ref_missing — secret refs listed but not resolvable (callers
    // are expected to have already verified absence before populating this
    // field; we report one issue per missing ref_id)
    for ref_id in &context.secret_ref_ids {
        issues.push(DegradedIssue {
            issue_code: "keychain_ref_missing".to_string(),
            worktree_id: None,
            project_id: None,
            // details contains only the ref ID, not any secret value
            details: format!("secret ref not in keychain: {ref_id}"),
        });
    }

    // invalid_workflow_reload — workflow is in a degraded state
    if let Some(issue) = workflow_health_to_recovery_issue(&context.workflow_health) {
        issues.push(issue);
    }

    // stale_claim_reconcile — sessions with unresolved stale claims
    for session_id in &context.stale_claim_session_ids {
        issues.push(DegradedIssue {
            issue_code: "stale_claim_reconcile".to_string(),
            worktree_id: None,
            project_id: None,
            details: format!("stale claim for session: {session_id}"),
        });
    }

    // crashed_restore
    if context.has_crashed_restore {
        issues.push(DegradedIssue {
            issue_code: "crashed_restore".to_string(),
            worktree_id: None,
            project_id: None,
            details: "crash-restore detected".to_string(),
        });
    }

    // worktree_unreachable
    if context.has_worktree_unreachable {
        issues.push(DegradedIssue {
            issue_code: "worktree_unreachable".to_string(),
            worktree_id: None,
            project_id: None,
            details: "worktree is unreachable".to_string(),
        });
    }

    // deleted_repo — git repo no longer exists at the project root
    for path in &context.deleted_repo_paths {
        issues.push(DegradedIssue {
            issue_code: "deleted_repo".to_string(),
            worktree_id: None,
            project_id: None,
            details: format!("repo not found at: {path}"),
        });
    }

    // before_run_hook_failure
    if context.has_before_run_hook_failure {
        issues.push(DegradedIssue {
            issue_code: "before_run_hook_failure".to_string(),
            worktree_id: None,
            project_id: None,
            details: "before_run hook failed during workflow bootstrap".to_string(),
        });
    }

    issues
}

// ── action routing ───────────────────────────────────────────────────────────

/// Map a `DegradedIssue` to its recommended `RecoveryAction`.
pub fn recovery_action_for_issue(issue: &DegradedIssue) -> RecoveryAction {
    match issue.issue_code.as_str() {
        "preset_missing" => RecoveryAction::IgnoreAndContinue,
        "missing_project_path" => RecoveryAction::ResetPath,
        "deleted_repo" => RecoveryAction::MarkArchived,
        "keychain_ref_missing" => RecoveryAction::RefreshKeychainRef,
        "invalid_workflow_reload" => RecoveryAction::ReloadWorkflow,
        "before_run_hook_failure" => RecoveryAction::IgnoreAndContinue,
        "worktree_unreachable" => RecoveryAction::DeleteWorktree,
        "crashed_restore" => RecoveryAction::MarkArchived,
        "stale_claim_reconcile" => RecoveryAction::IgnoreAndContinue,
        // Safe default for unknown codes
        _ => RecoveryAction::IgnoreAndContinue,
    }
}

// ── workflow_projection helper ───────────────────────────────────────────────

/// Returns `Some(DegradedIssue)` when the workflow health indicates an invalid
/// reload condition; `None` for healthy or non-applicable states.
pub fn workflow_health_to_recovery_issue(health: &WorkflowHealth) -> Option<DegradedIssue> {
    match health {
        WorkflowHealth::InvalidKeptLastGood => Some(DegradedIssue {
            issue_code: "invalid_workflow_reload".to_string(),
            worktree_id: None,
            project_id: None,
            details: "workflow is invalid; last good version is active".to_string(),
        }),
        _ => None,
    }
}
