use hc_domain::{ClaimState, PolicyPack, Task, TaskAutomationDetails, TaskAutomationMode, TaskColumn, WorkflowHealth};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EligibilityContext {
    pub workflow_health: WorkflowHealth,
    pub selected_agent: String,
    pub claim_state: ClaimState,
    pub policy_pack: PolicyPack,
}

pub fn evaluate_task_eligibility(task: &Task, context: &EligibilityContext) -> TaskAutomationDetails {
    let blocker_reason = if task.column != TaskColumn::Ready {
        Some("task_not_ready_for_dispatch".to_string())
    } else if task.automation_mode == TaskAutomationMode::Manual {
        Some("manual_mode".to_string())
    } else if context.workflow_health != WorkflowHealth::Ok {
        Some("workflow_invalid".to_string())
    } else if !context.policy_pack.allowed_agents.is_empty()
        && !context
            .policy_pack
            .allowed_agents
            .iter()
            .any(|agent| agent == &context.selected_agent)
    {
        Some("task_not_eligible_for_dispatch".to_string())
    } else if context.claim_state == ClaimState::Claimed {
        Some("task_claim_conflict".to_string())
    } else {
        None
    };

    TaskAutomationDetails {
        automation_mode: task.automation_mode,
        claim_state: context.claim_state,
        tracker_binding_state: task.tracker_binding_state.clone(),
        policy_pack: context.policy_pack.clone(),
        blocker_reason,
    }
}
