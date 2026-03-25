use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TerminalSettings {
    pub shell: String,
    pub default_cols: u32,
    pub default_rows: u32,
    pub scrollback_lines: u32,
    pub font_name: String,
    pub theme: String,
    pub cursor_style: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SecretRef {
    pub ref_id: String,
    pub label: String,
    pub env_var_name: String,
    pub keychain_service: String,
    pub keychain_account: String,
    /// Project scope filter. Empty string means applies to all projects.
    #[serde(default)]
    pub scope: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorktreePolicy {
    pub policy_id: String,
    pub max_age_days: Option<u32>,
    pub max_count: Option<u32>,
    pub auto_prune: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct NotificationRule {
    pub rule_id: String,
    pub event_kind: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectRecord {
    pub project_id: String,
    pub name: String,
    pub root_path: String,
    pub last_focused_at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct LayoutRecord {
    pub layout_id: String,
    pub project_id: String,
    /// Opaque JSON blob. Callers are responsible for validating content before persisting.
    pub data_json: String,
    pub saved_at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SessionMetadataRecord {
    pub session_id: String,
    pub project_id: String,
    pub title: String,
    pub cwd: String,
    pub branch: Option<String>,
    pub last_active_at: Option<String>,
    pub is_recoverable: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AppStateRecord {
    pub active_route: String,
    pub last_project_id: Option<String>,
    pub last_session_id: Option<String>,
    pub saved_at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DegradedIssue {
    pub issue_code: String,
    pub worktree_id: Option<String>,
    pub project_id: Option<String>,
    pub details: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryAction {
    DeleteWorktree,
    ResetPath,
    ReloadWorkflow,
    RefreshKeychainRef,
    MarkArchived,
    IgnoreAndContinue,
}

impl RecoveryAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeleteWorktree => "delete_worktree",
            Self::ResetPath => "reset_path",
            Self::ReloadWorkflow => "reload_workflow",
            Self::RefreshKeychainRef => "refresh_keychain_ref",
            Self::MarkArchived => "mark_archived",
            Self::IgnoreAndContinue => "ignore_and_continue",
        }
    }

    pub const fn all() -> [&'static str; 6] {
        [
            Self::DeleteWorktree.as_str(),
            Self::ResetPath.as_str(),
            Self::ReloadWorkflow.as_str(),
            Self::RefreshKeychainRef.as_str(),
            Self::MarkArchived.as_str(),
            Self::IgnoreAndContinue.as_str(),
        ]
    }
}
