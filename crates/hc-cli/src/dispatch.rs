use crate::client::ControlClient;
use crate::output::extract_data;

pub fn run(client: &ControlClient, args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        Some("send") => {
            let task_id = value_after(args, "--task");
            let target = value_after(args, "--target").ok_or_else(|| "missing --target".to_string())?;
            let message = value_after(args, "--message").ok_or_else(|| "missing --message".to_string())?;
            let body = format!(
                r#"{{"target_session_id":"{target}","task_id":{},"target_live":true,"payload":"{message}"}}"#,
                task_id
                    .map(|value| format!(r#""{value}""#))
                    .unwrap_or_else(|| "null".to_string())
            );
            let json = client.post_json("/v1/dispatch", Some(&body))?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            let data = extract_data(&json)?;
            let state = data["events"]
                .as_array()
                .and_then(|events| events.last())
                .and_then(|event| event["state"].as_str())
                .unwrap_or("queued");
            Ok(format!("Dispatch {state} for session {target}."))
        }
        _ => Err("unsupported dispatch command".to_string()),
    }
}

fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
