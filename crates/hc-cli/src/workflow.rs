use crate::client::ControlClient;

pub fn run(client: &ControlClient, args: &[String]) -> Result<String, String> {
    let project = value_after(args, "--project").unwrap_or(".");
    match args.first().map(String::as_str) {
        Some("validate") => {
            let json = client.post_json(
                "/v1/workflow/validate",
                Some(&format!(r#"{{"project_root":"{project}"}}"#)),
            )?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            let data: serde_json::Value =
                serde_json::from_str(&json).map_err(|error| error.to_string())?;
            Ok(format!(
                "Workflow state: {}",
                data["data"]["state"].as_str().unwrap_or("unknown")
            ))
        }
        Some("reload") => {
            let json = client.post_json(
                "/v1/workflow/reload",
                Some(&format!(r#"{{"project_root":"{project}"}}"#)),
            )?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            let data: serde_json::Value =
                serde_json::from_str(&json).map_err(|error| error.to_string())?;
            Ok(format!(
                "Workflow state: {}",
                data["data"]["state"].as_str().unwrap_or("unknown")
            ))
        }
        _ => Err("unsupported workflow command".to_string()),
    }
}

fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
