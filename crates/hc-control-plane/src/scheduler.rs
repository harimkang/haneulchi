use hc_domain::{TaskAutomationMode, TaskColumn};
use serde::{Deserialize, Serialize};

use crate::shared_store::lock_shared_store;

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
    pub fn from_store(store: &hc_storage::SqliteStore) -> Result<Self, hc_storage::StorageError> {
        let board = store.tasks().board(None)?;
        let tasks = board
            .into_iter()
            .flat_map(|column| {
                column.tasks.into_iter().map(move |task| SchedulerTask {
                    task_id: task.id,
                    column: column.column,
                    automation_mode: task.automation_mode,
                    target_session_id: task.linked_session_id,
                })
            })
            .collect();

        Ok(Self { tasks })
    }

    pub fn tick(&self, running_slots: u32, max_slots: u32, live_session_ids: &[String]) -> SchedulerResult {
        let available_slots = max_slots.saturating_sub(running_slots) as usize;
        let mut launched_task_ids = Vec::new();
        let mut queued = Vec::new();
        let mut failures = Vec::new();

        for task in &self.tasks {
            if task.column != TaskColumn::Ready || task.automation_mode == TaskAutomationMode::Manual {
                continue;
            }

            if let Some(target_session_id) = task.target_session_id.as_ref() {
                if !live_session_ids.iter().any(|session_id| session_id == target_session_id) {
                failures.push(SchedulerIssue {
                    task_id: task.task_id.clone(),
                    reason_code: "stale_target_session".to_string(),
                });
                continue;
                }
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

pub fn shared_scheduler_tick(
    running_slots: u32,
    max_slots: u32,
    live_session_ids: &[String],
) -> Result<SchedulerResult, String> {
    let store = lock_shared_store().map_err(|error| error.to_string())?;
    let scheduler = BoundedScheduler::from_store(&store).map_err(|error| error.to_string())?;
    Ok(scheduler.tick(running_slots, max_slots, live_session_ids))
}
