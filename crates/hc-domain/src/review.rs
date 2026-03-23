use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    #[default]
    Pending,
    Accepted,
    ChangesRequested,
    ManualContinue,
    FollowUpCreated,
}

impl ReviewStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::ChangesRequested => "changes_requested",
            Self::ManualContinue => "manual_continue",
            Self::FollowUpCreated => "follow_up_created",
        }
    }
}

impl FromStr for ReviewStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "changes_requested" => Ok(Self::ChangesRequested),
            "manual_continue" => Ok(Self::ManualContinue),
            "follow_up_created" => Ok(Self::FollowUpCreated),
            _ => Err(value.to_string()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReviewItem {
    pub id: String,
    pub task_id: String,
    pub session_id: Option<String>,
    pub status: ReviewStatus,
    pub summary: String,
    pub touched_files: Vec<String>,
    pub diff_summary: Option<String>,
    pub tests_summary: Option<String>,
    pub command_summary: Option<String>,
    pub warnings: Vec<String>,
    pub evidence_manifest_path: Option<String>,
    pub review_checklist_result: Option<String>,
    pub created_at: String,
    pub decided_at: Option<String>,
}

impl ReviewItem {
    pub fn new_pending(
        id: impl Into<String>,
        task_id: impl Into<String>,
        session_id: Option<String>,
        summary: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            task_id: task_id.into(),
            session_id,
            status: ReviewStatus::Pending,
            summary: summary.into(),
            touched_files: Vec::new(),
            diff_summary: None,
            tests_summary: None,
            command_summary: None,
            warnings: Vec::new(),
            evidence_manifest_path: None,
            review_checklist_result: None,
            created_at: created_at.into(),
            decided_at: None,
        }
    }
}
