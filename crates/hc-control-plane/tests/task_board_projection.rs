use hc_control_plane::{
    EligibilityContext, ReviewDecision, ReviewQueueService, TaskBoardColumnSummary,
    TaskBoardService, evaluate_task_eligibility,
};
use hc_domain::{ClaimState, PolicyPack, TaskAutomationMode, TaskColumn, WorkflowHealth};

#[test]
fn board_projection_groups_tasks_by_fixed_columns() {
    let service = TaskBoardService::demo().expect("demo board");
    let projection = service.board(None).expect("board projection");

    assert_eq!(
        projection
            .columns
            .iter()
            .map(|column| column.column)
            .collect::<Vec<_>>(),
        vec![
            TaskColumn::Inbox,
            TaskColumn::Ready,
            TaskColumn::Running,
            TaskColumn::Review,
            TaskColumn::Blocked,
            TaskColumn::Done,
        ]
    );
    assert_eq!(
        projection.projects,
        vec![
            TaskBoardColumnSummary::new("proj_alpha", 1),
            TaskBoardColumnSummary::new("proj_demo", 2),
        ]
    );
}

#[test]
fn project_filter_and_move_mutation_update_storage_backed_projection() {
    let service = TaskBoardService::demo().expect("demo board");

    let filtered = service.board(Some("proj_demo")).expect("filtered board");
    assert_eq!(filtered.selected_project_id.as_deref(), Some("proj_demo"));
    assert_eq!(filtered.columns[0].tasks.len(), 1);
    assert_eq!(filtered.columns[1].tasks.len(), 1);
    assert!(filtered.columns[3].tasks.is_empty());

    service
        .move_task("task_ready", TaskColumn::Review, "operator_drag")
        .expect("move task");

    let updated = service.board(Some("proj_demo")).expect("updated board");
    assert!(updated.columns[1].tasks.is_empty());
    assert_eq!(updated.columns[3].tasks.len(), 1);
    assert_eq!(updated.columns[3].tasks[0].id, "task_ready");
}

#[test]
fn review_ready_queue_only_lists_pending_review_items() {
    let service = ReviewQueueService::demo().expect("review queue");

    let projection = service.review_ready_projection().expect("review queue projection");

    assert_eq!(projection.items.len(), 1);
    assert_eq!(projection.items[0].task_id, "task_review");
    assert_eq!(projection.items[0].touched_files, vec!["Sources/Auth.swift", "Tests/AuthTests.swift"]);
    assert_eq!(projection.items[0].warnings, vec!["snapshot drift"]);
}

#[test]
fn timeline_accept_request_changes_manual_continue_and_follow_up_update_projections() {
    let service = ReviewQueueService::demo().expect("review queue");

    let accepted = service
        .apply_decision("task_review", ReviewDecision::Accept)
        .expect("accept decision");
    assert_eq!(accepted.task.column, TaskColumn::Done);
    assert_eq!(accepted.task.linked_session_id, None);

    let request_changes = ReviewQueueService::demo()
        .expect("review queue")
        .apply_decision("task_review", ReviewDecision::RequestChanges)
        .expect("request changes");
    assert_eq!(request_changes.task.column, TaskColumn::Ready);

    let manual_continue = ReviewQueueService::demo()
        .expect("review queue")
        .apply_decision("task_review", ReviewDecision::ManualContinue)
        .expect("manual continue");
    assert_eq!(manual_continue.task.column, TaskColumn::Running);
    assert_eq!(manual_continue.task.linked_session_id.as_deref(), Some("ses_02"));

    let follow_up = ReviewQueueService::demo()
        .expect("review queue")
        .apply_decision("task_review", ReviewDecision::FollowUp)
        .expect("follow up");
    assert_eq!(follow_up.follow_up_task.as_ref().expect("follow up").column, TaskColumn::Inbox);

    let timeline = follow_up.timeline;
    assert!(timeline.iter().any(|item| item.kind == "task_created"));
    assert!(timeline.iter().any(|item| item.kind == "review_decided"));
    assert!(timeline.iter().any(|item| item.kind == "follow_up_created"));
}

#[test]
fn eligibility_policy_pack_and_automation_mode_projection_block_invalid_dispatch() {
    let service = TaskBoardService::demo().expect("demo board");

    let updated = service
        .set_automation_mode("task_ready", TaskAutomationMode::AutoEligible)
        .expect("automation mode update");
    assert_eq!(updated.automation_mode, TaskAutomationMode::AutoEligible);

    let details = service
        .automation_details(
            "task_ready",
            WorkflowHealth::Ok,
            "gemini",
            PolicyPack {
                require_review: true,
                max_runtime_minutes: Some(45),
                allowed_agents: vec!["codex".to_string()],
                unsafe_override_policy: Some("explicit_only".to_string()),
            },
            ClaimState::None,
        )
        .expect("automation details");

    assert_eq!(details.policy_pack.require_review, true);
    assert_eq!(details.policy_pack.max_runtime_minutes, Some(45));
    assert_eq!(details.policy_pack.allowed_agents, vec!["codex"]);
    assert_eq!(
        details.policy_pack.unsafe_override_policy.as_deref(),
        Some("explicit_only")
    );
    assert_eq!(
        details.blocker_reason.as_deref(),
        Some("task_not_eligible_for_dispatch")
    );
}

#[test]
fn eligibility_keeps_local_board_authoritative_even_when_tracker_binding_exists() {
    let service = TaskBoardService::demo().expect("demo board");
    let mut task = service
        .task("task_ready")
        .expect("task lookup")
        .expect("task row");
    task.tracker_binding_state = "bound".to_string();
    task.automation_mode = TaskAutomationMode::AutoEligible;

    let details = evaluate_task_eligibility(
        &task,
        &EligibilityContext {
            workflow_health: WorkflowHealth::InvalidKeptLastGood,
            selected_agent: "codex".to_string(),
            claim_state: ClaimState::Claimed,
            policy_pack: PolicyPack {
                require_review: false,
                max_runtime_minutes: Some(30),
                allowed_agents: vec!["codex".to_string()],
                unsafe_override_policy: Some("explicit_only".to_string()),
            },
        },
    );

    assert_eq!(details.tracker_binding_state, "bound");
    assert_eq!(details.blocker_reason.as_deref(), Some("workflow_invalid"));
}
