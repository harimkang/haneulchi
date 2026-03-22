use hc_domain::{
    ClaimState, TaskAutomationMode, TaskClaimLifecycleState, TaskColumn, project_claim_state,
};

#[test]
fn task_vocabulary_matches_sprint_three_docs() {
    let ordered_columns = TaskColumn::all();
    assert_eq!(
        ordered_columns.map(TaskColumn::as_str),
        ["inbox", "ready", "running", "review", "blocked", "done",]
    );
    assert_eq!(
        ordered_columns.map(TaskColumn::label),
        ["Inbox", "Ready", "Running", "Review", "Blocked", "Done",]
    );
    assert_eq!(
        TaskAutomationMode::all(),
        ["manual", "assisted", "auto_eligible",]
    );
}

#[test]
fn internal_claim_states_project_to_operator_vocabulary() {
    assert_eq!(project_claim_state(None, false), ClaimState::None);
    assert_eq!(
        project_claim_state(Some(TaskClaimLifecycleState::Unclaimed), false),
        ClaimState::None
    );
    assert_eq!(
        project_claim_state(Some(TaskClaimLifecycleState::Claimed), true),
        ClaimState::Claimed
    );
    assert_eq!(
        project_claim_state(Some(TaskClaimLifecycleState::Running), true),
        ClaimState::Claimed
    );
    assert_eq!(
        project_claim_state(Some(TaskClaimLifecycleState::Released), false),
        ClaimState::Released
    );
    assert_eq!(
        project_claim_state(Some(TaskClaimLifecycleState::Terminal), false),
        ClaimState::Released
    );
    assert_eq!(
        project_claim_state(Some(TaskClaimLifecycleState::Claimed), false),
        ClaimState::Stale
    );
    assert_eq!(
        project_claim_state(Some(TaskClaimLifecycleState::Running), false),
        ClaimState::Stale
    );
}
