pub mod orchestrator;
pub mod review;
pub mod task;
pub mod timeline;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub use orchestrator::{PolicyPack, TaskAutomationDetails};
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimState {
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
    pub title: String,
    pub cwd: String,
    pub workspace_root: String,
    pub base_root: String,
    pub branch: Option<String>,
    pub latest_summary: Option<String>,
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
            title: title.into(),
            cwd: String::new(),
            workspace_root: String::new(),
            base_root: ".".to_string(),
            branch: None,
            latest_summary: None,
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
pub struct OpsSummary {
    pub running_slots: u32,
    pub max_slots: u32,
    pub retry_queue_count: u32,
    pub workflow_health: WorkflowHealth,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AppState {
    pub active_route: String,
    pub focused_session_id: Option<String>,
    pub degraded_flags: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AppSnapshot {
    pub meta: AppSnapshotMeta,
    pub ops: OpsSummary,
    pub workflow: WorkflowRuntimeStatus,
    pub tracker: TrackerStatus,
    pub app: AppState,
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
            ops: OpsSummary {
                running_slots: 0,
                max_slots: 1,
                retry_queue_count: 0,
                workflow_health: workflow.state,
            },
            workflow,
            tracker,
            app: AppState {
                active_route: "project_focus".to_string(),
                focused_session_id: None,
                degraded_flags: Vec::new(),
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
}
