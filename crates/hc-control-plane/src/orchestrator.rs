use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationStatusSummary {
    pub status: String,
    pub queued_claim_count: u32,
    pub paused: bool,
}
