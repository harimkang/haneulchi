pub fn run(args: &[String]) -> Result<String, String> {
    let project = value_after(args, "--project").unwrap_or(".");
    match args.first().map(String::as_str) {
        Some("validate") => hc_ffi::workflow_validate_json(project),
        Some("reload") => hc_ffi::workflow_reload_json(project),
        _ => Err("unsupported workflow command".to_string()),
    }
}

fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
