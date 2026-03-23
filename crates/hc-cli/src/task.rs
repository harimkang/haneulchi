pub fn run(args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        Some("create") => {
            let project = value_after(args, "--project").ok_or_else(|| "missing --project".to_string())?;
            let title = value_after(args, "--title").ok_or_else(|| "missing --title".to_string())?;
            hc_api::task_create_json(project, title)
        }
        Some("move") => {
            let task_id = args.get(1).ok_or_else(|| "missing task id".to_string())?;
            let column = value_after(args, "--column").ok_or_else(|| "missing --column".to_string())?;
            hc_api::task_move_json(task_id, column)
        }
        Some("assign") => {
            let task_id = args.get(1).ok_or_else(|| "missing task id".to_string())?;
            let session_id = value_after(args, "--session").ok_or_else(|| "missing --session".to_string())?;
            hc_ffi::session_attach_task_json(session_id, task_id)
        }
        Some("automation-mode") => {
            let task_id = args.get(1).ok_or_else(|| "missing task id".to_string())?;
            let mode = value_after(args, "--mode").ok_or_else(|| "missing --mode".to_string())?;
            hc_api::task_automation_mode_json(task_id, mode)
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
