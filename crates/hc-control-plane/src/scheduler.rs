use hc_domain::{TaskAutomationMode, TaskColumn};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SchedulerTask {
    pub task_id: String,
    pub column: TaskColumn,
    pub automation_mode: TaskAutomationMode,
    pub target_session_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SchedulerIssue {
    pub task_id: String,
    pub reason_code: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SchedulerResult {
    pub launched_task_ids: Vec<String>,
    pub queued: Vec<SchedulerIssue>,
    pub failures: Vec<SchedulerIssue>,
}

pub struct BoundedScheduler {
    tasks: Vec<SchedulerTask>,
}

impl BoundedScheduler {
    pub fn demo() -> Self {
        Self {
            tasks: vec![
                SchedulerTask {
                    task_id: "task_auto_01".to_string(),
                    column: TaskColumn::Ready,
                    automation_mode: TaskAutomationMode::AutoEligible,
                    target_session_id: None,
                },
                SchedulerTask {
                    task_id: "task_auto_02".to_string(),
                    column: TaskColumn::Ready,
                    automation_mode: TaskAutomationMode::AutoEligible,
                    target_session_id: None,
                },
                SchedulerTask {
                    task_id: "task_stale".to_string(),
                    column: TaskColumn::Ready,
                    automation_mode: TaskAutomationMode::Assisted,
                    target_session_id: Some("stale-session".to_string()),
                },
            ],
        }
    }

    pub fn tick(&self, running_slots: u32, max_slots: u32) -> SchedulerResult {
        let available_slots = max_slots.saturating_sub(running_slots) as usize;
        let mut launched_task_ids = Vec::new();
        let mut queued = Vec::new();
        let mut failures = Vec::new();

        for task in &self.tasks {
            if task.column != TaskColumn::Ready || task.automation_mode == TaskAutomationMode::Manual {
                continue;
            }

            if matches!(task.target_session_id.as_deref(), Some("stale-session")) {
                failures.push(SchedulerIssue {
                    task_id: task.task_id.clone(),
                    reason_code: "stale_target_session".to_string(),
                });
                continue;
            }

            if launched_task_ids.len() < available_slots {
                launched_task_ids.push(task.task_id.clone());
            } else {
                queued.push(SchedulerIssue {
                    task_id: task.task_id.clone(),
                    reason_code: "slot_capacity_exhausted".to_string(),
                });
            }
        }

        SchedulerResult {
            launched_task_ids,
            queued,
            failures,
        }
    }
}
