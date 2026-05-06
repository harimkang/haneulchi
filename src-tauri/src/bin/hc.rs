use std::{
    env,
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    process,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliCommand {
    Get { path: String },
    Post { path: String, body: String },
    Patch { path: String, body: String },
}

fn main() {
    match run(env::args()) {
        Ok(output) => {
            println!("{output}");
        }
        Err(error) => {
            eprintln!("hc: {error}");
            process::exit(1);
        }
    }
}

fn run<I, S>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    match parse_command(args)? {
        CliCommand::Get { path } => request_json(&path),
        CliCommand::Post { path, body } => request_json_with_body(&path, &body),
        CliCommand::Patch { path, body } => request_json_with_patch(&path, &body),
    }
}

fn parse_command<I, S>(args: I) -> Result<CliCommand, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_string())
        .collect::<Vec<_>>();
    let command = args.get(1).map(String::as_str);
    let json = args.iter().any(|arg| arg == "--json");
    if !json {
        return Err("only JSON output is available in the MVP CLI; pass --json".to_string());
    }

    match command {
        Some("state") => Ok(CliCommand::Get {
            path: project_scoped_path(&args, "/v1/state"),
        }),
        Some("health") => Ok(CliCommand::Get {
            path: "/v1/health".to_string(),
        }),
        Some("project") => parse_project_command(&args),
        Some("dispatch") => parse_dispatch_command(&args),
        Some("run") => parse_run_command(&args),
        Some("session") => parse_session_command(&args),
        Some("evidence") => parse_evidence_command(&args),
        Some("policy") => parse_policy_command(&args),
        Some("agent") => parse_agent_command(&args),
        Some("provider-model") => parse_provider_model_command(&args),
        Some("terminal-theme") => parse_terminal_theme_command(&args),
        Some("budget") => parse_budget_command(&args),
        Some("secret") => parse_secret_command(&args),
        Some("knowledge") => parse_knowledge_command(&args),
        Some("context") => parse_context_command(&args),
        Some("workflow") => parse_workflow_command(&args),
        Some("task") => parse_task_command(&args),
        Some("initiative") => parse_initiative_command(&args),
        Some("review") => parse_review_command(&args),
        Some("release-gate") => parse_release_gate_command(&args),
        Some("terminal") => parse_terminal_command(&args),
        Some("distribution") => parse_distribution_command(&args),
        Some("recovery") => parse_recovery_command(&args),
        Some("benchmark") => parse_benchmark_command(&args),
        Some("dogfood") => parse_dogfood_command(&args),
        Some("visual") => parse_visual_command(&args),
        Some("tracker") => parse_tracker_command(&args),
        Some("browser") => parse_browser_command(&args),
        Some("block") => parse_block_command(&args),
        Some("update") => parse_update_command(&args),
        _ => Err(root_usage()),
    }
}

fn root_usage() -> String {
    "usage: hc state --json | hc health --json | hc project ... --json | hc dispatch ... --json | hc run ... --json | hc session ... --json | hc evidence ... --json | hc policy ... --json | hc agent ... --json | hc provider-model ... --json | hc terminal-theme ... --json | hc budget ... --json | hc secret ... --json | hc knowledge ... --json | hc context ... --json | hc workflow ... --json | hc task ... --json | hc initiative ... --json | hc review ... --json | hc release-gate ... --json | hc terminal ... --json | hc distribution ... --json | hc recovery ... --json | hc benchmark ... --json | hc dogfood ... --json | hc visual ... --json | hc tracker ... --json | hc browser run --json | hc block ... --json | hc update check --json".to_string()
}

fn parse_update_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("check") => {
            let channel = value_after(args, "--channel").unwrap_or_else(|| "stable".to_string());
            Ok(CliCommand::Get {
                path: format!("/v1/update/check?channel={}", encode_query_value(&channel)),
            })
        }
        _ => Err(update_usage()),
    }
}

fn update_usage() -> String {
    "usage: hc update check [--channel <stable|beta>] --json".to_string()
}

fn parse_project_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("list") => Ok(CliCommand::Get {
            path: "/v1/projects".to_string(),
        }),
        Some("add") => {
            let key = value_after(args, "--key").ok_or_else(project_usage)?;
            let name = value_after(args, "--name").ok_or_else(project_usage)?;
            let path_value = value_after(args, "--path").ok_or_else(project_usage)?;
            let color = value_after(args, "--color");
            Ok(CliCommand::Post {
                path: "/v1/projects".to_string(),
                body: serde_json::json!({
                    "color": color,
                    "key": key,
                    "name": name,
                    "path": path_value
                })
                .to_string(),
            })
        }
        Some("focus") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/projects/{project_id}/focus"),
                body: "{}".to_string(),
            })
        }
        Some("detach") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/projects/{project_id}/detach"),
                body: "{}".to_string(),
            })
        }
        Some("tab-group") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let group_name = value_after(args, "--name").ok_or_else(project_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/projects/{project_id}/tab-group"),
                body: serde_json::json!({
                    "groupName": group_name
                })
                .to_string(),
            })
        }
        Some("files") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let path = value_after(args, "--path")
                .map(|value| {
                    format!(
                        "/v1/projects/{project_id}/files?path={}",
                        encode_query_value(&value)
                    )
                })
                .unwrap_or_else(|| format!("/v1/projects/{project_id}/files"));
            Ok(CliCommand::Get { path })
        }
        Some("file") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let file_path = value_after(args, "--path").ok_or_else(project_usage)?;
            Ok(CliCommand::Get {
                path: format!(
                    "/v1/projects/{project_id}/file?path={}",
                    encode_query_value(&file_path)
                ),
            })
        }
        Some("write-file") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let file_path = value_after(args, "--path").ok_or_else(project_usage)?;
            let body = value_after(args, "--body").ok_or_else(project_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/projects/{project_id}/file"),
                body: serde_json::json!({
                    "body": body,
                    "path": file_path
                })
                .to_string(),
            })
        }
        Some("diff") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let path = value_after(args, "--path")
                .map(|value| {
                    format!(
                        "/v1/projects/{project_id}/diff?path={}",
                        encode_query_value(&value)
                    )
                })
                .unwrap_or_else(|| format!("/v1/projects/{project_id}/diff"));
            Ok(CliCommand::Get { path })
        }
        Some("lsp") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let path = value_after(args, "--path")
                .map(|value| {
                    format!(
                        "/v1/projects/{project_id}/lsp-diagnostics?path={}",
                        encode_query_value(&value)
                    )
                })
                .unwrap_or_else(|| format!("/v1/projects/{project_id}/lsp-diagnostics"));
            Ok(CliCommand::Get { path })
        }
        Some("patch-export") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let path = value_after(args, "--path")
                .map(|value| {
                    format!(
                        "/v1/projects/{project_id}/patch/export?path={}",
                        encode_query_value(&value)
                    )
                })
                .unwrap_or_else(|| format!("/v1/projects/{project_id}/patch/export"));
            Ok(CliCommand::Get { path })
        }
        Some("patch-import") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let body = value_after(args, "--body").ok_or_else(project_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/projects/{project_id}/patch/import"),
                body: serde_json::json!({ "body": body }).to_string(),
            })
        }
        Some("pr-plan") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let title = value_after(args, "--title").ok_or_else(project_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/projects/{project_id}/pr/landing-plan"),
                body: serde_json::json!({
                    "draft": args.iter().any(|arg| arg == "--draft"),
                    "title": title
                })
                .to_string(),
            })
        }
        Some("search-files") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let query = value_after(args, "--query").ok_or_else(project_usage)?;
            Ok(CliCommand::Get {
                path: format!(
                    "/v1/projects/{project_id}/files/search?query={}",
                    encode_query_value(&query)
                ),
            })
        }
        Some("layout") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let preset = value_after(args, "--preset").ok_or_else(project_usage)?;
            if preset != "grid" && preset != "maximized" {
                return Err(project_usage());
            }
            let focused_session =
                value_after(args, "--focused-session").ok_or_else(project_usage)?;
            let maximized_session_id = if preset == "maximized" {
                serde_json::Value::String(focused_session.clone())
            } else {
                serde_json::Value::Null
            };
            Ok(CliCommand::Post {
                path: format!("/v1/projects/{project_id}/layout"),
                body: serde_json::json!({
                    "layoutJson": {
                        "focusedSessionId": focused_session,
                        "maximizedSessionId": maximized_session_id,
                        "mode": preset,
                        "panes": [focused_session]
                    }
                })
                .to_string(),
            })
        }
        Some("layout-presets") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            Ok(CliCommand::Get {
                path: format!("/v1/projects/{project_id}/layout-presets"),
            })
        }
        Some("layout-preset") => {
            let project_id = args.get(3).ok_or_else(project_usage)?;
            let name = value_after(args, "--name").ok_or_else(project_usage)?;
            let preset = value_after(args, "--preset").ok_or_else(project_usage)?;
            if preset != "grid" && preset != "maximized" {
                return Err(project_usage());
            }
            let focused_session =
                value_after(args, "--focused-session").ok_or_else(project_usage)?;
            let maximized_session_id = if preset == "maximized" {
                serde_json::Value::String(focused_session.clone())
            } else {
                serde_json::Value::Null
            };
            Ok(CliCommand::Post {
                path: format!("/v1/projects/{project_id}/layout-presets"),
                body: serde_json::json!({
                    "layoutJson": {
                        "focusedSessionId": focused_session,
                        "maximizedSessionId": maximized_session_id,
                        "mode": preset,
                        "panes": [focused_session]
                    },
                    "name": name
                })
                .to_string(),
            })
        }
        _ => Err(project_usage()),
    }
}

fn project_usage() -> String {
    "usage: hc project list --json | hc project add --key <key> --name <name> --path <path> [--color <hex>] --json | hc project focus <project-id> --json | hc project detach <project-id> --json | hc project tab-group <project-id> --name <group-name> --json | hc project files <project-id> [--path <relative>] --json | hc project file <project-id> --path <relative-file> --json | hc project write-file <project-id> --path <relative-file> --body <text> --json | hc project diff <project-id> [--path <relative-file>] --json | hc project lsp <project-id> [--path <relative-file>] --json | hc project patch-export <project-id> [--path <relative-file>] --json | hc project patch-import <project-id> --body <diff> --json | hc project pr-plan <project-id> --title <title> [--draft] --json | hc project search-files <project-id> --query <query> --json | hc project layout <project-id> --preset <grid|maximized> --focused-session <session-id> --json | hc project layout-presets <project-id> --json | hc project layout-preset <project-id> --name <preset-name> --preset <grid|maximized> --focused-session <session-id> --json".to_string()
}

fn parse_workflow_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("status") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/workflow/status"),
        }),
        Some("reload") => {
            let path = value_after(args, "--path").ok_or_else(workflow_usage)?;
            Ok(CliCommand::Post {
                path: "/v1/workflow/reload".to_string(),
                body: serde_json::json!({
                    "projectId": selected_project_id(args),
                    "sourcePath": path
                })
                .to_string(),
            })
        }
        Some("validate") => {
            let path = value_after(args, "--path").ok_or_else(workflow_usage)?;
            Ok(CliCommand::Post {
                path: "/v1/workflow/validate".to_string(),
                body: serde_json::json!({
                    "projectId": selected_project_id(args),
                    "sourcePath": path
                })
                .to_string(),
            })
        }
        Some("negative-tests") => Ok(CliCommand::Post {
            path: "/v1/workflow/negative-tests/run".to_string(),
            body: serde_json::json!({
                "projectId": selected_project_id(args)
            })
            .to_string(),
        }),
        Some("negative-test-runs") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/workflow/negative-tests/runs"),
        }),
        _ => Err(workflow_usage()),
    }
}

fn workflow_usage() -> String {
    "usage: hc workflow status [--project <project-id>] --json | hc workflow validate --path <WORKFLOW.md> [--project <project-id>] --json | hc workflow reload --path <WORKFLOW.md> [--project <project-id>] --json | hc workflow negative-tests [--project <project-id>] --json | hc workflow negative-test-runs [--project <project-id>] --json".to_string()
}

fn parse_dispatch_command(args: &[String]) -> Result<CliCommand, String> {
    let usage = || {
        "usage: hc dispatch <task-id> --agent <agent> [--context <context>] [--workspace <path>] --json"
            .to_string()
    };
    let task_id = args.get(2).ok_or_else(|| usage())?;
    let agent_profile_id = value_after(args, "--agent").ok_or_else(|| usage())?;
    let context_pack_id = value_after(args, "--context");
    let workspace_path = value_after(args, "--workspace");
    let mut body = serde_json::json!({
        "taskId": task_id,
        "agentProfileId": agent_profile_id,
        "contextPackId": context_pack_id
    });
    if let Some(workspace_path) = workspace_path {
        body["workspacePath"] = serde_json::json!(workspace_path);
    }
    Ok(CliCommand::Post {
        path: "/v1/dispatch".to_string(),
        body: body.to_string(),
    })
}

fn parse_run_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("list") => {
            let mut params = Vec::new();
            if let Some(project_id) = value_after(args, "--project") {
                params.push(format!("projectId={}", encode_query_value(&project_id)));
            }
            if let Some(lifecycle) = value_after(args, "--lifecycle") {
                params.push(format!("lifecycle={}", encode_query_value(&lifecycle)));
            }
            if let Some(task_id) = value_after(args, "--task") {
                params.push(format!("taskId={}", encode_query_value(&task_id)));
            }
            if let Some(agent_profile_id) = value_after(args, "--agent") {
                params.push(format!(
                    "agentProfileId={}",
                    encode_query_value(&agent_profile_id)
                ));
            }
            let path = if params.is_empty() {
                "/v1/runs".to_string()
            } else {
                format!("/v1/runs?{}", params.join("&"))
            };
            Ok(CliCommand::Get { path })
        }
        Some("open") => {
            let run_id = args.get(3).ok_or_else(run_usage)?;
            Ok(CliCommand::Get {
                path: format!("/v1/runs/{run_id}"),
            })
        }
        Some("replay") => {
            let run_id = args.get(3).ok_or_else(run_usage)?;
            Ok(CliCommand::Get {
                path: format!("/v1/runs/{run_id}/replay"),
            })
        }
        Some("usage") => {
            let run_id = args.get(3).ok_or_else(run_usage)?;
            Ok(CliCommand::Get {
                path: format!("/v1/runs/{run_id}/token-usage"),
            })
        }
        Some("transition") => {
            let run_id = args.get(3).ok_or_else(run_usage)?;
            let lifecycle = value_after(args, "--lifecycle").ok_or_else(run_usage)?;
            let status_detail = value_after(args, "--detail");
            Ok(CliCommand::Post {
                path: format!("/v1/runs/{run_id}/transition"),
                body: serde_json::json!({
                    "lifecycle": lifecycle,
                    "statusDetail": status_detail
                })
                .to_string(),
            })
        }
        Some("cancel") => {
            let run_id = args.get(3).ok_or_else(run_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/runs/{run_id}/cancel"),
                body: "{}".to_string(),
            })
        }
        Some("retry") => {
            let run_id = args.get(3).ok_or_else(run_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/runs/{run_id}/retry"),
                body: "{}".to_string(),
            })
        }
        Some("status") => {
            let run_id = args.get(3).ok_or_else(run_usage)?;
            let body_md = value_after(args, "--body").ok_or_else(run_usage)?;
            let lifecycle = value_after(args, "--lifecycle");
            let status_detail = value_after(args, "--detail");
            Ok(CliCommand::Post {
                path: format!("/v1/runs/{run_id}/status-updates"),
                body: serde_json::json!({
                    "bodyMd": body_md,
                    "lifecycle": lifecycle,
                    "statusDetail": status_detail
                })
                .to_string(),
            })
        }
        Some("hook") => {
            let run_id = args.get(3).ok_or_else(run_usage)?;
            let hook_name = value_after(args, "--hook").ok_or_else(run_usage)?;
            let repo_root = value_after(args, "--repo-root").ok_or_else(run_usage)?;
            let workspace_path = value_after(args, "--workspace");
            Ok(CliCommand::Post {
                path: format!("/v1/runs/{run_id}/hooks/{hook_name}/run"),
                body: serde_json::json!({
                    "repoRoot": repo_root,
                    "workspacePath": workspace_path
                })
                .to_string(),
            })
        }
        _ => Err(run_usage()),
    }
}

fn run_usage() -> String {
    "usage: hc run list [--project <project-id>] [--lifecycle <lifecycle>] [--task <task-id>] [--agent <agent-id>] --json | hc run open <run-id> --json | hc run replay <run-id> --json | hc run usage <run-id> --json | hc run transition <run-id> --lifecycle <lifecycle> [--detail <text>] --json | hc run cancel <run-id> --json | hc run retry <run-id> --json | hc run status <run-id> --body <body> [--lifecycle <lifecycle>] [--detail <text>] --json | hc run hook <run-id> --hook <hook> --repo-root <path> [--workspace <path>] --json".to_string()
}

fn parse_session_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("list") => {
            let mut params = Vec::new();
            if let Some(project_id) = value_after(args, "--project") {
                params.push(format!("projectId={}", encode_query_value(&project_id)));
            }
            if let Some(state) = value_after(args, "--state") {
                params.push(format!("state={}", encode_query_value(&state)));
            }
            if let Some(task_id) = value_after(args, "--task") {
                params.push(format!("taskId={}", encode_query_value(&task_id)));
            }
            let path = if params.is_empty() {
                "/v1/sessions".to_string()
            } else {
                format!("/v1/sessions?{}", params.join("&"))
            };
            Ok(CliCommand::Get { path })
        }
        Some("new") => {
            let title = value_after(args, "--title").ok_or_else(session_usage)?;
            let mode = value_after(args, "--mode").unwrap_or_else(|| "shell".to_string());
            let cwd = value_after(args, "--cwd");
            let ssh_target = value_after(args, "--ssh");
            let branch = value_after(args, "--branch");
            let agent_profile_id = value_after(args, "--agent");
            let session_cwd = if mode == "ssh" {
                ssh_target
                    .map(|target| {
                        let remote_path = cwd
                            .as_deref()
                            .filter(|path| !path.trim().is_empty())
                            .unwrap_or("~");
                        format!("ssh://{}{}", target.trim(), remote_path)
                    })
                    .or(cwd)
            } else {
                cwd
            };
            Ok(CliCommand::Post {
                path: "/v1/sessions".to_string(),
                body: serde_json::json!({
                    "agentProfileId": agent_profile_id,
                    "branch": branch,
                    "cwd": session_cwd,
                    "mode": mode,
                    "projectId": selected_project_id(args),
                    "title": title
                })
                .to_string(),
            })
        }
        Some("focus") => {
            let session_id = args.get(3).ok_or_else(session_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/sessions/{session_id}/focus"),
                body: "{}".to_string(),
            })
        }
        Some("attention") => {
            let session_id = args.get(3).ok_or_else(session_usage)?;
            let attention_state = value_after(args, "--state").ok_or_else(session_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/sessions/{session_id}/attention"),
                body: serde_json::json!({
                    "attentionState": attention_state
                })
                .to_string(),
            })
        }
        Some("resize") => {
            let session_id = args.get(3).ok_or_else(session_usage)?;
            let cols = value_after(args, "--cols")
                .ok_or_else(session_usage)?
                .parse::<u16>()
                .map_err(|_| session_usage())?;
            let rows = value_after(args, "--rows")
                .ok_or_else(session_usage)?
                .parse::<u16>()
                .map_err(|_| session_usage())?;
            Ok(CliCommand::Post {
                path: format!("/v1/sessions/{session_id}/resize"),
                body: serde_json::json!({
                    "cols": cols,
                    "rows": rows
                })
                .to_string(),
            })
        }
        Some("usage") => {
            let session_id = args.get(3).ok_or_else(session_usage)?;
            Ok(CliCommand::Get {
                path: format!("/v1/sessions/{session_id}/token-usage"),
            })
        }
        Some("input") => {
            let session_id = args.get(3).ok_or_else(session_usage)?;
            let text = value_after(args, "--text").ok_or_else(session_usage)?;
            let allow_dangerous = args.iter().any(|arg| arg == "--allow-dangerous");
            Ok(CliCommand::Post {
                path: format!("/v1/sessions/{session_id}/input"),
                body: serde_json::json!({
                    "allowDangerous": allow_dangerous,
                    "text": text
                })
                .to_string(),
            })
        }
        Some("stream") => parse_session_stream_command(args),
        Some("attach-task") => {
            let session_id = args.get(3).ok_or_else(session_usage)?;
            let task_id = value_after(args, "--task").ok_or_else(session_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/sessions/{session_id}/attach-task"),
                body: serde_json::json!({
                    "taskId": task_id
                })
                .to_string(),
            })
        }
        Some("detach-task") => {
            let session_id = args.get(3).ok_or_else(session_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/sessions/{session_id}/detach-task"),
                body: "{}".to_string(),
            })
        }
        Some("takeover") => {
            let session_id = args.get(3).ok_or_else(session_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/sessions/{session_id}/takeover"),
                body: "{}".to_string(),
            })
        }
        Some("release") => {
            let session_id = args.get(3).ok_or_else(session_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/sessions/{session_id}/release"),
                body: "{}".to_string(),
            })
        }
        Some("kill") => {
            let session_id = args.get(3).ok_or_else(session_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/sessions/{session_id}/kill"),
                body: "{}".to_string(),
            })
        }
        _ => Err(session_usage()),
    }
}

fn parse_session_stream_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(3).map(String::as_str) {
        Some("list") => {
            let session_id = args.get(4).ok_or_else(session_usage)?;
            let path = match value_after(args, "--limit") {
                Some(limit) => {
                    limit.parse::<usize>().map_err(|_| session_usage())?;
                    format!(
                        "/v1/sessions/{session_id}/stream-chunks?limit={}",
                        encode_query_value(&limit)
                    )
                }
                None => format!("/v1/sessions/{session_id}/stream-chunks"),
            };
            Ok(CliCommand::Get { path })
        }
        Some("record") => {
            let session_id = args.get(4).ok_or_else(session_usage)?;
            let seq_start = value_after(args, "--seq-start")
                .ok_or_else(session_usage)?
                .parse::<i64>()
                .map_err(|_| session_usage())?;
            let seq_end = value_after(args, "--seq-end")
                .ok_or_else(session_usage)?
                .parse::<i64>()
                .map_err(|_| session_usage())?;
            let body = value_after(args, "--body").ok_or_else(session_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/sessions/{session_id}/stream-chunks"),
                body: serde_json::json!({
                    "body": body,
                    "seqEnd": seq_end,
                    "seqStart": seq_start
                })
                .to_string(),
            })
        }
        _ => Err(session_usage()),
    }
}

fn session_usage() -> String {
    "usage: hc session list [--project <project-id>] [--state <state>] [--task <task-id>] --json | hc session new --title <title> [--project <project-id>] [--mode <shell|agent|ssh|dev_server|test|review>] [--ssh user@host] [--cwd <path>] [--branch <branch>] [--agent <agent-id>] --json | hc session focus|takeover|release|kill <session-id> --json | hc session attention <session-id> --state <none|unread> --json | hc session resize <session-id> --cols <n> --rows <n> --json | hc session usage <session-id> --json | hc session input <session-id> --text <text> [--allow-dangerous] --json | hc session stream list <session-id> [--limit <n>] --json | hc session stream record <session-id> --seq-start <n> --seq-end <n> --body <text> --json | hc session attach-task <session-id> --task <task-id> --json | hc session detach-task <session-id> --json".to_string()
}

fn parse_evidence_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("generate") => {
            let run_id = args.get(3).ok_or_else(evidence_usage)?;
            let evidence_pack_id = value_after(args, "--id");
            Ok(CliCommand::Post {
                path: format!("/v1/runs/{run_id}/evidence/generate"),
                body: serde_json::json!({
                    "evidencePackId": evidence_pack_id
                })
                .to_string(),
            })
        }
        Some("review") => {
            let evidence_id = args.get(3).ok_or_else(evidence_usage)?;
            let decision = value_after(args, "--decision").ok_or_else(evidence_usage)?;
            let reviewer_id = value_after(args, "--reviewer");
            let body_md = value_after(args, "--body");
            Ok(CliCommand::Post {
                path: format!("/v1/evidence/{evidence_id}/review-decision"),
                body: serde_json::json!({
                    "decision": decision,
                    "reviewerId": reviewer_id,
                    "bodyMd": body_md
                })
                .to_string(),
            })
        }
        _ => Err(evidence_usage()),
    }
}

fn evidence_usage() -> String {
    "usage: hc evidence generate <run-id> [--id <evidence-id>] --json | hc evidence review <evidence-id> --decision <approved|changes_requested|reopened|blocked> [--reviewer <id>] [--body <text>] --json".to_string()
}

fn parse_policy_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("approvals") => {
            let mut params = Vec::new();
            if let Some(project_id) = value_after(args, "--project") {
                params.push(format!("project={}", encode_query_value(&project_id)));
            }
            if let Some(state) = value_after(args, "--state") {
                params.push(format!("state={}", encode_query_value(&state)));
            }
            let path = if params.is_empty() {
                "/v1/policy/approvals".to_string()
            } else {
                format!("/v1/policy/approvals?{}", params.join("&"))
            };
            Ok(CliCommand::Get { path })
        }
        Some("packs") => {
            let mut params = Vec::new();
            if let Some(project_id) = value_after(args, "--project") {
                params.push(format!("project={}", encode_query_value(&project_id)));
            }
            if let Some(active) = value_after(args, "--active") {
                params.push(format!("active={}", encode_query_value(&active)));
            }
            let path = if params.is_empty() {
                "/v1/policy/packs".to_string()
            } else {
                format!("/v1/policy/packs?{}", params.join("&"))
            };
            Ok(CliCommand::Get { path })
        }
        Some("audit") => {
            let mut params = Vec::new();
            if let Some(project_id) = value_after(args, "--project") {
                params.push(format!("project={}", encode_query_value(&project_id)));
            }
            if let Some(decision) = value_after(args, "--decision") {
                params.push(format!("decision={}", encode_query_value(&decision)));
            }
            if let Some(action_kind) = value_after(args, "--action") {
                params.push(format!("actionKind={}", encode_query_value(&action_kind)));
            }
            if let Some(run_id) = value_after(args, "--run") {
                params.push(format!("run={}", encode_query_value(&run_id)));
            }
            if let Some(task_id) = value_after(args, "--task") {
                params.push(format!("task={}", encode_query_value(&task_id)));
            }
            let path = if params.is_empty() {
                "/v1/policy/audit".to_string()
            } else {
                format!("/v1/policy/audit?{}", params.join("&"))
            };
            Ok(CliCommand::Get { path })
        }
        Some("pack") if args.get(3).map(String::as_str) == Some("set") => {
            let name = value_after(args, "--name").ok_or_else(policy_usage)?;
            let sandbox_mode = value_after(args, "--sandbox").ok_or_else(policy_usage)?;
            let network = value_after(args, "--network").unwrap_or_else(|| "ask".to_string());
            let network_profile =
                value_after(args, "--network-profile").unwrap_or_else(|| "internet".to_string());
            let file_write = value_after(args, "--file-write").unwrap_or_else(|| "ask".to_string());
            let approval_required = split_csv_flag(args, "--approvals");
            let forbidden_operations = split_csv_flag(args, "--forbidden");
            Ok(CliCommand::Post {
                path: "/v1/policy/packs".to_string(),
                body: serde_json::json!({
                    "approvalRequired": approval_required,
                    "fileWrite": file_write,
                    "forbiddenOperations": forbidden_operations,
                    "name": name,
                    "network": network,
                    "networkProfile": network_profile,
                    "projectId": selected_project_id(args),
                    "sandboxMode": sandbox_mode,
                    "setActive": true
                })
                .to_string(),
            })
        }
        Some("request") => {
            let run_id = value_after(args, "--run");
            let task_id = value_after(args, "--task");
            let action_kind = value_after(args, "--action").ok_or_else(policy_usage)?;
            let command = value_after(args, "--command");
            let risk_level = value_after(args, "--risk").ok_or_else(policy_usage)?;
            Ok(CliCommand::Post {
                path: "/v1/policy/approvals".to_string(),
                body: serde_json::json!({
                    "actionKind": action_kind,
                    "command": command,
                    "projectId": selected_project_id(args),
                    "requestedBy": "hc",
                    "riskLevel": risk_level,
                    "runId": run_id,
                    "taskId": task_id
                })
                .to_string(),
            })
        }
        Some("evaluate") => {
            let run_id = value_after(args, "--run");
            let task_id = value_after(args, "--task");
            let action_kind = value_after(args, "--action").ok_or_else(policy_usage)?;
            let command = value_after(args, "--command");
            Ok(CliCommand::Post {
                path: "/v1/policy/evaluate".to_string(),
                body: serde_json::json!({
                    "actionKind": action_kind,
                    "command": command,
                    "projectId": selected_project_id(args),
                    "requestedBy": "hc",
                    "runId": run_id,
                    "taskId": task_id
                })
                .to_string(),
            })
        }
        Some("decide") => {
            let approval_id = args.get(3).ok_or_else(policy_usage)?;
            let decision = value_after(args, "--decision").ok_or_else(policy_usage)?;
            let decision_note = value_after(args, "--note");
            Ok(CliCommand::Post {
                path: format!("/v1/policy/approvals/{approval_id}/decision"),
                body: serde_json::json!({
                    "decision": decision,
                    "decisionBy": "human",
                    "decisionNote": decision_note
                })
                .to_string(),
            })
        }
        _ => Err(policy_usage()),
    }
}

fn policy_usage() -> String {
    "usage: hc policy approvals [--project <project-id>] [--state <pending|approved|denied|expired>] --json | hc policy packs [--project <project-id>] [--active <true|false>] --json | hc policy audit [--project <project-id>] [--decision <allowed|approval_required|forbidden>] [--action <kind>] [--run <run-id>] [--task <task-id>] --json | hc policy pack set [--project <project-id>] --name <name> --sandbox <normal|ask-before-write|sandboxed> [--network <allowed|ask|blocked>] [--network-profile <internet|local-only|offline>] [--file-write <allowed|ask|blocked>] [--approvals <csv>] [--forbidden <csv>] --json | hc policy request [--project <project-id>] --action <kind> --risk <level> [--run <run-id>] [--task <task-id>] [--command <command>] --json | hc policy evaluate [--project <project-id>] --action <kind> [--run <run-id>] [--task <task-id>] [--command <command>] --json | hc policy decide <approval-id> --decision <approved|denied> [--note <text>] --json".to_string()
}

fn parse_agent_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("list") => Ok(CliCommand::Get {
            path: "/v1/agents".to_string(),
        }),
        Some("scan") => Ok(CliCommand::Post {
            path: "/v1/agents/scan".to_string(),
            body: "{}".to_string(),
        }),
        Some("register") => {
            let id = value_after(args, "--id").ok_or_else(agent_usage)?;
            let name = value_after(args, "--name").ok_or_else(agent_usage)?;
            let runtime = value_after(args, "--runtime").ok_or_else(agent_usage)?;
            let command = value_after(args, "--command").ok_or_else(agent_usage)?;
            let args_json = value_after(args, "--args")
                .map(|raw| serde_json::from_str::<serde_json::Value>(&raw))
                .transpose()
                .map_err(|_| agent_usage())?
                .unwrap_or_else(|| serde_json::json!([]));
            let env_policy_json = value_after(args, "--env-policy")
                .map(|raw| serde_json::from_str::<serde_json::Value>(&raw))
                .transpose()
                .map_err(|_| agent_usage())?
                .unwrap_or_else(|| serde_json::json!({"inherit": true}));
            let skills_json = value_after(args, "--skills")
                .map(|raw| serde_json::from_str::<serde_json::Value>(&raw))
                .transpose()
                .map_err(|_| agent_usage())?
                .unwrap_or_else(|| serde_json::json!([]));
            Ok(CliCommand::Post {
                path: "/v1/agents".to_string(),
                body: serde_json::json!({
                    "argsJson": args_json,
                    "command": command,
                    "envPolicyJson": env_policy_json,
                    "id": id,
                    "name": name,
                    "runtime": runtime,
                    "skillsJson": skills_json,
                    "status": "available"
                })
                .to_string(),
            })
        }
        Some("pause") => {
            let agent_id = args.get(3).ok_or_else(agent_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/agents/{agent_id}/pause"),
                body: "{}".to_string(),
            })
        }
        Some("resume") => {
            let agent_id = args.get(3).ok_or_else(agent_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/agents/{agent_id}/resume"),
                body: "{}".to_string(),
            })
        }
        Some("heartbeat") => {
            let agent_id = args.get(3).ok_or_else(agent_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/agents/{agent_id}/heartbeat"),
                body: "{}".to_string(),
            })
        }
        Some("skill-packs") => {
            let project_id = selected_project_id(args);
            Ok(CliCommand::Get {
                path: format!(
                    "/v1/skill-packs?projectId={}",
                    encode_query_value(&project_id)
                ),
            })
        }
        Some("runtime-pool") => {
            let project_id = selected_project_id(args);
            Ok(CliCommand::Get {
                path: format!(
                    "/v1/runtime-pool?projectId={}",
                    encode_query_value(&project_id)
                ),
            })
        }
        Some("skill-pack") if args.get(3).map(String::as_str) == Some("set") => {
            let name = value_after(args, "--name").ok_or_else(agent_usage)?;
            let skills_json = value_after(args, "--skills")
                .map(|raw| serde_json::from_str::<serde_json::Value>(&raw))
                .transpose()
                .map_err(|_| agent_usage())?
                .unwrap_or_else(|| serde_json::json!([]));
            let mut body = serde_json::json!({
                "name": name,
                "projectId": selected_project_id(args),
                "skillsJson": skills_json
            });
            if let Some(description) = value_after(args, "--description") {
                body["description"] = serde_json::json!(description);
            }
            if let Some(context_pack_id) = value_after(args, "--context-pack") {
                body["sourceContextPackId"] = serde_json::json!(context_pack_id);
            }
            Ok(CliCommand::Post {
                path: "/v1/skill-packs".to_string(),
                body: body.to_string(),
            })
        }
        Some("events") if args.get(3).map(String::as_str) == Some("ingest") => {
            let adapter = value_after(args, "--adapter").ok_or_else(agent_usage)?;
            let agent_id = value_after(args, "--agent").ok_or_else(agent_usage)?;
            let payload_raw = value_after(args, "--payload").ok_or_else(agent_usage)?;
            let payload = serde_json::from_str::<serde_json::Value>(&payload_raw)
                .map_err(|_| agent_usage())?;
            let session_id = value_after(args, "--session");
            let run_id = value_after(args, "--run");
            let mut body = serde_json::json!({
                "adapter": adapter,
                "agentProfileId": agent_id,
                "payload": payload,
                "projectId": selected_project_id(args)
            });
            if let Some(session_id) = session_id {
                body["sessionId"] = serde_json::json!(session_id);
            }
            if let Some(run_id) = run_id {
                body["runId"] = serde_json::json!(run_id);
            }
            Ok(CliCommand::Post {
                path: "/v1/agent-events/ingest".to_string(),
                body: body.to_string(),
            })
        }
        _ => Err(agent_usage()),
    }
}

fn agent_usage() -> String {
    "usage: hc agent list --json | hc agent scan --json | hc agent register --id <id> --name <name> --runtime <runtime> --command <command> [--args <json>] [--env-policy <json>] [--skills <json>] --json | hc agent pause <agent-id> --json | hc agent resume <agent-id> --json | hc agent heartbeat <agent-id> --json | hc agent skill-packs [--project <project-id>] --json | hc agent skill-pack set [--project <project-id>] --name <name> [--description <text>] [--skills <json>] [--context-pack <id>] --json | hc agent runtime-pool [--project <project-id>] --json | hc agent events ingest [--project <project-id>] --adapter <name> --agent <agent-id> --payload <json> [--session <session-id>] [--run <run-id>] --json".to_string()
}

fn parse_provider_model_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("get") => Ok(CliCommand::Get {
            path: "/v1/provider-model".to_string(),
        }),
        Some("set") => {
            let provider = value_after(args, "--provider").ok_or_else(provider_model_usage)?;
            let model = value_after(args, "--model").ok_or_else(provider_model_usage)?;
            let agent = value_after(args, "--agent").ok_or_else(provider_model_usage)?;
            Ok(CliCommand::Post {
                path: "/v1/provider-model".to_string(),
                body: serde_json::json!({
                    "agentProfileId": agent,
                    "model": model,
                    "provider": provider
                })
                .to_string(),
            })
        }
        _ => Err(provider_model_usage()),
    }
}

fn provider_model_usage() -> String {
    "usage: hc provider-model get --json | hc provider-model set --provider <provider> --model <model> --agent <agent-id> --json".to_string()
}

fn parse_terminal_theme_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("get") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/terminal-theme"),
        }),
        Some("set") => {
            let name = value_after(args, "--name").ok_or_else(terminal_theme_usage)?;
            let background = value_after(args, "--background").ok_or_else(terminal_theme_usage)?;
            let foreground = value_after(args, "--foreground").ok_or_else(terminal_theme_usage)?;
            let accent = value_after(args, "--accent").ok_or_else(terminal_theme_usage)?;
            let mut body = serde_json::json!({
                "accent": accent,
                "background": background,
                "foreground": foreground,
                "name": name
            });
            if let Some(project_id) = value_after(args, "--project") {
                body["projectId"] = serde_json::json!(project_id);
            }
            Ok(CliCommand::Post {
                path: "/v1/terminal-theme".to_string(),
                body: body.to_string(),
            })
        }
        _ => Err(terminal_theme_usage()),
    }
}

fn terminal_theme_usage() -> String {
    "usage: hc terminal-theme get [--project <project-id>] --json | hc terminal-theme set [--project <project-id>] --name <name> --background <#rrggbb> --foreground <#rrggbb> --accent <#rrggbb> --json".to_string()
}

fn parse_budget_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("status") | Some("explain") | Some("dashboard") => Ok(CliCommand::Get {
            path: "/v1/budgets".to_string(),
        }),
        Some("forecast") => Ok(CliCommand::Get {
            path: "/v1/budgets/forecast".to_string(),
        }),
        Some("prices") if args.get(3).map(String::as_str) == Some("update") => {
            let source = value_after(args, "--source").ok_or_else(budget_usage)?;
            let payload_raw = value_after(args, "--payload").ok_or_else(budget_usage)?;
            let prices = serde_json::from_str::<serde_json::Value>(&payload_raw)
                .map_err(|_| budget_usage())?;
            Ok(CliCommand::Post {
                path: "/v1/provider-prices/update".to_string(),
                body: serde_json::json!({
                    "source": source,
                    "prices": prices
                })
                .to_string(),
            })
        }
        Some("prices") => Ok(CliCommand::Get {
            path: "/v1/provider-prices".to_string(),
        }),
        Some("export") => {
            let run_id = value_after(args, "--run").ok_or_else(budget_usage)?;
            Ok(CliCommand::Get {
                path: format!("/v1/runs/{run_id}/token-usage"),
            })
        }
        Some("record") => {
            let provider = value_after(args, "--provider").ok_or_else(budget_usage)?;
            let model = value_after(args, "--model").ok_or_else(budget_usage)?;
            let input_tokens = parse_i64_flag(args, "--input", budget_usage)?;
            let output_tokens = parse_i64_flag(args, "--output", budget_usage)?;
            let cost_usd = value_after(args, "--cost-usd")
                .ok_or_else(budget_usage)?
                .parse::<f64>()
                .map_err(|_| budget_usage())?;
            let source = value_after(args, "--source").ok_or_else(budget_usage)?;
            Ok(CliCommand::Post {
                path: "/v1/token-usage".to_string(),
                body: serde_json::json!({
                    "agentProfileId": value_after(args, "--agent"),
                    "costUsd": cost_usd,
                    "inputTokens": input_tokens,
                    "model": model,
                    "outputTokens": output_tokens,
                    "projectId": selected_project_id(args),
                    "provider": provider,
                    "runId": value_after(args, "--run"),
                    "sessionId": value_after(args, "--session"),
                    "source": source,
                    "taskId": value_after(args, "--task")
                })
                .to_string(),
            })
        }
        Some("set") => {
            let scope_type = value_after(args, "--scope").ok_or_else(budget_usage)?;
            let scope_id = value_after(args, "--id");
            let max_usd = value_after(args, "--max-usd")
                .ok_or_else(budget_usage)?
                .parse::<f64>()
                .map_err(|_| budget_usage())?;
            let warn_pct = value_after(args, "--warn-pct")
                .unwrap_or_else(|| "0.8".to_string())
                .parse::<f64>()
                .map_err(|_| budget_usage())?;
            let hard_limit = args.iter().any(|arg| arg == "--hard-limit");
            Ok(CliCommand::Post {
                path: "/v1/budgets".to_string(),
                body: serde_json::json!({
                    "scopeType": scope_type,
                    "scopeId": scope_id,
                    "maxUsd": max_usd,
                    "warnPct": warn_pct,
                    "hardLimit": hard_limit
                })
                .to_string(),
            })
        }
        Some("ingest") => {
            let adapter = value_after(args, "--adapter").ok_or_else(budget_usage)?;
            let agent_profile_id = value_after(args, "--agent");
            let session_id = value_after(args, "--session");
            let task_id = value_after(args, "--task");
            let run_id = value_after(args, "--run");
            let payload_raw = value_after(args, "--payload").ok_or_else(budget_usage)?;
            let payload = serde_json::from_str::<serde_json::Value>(&payload_raw)
                .map_err(|_| budget_usage())?;
            let mut body = serde_json::json!({
                "adapter": adapter,
                "payload": payload,
                "projectId": selected_project_id(args)
            });
            if let Some(agent_profile_id) = agent_profile_id {
                body["agentProfileId"] = serde_json::json!(agent_profile_id);
            }
            if let Some(session_id) = session_id {
                body["sessionId"] = serde_json::json!(session_id);
            }
            if let Some(task_id) = task_id {
                body["taskId"] = serde_json::json!(task_id);
            }
            if let Some(run_id) = run_id {
                body["runId"] = serde_json::json!(run_id);
            }
            Ok(CliCommand::Post {
                path: "/v1/token-usage/ingest".to_string(),
                body: body.to_string(),
            })
        }
        _ => Err(budget_usage()),
    }
}

fn budget_usage() -> String {
    "usage: hc budget status|explain|dashboard|forecast|prices --json | hc budget prices update --source <source> --payload <json-array> --json | hc budget export --run <run-id> --json | hc budget record [--project <project-id>] --provider <provider> --model <model> --input <tokens> --output <tokens> --cost-usd <amount> --source <source> [--agent <id>] [--session <id>] [--task <id>] [--run <id>] --json | hc budget set --scope <workspace|project|goal|task|run|agent> [--id <scope-id>] --max-usd <amount> [--warn-pct <pct>] [--hard-limit] --json | hc budget ingest [--project <project-id>] --adapter <name> --payload <json> [--agent <id>] [--session <id>] [--task <id>] [--run <id>] --json".to_string()
}

fn parse_secret_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("list") => {
            let mut params = Vec::new();
            if let Some(project_id) = value_after(args, "--project") {
                params.push(format!("project={}", encode_query_value(&project_id)));
            }
            if let Some(name) = value_after(args, "--name") {
                params.push(format!("name={}", encode_query_value(&name)));
            }
            let path = if params.is_empty() {
                "/v1/secrets".to_string()
            } else {
                format!("/v1/secrets?{}", params.join("&"))
            };
            Ok(CliCommand::Get { path })
        }
        Some("set") => {
            let name = value_after(args, "--name").ok_or_else(secret_usage)?;
            let value = value_after(args, "--value").ok_or_else(secret_usage)?;
            Ok(CliCommand::Post {
                path: "/v1/secrets".to_string(),
                body: serde_json::json!({
                    "projectId": selected_project_id(args),
                    "name": name,
                    "value": value
                })
                .to_string(),
            })
        }
        _ => Err(secret_usage()),
    }
}

fn secret_usage() -> String {
    "usage: hc secret list [--project <project-id>] [--name <name>] --json | hc secret set [--project <project-id>] --name <name> --value <value> --json".to_string()
}

fn parse_knowledge_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("sources") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/knowledge/sources"),
        }),
        Some("source") if args.get(3).map(String::as_str) == Some("add") => {
            let kind = value_after(args, "--kind").ok_or_else(knowledge_usage)?;
            let path_or_ref = value_after(args, "--path").ok_or_else(knowledge_usage)?;
            let fingerprint = value_after(args, "--fingerprint").ok_or_else(knowledge_usage)?;
            let status = value_after(args, "--status").unwrap_or_else(|| "current".to_string());
            Ok(CliCommand::Post {
                path: "/v1/knowledge/sources".to_string(),
                body: serde_json::json!({
                    "fingerprint": fingerprint,
                    "kind": kind,
                    "pathOrRef": path_or_ref,
                    "projectId": selected_project_id(args),
                    "status": status
                })
                .to_string(),
            })
        }
        Some("page") if args.get(3).map(String::as_str) == Some("create") => {
            let slug = value_after(args, "--slug").ok_or_else(knowledge_usage)?;
            let title = value_after(args, "--title").ok_or_else(knowledge_usage)?;
            let body_md = value_after(args, "--body").ok_or_else(knowledge_usage)?;
            let source_ids = value_after(args, "--source")
                .map(|source| vec![source])
                .unwrap_or_default();
            Ok(CliCommand::Post {
                path: "/v1/knowledge/pages".to_string(),
                body: serde_json::json!({
                    "bodyMd": body_md,
                    "freshnessState": "current",
                    "projectId": selected_project_id(args),
                    "slug": slug,
                    "sourceIds": source_ids,
                    "title": title
                })
                .to_string(),
            })
        }
        Some("search") => {
            let mut params = Vec::new();
            if let Some(project_id) = value_after(args, "--project") {
                params.push(format!("projectId={}", encode_query_value(&project_id)));
            }
            if let Some(query) = value_after(args, "--query") {
                params.push(format!("query={}", encode_query_value(&query)));
            }
            let path = if params.is_empty() {
                "/v1/knowledge".to_string()
            } else {
                format!("/v1/knowledge?{}", params.join("&"))
            };
            Ok(CliCommand::Get { path })
        }
        Some("open") => {
            let page_id = args.get(3).ok_or_else(knowledge_usage)?;
            Ok(CliCommand::Get {
                path: format!("/v1/knowledge/{page_id}"),
            })
        }
        Some("explorations") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/knowledge/explorations"),
        }),
        Some("exploration") if args.get(3).map(String::as_str) == Some("create") => {
            let title = value_after(args, "--title").ok_or_else(knowledge_usage)?;
            let question = value_after(args, "--question").ok_or_else(knowledge_usage)?;
            let answer_md = value_after(args, "--answer").ok_or_else(knowledge_usage)?;
            let page_ids = value_after(args, "--page")
                .map(|page_id| vec![page_id])
                .unwrap_or_default();
            let context_pack_id = value_after(args, "--context");
            Ok(CliCommand::Post {
                path: "/v1/knowledge/explorations".to_string(),
                body: serde_json::json!({
                    "answerMd": answer_md,
                    "contextPackId": context_pack_id,
                    "pageIds": page_ids,
                    "projectId": selected_project_id(args),
                    "question": question,
                    "title": title
                })
                .to_string(),
            })
        }
        Some("concepts") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/knowledge/concepts"),
        }),
        Some("obsidian") if args.get(3).map(String::as_str) == Some("export") => {
            Ok(CliCommand::Post {
                path: "/v1/knowledge/obsidian/export".to_string(),
                body: serde_json::json!({
                    "projectId": selected_project_id(args)
                })
                .to_string(),
            })
        }
        Some("chat") => {
            let question = value_after(args, "--question").ok_or_else(knowledge_usage)?;
            let context_pack_id = value_after(args, "--context");
            Ok(CliCommand::Post {
                path: "/v1/knowledge/chat".to_string(),
                body: serde_json::json!({
                    "contextPackId": context_pack_id,
                    "projectId": selected_project_id(args),
                    "question": question
                })
                .to_string(),
            })
        }
        Some("lint") => {
            let stale_count = parse_i64_flag(args, "--stale", knowledge_usage)?;
            let gap_count = parse_i64_flag(args, "--gaps", knowledge_usage)?;
            let contradiction_count = value_after(args, "--contradictions")
                .map(|value| value.parse::<i64>().map_err(|_| knowledge_usage()))
                .transpose()?
                .unwrap_or(0);
            let body_md = value_after(args, "--body").ok_or_else(knowledge_usage)?;
            Ok(CliCommand::Post {
                path: "/v1/knowledge/lint".to_string(),
                body: serde_json::json!({
                    "bodyMd": body_md,
                    "contradictionCount": contradiction_count,
                    "gapCount": gap_count,
                    "projectId": selected_project_id(args),
                    "staleCount": stale_count
                })
                .to_string(),
            })
        }
        Some("compile") => Ok(CliCommand::Post {
            path: "/v1/knowledge/automation/run".to_string(),
            body: serde_json::json!({
                "projectId": selected_project_id(args),
                "watch": args.iter().any(|arg| arg == "--watch")
            })
            .to_string(),
        }),
        Some("ingest") => {
            let path_or_ref = value_after(args, "--path").ok_or_else(knowledge_usage)?;
            let kind = value_after(args, "--kind").unwrap_or_else(|| "markdown".to_string());
            let title = value_after(args, "--title");
            let body_md = value_after(args, "--body").ok_or_else(knowledge_usage)?;
            let max_chunk_chars = value_after(args, "--chunk")
                .map(|value| value.parse::<i64>().map_err(|_| knowledge_usage()))
                .transpose()?;
            Ok(CliCommand::Post {
                path: "/v1/knowledge/ingest".to_string(),
                body: serde_json::json!({
                    "bodyMd": body_md,
                    "kind": kind,
                    "maxChunkChars": max_chunk_chars,
                    "pathOrRef": path_or_ref,
                    "projectId": selected_project_id(args),
                    "title": title
                })
                .to_string(),
            })
        }
        _ => Err(knowledge_usage()),
    }
}

fn knowledge_usage() -> String {
    "usage: hc knowledge sources [--project <project-id>] --json | hc knowledge source add [--project <project-id>] --kind <kind> --path <path-or-ref> --fingerprint <fingerprint> [--status <status>] --json | hc knowledge search [--project <project-id>] [--query <query>] --json | hc knowledge open <page-id> --json | hc knowledge explorations [--project <project-id>] --json | hc knowledge exploration create [--project <project-id>] --title <title> --question <question> --answer <markdown> [--page <page-id>] [--context <context-pack>] --json | hc knowledge concepts [--project <project-id>] --json | hc knowledge obsidian export [--project <project-id>] --json | hc knowledge chat [--project <project-id>] --question <question> [--context <context-pack>] --json | hc knowledge page create [--project <project-id>] --slug <slug> --title <title> --body <markdown> [--source <source-id>] --json | hc knowledge lint [--project <project-id>] --stale <n> --gaps <n> [--contradictions <n>] --body <markdown> --json | hc knowledge compile [--project <project-id>] [--watch] --json | hc knowledge ingest [--project <project-id>] --path <path> [--kind <kind>] [--title <title>] --body <markdown> [--chunk <n>] --json".to_string()
}

fn parse_context_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("list") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/context-packs"),
        }),
        Some("show") => {
            let context_id = args.get(3).ok_or_else(context_usage)?;
            Ok(CliCommand::Get {
                path: format!("/v1/context-packs/{context_id}"),
            })
        }
        Some("create") => {
            let id = value_after(args, "--id");
            let name = value_after(args, "--name").ok_or_else(context_usage)?;
            let source = value_after(args, "--source").ok_or_else(context_usage)?;
            let max_tokens_hint = value_after(args, "--max-tokens")
                .map(|value| value.parse::<i64>().map_err(|_| context_usage()))
                .transpose()?;
            Ok(CliCommand::Post {
                path: "/v1/context-packs".to_string(),
                body: serde_json::json!({
                    "id": id,
                    "maxTokensHint": max_tokens_hint,
                    "name": name,
                    "projectId": selected_project_id(args),
                    "sourcesJson": [
                        {
                            "id": source,
                            "type": "knowledge_page"
                        }
                    ]
                })
                .to_string(),
            })
        }
        Some("attach") => {
            let task_id = args.get(3).ok_or_else(context_usage)?;
            let context_pack_id = value_after(args, "--context").ok_or_else(context_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/tasks/{task_id}/context"),
                body: serde_json::json!({ "contextPackId": context_pack_id }).to_string(),
            })
        }
        _ => Err(context_usage()),
    }
}

fn context_usage() -> String {
    "usage: hc context list [--project <project-id>] --json | hc context show <context-id> --json | hc context create [--project <project-id>] --name <name> --source <knowledge-page-id> [--id <context-id>] [--max-tokens <n>] --json | hc context attach <task-id> --context <context-pack> --json".to_string()
}

fn parse_block_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("search") => {
            let mut params = Vec::new();
            if let Some(query) = value_after(args, "--query") {
                params.push(format!("query={}", encode_query_value(&query)));
            }
            if let Some(task_id) = value_after(args, "--task") {
                params.push(format!("taskId={}", encode_query_value(&task_id)));
            }
            if let Some(session_id) = value_after(args, "--session") {
                params.push(format!("sessionId={}", encode_query_value(&session_id)));
            }
            let path = if params.is_empty() {
                "/v1/command-blocks".to_string()
            } else {
                format!("/v1/command-blocks?{}", params.join("&"))
            };
            Ok(CliCommand::Get { path })
        }
        Some("show") => {
            let command_block_id = args.get(3).ok_or_else(block_usage)?;
            Ok(CliCommand::Get {
                path: format!("/v1/command-blocks/{command_block_id}"),
            })
        }
        Some("mark") => {
            let command_block_id = args.get(3).ok_or_else(block_usage)?;
            let status = value_after(args, "--status").ok_or_else(block_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/command-blocks/{command_block_id}/mark"),
                body: serde_json::json!({
                    "status": status
                })
                .to_string(),
            })
        }
        Some("merge") => {
            let command_block_id = args.get(3).ok_or_else(block_usage)?;
            let second_command_block_id = value_after(args, "--with").ok_or_else(block_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/command-blocks/{command_block_id}/merge"),
                body: serde_json::json!({
                    "secondCommandBlockId": second_command_block_id
                })
                .to_string(),
            })
        }
        Some("split") => {
            let command_block_id = args.get(3).ok_or_else(block_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/command-blocks/{command_block_id}/split"),
                body: "{}".to_string(),
            })
        }
        Some("explain") => {
            let command_block_id = args.get(3).ok_or_else(block_usage)?;
            let provider = value_after(args, "--provider").ok_or_else(block_usage)?;
            let model = value_after(args, "--model").ok_or_else(block_usage)?;
            let agent = value_after(args, "--agent").ok_or_else(block_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/command-blocks/{command_block_id}/explain"),
                body: serde_json::json!({
                    "agentProfileId": agent,
                    "model": model,
                    "provider": provider
                })
                .to_string(),
            })
        }
        Some("export") => {
            let command_block_id = args.get(3).ok_or_else(block_usage)?;
            Ok(CliCommand::Get {
                path: format!("/v1/command-blocks/{command_block_id}/bundle"),
            })
        }
        Some("attach") => {
            let command_block_id = args.get(3).ok_or_else(block_usage)?;
            let evidence_pack_id = value_after(args, "--evidence").ok_or_else(block_usage)?;
            let task_id = value_after(args, "--task");
            let run_id = value_after(args, "--run");
            Ok(CliCommand::Post {
                path: format!("/v1/command-blocks/{command_block_id}/attach-evidence"),
                body: serde_json::json!({
                    "evidencePackId": evidence_pack_id,
                    "runId": run_id,
                    "taskId": task_id
                })
                .to_string(),
            })
        }
        _ => Err(block_usage()),
    }
}

fn block_usage() -> String {
    "usage: hc block search [--query <query>] [--task <task-id>] [--session <session-id>] --json | hc block show <command-block-id> --json | hc block mark <command-block-id> --status <running|completed|failed|unknown> --json | hc block merge <command-block-id> --with <command-block-id> --json | hc block split <command-block-id> --json | hc block explain <command-block-id> --provider <provider> --model <model> --agent <agent-id> --json | hc block export <command-block-id> --json | hc block attach <command-block-id> --evidence <evidence-pack-id> [--task <task-id>] [--run <run-id>] --json".to_string()
}

fn parse_task_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("list") => {
            let mut params = Vec::new();
            if let Some(project_id) = value_after(args, "--project") {
                params.push(format!("projectId={}", encode_query_value(&project_id)));
            }
            if let Some(status) = value_after(args, "--status") {
                params.push(format!("status={}", encode_query_value(&status)));
            }
            if let Some(query) = value_after(args, "--query") {
                params.push(format!("query={}", encode_query_value(&query)));
            }
            let path = if params.is_empty() {
                "/v1/tasks".to_string()
            } else {
                format!("/v1/tasks?{}", params.join("&"))
            };
            Ok(CliCommand::Get { path })
        }
        Some("create") => {
            let title = value_after(args, "--title").ok_or_else(|| {
                "usage: hc task create --title <title> [--project <project-id>] [--priority <priority>] --json".to_string()
            })?;
            let priority = value_after(args, "--priority");
            let project_id =
                value_after(args, "--project").unwrap_or_else(|| "proj_local".to_string());
            Ok(CliCommand::Post {
                path: "/v1/tasks".to_string(),
                body: serde_json::json!({
                    "projectId": project_id,
                    "title": title,
                    "priority": priority
                })
                .to_string(),
            })
        }
        Some("edit") => {
            let task_id = args.get(3).ok_or_else(task_usage)?;
            Ok(CliCommand::Patch {
                path: format!("/v1/tasks/{task_id}"),
                body: serde_json::json!({
                    "description": value_after(args, "--description"),
                    "priority": value_after(args, "--priority"),
                    "title": value_after(args, "--title")
                })
                .to_string(),
            })
        }
        Some("move") => {
            let task_id = args.get(3).ok_or_else(|| {
                "usage: hc task move <task-id> --status <status> --json".to_string()
            })?;
            let status = value_after(args, "--status").ok_or_else(|| {
                "usage: hc task move <task-id> --status <status> --json".to_string()
            })?;
            Ok(CliCommand::Post {
                path: format!("/v1/tasks/{task_id}/move"),
                body: serde_json::json!({ "status": status }).to_string(),
            })
        }
        Some("comment") => {
            let task_id = args.get(3).ok_or_else(|| {
                "usage: hc task comment <task-id> --body <body> --json".to_string()
            })?;
            let body = value_after(args, "--body").ok_or_else(|| {
                "usage: hc task comment <task-id> --body <body> --json".to_string()
            })?;
            Ok(CliCommand::Post {
                path: format!("/v1/tasks/{task_id}/comments"),
                body: serde_json::json!({ "body": body }).to_string(),
            })
        }
        Some("comments") => {
            let task_id = args
                .get(3)
                .ok_or_else(|| "usage: hc task comments <task-id> --json".to_string())?;
            Ok(CliCommand::Get {
                path: format!("/v1/tasks/{task_id}/comments"),
            })
        }
        Some("subtasks") => {
            let task_id = args
                .get(3)
                .ok_or_else(|| "usage: hc task subtasks <task-id> --json".to_string())?;
            Ok(CliCommand::Get {
                path: format!("/v1/tasks/{task_id}/subtasks"),
            })
        }
        Some("subtask") => match args.get(3).map(String::as_str) {
            Some("add") => {
                let task_id = args.get(4).ok_or_else(task_usage)?;
                let title = value_after(args, "--title").ok_or_else(task_usage)?;
                Ok(CliCommand::Post {
                    path: format!("/v1/tasks/{task_id}/subtasks"),
                    body: serde_json::json!({ "title": title }).to_string(),
                })
            }
            Some("status") => {
                let task_id = args.get(4).ok_or_else(task_usage)?;
                let subtask_id = args.get(5).ok_or_else(task_usage)?;
                let status = value_after(args, "--status").ok_or_else(task_usage)?;
                Ok(CliCommand::Post {
                    path: format!("/v1/tasks/{task_id}/subtasks/{subtask_id}/status"),
                    body: serde_json::json!({ "status": status }).to_string(),
                })
            }
            _ => Err(task_usage()),
        },
        Some("open") => {
            let task_id = args
                .get(3)
                .ok_or_else(|| "usage: hc task open <task-id> --json".to_string())?;
            Ok(CliCommand::Get {
                path: format!("/v1/tasks/{task_id}"),
            })
        }
        Some("workpad") => {
            let task_id = args.get(3).ok_or_else(|| {
                "usage: hc task workpad <task-id> --body <markdown> --json".to_string()
            })?;
            let body = value_after(args, "--body").ok_or_else(|| {
                "usage: hc task workpad <task-id> --body <markdown> --json".to_string()
            })?;
            Ok(CliCommand::Post {
                path: format!("/v1/tasks/{task_id}/workpad"),
                body: serde_json::json!({ "body": body }).to_string(),
            })
        }
        Some("context") => {
            let task_id = args.get(3).ok_or_else(|| {
                "usage: hc task context <task-id> --context <context-pack> --json".to_string()
            })?;
            let context_pack_id = value_after(args, "--context").ok_or_else(|| {
                "usage: hc task context <task-id> --context <context-pack> --json".to_string()
            })?;
            Ok(CliCommand::Post {
                path: format!("/v1/tasks/{task_id}/context"),
                body: serde_json::json!({ "contextPackId": context_pack_id }).to_string(),
            })
        }
        Some("assign") => {
            let task_id = args.get(3).ok_or_else(task_usage)?;
            let assignee_id = value_after(args, "--assignee").ok_or_else(task_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/tasks/{task_id}/planning"),
                body: serde_json::json!({
                    "assigneeId": assignee_id,
                    "assigneeType": "agent"
                })
                .to_string(),
            })
        }
        Some("planning") => {
            let task_id = args.get(3).ok_or_else(task_usage)?;
            let cycle_id = value_after(args, "--cycle");
            let module_id = value_after(args, "--module");
            let initiative_id = value_after(args, "--initiative");
            let assignee_id = value_after(args, "--assignee");
            let due_at = value_after(args, "--due");
            let estimate = value_after(args, "--estimate");
            let labels = value_after(args, "--labels").map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|label| !label.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            });
            if cycle_id.is_none()
                && module_id.is_none()
                && initiative_id.is_none()
                && assignee_id.is_none()
                && due_at.is_none()
                && estimate.is_none()
                && labels.is_none()
            {
                return Err(task_usage());
            }
            let assignee_type = assignee_id.as_ref().map(|_| "agent");
            let mut body = serde_json::Map::new();
            body.insert("assigneeId".to_string(), serde_json::json!(assignee_id));
            body.insert("assigneeType".to_string(), serde_json::json!(assignee_type));
            body.insert("cycleId".to_string(), serde_json::json!(cycle_id));
            if let Some(due_at) = due_at {
                body.insert("dueAt".to_string(), serde_json::json!(due_at));
            }
            if let Some(estimate) = estimate {
                body.insert("estimate".to_string(), serde_json::json!(estimate));
            }
            body.insert("initiativeId".to_string(), serde_json::json!(initiative_id));
            if let Some(labels) = labels {
                body.insert("labels".to_string(), serde_json::json!(labels));
            }
            body.insert("moduleId".to_string(), serde_json::json!(module_id));
            Ok(CliCommand::Post {
                path: format!("/v1/tasks/{task_id}/planning"),
                body: serde_json::Value::Object(body).to_string(),
            })
        }
        Some("cycles") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/cycles"),
        }),
        Some("cycle") => match args.get(3).map(String::as_str) {
            Some("create") => {
                let name = value_after(args, "--name").ok_or_else(task_usage)?;
                Ok(CliCommand::Post {
                    path: "/v1/cycles".to_string(),
                    body: serde_json::json!({
                        "endsAt": value_after(args, "--ends"),
                        "name": name,
                        "projectId": selected_project_id(args),
                        "startsAt": value_after(args, "--starts"),
                        "status": value_after(args, "--status")
                    })
                    .to_string(),
                })
            }
            _ => Err(task_usage()),
        },
        Some("modules") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/modules"),
        }),
        Some("module") => match args.get(3).map(String::as_str) {
            Some("create") => {
                let name = value_after(args, "--name").ok_or_else(task_usage)?;
                Ok(CliCommand::Post {
                    path: "/v1/modules".to_string(),
                    body: serde_json::json!({
                        "description": value_after(args, "--description"),
                        "name": name,
                        "projectId": selected_project_id(args),
                        "status": value_after(args, "--status")
                    })
                    .to_string(),
                })
            }
            _ => Err(task_usage()),
        },
        Some("lifecycle-e2e") => Ok(CliCommand::Post {
            path: "/v1/task-lifecycle/e2e/run".to_string(),
            body: serde_json::json!({
                "projectId": selected_project_id(args)
            })
            .to_string(),
        }),
        Some("lifecycle-e2e-runs") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/task-lifecycle/e2e/runs"),
        }),
        _ => Err(task_usage()),
    }
}

fn task_usage() -> String {
    "usage: hc task list|create|edit|move|assign|comment|comments|subtasks|open|workpad|context|planning [--cycle <cycle>] [--module <module>] [--initiative <initiative>] [--assignee <agent>] [--due <date>] [--estimate <text>] [--labels <comma-list>] --json | hc task subtask add <task-id> --title <title> --json | hc task subtask status <task-id> <subtask-id> --status <open|done> --json | hc task cycles [--project <project-id>] --json | hc task cycle create --name <name> [--project <project-id>] [--starts <date>] [--ends <date>] [--status <status>] --json | hc task modules [--project <project-id>] --json | hc task module create --name <name> [--project <project-id>] [--description <text>] [--status <status>] --json | hc task lifecycle-e2e [--project <project-id>] --json | hc task lifecycle-e2e-runs [--project <project-id>] --json".to_string()
}

fn parse_initiative_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("list") => {
            let project_id = value_after(args, "--project");
            let path = project_id
                .map(|project_id| {
                    format!(
                        "/v1/initiatives?projectId={}",
                        encode_query_value(&project_id)
                    )
                })
                .unwrap_or_else(|| "/v1/initiatives".to_string());
            Ok(CliCommand::Get { path })
        }
        Some("create") => {
            let name = value_after(args, "--name").ok_or_else(initiative_usage)?;
            let project_id =
                value_after(args, "--project").unwrap_or_else(|| "proj_local".to_string());
            let description = value_after(args, "--description");
            let budget_id = value_after(args, "--budget");
            let status = value_after(args, "--status");
            Ok(CliCommand::Post {
                path: "/v1/initiatives".to_string(),
                body: serde_json::json!({
                    "budgetId": budget_id,
                    "description": description,
                    "name": name,
                    "projectId": project_id,
                    "status": status
                })
                .to_string(),
            })
        }
        _ => Err(initiative_usage()),
    }
}

fn initiative_usage() -> String {
    "usage: hc initiative list [--project <project-id>] --json | hc initiative create --name <name> [--project <project-id>] [--description <text>] [--budget <budget-id>] [--status <status>] --json".to_string()
}

fn parse_tracker_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("bindings") | Some("list") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/tracker-bindings"),
        }),
        Some(provider @ ("linear" | "github" | "plane"))
            if args.get(3).map(String::as_str) == Some("sync") =>
        {
            Ok(CliCommand::Post {
                path: format!("/v1/tracker-sync/{provider}/run"),
                body: serde_json::json!({
                    "dryRun": args.iter().any(|arg| arg == "--dry-run"),
                    "projectId": selected_project_id(args)
                })
                .to_string(),
            })
        }
        Some("bind") => {
            let task_id = value_after(args, "--task");
            let project_id = value_after(args, "--project");
            let (local_kind, local_id, binding_project_id) = match (task_id, project_id) {
                (Some(task_id), project_id) => (
                    "task",
                    task_id,
                    project_id.unwrap_or_else(|| "proj_local".to_string()),
                ),
                (None, Some(project_id)) => ("project", project_id.clone(), project_id),
                (None, None) => return Err(tracker_usage()),
            };
            let provider = value_after(args, "--provider").ok_or_else(tracker_usage)?;
            let external_id = value_after(args, "--external-id").ok_or_else(tracker_usage)?;
            let external_url = value_after(args, "--url");
            let sync_mode = value_after(args, "--mode").unwrap_or_else(|| "manual".to_string());
            Ok(CliCommand::Post {
                path: "/v1/tracker-bindings".to_string(),
                body: serde_json::json!({
                    "externalId": external_id,
                    "externalUrl": external_url,
                    "localId": local_id,
                    "localKind": local_kind,
                    "projectId": binding_project_id,
                    "provider": provider,
                    "syncMode": sync_mode
                })
                .to_string(),
            })
        }
        _ => Err(tracker_usage()),
    }
}

fn tracker_usage() -> String {
    "usage: hc tracker bindings [--project <project-id>] --json | hc tracker bind --task <task-id> [--project <project-id>] --provider <linear|github|plane|custom|manual> --external-id <id> [--url <url>] [--mode <manual|mirror|import|export>] --json | hc tracker bind --project <project-id> --provider <linear|github|plane|custom|manual> --external-id <id> [--url <url>] [--mode <manual|mirror|import|export>] --json | hc tracker <linear|github|plane> sync [--project <project-id>] [--dry-run] --json".to_string()
}

fn parse_browser_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("run") => {
            let url = value_after(args, "--url").ok_or_else(browser_usage)?;
            let project_id =
                value_after(args, "--project").unwrap_or_else(|| "proj_local".to_string());
            let scenario = value_after(args, "--scenario").unwrap_or_else(|| "smoke".to_string());
            Ok(CliCommand::Post {
                path: "/v1/browser-automation/run".to_string(),
                body: serde_json::json!({
                    "projectId": project_id,
                    "scenario": scenario,
                    "url": url
                })
                .to_string(),
            })
        }
        _ => Err(browser_usage()),
    }
}

fn browser_usage() -> String {
    "usage: hc browser run --url <localhost-url> [--project <project-id>] [--scenario <name>] --json".to_string()
}

fn parse_review_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("list") => {
            let mut params = Vec::new();
            if let Some(project_id) = value_after(args, "--project") {
                params.push(format!("projectId={}", encode_query_value(&project_id)));
            }
            if let Some(state) = value_after(args, "--state") {
                params.push(format!("state={}", encode_query_value(&state)));
            }
            if let Some(completeness) = value_after(args, "--completeness") {
                params.push(format!(
                    "completeness={}",
                    encode_query_value(&completeness)
                ));
            }
            let path = if params.is_empty() {
                "/v1/reviews".to_string()
            } else {
                format!("/v1/reviews?{}", params.join("&"))
            };
            Ok(CliCommand::Get { path })
        }
        Some("accept") | Some("changes") | Some("block") => {
            let review_id = args.get(3).ok_or_else(review_usage)?;
            let decision = match args.get(2).map(String::as_str) {
                Some("accept") => "approved",
                Some("changes") => "changes_requested",
                Some("block") => "blocked",
                _ => unreachable!(),
            };
            let body_md = value_after(args, "--body");
            let reviewer_id =
                value_after(args, "--reviewer").unwrap_or_else(|| "human".to_string());
            Ok(CliCommand::Post {
                path: format!("/v1/reviews/{review_id}/decision"),
                body: serde_json::json!({
                    "bodyMd": body_md,
                    "decision": decision,
                    "reviewerId": reviewer_id
                })
                .to_string(),
            })
        }
        Some("follow-up") => {
            let review_id = args.get(3).ok_or_else(review_usage)?;
            let title = value_after(args, "--title").ok_or_else(review_usage)?;
            let priority = value_after(args, "--priority");
            Ok(CliCommand::Post {
                path: format!("/v1/reviews/{review_id}/follow-up-task"),
                body: serde_json::json!({
                    "priority": priority,
                    "title": title
                })
                .to_string(),
            })
        }
        Some("pr-plan") => {
            let review_id = args.get(3).ok_or_else(review_usage)?;
            let title = value_after(args, "--title").ok_or_else(review_usage)?;
            Ok(CliCommand::Post {
                path: format!("/v1/reviews/{review_id}/pr/landing-plan"),
                body: serde_json::json!({
                    "draft": args.iter().any(|arg| arg == "--draft"),
                    "title": title
                })
                .to_string(),
            })
        }
        _ => Err(review_usage()),
    }
}

fn review_usage() -> String {
    "usage: hc review list [--project <project-id>] [--state <pending|approved|changes_requested|reopened|blocked|incomplete>] [--completeness <complete|incomplete>] --json | hc review accept|changes|block <review-id> [--reviewer <id>] [--body <text>] --json | hc review follow-up <review-id> --title <title> [--priority <priority>] --json | hc review pr-plan <review-id> --title <title> [--draft] --json".to_string()
}

fn parse_release_gate_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("run") => Ok(CliCommand::Post {
            path: "/v1/release-gates/run".to_string(),
            body: serde_json::json!({
                "projectId": selected_project_id(args)
            })
            .to_string(),
        }),
        Some("list") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/release-gates/runs"),
        }),
        _ => Err(release_gate_usage()),
    }
}

fn release_gate_usage() -> String {
    "usage: hc release-gate run [--project <project-id>] --json | hc release-gate list [--project <project-id>] --json".to_string()
}

fn parse_terminal_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("smoke") => Ok(CliCommand::Post {
            path: "/v1/terminal-fidelity/smoke/run".to_string(),
            body: serde_json::json!({
                "projectId": selected_project_id(args)
            })
            .to_string(),
        }),
        Some("smoke-runs") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/terminal-fidelity/smoke/runs"),
        }),
        _ => Err(terminal_usage()),
    }
}

fn terminal_usage() -> String {
    "usage: hc terminal smoke [--project <project-id>] --json | hc terminal smoke-runs [--project <project-id>] --json".to_string()
}

fn parse_distribution_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("dmg-smoke") => Ok(CliCommand::Post {
            path: "/v1/distribution/dmg-smoke/run".to_string(),
            body: serde_json::json!({
                "projectId": selected_project_id(args)
            })
            .to_string(),
        }),
        Some("dmg-smoke-runs") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/distribution/dmg-smoke/runs"),
        }),
        _ => Err(distribution_usage()),
    }
}

fn distribution_usage() -> String {
    "usage: hc distribution dmg-smoke [--project <project-id>] --json | hc distribution dmg-smoke-runs [--project <project-id>] --json".to_string()
}

fn parse_recovery_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("drills") => Ok(CliCommand::Post {
            path: "/v1/recovery/drills/run".to_string(),
            body: serde_json::json!({
                "projectId": selected_project_id(args)
            })
            .to_string(),
        }),
        Some("drill-runs") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/recovery/drills/runs"),
        }),
        _ => Err(recovery_usage()),
    }
}

fn recovery_usage() -> String {
    "usage: hc recovery drills [--project <project-id>] --json | hc recovery drill-runs [--project <project-id>] --json".to_string()
}

fn parse_benchmark_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("run") => Ok(CliCommand::Post {
            path: "/v1/benchmarks/run".to_string(),
            body: serde_json::json!({
                "projectId": selected_project_id(args)
            })
            .to_string(),
        }),
        Some("runs") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/benchmarks/runs"),
        }),
        _ => Err(benchmark_usage()),
    }
}

fn benchmark_usage() -> String {
    "usage: hc benchmark run [--project <project-id>] --json | hc benchmark runs [--project <project-id>] --json".to_string()
}

fn parse_dogfood_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("telemetry-review") => Ok(CliCommand::Post {
            path: "/v1/dogfood/telemetry-review/run".to_string(),
            body: serde_json::json!({
                "projectId": selected_project_id(args)
            })
            .to_string(),
        }),
        Some("telemetry-reviews") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/dogfood/telemetry-reviews"),
        }),
        _ => Err(dogfood_usage()),
    }
}

fn dogfood_usage() -> String {
    "usage: hc dogfood telemetry-review [--project <project-id>] --json | hc dogfood telemetry-reviews [--project <project-id>] --json".to_string()
}

fn parse_visual_command(args: &[String]) -> Result<CliCommand, String> {
    match args.get(2).map(String::as_str) {
        Some("graph") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/visual-harness/graph"),
        }),
        Some("links") => Ok(CliCommand::Get {
            path: project_scoped_path(args, "/v1/visual-harness/links"),
        }),
        Some("link") => {
            let source_id = value_after(args, "--source").ok_or_else(visual_usage)?;
            let target_id = value_after(args, "--target").ok_or_else(visual_usage)?;
            let kind = value_after(args, "--kind").ok_or_else(visual_usage)?;
            Ok(CliCommand::Post {
                path: "/v1/visual-harness/links".to_string(),
                body: serde_json::json!({
                    "projectId": selected_project_id(args),
                    "sourceId": source_id,
                    "targetId": target_id,
                    "kind": kind
                })
                .to_string(),
            })
        }
        _ => Err(visual_usage()),
    }
}

fn visual_usage() -> String {
    "usage: hc visual graph [--project <project-id>] --json | hc visual links [--project <project-id>] --json | hc visual link [--project <project-id>] --source <id> --target <id> --kind <context|tool|task|workflow|dependency> --json".to_string()
}

fn value_after(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
}

fn split_csv_flag(args: &[String], flag: &str) -> Vec<String> {
    value_after(args, flag)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn parse_i64_flag(args: &[String], flag: &str, usage: fn() -> String) -> Result<i64, String> {
    value_after(args, flag)
        .ok_or_else(usage)?
        .parse::<i64>()
        .map_err(|_| usage())
}

fn encode_query_value(value: &str) -> String {
    value.replace(' ', "+")
}

fn selected_project_id(args: &[String]) -> String {
    value_after(args, "--project").unwrap_or_else(|| "proj_local".to_string())
}

fn project_scoped_path(args: &[String], base_path: &str) -> String {
    value_after(args, "--project")
        .map(|project_id| format!("{base_path}?projectId={}", encode_query_value(&project_id)))
        .unwrap_or_else(|| base_path.to_string())
}

fn request_json(path: &str) -> Result<String, String> {
    request(http_get_request(path))
}

fn request_json_with_body(path: &str, body: &str) -> Result<String, String> {
    request(http_post_request(path, body))
}

fn request_json_with_patch(path: &str, body: &str) -> Result<String, String> {
    request(http_patch_request(path, body))
}

fn request(request: String) -> Result<String, String> {
    let mut stream = UnixStream::connect(socket_path()).map_err(|error| {
        format!(
            "failed to connect to Haneulchi control API at {}: {error}",
            socket_path().display()
        )
    })?;
    stream
        .write_all(request.as_bytes())
        .and_then(|_| stream.flush())
        .map_err(|error| format!("failed to send control API request: {error}"))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| format!("failed to read control API response: {error}"))?;
    parse_http_body(&response)
}

fn http_get_request(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\nhost: haneulchi.local\r\naccept: application/json\r\nconnection: close\r\n\r\n"
    )
}

fn http_post_request(path: &str, body: &str) -> String {
    format!(
        "POST {path} HTTP/1.1\r\nhost: haneulchi.local\r\naccept: application/json\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn http_patch_request(path: &str, body: &str) -> String {
    format!(
        "PATCH {path} HTTP/1.1\r\nhost: haneulchi.local\r\naccept: application/json\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn parse_http_body(response: &str) -> Result<String, String> {
    let mut parts = response.splitn(2, "\r\n\r\n");
    let headers = parts.next().unwrap_or_default();
    let body = parts
        .next()
        .ok_or_else(|| "control API response did not include an HTTP body".to_string())?;
    let success = headers
        .lines()
        .next()
        .and_then(|status_line| status_line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .map(|code| (200..300).contains(&code))
        .unwrap_or(false);
    if !success {
        return Err(body.to_string());
    }
    Ok(body.to_string())
}

fn socket_path() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("Application Support")
        .join("Haneulchi")
        .join("run")
        .join("haneulchi.sock")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_state_and_health_commands() {
        assert_eq!(
            parse_command(["hc", "state", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/state".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "state", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/state?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "health", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/health".to_string()
            })
        );
        let root_usage = parse_command(["hc", "unknown", "--json"])
            .expect_err("unknown top-level commands should return root usage");
        for command_group in [
            "hc project",
            "hc dispatch",
            "hc run",
            "hc session",
            "hc evidence",
            "hc policy",
            "hc agent",
            "hc provider-model",
            "hc terminal-theme",
            "hc budget",
            "hc secret",
            "hc knowledge",
            "hc context",
            "hc workflow",
            "hc task",
            "hc initiative",
            "hc review",
            "hc release-gate",
            "hc terminal",
            "hc distribution",
            "hc recovery",
            "hc benchmark",
            "hc dogfood",
            "hc visual",
            "hc tracker",
            "hc browser",
            "hc block",
            "hc update",
        ] {
            assert!(
                root_usage.contains(command_group),
                "root usage should list {command_group}: {root_usage}"
            );
        }
    }

    #[test]
    fn parses_update_check_command() {
        assert_eq!(
            parse_command(["hc", "update", "check", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/update/check?channel=stable".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "update", "check", "--channel", "beta", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/update/check?channel=beta".to_string()
            })
        );
    }

    #[test]
    fn parses_task_commands() {
        assert_eq!(
            parse_command(["hc", "task", "list", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/tasks".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "task", "list", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/tasks?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "list",
                "--project",
                "proj_auth",
                "--status",
                "ready",
                "--query",
                "auth flow",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/tasks?projectId=proj_auth&status=ready&query=auth+flow".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "create",
                "--title",
                "New task",
                "--project",
                "proj_auth",
                "--priority",
                "high",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tasks".to_string(),
                body: "{\"priority\":\"high\",\"projectId\":\"proj_auth\",\"title\":\"New task\"}"
                    .to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "edit",
                "task_1",
                "--title",
                "Updated title",
                "--description",
                "Updated description",
                "--priority",
                "urgent",
                "--json"
            ]),
            Ok(CliCommand::Patch {
                path: "/v1/tasks/task_1".to_string(),
                body: "{\"description\":\"Updated description\",\"priority\":\"urgent\",\"title\":\"Updated title\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "task", "move", "task_1", "--status", "ready", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/tasks/task_1/move".to_string(),
                body: "{\"status\":\"ready\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "task", "comment", "task_1", "--body", "Done", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/tasks/task_1/comments".to_string(),
                body: "{\"body\":\"Done\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "task", "comments", "task_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/tasks/task_1/comments".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "task", "subtasks", "task_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/tasks/task_1/subtasks".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "subtask",
                "add",
                "task_1",
                "--title",
                "Attach screenshots",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tasks/task_1/subtasks".to_string(),
                body: "{\"title\":\"Attach screenshots\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "subtask",
                "status",
                "task_1",
                "subtask_1",
                "--status",
                "done",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tasks/task_1/subtasks/subtask_1/status".to_string(),
                body: "{\"status\":\"done\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "workpad",
                "task_1",
                "--body",
                "## Notes\n- Attach evidence",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tasks/task_1/workpad".to_string(),
                body: "{\"body\":\"## Notes\\n- Attach evidence\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "context",
                "task_1",
                "--context",
                "ctx_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tasks/task_1/context".to_string(),
                body: "{\"contextPackId\":\"ctx_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "planning",
                "task_1",
                "--cycle",
                "Sprint 5",
                "--module",
                "Control API",
                "--initiative",
                "init_auth",
                "--assignee",
                "agent_codex",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tasks/task_1/planning".to_string(),
                body: "{\"assigneeId\":\"agent_codex\",\"assigneeType\":\"agent\",\"cycleId\":\"Sprint 5\",\"initiativeId\":\"init_auth\",\"moduleId\":\"Control API\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "planning",
                "task_1",
                "--due",
                "2026-05-15",
                "--estimate",
                "3 pts",
                "--labels",
                "release,evidence",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tasks/task_1/planning".to_string(),
                body: "{\"assigneeId\":null,\"assigneeType\":null,\"cycleId\":null,\"dueAt\":\"2026-05-15\",\"estimate\":\"3 pts\",\"initiativeId\":null,\"labels\":[\"release\",\"evidence\"],\"moduleId\":null}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "task", "cycles", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/cycles?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "cycle",
                "create",
                "--project",
                "proj_auth",
                "--name",
                "Sprint 13",
                "--starts",
                "2026-05-01",
                "--ends",
                "2026-05-15",
                "--status",
                "planned",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/cycles".to_string(),
                body: "{\"endsAt\":\"2026-05-15\",\"name\":\"Sprint 13\",\"projectId\":\"proj_auth\",\"startsAt\":\"2026-05-01\",\"status\":\"planned\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "task", "modules", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/modules?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "module",
                "create",
                "--project",
                "proj_auth",
                "--name",
                "Release",
                "--description",
                "Release gate work",
                "--status",
                "active",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/modules".to_string(),
                body: "{\"description\":\"Release gate work\",\"name\":\"Release\",\"projectId\":\"proj_auth\",\"status\":\"active\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "assign",
                "task_1",
                "--assignee",
                "agent_codex",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tasks/task_1/planning".to_string(),
                body: "{\"assigneeId\":\"agent_codex\",\"assigneeType\":\"agent\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "task", "open", "task_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/tasks/task_1".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "task", "lifecycle-e2e", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/task-lifecycle/e2e/run".to_string(),
                body: "{\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "lifecycle-e2e",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/task-lifecycle/e2e/run".to_string(),
                body: "{\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "task", "lifecycle-e2e-runs", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/task-lifecycle/e2e/runs".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "task",
                "lifecycle-e2e-runs",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/task-lifecycle/e2e/runs?projectId=proj_auth".to_string()
            })
        );
    }

    #[test]
    fn parses_initiative_commands() {
        assert_eq!(
            parse_command(["hc", "initiative", "list", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/initiatives".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "initiative",
                "list",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/initiatives?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "initiative",
                "create",
                "--name",
                "Auth reliability goal",
                "--description",
                "Why auth tasks matter",
                "--budget",
                "budget_auth",
                "--status",
                "active",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/initiatives".to_string(),
                body: "{\"budgetId\":\"budget_auth\",\"description\":\"Why auth tasks matter\",\"name\":\"Auth reliability goal\",\"projectId\":\"proj_local\",\"status\":\"active\"}".to_string()
            })
        );
    }

    #[test]
    fn parses_tracker_binding_commands() {
        assert_eq!(
            parse_command(["hc", "tracker", "bindings", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/tracker-bindings".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "tracker",
                "bindings",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/tracker-bindings?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "tracker",
                "bind",
                "--task",
                "task_1",
                "--provider",
                "linear",
                "--external-id",
                "LIN-42",
                "--url",
                "https://linear.app/acme/issue/LIN-42",
                "--mode",
                "mirror",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tracker-bindings".to_string(),
                body: "{\"externalId\":\"LIN-42\",\"externalUrl\":\"https://linear.app/acme/issue/LIN-42\",\"localId\":\"task_1\",\"localKind\":\"task\",\"projectId\":\"proj_local\",\"provider\":\"linear\",\"syncMode\":\"mirror\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "tracker",
                "bind",
                "--task",
                "task_auth",
                "--project",
                "proj_auth",
                "--provider",
                "linear",
                "--external-id",
                "AUTH-42",
                "--mode",
                "mirror",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tracker-bindings".to_string(),
                body: "{\"externalId\":\"AUTH-42\",\"externalUrl\":null,\"localId\":\"task_auth\",\"localKind\":\"task\",\"projectId\":\"proj_auth\",\"provider\":\"linear\",\"syncMode\":\"mirror\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "tracker",
                "bind",
                "--project",
                "proj_auth",
                "--provider",
                "github",
                "--external-id",
                "octo/repo#123",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tracker-bindings".to_string(),
                body: "{\"externalId\":\"octo/repo#123\",\"externalUrl\":null,\"localId\":\"proj_auth\",\"localKind\":\"project\",\"projectId\":\"proj_auth\",\"provider\":\"github\",\"syncMode\":\"manual\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "tracker", "linear", "sync", "--dry-run", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/tracker-sync/linear/run".to_string(),
                body: "{\"dryRun\":true,\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "tracker",
                "linear",
                "sync",
                "--project",
                "proj_auth",
                "--dry-run",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tracker-sync/linear/run".to_string(),
                body: "{\"dryRun\":true,\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "tracker", "github", "sync", "--dry-run", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/tracker-sync/github/run".to_string(),
                body: "{\"dryRun\":true,\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "tracker", "plane", "sync", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/tracker-sync/plane/run".to_string(),
                body: "{\"dryRun\":false,\"projectId\":\"proj_local\"}".to_string()
            })
        );
    }

    #[test]
    fn parses_release_gate_runner_commands() {
        assert_eq!(
            parse_command(["hc", "release-gate", "run", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/release-gates/run".to_string(),
                body: "{\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "release-gate",
                "run",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/release-gates/run".to_string(),
                body: "{\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "release-gate", "list", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/release-gates/runs".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "release-gate",
                "list",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/release-gates/runs?projectId=proj_auth".to_string()
            })
        );
    }

    #[test]
    fn parses_terminal_fidelity_smoke_commands() {
        assert_eq!(
            parse_command(["hc", "terminal", "smoke", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/terminal-fidelity/smoke/run".to_string(),
                body: "{\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "terminal",
                "smoke",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/terminal-fidelity/smoke/run".to_string(),
                body: "{\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "terminal", "smoke-runs", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/terminal-fidelity/smoke/runs".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "terminal",
                "smoke-runs",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/terminal-fidelity/smoke/runs?projectId=proj_auth".to_string()
            })
        );
    }

    #[test]
    fn parses_distribution_dmg_smoke_commands() {
        assert_eq!(
            parse_command(["hc", "distribution", "dmg-smoke", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/distribution/dmg-smoke/run".to_string(),
                body: "{\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "distribution",
                "dmg-smoke",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/distribution/dmg-smoke/run".to_string(),
                body: "{\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "distribution", "dmg-smoke-runs", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/distribution/dmg-smoke/runs".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "distribution",
                "dmg-smoke-runs",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/distribution/dmg-smoke/runs?projectId=proj_auth".to_string()
            })
        );
    }

    #[test]
    fn parses_recovery_drill_commands() {
        assert_eq!(
            parse_command(["hc", "recovery", "drills", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/recovery/drills/run".to_string(),
                body: "{\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "recovery",
                "drills",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/recovery/drills/run".to_string(),
                body: "{\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "recovery", "drill-runs", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/recovery/drills/runs".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "recovery",
                "drill-runs",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/recovery/drills/runs?projectId=proj_auth".to_string()
            })
        );
    }

    #[test]
    fn parses_benchmark_suite_commands() {
        assert_eq!(
            parse_command(["hc", "benchmark", "run", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/benchmarks/run".to_string(),
                body: "{\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "benchmark", "run", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/benchmarks/run".to_string(),
                body: "{\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "benchmark", "runs", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/benchmarks/runs".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "benchmark",
                "runs",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/benchmarks/runs?projectId=proj_auth".to_string()
            })
        );
    }

    #[test]
    fn parses_dogfood_telemetry_review_commands() {
        assert_eq!(
            parse_command(["hc", "dogfood", "telemetry-review", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/dogfood/telemetry-review/run".to_string(),
                body: "{\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "dogfood",
                "telemetry-review",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/dogfood/telemetry-review/run".to_string(),
                body: "{\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "dogfood", "telemetry-reviews", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/dogfood/telemetry-reviews".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "dogfood",
                "telemetry-reviews",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/dogfood/telemetry-reviews?projectId=proj_auth".to_string()
            })
        );
    }

    #[test]
    fn parses_visual_harness_link_commands() {
        assert_eq!(
            parse_command(["hc", "visual", "graph", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/visual-harness/graph".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "visual", "graph", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/visual-harness/graph?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "visual", "links", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/visual-harness/links".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "visual", "links", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/visual-harness/links?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "visual",
                "link",
                "--source",
                "ctx_default",
                "--target",
                "task_1",
                "--kind",
                "context",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/visual-harness/links".to_string(),
                body: "{\"kind\":\"context\",\"projectId\":\"proj_local\",\"sourceId\":\"ctx_default\",\"targetId\":\"task_1\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "visual",
                "link",
                "--project",
                "proj_auth",
                "--source",
                "ctx_auth",
                "--target",
                "task_2",
                "--kind",
                "task",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/visual-harness/links".to_string(),
                body: "{\"kind\":\"task\",\"projectId\":\"proj_auth\",\"sourceId\":\"ctx_auth\",\"targetId\":\"task_2\"}".to_string()
            })
        );
    }

    #[test]
    fn parses_project_commands() {
        assert_eq!(
            parse_command(["hc", "project", "list", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/projects".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "add",
                "--key",
                "AUTH",
                "--name",
                "Auth Service",
                "--path",
                "/repo/auth-service",
                "--color",
                "#059669",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/projects".to_string(),
                body: "{\"color\":\"#059669\",\"key\":\"AUTH\",\"name\":\"Auth Service\",\"path\":\"/repo/auth-service\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "project", "focus", "proj_auth", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/projects/proj_auth/focus".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "project", "detach", "proj_auth", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/projects/proj_auth/detach".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "tab-group",
                "proj_auth",
                "--name",
                "Backend",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/projects/proj_auth/tab-group".to_string(),
                body: "{\"groupName\":\"Backend\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "layout",
                "proj_auth",
                "--preset",
                "maximized",
                "--focused-session",
                "session_1",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/projects/proj_auth/layout".to_string(),
                body: "{\"layoutJson\":{\"focusedSessionId\":\"session_1\",\"maximizedSessionId\":\"session_1\",\"mode\":\"maximized\",\"panes\":[\"session_1\"]}}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "project", "layout-presets", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/projects/proj_auth/layout-presets".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "layout-preset",
                "proj_auth",
                "--name",
                "Review grid",
                "--preset",
                "grid",
                "--focused-session",
                "session_1",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/projects/proj_auth/layout-presets".to_string(),
                body: "{\"layoutJson\":{\"focusedSessionId\":\"session_1\",\"maximizedSessionId\":null,\"mode\":\"grid\",\"panes\":[\"session_1\"]},\"name\":\"Review grid\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "files",
                "proj_auth",
                "--path",
                "src",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/projects/proj_auth/files?path=src".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "file",
                "proj_auth",
                "--path",
                "src/main.rs",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/projects/proj_auth/file?path=src/main.rs".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "write-file",
                "proj_auth",
                "--path",
                "src/main.ts",
                "--body",
                "export const value = 2;\n",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/projects/proj_auth/file".to_string(),
                body: "{\"body\":\"export const value = 2;\\n\",\"path\":\"src/main.ts\"}"
                    .to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "diff",
                "proj_auth",
                "--path",
                "src/main.rs",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/projects/proj_auth/diff?path=src/main.rs".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "lsp",
                "proj_auth",
                "--path",
                "src/app.ts",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/projects/proj_auth/lsp-diagnostics?path=src/app.ts".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "project", "patch-export", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/projects/proj_auth/patch/export".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "patch-export",
                "proj_auth",
                "--path",
                "src/main.rs",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/projects/proj_auth/patch/export?path=src/main.rs".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "patch-import",
                "proj_auth",
                "--body",
                "diff --git a/a b/a",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/projects/proj_auth/patch/import".to_string(),
                body: "{\"body\":\"diff --git a/a b/a\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "pr-plan",
                "proj_auth",
                "--title",
                "Ship auth",
                "--draft",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/projects/proj_auth/pr/landing-plan".to_string(),
                body: "{\"draft\":true,\"title\":\"Ship auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "project",
                "search-files",
                "proj_auth",
                "--query",
                "login form",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/projects/proj_auth/files/search?query=login+form".to_string()
            })
        );
    }

    #[test]
    fn parses_browser_automation_commands() {
        assert_eq!(
            parse_command([
                "hc",
                "browser",
                "run",
                "--project",
                "proj_auth",
                "--url",
                "http://localhost:3000/docs",
                "--scenario",
                "smoke",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/browser-automation/run".to_string(),
                body: "{\"projectId\":\"proj_auth\",\"scenario\":\"smoke\",\"url\":\"http://localhost:3000/docs\"}".to_string()
            })
        );
    }

    #[test]
    fn parses_block_search_command() {
        assert_eq!(
            parse_command([
                "hc",
                "block",
                "search",
                "--query",
                "frontend diagnostics",
                "--task",
                "task_frontend",
                "--session",
                "pty_1",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path:
                    "/v1/command-blocks?query=frontend+diagnostics&taskId=task_frontend&sessionId=pty_1"
                        .to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "block", "search", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/command-blocks".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "block", "show", "cmdblk_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/command-blocks/cmdblk_1".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "block",
                "mark",
                "cmdblk_1",
                "--status",
                "completed",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/command-blocks/cmdblk_1/mark".to_string(),
                body: "{\"status\":\"completed\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "block", "merge", "cmdblk_1", "--with", "cmdblk_2", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/command-blocks/cmdblk_1/merge".to_string(),
                body: "{\"secondCommandBlockId\":\"cmdblk_2\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "block", "split", "cmdblk_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/command-blocks/cmdblk_1/split".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "block",
                "explain",
                "cmdblk_1",
                "--provider",
                "openai",
                "--model",
                "gpt-5.4",
                "--agent",
                "agent_codex",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/command-blocks/cmdblk_1/explain".to_string(),
                body: "{\"agentProfileId\":\"agent_codex\",\"model\":\"gpt-5.4\",\"provider\":\"openai\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "block", "export", "cmdblk_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/command-blocks/cmdblk_1/bundle".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "block",
                "attach",
                "cmdblk_1",
                "--evidence",
                "ev_local",
                "--task",
                "task_review",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/command-blocks/cmdblk_1/attach-evidence".to_string(),
                body: "{\"evidencePackId\":\"ev_local\",\"runId\":null,\"taskId\":\"task_review\"}"
                    .to_string()
            })
        );
    }

    #[test]
    fn parses_dispatch_and_run_list_commands() {
        assert_eq!(
            parse_command([
                "hc",
                "dispatch",
                "task_ready",
                "--agent",
                "agent_codex",
                "--context",
                "ctx_default",
                "--workspace",
                "/repo/.haneulchi/worktrees/run_1",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/dispatch".to_string(),
                body: "{\"agentProfileId\":\"agent_codex\",\"contextPackId\":\"ctx_default\",\"taskId\":\"task_ready\",\"workspacePath\":\"/repo/.haneulchi/worktrees/run_1\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "run", "list", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/runs".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "run", "list", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/runs?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "run",
                "list",
                "--project",
                "proj_local",
                "--lifecycle",
                "review_ready",
                "--task",
                "task_ready",
                "--agent",
                "agent_claude",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/runs?projectId=proj_local&lifecycle=review_ready&taskId=task_ready&agentProfileId=agent_claude".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "run", "open", "run_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/runs/run_1".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "run", "replay", "run_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/runs/run_1/replay".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "run", "usage", "run_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/runs/run_1/token-usage".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "run",
                "transition",
                "run_1",
                "--lifecycle",
                "review_ready",
                "--detail",
                "Evidence pack is ready",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/runs/run_1/transition".to_string(),
                body:
                    "{\"lifecycle\":\"review_ready\",\"statusDetail\":\"Evidence pack is ready\"}"
                        .to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "run", "cancel", "run_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/runs/run_1/cancel".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "run",
                "status",
                "run_1",
                "--body",
                "Investigating OAuth fixture failure.",
                "--lifecycle",
                "waiting_input",
                "--detail",
                "Needs OAuth test account",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/runs/run_1/status-updates".to_string(),
                body: "{\"bodyMd\":\"Investigating OAuth fixture failure.\",\"lifecycle\":\"waiting_input\",\"statusDetail\":\"Needs OAuth test account\"}".to_string()
            })
        );
        let status_usage = parse_command(["hc", "run", "status", "run_1", "--json"])
            .expect_err("run status without --body should return usage");
        assert!(
            status_usage.contains("hc run status <run-id> --body <body>"),
            "run status usage should document the status command: {status_usage}"
        );
        assert_eq!(
            parse_command(["hc", "run", "retry", "run_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/runs/run_1/retry".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "run",
                "hook",
                "run_1",
                "--hook",
                "before_run",
                "--repo-root",
                "/repo",
                "--workspace",
                "/repo/.haneulchi/worktrees/run_1",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/runs/run_1/hooks/before_run/run".to_string(),
                body: "{\"repoRoot\":\"/repo\",\"workspacePath\":\"/repo/.haneulchi/worktrees/run_1\"}".to_string()
            })
        );
    }

    #[test]
    fn parses_evidence_generate_and_review_commands() {
        assert_eq!(
            parse_command(["hc", "evidence", "generate", "run_1", "--id", "ev_run_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/runs/run_1/evidence/generate".to_string(),
                body: "{\"evidencePackId\":\"ev_run_1\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "evidence",
                "review",
                "ev_run_1",
                "--decision",
                "approved",
                "--reviewer",
                "human",
                "--body",
                "Looks complete",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/evidence/ev_run_1/review-decision".to_string(),
                body: "{\"bodyMd\":\"Looks complete\",\"decision\":\"approved\",\"reviewerId\":\"human\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "evidence",
                "review",
                "ev_run_1",
                "--decision",
                "blocked",
                "--body",
                "Fixture missing",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/evidence/ev_run_1/review-decision".to_string(),
                body:
                    "{\"bodyMd\":\"Fixture missing\",\"decision\":\"blocked\",\"reviewerId\":null}"
                        .to_string()
            })
        );
    }

    #[test]
    fn parses_policy_approval_commands() {
        assert_eq!(
            parse_command(["hc", "policy", "approvals", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/policy/approvals".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "policy", "approvals", "--state", "pending", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/policy/approvals?state=pending".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "policy",
                "approvals",
                "--project",
                "proj_auth",
                "--state",
                "pending",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/policy/approvals?project=proj_auth&state=pending".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "policy",
                "request",
                "--project",
                "proj_auth",
                "--run",
                "run_1",
                "--task",
                "task_1",
                "--action",
                "shell_command",
                "--command",
                "rm -rf build/cache",
                "--risk",
                "high",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/policy/approvals".to_string(),
                body: "{\"actionKind\":\"shell_command\",\"command\":\"rm -rf build/cache\",\"projectId\":\"proj_auth\",\"requestedBy\":\"hc\",\"riskLevel\":\"high\",\"runId\":\"run_1\",\"taskId\":\"task_1\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "policy",
                "evaluate",
                "--project",
                "proj_auth",
                "--run",
                "run_policy",
                "--task",
                "task_policy",
                "--action",
                "network",
                "--command",
                "curl https://example.com",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/policy/evaluate".to_string(),
                body: "{\"actionKind\":\"network\",\"command\":\"curl https://example.com\",\"projectId\":\"proj_auth\",\"requestedBy\":\"hc\",\"runId\":\"run_policy\",\"taskId\":\"task_policy\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "policy",
                "decide",
                "policy_approval_1",
                "--decision",
                "approved",
                "--note",
                "Allowed",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/policy/approvals/policy_approval_1/decision".to_string(),
                body: "{\"decision\":\"approved\",\"decisionBy\":\"human\",\"decisionNote\":\"Allowed\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "policy", "packs", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/policy/packs".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "policy", "packs", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/policy/packs?project=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "policy",
                "packs",
                "--project",
                "proj_auth",
                "--active",
                "true",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/policy/packs?project=proj_auth&active=true".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "policy", "audit", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/policy/audit".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "policy",
                "audit",
                "--project",
                "proj_auth",
                "--decision",
                "forbidden",
                "--action",
                "network",
                "--run",
                "run_policy",
                "--task",
                "task_policy",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/policy/audit?project=proj_auth&decision=forbidden&actionKind=network&run=run_policy&task=task_policy".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "policy",
                "pack",
                "set",
                "--project",
                "proj_auth",
                "--name",
                "Ask before write",
                "--sandbox",
                "ask-before-write",
                "--network",
                "blocked",
                "--network-profile",
                "local-only",
                "--file-write",
                "ask",
                "--approvals",
                "shell_command,file_write",
                "--forbidden",
                "network",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/policy/packs".to_string(),
                body: "{\"approvalRequired\":[\"shell_command\",\"file_write\"],\"fileWrite\":\"ask\",\"forbiddenOperations\":[\"network\"],\"name\":\"Ask before write\",\"network\":\"blocked\",\"networkProfile\":\"local-only\",\"projectId\":\"proj_auth\",\"sandboxMode\":\"ask-before-write\",\"setActive\":true}".to_string()
            })
        );
    }

    #[test]
    fn parses_review_queue_commands() {
        assert_eq!(
            parse_command(["hc", "review", "list", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/reviews".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "review", "list", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/reviews?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "review",
                "list",
                "--project",
                "proj_local",
                "--state",
                "approved",
                "--completeness",
                "complete",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/reviews?projectId=proj_local&state=approved&completeness=complete"
                    .to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "review",
                "accept",
                "review_ev_run_1",
                "--reviewer",
                "human",
                "--body",
                "Looks complete",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/reviews/review_ev_run_1/decision".to_string(),
                body: "{\"bodyMd\":\"Looks complete\",\"decision\":\"approved\",\"reviewerId\":\"human\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "review", "changes", "review_ev_run_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/reviews/review_ev_run_1/decision".to_string(),
                body:
                    "{\"bodyMd\":null,\"decision\":\"changes_requested\",\"reviewerId\":\"human\"}"
                        .to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "review", "block", "review_ev_run_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/reviews/review_ev_run_1/decision".to_string(),
                body: "{\"bodyMd\":null,\"decision\":\"blocked\",\"reviewerId\":\"human\"}"
                    .to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "review",
                "follow-up",
                "review_ev_run_1",
                "--title",
                "Address reviewer notes",
                "--priority",
                "urgent",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/reviews/review_ev_run_1/follow-up-task".to_string(),
                body: "{\"priority\":\"urgent\",\"title\":\"Address reviewer notes\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "review",
                "pr-plan",
                "review_ev_run_1",
                "--title",
                "Ship review evidence",
                "--draft",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/reviews/review_ev_run_1/pr/landing-plan".to_string(),
                body: "{\"draft\":true,\"title\":\"Ship review evidence\"}".to_string()
            })
        );
    }

    #[test]
    fn parses_agent_directory_commands() {
        assert_eq!(
            parse_command(["hc", "agent", "list", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/agents".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "agent", "scan", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/agents/scan".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "agent", "pause", "agent_codex", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/agents/agent_codex/pause".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "agent", "resume", "agent_codex", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/agents/agent_codex/resume".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "agent", "heartbeat", "agent_codex", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/agents/agent_codex/heartbeat".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "agent",
                "skill-packs",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/skill-packs?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "agent",
                "runtime-pool",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/runtime-pool?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "agent",
                "skill-pack",
                "set",
                "--project",
                "proj_auth",
                "--name",
                "Auth reviewer",
                "--description",
                "Review auth flows",
                "--skills",
                "[\"code-review\",\"auth\"]",
                "--context-pack",
                "ctx_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/skill-packs".to_string(),
                body: "{\"description\":\"Review auth flows\",\"name\":\"Auth reviewer\",\"projectId\":\"proj_auth\",\"skillsJson\":[\"code-review\",\"auth\"],\"sourceContextPackId\":\"ctx_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "agent",
                "events",
                "ingest",
                "--adapter",
                "raw-jsonl",
                "--agent",
                "agent_codex",
                "--session",
                "session_1",
                "--run",
                "run_1",
                "--payload",
                "{\"raw\":\"{\\\"type\\\":\\\"status\\\",\\\"status\\\":\\\"needs_input\\\",\\\"message\\\":\\\"Waiting for review\\\"}\\n\"}",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/agent-events/ingest".to_string(),
                body: "{\"adapter\":\"raw-jsonl\",\"agentProfileId\":\"agent_codex\",\"payload\":{\"raw\":\"{\\\"type\\\":\\\"status\\\",\\\"status\\\":\\\"needs_input\\\",\\\"message\\\":\\\"Waiting for review\\\"}\\n\"},\"projectId\":\"proj_local\",\"runId\":\"run_1\",\"sessionId\":\"session_1\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "agent",
                "events",
                "ingest",
                "--project",
                "proj_auth",
                "--adapter",
                "raw-jsonl",
                "--agent",
                "agent_codex",
                "--payload",
                "{\"raw\":\"event\"}",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/agent-events/ingest".to_string(),
                body: "{\"adapter\":\"raw-jsonl\",\"agentProfileId\":\"agent_codex\",\"payload\":{\"raw\":\"event\"},\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "provider-model", "get", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/provider-model".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "provider-model",
                "set",
                "--provider",
                "anthropic",
                "--model",
                "claude-3-7-sonnet-latest",
                "--agent",
                "agent_claude",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/provider-model".to_string(),
                body: "{\"agentProfileId\":\"agent_claude\",\"model\":\"claude-3-7-sonnet-latest\",\"provider\":\"anthropic\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "terminal-theme",
                "get",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/terminal-theme?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "terminal-theme",
                "set",
                "--project",
                "proj_auth",
                "--name",
                "Auth Focus",
                "--background",
                "#09111f",
                "--foreground",
                "#eaf6ff",
                "--accent",
                "#19c37d",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/terminal-theme".to_string(),
                body: "{\"accent\":\"#19c37d\",\"background\":\"#09111f\",\"foreground\":\"#eaf6ff\",\"name\":\"Auth Focus\",\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "agent",
                "register",
                "--id",
                "agent_acme",
                "--name",
                "Acme CLI",
                "--runtime",
                "generic-cli",
                "--command",
                "acme-agent",
                "--args",
                "[\"--json\"]",
                "--env-policy",
                "{\"inherit\":false,\"allow\":[\"ACME_API_KEY\"]}",
                "--skills",
                "[\"code-review\"]",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/agents".to_string(),
                body: "{\"argsJson\":[\"--json\"],\"command\":\"acme-agent\",\"envPolicyJson\":{\"allow\":[\"ACME_API_KEY\"],\"inherit\":false},\"id\":\"agent_acme\",\"name\":\"Acme CLI\",\"runtime\":\"generic-cli\",\"skillsJson\":[\"code-review\"],\"status\":\"available\"}".to_string()
            })
        );
    }

    #[test]
    fn parses_session_control_commands() {
        assert_eq!(
            parse_command(["hc", "session", "list", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/sessions".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "session", "list", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/sessions?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "session",
                "list",
                "--project",
                "proj_local",
                "--state",
                "running",
                "--task",
                "task_attached",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/sessions?projectId=proj_local&state=running&taskId=task_attached"
                    .to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "session",
                "new",
                "--title",
                "Codex AUTH-104",
                "--mode",
                "agent",
                "--cwd",
                "/repo/auth-service",
                "--branch",
                "fix/auth",
                "--agent",
                "agent_codex",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/sessions".to_string(),
                body: "{\"agentProfileId\":\"agent_codex\",\"branch\":\"fix/auth\",\"cwd\":\"/repo/auth-service\",\"mode\":\"agent\",\"projectId\":\"proj_local\",\"title\":\"Codex AUTH-104\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "session",
                "attention",
                "session_1",
                "--state",
                "unread",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/sessions/session_1/attention".to_string(),
                body: "{\"attentionState\":\"unread\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "session",
                "resize",
                "session_1",
                "--cols",
                "132",
                "--rows",
                "40",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/sessions/session_1/resize".to_string(),
                body: "{\"cols\":132,\"rows\":40}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "session",
                "new",
                "--title",
                "Codex AUTH-104",
                "--project",
                "proj_auth",
                "--mode",
                "agent",
                "--cwd",
                "/repo/auth-service",
                "--branch",
                "fix/auth",
                "--agent",
                "agent_codex",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/sessions".to_string(),
                body: "{\"agentProfileId\":\"agent_codex\",\"branch\":\"fix/auth\",\"cwd\":\"/repo/auth-service\",\"mode\":\"agent\",\"projectId\":\"proj_auth\",\"title\":\"Codex AUTH-104\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "session",
                "new",
                "--title",
                "SSH staging",
                "--mode",
                "ssh",
                "--ssh",
                "deploy@staging.example.com",
                "--cwd",
                "/srv/app",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/sessions".to_string(),
                body: "{\"agentProfileId\":null,\"branch\":null,\"cwd\":\"ssh://deploy@staging.example.com/srv/app\",\"mode\":\"ssh\",\"projectId\":\"proj_local\",\"title\":\"SSH staging\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "session", "focus", "session_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/sessions/session_1/focus".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "session", "usage", "session_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/sessions/session_1/token-usage".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "session",
                "attach-task",
                "session_1",
                "--task",
                "task_attached",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/sessions/session_1/attach-task".to_string(),
                body: "{\"taskId\":\"task_attached\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "session", "detach-task", "session_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/sessions/session_1/detach-task".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "session",
                "input",
                "session_1",
                "--text",
                "rm -rf /tmp/build",
                "--allow-dangerous",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/sessions/session_1/input".to_string(),
                body: "{\"allowDangerous\":true,\"text\":\"rm -rf /tmp/build\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "session", "stream", "list", "session_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/sessions/session_1/stream-chunks".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "session",
                "stream",
                "list",
                "session_1",
                "--limit",
                "25",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/sessions/session_1/stream-chunks?limit=25".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "session",
                "stream",
                "record",
                "session_1",
                "--seq-start",
                "1",
                "--seq-end",
                "4",
                "--body",
                "npm test\nPASS\n",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/sessions/session_1/stream-chunks".to_string(),
                body: "{\"body\":\"npm test\\nPASS\\n\",\"seqEnd\":4,\"seqStart\":1}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "session", "takeover", "session_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/sessions/session_1/takeover".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "session", "release", "session_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/sessions/session_1/release".to_string(),
                body: "{}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "session", "kill", "session_1", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/sessions/session_1/kill".to_string(),
                body: "{}".to_string()
            })
        );
    }

    #[test]
    fn parses_budget_status_and_set_commands() {
        assert_eq!(
            parse_command(["hc", "budget", "status", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/budgets".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "budget", "explain", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/budgets".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "budget", "dashboard", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/budgets".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "budget", "forecast", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/budgets/forecast".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "budget", "prices", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/provider-prices".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "budget",
                "prices",
                "update",
                "--source",
                "local-fixture",
                "--payload",
                "[{\"provider\":\"openai\",\"model\":\"gpt-5.4\",\"inputUsdPerMillion\":2.0,\"outputUsdPerMillion\":8.0}]",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/provider-prices/update".to_string(),
                body: "{\"prices\":[{\"inputUsdPerMillion\":2.0,\"model\":\"gpt-5.4\",\"outputUsdPerMillion\":8.0,\"provider\":\"openai\"}],\"source\":\"local-fixture\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "budget", "export", "--run", "run_1", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/runs/run_1/token-usage".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "budget",
                "record",
                "--provider",
                "openai",
                "--model",
                "gpt-5.4",
                "--input",
                "1200",
                "--output",
                "800",
                "--cost-usd",
                "8.5",
                "--source",
                "adapter",
                "--agent",
                "agent_codex",
                "--session",
                "session_1",
                "--task",
                "task_budget",
                "--run",
                "run_budget",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/token-usage".to_string(),
                body: "{\"agentProfileId\":\"agent_codex\",\"costUsd\":8.5,\"inputTokens\":1200,\"model\":\"gpt-5.4\",\"outputTokens\":800,\"projectId\":\"proj_local\",\"provider\":\"openai\",\"runId\":\"run_budget\",\"sessionId\":\"session_1\",\"source\":\"adapter\",\"taskId\":\"task_budget\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "budget",
                "record",
                "--project",
                "proj_auth",
                "--provider",
                "openai",
                "--model",
                "gpt-5.4",
                "--input",
                "1200",
                "--output",
                "800",
                "--cost-usd",
                "8.5",
                "--source",
                "adapter",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/token-usage".to_string(),
                body: "{\"agentProfileId\":null,\"costUsd\":8.5,\"inputTokens\":1200,\"model\":\"gpt-5.4\",\"outputTokens\":800,\"projectId\":\"proj_auth\",\"provider\":\"openai\",\"runId\":null,\"sessionId\":null,\"source\":\"adapter\",\"taskId\":null}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "budget",
                "set",
                "--scope",
                "project",
                "--id",
                "proj_local",
                "--max-usd",
                "10",
                "--warn-pct",
                "0.8",
                "--hard-limit",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/budgets".to_string(),
                body: "{\"hardLimit\":true,\"maxUsd\":10.0,\"scopeId\":\"proj_local\",\"scopeType\":\"project\",\"warnPct\":0.8}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "budget",
                "set",
                "--scope",
                "task",
                "--id",
                "task_budget",
                "--max-usd",
                "3",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/budgets".to_string(),
                body: "{\"hardLimit\":false,\"maxUsd\":3.0,\"scopeId\":\"task_budget\",\"scopeType\":\"task\",\"warnPct\":0.8}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "budget",
                "set",
                "--scope",
                "run",
                "--id",
                "run_budget",
                "--max-usd",
                "2",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/budgets".to_string(),
                body: "{\"hardLimit\":false,\"maxUsd\":2.0,\"scopeId\":\"run_budget\",\"scopeType\":\"run\",\"warnPct\":0.8}".to_string()
            })
        );
        assert!(
            budget_usage().contains("<workspace|project|goal|task|run|agent>"),
            "budget usage should document every supported scope"
        );
        assert_eq!(
            parse_command([
                "hc",
                "budget",
                "ingest",
                "--adapter",
                "openai.responses",
                "--agent",
                "agent_codex",
                "--payload",
                "{\"model\":\"gpt-5.4\",\"usage\":{\"input_tokens\":1200,\"output_tokens\":800},\"cost_usd\":8.5}",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/token-usage/ingest".to_string(),
                body: "{\"adapter\":\"openai.responses\",\"agentProfileId\":\"agent_codex\",\"payload\":{\"cost_usd\":8.5,\"model\":\"gpt-5.4\",\"usage\":{\"input_tokens\":1200,\"output_tokens\":800}},\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "budget",
                "ingest",
                "--project",
                "proj_auth",
                "--adapter",
                "openai.responses",
                "--payload",
                "{\"model\":\"gpt-5.4\"}",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/token-usage/ingest".to_string(),
                body: "{\"adapter\":\"openai.responses\",\"payload\":{\"model\":\"gpt-5.4\"},\"projectId\":\"proj_auth\"}".to_string()
            })
        );
    }

    #[test]
    fn parses_secret_list_and_set_commands() {
        assert_eq!(
            parse_command(["hc", "secret", "list", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/secrets".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "secret", "list", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/secrets?project=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "secret",
                "list",
                "--project",
                "proj_auth",
                "--name",
                "OPENAI_API_KEY",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/secrets?project=proj_auth&name=OPENAI_API_KEY".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "secret",
                "set",
                "--name",
                "OPENAI_API_KEY",
                "--value",
                "openai-secret-fixture-value",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/secrets".to_string(),
                body: "{\"name\":\"OPENAI_API_KEY\",\"projectId\":\"proj_local\",\"value\":\"openai-secret-fixture-value\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "secret",
                "set",
                "--project",
                "proj_auth",
                "--name",
                "ANTHROPIC_API_KEY",
                "--value",
                "anthropic-secret-fixture-value",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/secrets".to_string(),
                body: "{\"name\":\"ANTHROPIC_API_KEY\",\"projectId\":\"proj_auth\",\"value\":\"anthropic-secret-fixture-value\"}".to_string()
            })
        );
    }

    #[test]
    fn parses_knowledge_search_and_context_create_commands() {
        assert_eq!(
            parse_command(["hc", "knowledge", "sources", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/knowledge/sources".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "sources",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/knowledge/sources?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "source",
                "add",
                "--kind",
                "file",
                "--path",
                "docs/auth.md",
                "--fingerprint",
                "sha256:abc",
                "--status",
                "current",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/sources".to_string(),
                body: "{\"fingerprint\":\"sha256:abc\",\"kind\":\"file\",\"pathOrRef\":\"docs/auth.md\",\"projectId\":\"proj_local\",\"status\":\"current\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "source",
                "add",
                "--project",
                "proj_auth",
                "--kind",
                "file",
                "--path",
                "docs/auth.md",
                "--fingerprint",
                "sha256:abc",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/sources".to_string(),
                body: "{\"fingerprint\":\"sha256:abc\",\"kind\":\"file\",\"pathOrRef\":\"docs/auth.md\",\"projectId\":\"proj_auth\",\"status\":\"current\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "page",
                "create",
                "--slug",
                "auth-flow",
                "--title",
                "Auth Flow",
                "--body",
                "# Auth Flow",
                "--source",
                "ks_1",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/pages".to_string(),
                body: "{\"bodyMd\":\"# Auth Flow\",\"freshnessState\":\"current\",\"projectId\":\"proj_local\",\"slug\":\"auth-flow\",\"sourceIds\":[\"ks_1\"],\"title\":\"Auth Flow\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "page",
                "create",
                "--project",
                "proj_auth",
                "--slug",
                "auth-flow",
                "--title",
                "Auth Flow",
                "--body",
                "# Auth Flow",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/pages".to_string(),
                body: "{\"bodyMd\":\"# Auth Flow\",\"freshnessState\":\"current\",\"projectId\":\"proj_auth\",\"slug\":\"auth-flow\",\"sourceIds\":[],\"title\":\"Auth Flow\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "search",
                "--query",
                "auth flow",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/knowledge?query=auth+flow".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "search",
                "--project",
                "proj_auth",
                "--query",
                "auth flow",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/knowledge?projectId=proj_auth&query=auth+flow".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "knowledge", "open", "kp_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/knowledge/kp_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "chat",
                "--question",
                "How should token rollback work?",
                "--context",
                "ctx_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/chat".to_string(),
                body: "{\"contextPackId\":\"ctx_auth\",\"projectId\":\"proj_local\",\"question\":\"How should token rollback work?\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "chat",
                "--project",
                "proj_auth",
                "--question",
                "How should token rollback work?",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/chat".to_string(),
                body: "{\"contextPackId\":null,\"projectId\":\"proj_auth\",\"question\":\"How should token rollback work?\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "knowledge", "explorations", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/knowledge/explorations".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "explorations",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/knowledge/explorations?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "exploration",
                "create",
                "--title",
                "Token rollout investigation",
                "--question",
                "How should rollback handle token rotation?",
                "--answer",
                "Keep both issuers during rollback.",
                "--page",
                "kp_1",
                "--context",
                "ctx_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/explorations".to_string(),
                body: "{\"answerMd\":\"Keep both issuers during rollback.\",\"contextPackId\":\"ctx_auth\",\"pageIds\":[\"kp_1\"],\"projectId\":\"proj_local\",\"question\":\"How should rollback handle token rotation?\",\"title\":\"Token rollout investigation\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "exploration",
                "create",
                "--project",
                "proj_auth",
                "--title",
                "Token rollout investigation",
                "--question",
                "How should rollback handle token rotation?",
                "--answer",
                "Keep both issuers during rollback.",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/explorations".to_string(),
                body: "{\"answerMd\":\"Keep both issuers during rollback.\",\"contextPackId\":null,\"pageIds\":[],\"projectId\":\"proj_auth\",\"question\":\"How should rollback handle token rotation?\",\"title\":\"Token rollout investigation\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "knowledge", "concepts", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/knowledge/concepts".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "concepts",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/knowledge/concepts?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "knowledge", "obsidian", "export", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/obsidian/export".to_string(),
                body: "{\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "obsidian",
                "export",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/obsidian/export".to_string(),
                body: "{\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "lint",
                "--stale",
                "3",
                "--gaps",
                "2",
                "--contradictions",
                "1",
                "--body",
                "Gap: rollback path",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/lint".to_string(),
                body: "{\"bodyMd\":\"Gap: rollback path\",\"contradictionCount\":1,\"gapCount\":2,\"projectId\":\"proj_local\",\"staleCount\":3}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "lint",
                "--project",
                "proj_auth",
                "--stale",
                "3",
                "--gaps",
                "2",
                "--body",
                "Gap: rollback path",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/lint".to_string(),
                body: "{\"bodyMd\":\"Gap: rollback path\",\"contradictionCount\":0,\"gapCount\":2,\"projectId\":\"proj_auth\",\"staleCount\":3}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "knowledge", "compile", "--watch", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/automation/run".to_string(),
                body: "{\"projectId\":\"proj_local\",\"watch\":true}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "compile",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/automation/run".to_string(),
                body: "{\"projectId\":\"proj_auth\",\"watch\":false}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "ingest",
                "--path",
                "docs/runbook.pdf",
                "--kind",
                "pdf",
                "--title",
                "Runbook PDF",
                "--body",
                "Runbook body",
                "--chunk",
                "1200",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/ingest".to_string(),
                body: "{\"bodyMd\":\"Runbook body\",\"kind\":\"pdf\",\"maxChunkChars\":1200,\"pathOrRef\":\"docs/runbook.pdf\",\"projectId\":\"proj_local\",\"title\":\"Runbook PDF\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "knowledge",
                "ingest",
                "--project",
                "proj_auth",
                "--path",
                "docs/runbook.md",
                "--body",
                "Runbook body",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/knowledge/ingest".to_string(),
                body: "{\"bodyMd\":\"Runbook body\",\"kind\":\"markdown\",\"maxChunkChars\":null,\"pathOrRef\":\"docs/runbook.md\",\"projectId\":\"proj_auth\",\"title\":null}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "context", "list", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/context-packs".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "context", "list", "--project", "proj_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/context-packs?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "context", "show", "ctx_auth", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/context-packs/ctx_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "context",
                "create",
                "--id",
                "ctx_auth",
                "--name",
                "auth-default",
                "--source",
                "kp_1",
                "--max-tokens",
                "24000",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/context-packs".to_string(),
                body: "{\"id\":\"ctx_auth\",\"maxTokensHint\":24000,\"name\":\"auth-default\",\"projectId\":\"proj_local\",\"sourcesJson\":[{\"id\":\"kp_1\",\"type\":\"knowledge_page\"}]}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "context",
                "create",
                "--project",
                "proj_auth",
                "--name",
                "auth-default",
                "--source",
                "kp_1",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/context-packs".to_string(),
                body: "{\"id\":null,\"maxTokensHint\":null,\"name\":\"auth-default\",\"projectId\":\"proj_auth\",\"sourcesJson\":[{\"id\":\"kp_1\",\"type\":\"knowledge_page\"}]}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "context",
                "attach",
                "task_1",
                "--context",
                "ctx_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/tasks/task_1/context".to_string(),
                body: "{\"contextPackId\":\"ctx_auth\"}".to_string()
            })
        );
    }

    #[test]
    fn accepts_successful_created_http_responses() {
        assert_eq!(
            parse_http_body("HTTP/1.1 201 Created\r\ncontent-type: application/json\r\n\r\n{\"id\":\"ev_run_1\"}"),
            Ok("{\"id\":\"ev_run_1\"}".to_string())
        );
    }

    #[test]
    fn parses_workflow_status_and_reload_commands() {
        assert_eq!(
            parse_command(["hc", "workflow", "status", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/workflow/status".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "workflow",
                "status",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/workflow/status?projectId=proj_auth".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "workflow",
                "reload",
                "--path",
                "WORKFLOW.md",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/workflow/reload".to_string(),
                body: "{\"projectId\":\"proj_local\",\"sourcePath\":\"WORKFLOW.md\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "workflow",
                "validate",
                "--path",
                "WORKFLOW.md",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/workflow/validate".to_string(),
                body: "{\"projectId\":\"proj_local\",\"sourcePath\":\"WORKFLOW.md\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "workflow", "negative-tests", "--json"]),
            Ok(CliCommand::Post {
                path: "/v1/workflow/negative-tests/run".to_string(),
                body: "{\"projectId\":\"proj_local\"}".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "workflow",
                "negative-tests",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Post {
                path: "/v1/workflow/negative-tests/run".to_string(),
                body: "{\"projectId\":\"proj_auth\"}".to_string()
            })
        );
        assert_eq!(
            parse_command(["hc", "workflow", "negative-test-runs", "--json"]),
            Ok(CliCommand::Get {
                path: "/v1/workflow/negative-tests/runs".to_string()
            })
        );
        assert_eq!(
            parse_command([
                "hc",
                "workflow",
                "negative-test-runs",
                "--project",
                "proj_auth",
                "--json"
            ]),
            Ok(CliCommand::Get {
                path: "/v1/workflow/negative-tests/runs?projectId=proj_auth".to_string()
            })
        );
    }

    #[test]
    fn rejects_commands_without_json_flag() {
        assert_eq!(
            parse_command(["hc", "state"]),
            Err("only JSON output is available in the MVP CLI; pass --json".to_string())
        );
    }

    #[test]
    fn formats_local_http_get_requests() {
        assert_eq!(
            http_get_request("/v1/state"),
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi.local\r\naccept: application/json\r\nconnection: close\r\n\r\n"
        );
    }

    #[test]
    fn formats_local_http_post_requests() {
        assert_eq!(
            http_post_request("/v1/tasks", "{\"title\":\"New task\"}"),
            "POST /v1/tasks HTTP/1.1\r\nhost: haneulchi.local\r\naccept: application/json\r\ncontent-type: application/json\r\ncontent-length: 20\r\nconnection: close\r\n\r\n{\"title\":\"New task\"}"
        );
    }

    #[test]
    fn formats_local_http_patch_requests() {
        assert_eq!(
            http_patch_request("/v1/tasks/task_1", "{\"title\":\"Updated task\"}"),
            "PATCH /v1/tasks/task_1 HTTP/1.1\r\nhost: haneulchi.local\r\naccept: application/json\r\ncontent-type: application/json\r\ncontent-length: 24\r\nconnection: close\r\n\r\n{\"title\":\"Updated task\"}"
        );
    }

    #[test]
    fn extracts_successful_http_response_body() {
        assert_eq!(
            parse_http_body("HTTP/1.1 200 OK\r\ncontent-length: 11\r\n\r\n{\"ok\":true}"),
            Ok("{\"ok\":true}".to_string())
        );
    }
}
