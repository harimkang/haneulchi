pub fn run(args: &[String]) -> Result<String, String> {
    let json = hc_ffi::state_snapshot_json()?;
    if args.iter().any(|arg| arg == "--json") {
        return Ok(json);
    }

    let value: serde_json::Value = serde_json::from_str(&json).map_err(|error| error.to_string())?;
    let snapshot_rev = value["meta"]["snapshot_rev"].as_u64().unwrap_or_default();
    let running_slots = value["ops"]["running_slots"].as_u64().unwrap_or_default();
    let max_slots = value["ops"]["max_slots"].as_u64().unwrap_or_default();
    let project_count = value["projects"].as_array().map(|items| items.len()).unwrap_or_default();
    let session_count = value["sessions"].as_array().map(|items| items.len()).unwrap_or_default();

    Ok(format!(
        "snapshot_rev={snapshot_rev} projects={project_count} sessions={session_count} slots={running_slots}/{max_slots}"
    ))
}
