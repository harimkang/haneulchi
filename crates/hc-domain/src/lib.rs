pub mod orchestrator;
pub mod review;
pub mod task;
pub mod time;
pub mod timeline;

use std::collections::BTreeMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub use orchestrator::{
    OrchestratorRuntime, PolicyPack, TaskAutomationDetails, TrackerBinding, WorkflowReloadEvent,
};
pub use review::{ReviewItem, ReviewStatus};
pub use task::{
    Task, TaskAutomationMode, TaskBoardColumnProjection, TaskClaimLifecycleState, TaskColumn,
    TaskDrawerProjection, project_claim_state,
};
pub use timeline::{TimelineEvent, TimelineEventKind};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionRuntimeState {
    Launching,
    Running,
    WaitingInput,
    ReviewReady,
    Blocked,
    Done,
    Error,
    Exited,
}

impl SessionRuntimeState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Launching => "launching",
            Self::Running => "running",
            Self::WaitingInput => "waiting_input",
            Self::ReviewReady => "review_ready",
            Self::Blocked => "blocked",
            Self::Done => "done",
            Self::Error => "error",
            Self::Exited => "exited",
        }
    }

    pub const fn all() -> [&'static str; 8] {
        [
            Self::Launching.as_str(),
            Self::Running.as_str(),
            Self::WaitingInput.as_str(),
            Self::ReviewReady.as_str(),
            Self::Blocked.as_str(),
            Self::Done.as_str(),
            Self::Error.as_str(),
            Self::Exited.as_str(),
        ]
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowHealth {
    None,
    Ok,
    InvalidKeptLastGood,
    ReloadPending,
}

impl WorkflowHealth {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Ok => "ok",
            Self::InvalidKeptLastGood => "invalid_kept_last_good",
            Self::ReloadPending => "reload_pending",
        }
    }

    pub const fn all() -> [&'static str; 4] {
        [
            Self::None.as_str(),
            Self::Ok.as_str(),
            Self::InvalidKeptLastGood.as_str(),
            Self::ReloadPending.as_str(),
        ]
    }
}

impl Default for WorkflowHealth {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimState {
    #[default]
    None,
    Claimed,
    Released,
    Stale,
}

impl ClaimState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Claimed => "claimed",
            Self::Released => "released",
            Self::Stale => "stale",
        }
    }

    pub const fn all() -> [&'static str; 4] {
        [
            Self::None.as_str(),
            Self::Claimed.as_str(),
            Self::Released.as_str(),
            Self::Stale.as_str(),
        ]
    }
}

impl FromStr for ClaimState {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "claimed" => Ok(Self::Claimed),
            "released" => Ok(Self::Released),
            "stale" => Ok(Self::Stale),
            _ => Err(value.to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionFocusState {
    Focused,
    Background,
}

impl SessionFocusState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Focused => "focused",
            Self::Background => "background",
        }
    }

    pub const fn all() -> [&'static str; 2] {
        [Self::Focused.as_str(), Self::Background.as_str()]
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkflowRuntimeStatus {
    pub state: WorkflowHealth,
    pub path: String,
    pub last_good_hash: Option<String>,
    pub last_reload_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TrackerStatus {
    pub state: String,
    pub last_sync_at: Option<String>,
    pub health: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectSummary {
    pub project_id: String,
    pub name: String,
    pub root_path: String,
    pub status: String,
    pub workflow_state: WorkflowHealth,
    pub session_count: u32,
    pub task_counts: BTreeMap<String, u32>,
    pub attention_count: u32,
}

impl ProjectSummary {
    pub fn new(
        project_id: impl Into<String>,
        name: impl Into<String>,
        root_path: impl Into<String>,
        workflow_state: WorkflowHealth,
        session_count: u32,
        attention_count: u32,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            name: name.into(),
            root_path: root_path.into(),
            status: "active".to_string(),
            workflow_state,
            session_count,
            task_counts: BTreeMap::new(),
            attention_count,
        }
    }

    pub fn with_task_count(mut self, column: impl Into<String>, count: u32) -> Self {
        self.task_counts.insert(column.into(), count);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub project_id: String,
    pub task_id: Option<String>,
    pub mode: String,
    pub runtime_state: SessionRuntimeState,
    pub manual_control: String,
    pub dispatch_state: String,
    pub claim_state: ClaimState,
    pub adapter_kind: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub dispatch_reason: Option<String>,
    pub title: String,
    pub cwd: String,
    pub workspace_root: String,
    pub base_root: String,
    pub branch: Option<String>,
    pub latest_summary: Option<String>,
    pub latest_commentary: Option<String>,
    pub commentary_updated_at: Option<String>,
    pub active_window_title: Option<String>,
    pub unread_count: u32,
    pub last_activity_at: Option<String>,
    pub focus_state: SessionFocusState,
    pub can_focus: bool,
    pub can_takeover: bool,
    pub can_release_takeover: bool,
}

impl SessionSummary {
    pub fn new(
        session_id: impl Into<String>,
        project_id: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            project_id: project_id.into(),
            task_id: None,
            mode: "generic".to_string(),
            runtime_state: SessionRuntimeState::Launching,
            manual_control: "none".to_string(),
            dispatch_state: "not_dispatchable".to_string(),
            claim_state: ClaimState::None,
            adapter_kind: None,
            provider_id: None,
            model_id: None,
            dispatch_reason: None,
            title: title.into(),
            cwd: String::new(),
            workspace_root: String::new(),
            base_root: ".".to_string(),
            branch: None,
            latest_summary: None,
            latest_commentary: None,
            commentary_updated_at: None,
            active_window_title: None,
            unread_count: 0,
            last_activity_at: None,
            focus_state: SessionFocusState::Background,
            can_focus: true,
            can_takeover: false,
            can_release_takeover: false,
        }
    }

    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = mode.into();
        self
    }

    pub fn with_runtime_state(mut self, runtime_state: SessionRuntimeState) -> Self {
        self.runtime_state = runtime_state;
        self
    }

    pub fn with_claim_state(mut self, claim_state: ClaimState) -> Self {
        self.claim_state = claim_state;
        self
    }

    pub fn with_focus_state(mut self, focus_state: SessionFocusState) -> Self {
        self.focus_state = focus_state;
        self
    }

    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = cwd.into();
        self
    }

    pub fn with_workspace_root(mut self, workspace_root: impl Into<String>) -> Self {
        self.workspace_root = workspace_root.into();
        self
    }

    pub fn with_base_root(mut self, base_root: impl Into<String>) -> Self {
        self.base_root = base_root.into();
        self
    }

    pub fn with_unread_count(mut self, unread_count: u32) -> Self {
        self.unread_count = unread_count;
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AttentionSummary {
    pub attention_id: String,
    pub kind: String,
    pub project_id: String,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub title: String,
    pub summary: String,
    pub created_at: Option<String>,
    pub severity: String,
    pub action_hint: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RetryQueueEntry {
    pub task_id: String,
    pub project_id: String,
    pub attempt: u32,
    pub reason_code: String,
    pub due_at: Option<String>,
    pub backoff_ms: u64,
    pub claim_state: ClaimState,
    pub retry_state: RetryState,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryState {
    #[default]
    None,
    Due,
    BackingOff,
    Exhausted,
}

impl RetryState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Due => "due",
            Self::BackingOff => "backing_off",
            Self::Exhausted => "exhausted",
        }
    }
}

impl FromStr for RetryState {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "due" => Ok(Self::Due),
            "backing_off" => Ok(Self::BackingOff),
            "exhausted" => Ok(Self::Exhausted),
            _ => Err(value.to_string()),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct WarningSummary {
    pub warning_id: String,
    pub severity: String,
    pub headline: String,
    pub next_action: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AppSnapshotMeta {
    pub snapshot_rev: u64,
    pub runtime_rev: u64,
    pub projection_rev: u64,
    pub snapshot_at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationOpsSummary {
    pub status: String,
    pub cadence_ms: u64,
    pub last_tick_at: Option<String>,
    pub last_reconcile_at: Option<String>,
    pub running_slots: u32,
    pub max_slots: u32,
    pub retry_due_count: u32,
    pub queued_claim_count: u32,
    pub paused: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct OpsEnvelope {
    pub automation: AutomationOpsSummary,
    pub workflow: WorkflowRuntimeStatus,
    pub tracker: TrackerStatus,
    pub app: AppState,
}

pub type OpsSummary = AutomationOpsSummary;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AppState {
    pub active_route: String,
    pub focused_session_id: Option<String>,
    pub degraded_flags: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AppSnapshot {
    pub meta: AppSnapshotMeta,
    pub ops: OpsEnvelope,
    pub projects: Vec<ProjectSummary>,
    pub sessions: Vec<SessionSummary>,
    pub attention: Vec<AttentionSummary>,
    pub retry_queue: Vec<RetryQueueEntry>,
    pub warnings: Vec<WarningSummary>,
}

impl AppSnapshot {
    pub fn new(workflow: WorkflowRuntimeStatus, tracker: TrackerStatus) -> Self {
        Self {
            meta: AppSnapshotMeta {
                snapshot_rev: 1,
                runtime_rev: 1,
                projection_rev: 1,
                snapshot_at: None,
            },
            ops: OpsEnvelope {
                automation: AutomationOpsSummary {
                    status: "running".to_string(),
                    cadence_ms: 15_000,
                    last_tick_at: None,
                    last_reconcile_at: None,
                    running_slots: 0,
                    max_slots: 1,
                    retry_due_count: 0,
                    queued_claim_count: 0,
                    paused: false,
                },
                workflow,
                tracker,
                app: AppState {
                    active_route: "project_focus".to_string(),
                    focused_session_id: None,
                    degraded_flags: Vec::new(),
                },
            },
            projects: Vec::new(),
            sessions: Vec::new(),
            attention: Vec::new(),
            retry_queue: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_project(mut self, project: ProjectSummary) -> Self {
        self.projects.push(project);
        self
    }

    pub fn with_session(mut self, session: SessionSummary) -> Self {
        self.sessions.push(session);
        self
    }

    pub fn with_automation(mut self, automation: AutomationOpsSummary) -> Self {
        self.ops.automation = automation;
        self
    }

    pub fn with_app_state(mut self, app: AppState) -> Self {
        self.ops.app = app;
        self
    }

    pub fn with_retry_entry(mut self, entry: RetryQueueEntry) -> Self {
        self.retry_queue.push(entry);
        self
    }
}
