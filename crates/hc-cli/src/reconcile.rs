use crate::client::ControlClient;
use crate::output::extract_data;

pub fn run(client: &ControlClient, args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        Some("now") => {
            let project_id = value_after(args, "--project");
            let body = project_id
                .map(|project_id| format!(r#"{{"project_id":"{project_id}"}}"#))
                .unwrap_or_else(|| "{}".to_string());
            let json = client.post_json("/v1/reconcile", Some(&body))?;
            if args.iter().any(|arg| arg == "--json") {
                return Ok(json);
            }
            let data = extract_data(&json)?;
            if let Some(project_id) = data["project_id"].as_str() {
                Ok(format!("Reconcile requested for project {project_id}."))
            } else {
                Ok("Reconcile requested.".to_string())
            }
        }
        _ => Err("unsupported reconcile command".to_string()),
    }
}

fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
