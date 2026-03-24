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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct OrchestratorRuntime {
    pub singleton_key: String,
    pub cadence_ms: u64,
    pub last_tick_at: Option<String>,
    pub last_reconcile_at: Option<String>,
    pub max_slots: u32,
    pub running_slots: u32,
    pub workflow_state: String,
    pub tracker_state: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkflowReloadEvent {
    pub id: String,
    pub project_id: String,
    pub file_path: String,
    pub status: String,
    pub loaded_hash: Option<String>,
    pub kept_last_good_hash: Option<String>,
    pub message: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TrackerBinding {
    pub id: String,
    pub task_id: String,
    pub provider: String,
    pub external_id: String,
    pub external_key: String,
    pub sync_mode: String,
    pub state: String,
    pub last_sync_at: Option<String>,
}
