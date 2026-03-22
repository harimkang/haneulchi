use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use hc_domain::{Task, TaskAutomationMode, TaskBoardColumnProjection, TaskColumn};
use hc_storage::{NewTaskRecord, SqliteStore};
use serde::{Deserialize, Serialize};

const DEMO_CREATED_AT: &str = "2026-03-23T02:00:00Z";
const DEMO_UPDATED_AT: &str = "2026-03-23T02:05:00Z";
const MOVE_UPDATED_AT: &str = "2026-03-23T02:10:00Z";

#[derive(Debug, thiserror::Error)]
pub enum TaskBoardError {
    #[error("task board lock poisoned")]
    LockPoisoned,
    #[error("storage error: {0}")]
    Storage(#[from] hc_storage::StorageError),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskBoardColumnSummary {
    pub project_id: String,
    pub task_count: usize,
}

impl TaskBoardColumnSummary {
    pub fn new(project_id: impl Into<String>, task_count: usize) -> Self {
        Self {
            project_id: project_id.into(),
            task_count,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskBoardProjection {
    pub selected_project_id: Option<String>,
    pub projects: Vec<TaskBoardColumnSummary>,
    pub columns: Vec<TaskBoardColumnProjection>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskBoardMutationResult {
    pub task: Task,
}

pub struct TaskBoardService {
    store: SqliteStore,
}

impl TaskBoardService {
    pub fn new(store: SqliteStore) -> Self {
        Self { store }
    }

    pub fn demo() -> Result<Self, TaskBoardError> {
        let store = SqliteStore::in_memory()?;
        seed_demo_board(&store)?;
        Ok(Self::new(store))
    }

    pub fn board(&self, project_id: Option<&str>) -> Result<TaskBoardProjection, TaskBoardError> {
        let columns = self.store.tasks().board(project_id)?;
        let all_columns = self.store.tasks().board(None)?;
        let mut project_counts = BTreeMap::new();

        for column in &all_columns {
            for task in &column.tasks {
                *project_counts
                    .entry(task.project_id.clone())
                    .or_insert(0usize) += 1;
            }
        }

        let projects = project_counts
            .into_iter()
            .map(|(project_id, task_count)| TaskBoardColumnSummary::new(project_id, task_count))
            .collect();

        Ok(TaskBoardProjection {
            selected_project_id: project_id.map(ToOwned::to_owned),
            projects,
            columns,
        })
    }

    pub fn move_task(
        &self,
        task_id: &str,
        column: TaskColumn,
        actor: &str,
    ) -> Result<TaskBoardMutationResult, TaskBoardError> {
        let task = self
            .store
            .tasks()
            .move_to(task_id, column, actor, MOVE_UPDATED_AT)?;

        Ok(TaskBoardMutationResult { task })
    }
}

pub fn shared_task_board_projection(
    project_id: Option<&str>,
) -> Result<TaskBoardProjection, TaskBoardError> {
    lock_shared_board_service()?.board(project_id)
}

pub fn shared_task_move(
    task_id: &str,
    column: TaskColumn,
    actor: &str,
) -> Result<TaskBoardMutationResult, TaskBoardError> {
    lock_shared_board_service()?.move_task(task_id, column, actor)
}

pub fn reset_task_board_for_tests() {
    if let Ok(mut service) = lock_shared_board_service() {
        if let Ok(demo) = TaskBoardService::demo() {
            *service = demo;
        }
    }
}

fn shared_board_service() -> &'static Mutex<TaskBoardService> {
    static TASK_BOARD: OnceLock<Mutex<TaskBoardService>> = OnceLock::new();
    TASK_BOARD.get_or_init(|| Mutex::new(TaskBoardService::demo().expect("demo task board")))
}

fn lock_shared_board_service()
-> Result<std::sync::MutexGuard<'static, TaskBoardService>, TaskBoardError> {
    shared_board_service()
        .lock()
        .map_err(|_| TaskBoardError::LockPoisoned)
}

fn seed_demo_board(store: &SqliteStore) -> Result<(), TaskBoardError> {
    store.tasks().create(NewTaskRecord {
        id: "task_inbox".to_string(),
        project_id: "proj_demo".to_string(),
        display_key: "TASK-INBOX".to_string(),
        title: "Task draft".to_string(),
        description: "Inbox task for board filtering".to_string(),
        priority: "p2".to_string(),
        automation_mode: TaskAutomationMode::Manual,
        created_at: DEMO_CREATED_AT.to_string(),
        updated_at: DEMO_CREATED_AT.to_string(),
    })?;
    store.tasks().create(NewTaskRecord {
        id: "task_ready".to_string(),
        project_id: "proj_demo".to_string(),
        display_key: "TASK-READY".to_string(),
        title: "Ready handoff".to_string(),
        description: "Task ready to move across the board".to_string(),
        priority: "p1".to_string(),
        automation_mode: TaskAutomationMode::Assisted,
        created_at: DEMO_CREATED_AT.to_string(),
        updated_at: DEMO_CREATED_AT.to_string(),
    })?;
    store.tasks().move_to(
        "task_ready",
        TaskColumn::Ready,
        "seed_demo",
        DEMO_UPDATED_AT,
    )?;
    store.tasks().create(NewTaskRecord {
        id: "task_running".to_string(),
        project_id: "proj_alpha".to_string(),
        display_key: "TASK-RUNNING".to_string(),
        title: "Running automation".to_string(),
        description: "Second project task for filter coverage".to_string(),
        priority: "p0".to_string(),
        automation_mode: TaskAutomationMode::AutoEligible,
        created_at: DEMO_CREATED_AT.to_string(),
        updated_at: DEMO_CREATED_AT.to_string(),
    })?;
    store.tasks().move_to(
        "task_running",
        TaskColumn::Running,
        "seed_demo",
        DEMO_UPDATED_AT,
    )?;

    Ok(())
}
