use crate::client::ControlClient;
use crate::output::extract_data;

pub fn run(client: &ControlClient, args: &[String]) -> Result<String, String> {
    let mut path = "/v1/state".to_string();
    let mut query = Vec::new();
    if args.iter().any(|arg| arg == "--compact") {
        query.push("view=compact".to_string());
    }
    if let Some(project_id) = value_after(args, "--project") {
        query.push(format!("project_id={project_id}"));
    }
    if !query.is_empty() {
        path.push('?');
        path.push_str(&query.join("&"));
    }
    let json = client.get_json(&path)?;
    if args.iter().any(|arg| arg == "--json") {
        return Ok(json);
    }

    let envelope: serde_json::Value =
        serde_json::from_str(&json).map_err(|error| error.to_string())?;
    let value = extract_data(&json)?;
    let snapshot_rev = envelope["meta"]["snapshot_rev"].as_u64().unwrap_or_default();
    let running_slots = value["ops"]["automation"]["running_slots"]
        .as_u64()
        .unwrap_or_default();
    let max_slots = value["ops"]["automation"]["max_slots"]
        .as_u64()
        .unwrap_or_default();
    let project_count = value["projects"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or_default();
    let session_count = value["sessions"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or_default();

    Ok(format!(
        "snapshot_rev={snapshot_rev} projects={project_count} sessions={session_count} slots={running_slots}/{max_slots}"
    ))
}

fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
