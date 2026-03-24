use crate::client::ControlClient;
use crate::output::extract_data;

pub fn run(client: &ControlClient, args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        Some("create") => {
            let project = value_after(args, "--project").ok_or_else(|| "missing --project".to_string())?;
            let title = value_after(args, "--title").ok_or_else(|| "missing --title".to_string())?;
            let priority = value_after(args, "--priority").unwrap_or("normal");
            let json = client.post_json(
                "/v1/tasks",
                Some(&format!(r#"{{"project_id":"{project}","title":"{title}","priority":"{priority}"}}"#)),
            )?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            let data = extract_data(&json)?;
            Ok(format!(
                "Created task {} with priority {}.",
                data["id"].as_str().unwrap_or(""),
                data["priority"].as_str().unwrap_or(priority)
            ))
        }
        Some("move") => {
            let task_id = args.get(1).ok_or_else(|| "missing task id".to_string())?;
            let column = value_after(args, "--column").ok_or_else(|| "missing --column".to_string())?;
            let json = client.post_json(&format!("/v1/tasks/{task_id}/move"), Some(&format!(r#"{{"column":"{column}"}}"#)))?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            Ok(format!("Moved task {task_id} to {column}."))
        }
        Some("assign") => {
            let task_id = args.get(1).ok_or_else(|| "missing task id".to_string())?;
            let session_id = value_after(args, "--session").ok_or_else(|| "missing --session".to_string())?;
            let json = client.post_json(
                &format!("/v1/sessions/{session_id}/attach-task"),
                Some(&format!(r#"{{"task_id":"{task_id}"}}"#)),
            )?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            Ok(format!("Assigned task {task_id} to session {session_id}."))
        }
        Some("automation-mode") => {
            let task_id = args.get(1).ok_or_else(|| "missing task id".to_string())?;
            let mode = value_after(args, "--mode").ok_or_else(|| "missing --mode".to_string())?;
            let json = client.post_json(
                &format!("/v1/tasks/{task_id}/automation-mode"),
                Some(&format!(r#"{{"mode":"{mode}"}}"#)),
            )?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            Ok(format!("Automation mode for task {task_id} set to {mode}."))
        }
        _ => Err("unsupported task command".to_string()),
    }
}

fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
