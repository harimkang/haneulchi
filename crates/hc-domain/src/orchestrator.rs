use serde::{Deserialize, Serialize};

use crate::{ClaimState, TaskAutomationMode};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PolicyPack {
    pub require_review: bool,
    pub max_runtime_minutes: Option<u64>,
    pub allowed_agents: Vec<String>,
    pub unsafe_override_policy: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskAutomationDetails {
    pub automation_mode: TaskAutomationMode,
    pub claim_state: ClaimState,
    pub tracker_binding_state: String,
    pub policy_pack: PolicyPack,
    pub blocker_reason: Option<String>,
}

impl Default for TaskAutomationDetails {
    fn default() -> Self {
        Self {
            automation_mode: TaskAutomationMode::Manual,
            claim_state: ClaimState::None,
            tracker_binding_state: "local_only".to_string(),
            policy_pack: PolicyPack::default(),
            blocker_reason: None,
        }
    }
}
