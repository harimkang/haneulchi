use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ClaimState;
use crate::review::ReviewItem;

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum TaskColumn {
    #[default]
    Inbox,
    Ready,
    Running,
    Review,
    Blocked,
    Done,
}

impl TaskColumn {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Review => "review",
            Self::Blocked => "blocked",
            Self::Done => "done",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Inbox => "Inbox",
            Self::Ready => "Ready",
            Self::Running => "Running",
            Self::Review => "Review",
            Self::Blocked => "Blocked",
            Self::Done => "Done",
        }
    }

    pub const fn all() -> [Self; 6] {
        [
            Self::Inbox,
            Self::Ready,
            Self::Running,
            Self::Review,
            Self::Blocked,
            Self::Done,
        ]
    }
}

impl FromStr for TaskColumn {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "inbox" => Ok(Self::Inbox),
            "ready" => Ok(Self::Ready),
            "running" => Ok(Self::Running),
            "review" => Ok(Self::Review),
            "blocked" => Ok(Self::Blocked),
            "done" => Ok(Self::Done),
            _ => Err(value.to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAutomationMode {
    #[default]
    Manual,
    Assisted,
    AutoEligible,
}

impl TaskAutomationMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Assisted => "assisted",
            Self::AutoEligible => "auto_eligible",
        }
    }

    pub const fn all() -> [&'static str; 3] {
        [
            Self::Manual.as_str(),
            Self::Assisted.as_str(),
            Self::AutoEligible.as_str(),
        ]
    }
}

impl FromStr for TaskAutomationMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "manual" => Ok(Self::Manual),
            "assisted" => Ok(Self::Assisted),
            "auto_eligible" => Ok(Self::AutoEligible),
            _ => Err(value.to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskClaimLifecycleState {
    #[default]
    Unclaimed,
    Claimed,
    Running,
    Released,
    Terminal,
}

impl TaskClaimLifecycleState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unclaimed => "unclaimed",
            Self::Claimed => "claimed",
            Self::Running => "running",
            Self::Released => "released",
            Self::Terminal => "terminal",
        }
    }
}

impl FromStr for TaskClaimLifecycleState {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "unclaimed" => Ok(Self::Unclaimed),
            "claimed" => Ok(Self::Claimed),
            "running" => Ok(Self::Running),
            "released" => Ok(Self::Released),
            "terminal" => Ok(Self::Terminal),
            _ => Err(value.to_string()),
        }
    }
}

pub fn project_claim_state(
    internal_state: Option<TaskClaimLifecycleState>,
    has_live_owner: bool,
) -> ClaimState {
    match (internal_state, has_live_owner) {
        (None, _) | (Some(TaskClaimLifecycleState::Unclaimed), _) => ClaimState::None,
        (Some(TaskClaimLifecycleState::Claimed), true)
        | (Some(TaskClaimLifecycleState::Running), true) => ClaimState::Claimed,
        (Some(TaskClaimLifecycleState::Released), false)
        | (Some(TaskClaimLifecycleState::Terminal), false) => ClaimState::Released,
        _ => ClaimState::Stale,
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub display_key: String,
    pub column: TaskColumn,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub automation_mode: TaskAutomationMode,
    pub tracker_binding_state: String,
    pub linked_session_id: Option<String>,
    pub linked_worktree_id: Option<String>,
    pub latest_review_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Task {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        project_id: impl Into<String>,
        display_key: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        priority: impl Into<String>,
        automation_mode: TaskAutomationMode,
        created_at: impl Into<String>,
        updated_at: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            project_id: project_id.into(),
            display_key: display_key.into(),
            column: TaskColumn::Inbox,
            title: title.into(),
            description: description.into(),
            priority: priority.into(),
            automation_mode,
            tracker_binding_state: "local_only".to_string(),
            linked_session_id: None,
            linked_worktree_id: None,
            latest_review_id: None,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskBoardColumnProjection {
    pub column: TaskColumn,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskDrawerProjection {
    pub task: Task,
    pub claim_state: ClaimState,
    pub latest_review: Option<ReviewItem>,
}
