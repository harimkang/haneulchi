use hc_storage::SqliteStore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskTimelineEntry {
    pub id: String,
    pub kind: String,
    pub actor: String,
    pub summary: String,
    pub warning_reason: Option<String>,
    pub created_at: String,
}

pub fn project_task_timeline(
    store: &SqliteStore,
    task_id: &str,
) -> Result<Vec<TaskTimelineEntry>, hc_storage::StorageError> {
    let events = store.timeline().list_for_task(task_id)?;
    let mut timeline = Vec::new();

    for event in events {
        let broken_link = event
            .review_item_id
            .as_deref()
            .map(|review_id| !review_item_exists(store, review_id))
            .unwrap_or(false);
        let warning_reason = if broken_link {
            Some("broken_link".to_string())
        } else {
            None
        };

        timeline.push(TaskTimelineEntry {
            id: event.id,
            kind: if warning_reason.is_some() {
                "warning".to_string()
            } else {
                event.kind.as_str().to_string()
            },
            actor: event.actor,
            summary: summarize_event(&event.kind.as_str().to_string(), event.reason_code.as_deref(), event.payload_json.as_deref()),
            warning_reason,
            created_at: event.created_at,
        });
    }

    Ok(timeline)
}

fn review_item_exists(store: &SqliteStore, review_id: &str) -> bool {
    let connection = store.connection();
    let mut statement = match connection.prepare("SELECT 1 FROM review_items WHERE id = ?1 LIMIT 1") {
        Ok(statement) => statement,
        Err(_) => return false,
    };

    statement
        .query_row(rusqlite::params![review_id], |_| Ok(()))
        .is_ok()
}

fn summarize_event(kind: &str, reason_code: Option<&str>, payload_json: Option<&str>) -> String {
    match kind {
        "task_created" => "Task created".to_string(),
        "task_moved" => payload_json
            .and_then(extract_column)
            .map(|column| format!("Moved to {column}"))
            .unwrap_or_else(|| "Task moved".to_string()),
        "task_attached" => "Session attached".to_string(),
        "task_detached" => "Session detached".to_string(),
        "worktree_created" => "Dedicated worktree provisioned".to_string(),
        "review_ready" => "Review ready".to_string(),
        "review_decided" => reason_code
            .map(|reason| format!("Review decision: {reason}"))
            .unwrap_or_else(|| "Review decided".to_string()),
        "follow_up_created" => "Follow-up task created".to_string(),
        other => reason_code
            .map(|reason| format!("{other}: {reason}"))
            .unwrap_or_else(|| other.replace('_', " ")),
    }
}

fn extract_column(payload_json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(payload_json)
        .ok()?
        .get("column")?
        .as_str()
        .map(|value| value.to_string())
}
