use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineEventKind {
    TaskCreated,
    TaskUpdated,
    TaskMoved,
    TaskAttached,
    TaskDetached,
    ClaimAcquired,
    ClaimReleased,
    SessionLaunched,
    WorktreeCreated,
    HookFinished,
    ReviewReady,
    ReviewDecided,
    FollowUpCreated,
    DispatchSent,
    AttentionCreated,
}

impl TimelineEventKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TaskCreated => "task_created",
            Self::TaskUpdated => "task_updated",
            Self::TaskMoved => "task_moved",
            Self::TaskAttached => "task_attached",
            Self::TaskDetached => "task_detached",
            Self::ClaimAcquired => "claim_acquired",
            Self::ClaimReleased => "claim_released",
            Self::SessionLaunched => "session_launched",
            Self::WorktreeCreated => "worktree_created",
            Self::HookFinished => "hook_finished",
            Self::ReviewReady => "review_ready",
            Self::ReviewDecided => "review_decided",
            Self::FollowUpCreated => "follow_up_created",
            Self::DispatchSent => "dispatch_sent",
            Self::AttentionCreated => "attention_created",
        }
    }
}

impl FromStr for TimelineEventKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "task_created" => Ok(Self::TaskCreated),
            "task_updated" => Ok(Self::TaskUpdated),
            "task_moved" => Ok(Self::TaskMoved),
            "task_attached" => Ok(Self::TaskAttached),
            "task_detached" => Ok(Self::TaskDetached),
            "claim_acquired" => Ok(Self::ClaimAcquired),
            "claim_released" => Ok(Self::ClaimReleased),
            "session_launched" => Ok(Self::SessionLaunched),
            "worktree_created" => Ok(Self::WorktreeCreated),
            "hook_finished" => Ok(Self::HookFinished),
            "review_ready" => Ok(Self::ReviewReady),
            "review_decided" => Ok(Self::ReviewDecided),
            "follow_up_created" => Ok(Self::FollowUpCreated),
            "dispatch_sent" => Ok(Self::DispatchSent),
            "attention_created" => Ok(Self::AttentionCreated),
            _ => Err(value.to_string()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TimelineEvent {
    pub id: String,
    pub task_id: String,
    pub session_id: Option<String>,
    pub review_item_id: Option<String>,
    pub worktree_id: Option<String>,
    pub kind: TimelineEventKind,
    pub actor: String,
    pub reason_code: Option<String>,
    pub payload_json: Option<String>,
    pub created_at: String,
}
