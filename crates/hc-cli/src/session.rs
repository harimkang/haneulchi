use crate::client::ControlClient;
use crate::output::extract_data;

pub fn run(client: &ControlClient, args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        Some("list") => {
            let mut path = "/v1/sessions".to_string();
            let mut query = Vec::new();
            if let Some(project_id) = value_after(args, "--project") {
                query.push(format!("project_id={project_id}"));
            }
            if let Some(state) = value_after(args, "--state") {
                query.push(format!("state={state}"));
            }
            if let Some(mode) = value_after(args, "--mode") {
                query.push(format!("mode={mode}"));
            }
            if let Some(task_id) = value_after(args, "--task") {
                query.push(format!("task_id={task_id}"));
            }
            if args.iter().any(|arg| arg == "--dispatchable") {
                query.push("dispatchable=true".to_string());
            }
            if !query.is_empty() {
                path.push('?');
                path.push_str(&query.join("&"));
            }
            let json = client.get_json(&path)?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            let data = extract_data(&json)?;
            let sessions = data.as_array().cloned().unwrap_or_default();
            let mut lines =
                vec!["SESSION PROJECT MODE STATE TASK BRANCH UNREAD SUMMARY".to_string()];
            for session in sessions {
                lines.push(format!(
                    "{} {} {} {} {} {} {} {}",
                    session["session_id"].as_str().unwrap_or(""),
                    session["project_id"].as_str().unwrap_or(""),
                    session["mode"].as_str().unwrap_or(""),
                    session["runtime_state"].as_str().unwrap_or(""),
                    session["task_id"].as_str().unwrap_or("-"),
                    session["branch"].as_str().unwrap_or("-"),
                    session["unread_count"].as_u64().unwrap_or_default(),
                    session["latest_summary"].as_str().unwrap_or("")
                ));
            }
            Ok(lines.join("\n"))
        }
        Some("get") => {
            let session_id = args
                .get(1)
                .ok_or_else(|| "missing session id".to_string())?;
            let json = client.get_json(&format!("/v1/sessions/{session_id}"))?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            let data = extract_data(&json)?;
            Ok(format!(
                "{} {}",
                data["session_id"].as_str().unwrap_or(""),
                data["title"].as_str().unwrap_or("")
            ))
        }
        Some("focus") => {
            let session_id = args
                .get(1)
                .ok_or_else(|| "missing session id".to_string())?;
            let json = client.post_json(
                &format!("/v1/sessions/{session_id}/focus"),
                Some(r#"{"activate_app":true}"#),
            )?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            Ok(format!("Focus requested for session {session_id}."))
        }
        Some("takeover") => {
            let session_id = args
                .get(1)
                .ok_or_else(|| "missing session id".to_string())?;
            let json = client.post_json(
                &format!("/v1/sessions/{session_id}/takeover"),
                Some(r#"{"reason":"manual review"}"#),
            )?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            Ok(format!("Takeover enabled for session {session_id}."))
        }
        Some("release-takeover") => {
            let session_id = args
                .get(1)
                .ok_or_else(|| "missing session id".to_string())?;
            let json = client.post_json(
                &format!("/v1/sessions/{session_id}/release-takeover"),
                Some(r#"{"resume_mode":"normal"}"#),
            )?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            Ok(format!("Takeover released for session {session_id}."))
        }
        Some("attach-task") => {
            let session_id = args
                .get(1)
                .ok_or_else(|| "missing session id".to_string())?;
            let task_id =
                value_after(args, "--task").ok_or_else(|| "missing --task".to_string())?;
            let json = client.post_json(
                &format!("/v1/sessions/{session_id}/attach-task"),
                Some(&format!(r#"{{"task_id":"{task_id}"}}"#)),
            )?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            Ok(format!("Attached task {task_id} to session {session_id}."))
        }
        Some("detach-task") => {
            let session_id = args
                .get(1)
                .ok_or_else(|| "missing session id".to_string())?;
            let json = client.post_json(
                &format!("/v1/sessions/{session_id}/detach-task"),
                Some("{}"),
            )?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            Ok(format!("Detached task from session {session_id}."))
        }
        _ => Err("unsupported session command".to_string()),
    }
}

fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
