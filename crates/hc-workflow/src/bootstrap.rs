use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::contract::{HookPhase, LoadedWorkflow};
use crate::hooks::run_hook;
use crate::template::{RenderContext, render_template};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapRequest {
    pub workflow: LoadedWorkflow,
    pub project_name: String,
    pub task_id: String,
    pub task_title: String,
    pub repo_root: PathBuf,
    pub workspace_root: PathBuf,
    pub launch_program: String,
    pub launch_args: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrepareBootstrapRequest {
    pub workflow: LoadedWorkflow,
    pub project_name: String,
    pub task_id: String,
    pub task_title: String,
    pub repo_root: PathBuf,
    pub workspace_root: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HookPhaseResult {
    pub phase: String,
    pub command_path: Option<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub succeeded: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct BootstrapStatusSummary {
    pub workspace_root: String,
    pub base_root: String,
    pub session_cwd: String,
    pub rendered_prompt_path: String,
    pub phase_sequence: Vec<String>,
    pub hook_phase_results: Vec<HookPhaseResult>,
    pub outcome_code: String,
    pub warning_codes: Vec<String>,
    pub claim_released: bool,
    pub launch_exit_code: Option<i32>,
    pub last_known_good_hash: Option<String>,
}

pub type BootstrapResult = BootstrapStatusSummary;

pub fn prepare_bootstrap(request: PrepareBootstrapRequest) -> Result<BootstrapResult, String> {
    let mut phases = vec!["resolve".to_string(), "normalize".to_string()];
    let session_id = format!("bootstrap-{}", request.task_id);
    let now_iso8601 = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| error.to_string())?;
    let workspace_root = request.workspace_root.clone();
    fs::create_dir_all(&workspace_root).map_err(|error| error.to_string())?;
    phases.push("workspace".to_string());

    let base_root = request
        .workflow
        .effective_config
        .workspace
        .base_root
        .clone();
    let session_cwd = if base_root == "." {
        workspace_root.clone()
    } else {
        workspace_root.join(&base_root)
    };
    fs::create_dir_all(&session_cwd).map_err(|error| error.to_string())?;
    phases.push("paths".to_string());

    let env = bootstrap_env_prepare(
        &request,
        &workspace_root,
        &session_cwd,
        &session_id,
        &now_iso8601,
    );
    let mut hook_phase_results = Vec::new();
    let mut warning_codes = Vec::new();
    let last_known_good_hash = Some(request.workflow.contract_hash.clone());

    if let Some((result, optional)) = run_prepare_phase_if_present(
        &request,
        HookPhase::AfterCreate,
        &workspace_root,
        &session_cwd,
        &env,
        &mut hook_phase_results,
    ) {
        phases.push("after_create".to_string());
        if !result.succeeded {
            if optional {
                warning_codes.push("after_create_failed_optional".to_string());
            } else {
                return Ok(BootstrapStatusSummary {
                    workspace_root: workspace_root.display().to_string(),
                    base_root,
                    session_cwd: session_cwd.display().to_string(),
                    rendered_prompt_path: String::new(),
                    phase_sequence: phases,
                    hook_phase_results,
                    outcome_code: "workflow_setup_failed".to_string(),
                    warning_codes,
                    claim_released: true,
                    launch_exit_code: None,
                    last_known_good_hash,
                });
            }
        }
    } else {
        phases.push("after_create".to_string());
    }

    let rendered_prompt_path = session_cwd.join("prompt.rendered.md");
    let rendered_prompt = render_prepare_prompt(
        &request,
        &workspace_root,
        &session_cwd,
        &session_id,
        &now_iso8601,
    )?;
    fs::write(&rendered_prompt_path, rendered_prompt).map_err(|error| error.to_string())?;
    phases.push("prompt".to_string());

    if let Some((result, optional)) = run_prepare_phase_if_present(
        &request,
        HookPhase::BeforeRun,
        &workspace_root,
        &session_cwd,
        &env,
        &mut hook_phase_results,
    ) {
        phases.push("before_run".to_string());
        if !result.succeeded {
            if optional {
                warning_codes.push("before_run_failed_optional".to_string());
            } else {
                return Ok(BootstrapStatusSummary {
                    workspace_root: workspace_root.display().to_string(),
                    base_root,
                    session_cwd: session_cwd.display().to_string(),
                    rendered_prompt_path: rendered_prompt_path.display().to_string(),
                    phase_sequence: phases,
                    hook_phase_results,
                    outcome_code: "workflow_setup_failed".to_string(),
                    warning_codes,
                    claim_released: true,
                    launch_exit_code: None,
                    last_known_good_hash,
                });
            }
        }
    } else {
        phases.push("before_run".to_string());
    }

    Ok(BootstrapStatusSummary {
        workspace_root: workspace_root.display().to_string(),
        base_root,
        session_cwd: session_cwd.display().to_string(),
        rendered_prompt_path: rendered_prompt_path.display().to_string(),
        phase_sequence: phases,
        hook_phase_results,
        outcome_code: "launch_prepared".to_string(),
        warning_codes,
        claim_released: false,
        launch_exit_code: None,
        last_known_good_hash,
    })
}

pub fn run_bootstrap(request: BootstrapRequest) -> Result<BootstrapResult, String> {
    let mut phases = vec!["resolve".to_string(), "normalize".to_string()];
    let session_id = format!("bootstrap-{}", request.task_id);
    let now_iso8601 = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| error.to_string())?;
    let workspace_root = request.workspace_root.clone();
    fs::create_dir_all(&workspace_root).map_err(|error| error.to_string())?;
    phases.push("workspace".to_string());

    let base_root = request
        .workflow
        .effective_config
        .workspace
        .base_root
        .clone();
    let session_cwd = if base_root == "." {
        workspace_root.clone()
    } else {
        workspace_root.join(&base_root)
    };
    fs::create_dir_all(&session_cwd).map_err(|error| error.to_string())?;
    phases.push("paths".to_string());

    let env = bootstrap_env(
        &request,
        &workspace_root,
        &session_cwd,
        &session_id,
        &now_iso8601,
    );
    let mut hook_phase_results = Vec::new();
    let mut warning_codes = Vec::new();
    let last_known_good_hash = Some(request.workflow.contract_hash.clone());

    if let Some((result, optional)) = run_phase_if_present(
        &request,
        HookPhase::AfterCreate,
        &workspace_root,
        &session_cwd,
        &env,
        &mut hook_phase_results,
    ) {
        phases.push("after_create".to_string());
        if !result.succeeded {
            if optional {
                warning_codes.push("after_create_failed_optional".to_string());
            } else {
                return Ok(BootstrapStatusSummary {
                    workspace_root: workspace_root.display().to_string(),
                    base_root,
                    session_cwd: session_cwd.display().to_string(),
                    rendered_prompt_path: String::new(),
                    phase_sequence: phases,
                    hook_phase_results,
                    outcome_code: "workflow_setup_failed".to_string(),
                    warning_codes,
                    claim_released: true,
                    launch_exit_code: None,
                    last_known_good_hash,
                });
            }
        }
    } else {
        phases.push("after_create".to_string());
    }

    let rendered_prompt_path = session_cwd.join("prompt.rendered.md");
    let rendered_prompt =
        render_prompt(&request, &workspace_root, &session_cwd, &session_id, &now_iso8601)?;
    fs::write(&rendered_prompt_path, rendered_prompt).map_err(|error| error.to_string())?;
    phases.push("prompt".to_string());

    if let Some((result, optional)) = run_phase_if_present(
        &request,
        HookPhase::BeforeRun,
        &workspace_root,
        &session_cwd,
        &env,
        &mut hook_phase_results,
    ) {
        phases.push("before_run".to_string());
        if !result.succeeded {
            if optional {
                warning_codes.push("before_run_failed_optional".to_string());
            } else {
                return Ok(BootstrapStatusSummary {
                    workspace_root: workspace_root.display().to_string(),
                    base_root,
                    session_cwd: session_cwd.display().to_string(),
                    rendered_prompt_path: rendered_prompt_path.display().to_string(),
                    phase_sequence: phases,
                    hook_phase_results,
                    outcome_code: "workflow_setup_failed".to_string(),
                    warning_codes,
                    claim_released: true,
                    launch_exit_code: None,
                    last_known_good_hash,
                });
            }
        }
    } else {
        phases.push("before_run".to_string());
    }

    let launch_output = Command::new(&request.launch_program)
        .current_dir(&session_cwd)
        .args(&request.launch_args)
        .envs(&env)
        .output()
        .map_err(|error| error.to_string())?;
    phases.push("launch".to_string());
    let launch_exit_code = launch_output.status.code();

    if let Some((result, _optional)) = run_phase_if_present(
        &request,
        HookPhase::AfterRun,
        &workspace_root,
        &session_cwd,
        &env,
        &mut hook_phase_results,
    ) {
        phases.push("after_run".to_string());
        if !result.succeeded {
            warning_codes.push("after_run_failed".to_string());
        }
    } else {
        phases.push("after_run".to_string());
    }

    let outcome_code = if launch_output.status.success() {
        "launch_succeeded"
    } else {
        "launch_failed"
    };

    let result = BootstrapStatusSummary {
        workspace_root: workspace_root.display().to_string(),
        base_root,
        session_cwd: session_cwd.display().to_string(),
        rendered_prompt_path: rendered_prompt_path.display().to_string(),
        phase_sequence: {
            let mut phases = phases;
            phases.push("evidence".to_string());
            phases
        },
        hook_phase_results,
        outcome_code: outcome_code.to_string(),
        warning_codes,
        claim_released: false,
        launch_exit_code,
        last_known_good_hash,
    };

    let evidence_path = session_cwd.join("bootstrap.report.json");
    fs::write(
        evidence_path,
        serde_json::to_string_pretty(&result).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    Ok(result)
}

fn run_phase_if_present(
    request: &BootstrapRequest,
    phase: HookPhase,
    workspace_root: &Path,
    session_cwd: &Path,
    env: &BTreeMap<String, String>,
    results: &mut Vec<HookPhaseResult>,
) -> Option<(HookPhaseResult, bool)> {
    let definition = request.workflow.effective_config.hooks.hook(phase)?;
    let path = match phase {
        HookPhase::AfterCreate => request.workflow.resolved_paths.after_create.as_ref(),
        HookPhase::BeforeRun => request.workflow.resolved_paths.before_run.as_ref(),
        HookPhase::AfterRun => request.workflow.resolved_paths.after_run.as_ref(),
    }?;
    let mirrored_path = match mirror_hook_path(&request.repo_root, workspace_root, path) {
        Ok(path) => path,
        Err(error) => {
            let result = HookPhaseResult {
                phase: match phase {
                    HookPhase::AfterCreate => "after_create".to_string(),
                    HookPhase::BeforeRun => "before_run".to_string(),
                    HookPhase::AfterRun => "after_run".to_string(),
                },
                command_path: Some(path.display().to_string()),
                exit_code: None,
                stdout: String::new(),
                stderr: error,
                succeeded: false,
            };
            results.push(result.clone());
            return Some((result, definition.optional));
        }
    };
    let mut phase_env = env.clone();
    phase_env.insert("HC_HOOK_PHASE".to_string(), phase_name(phase).to_string());
    phase_env.insert(
        "HANEULCHI_HOOK_PHASE".to_string(),
        phase_name(phase).to_string(),
    );
    let result = run_hook(phase, definition, &mirrored_path, session_cwd, &phase_env);
    results.push(result.clone());
    Some((result, definition.optional))
}

fn run_prepare_phase_if_present(
    request: &PrepareBootstrapRequest,
    phase: HookPhase,
    workspace_root: &Path,
    session_cwd: &Path,
    env: &BTreeMap<String, String>,
    results: &mut Vec<HookPhaseResult>,
) -> Option<(HookPhaseResult, bool)> {
    let definition = request.workflow.effective_config.hooks.hook(phase)?;
    let path = match phase {
        HookPhase::AfterCreate => request.workflow.resolved_paths.after_create.as_ref(),
        HookPhase::BeforeRun => request.workflow.resolved_paths.before_run.as_ref(),
        HookPhase::AfterRun => request.workflow.resolved_paths.after_run.as_ref(),
    }?;
    let mirrored_path = match mirror_hook_path(&request.repo_root, workspace_root, path) {
        Ok(path) => path,
        Err(error) => {
            let result = HookPhaseResult {
                phase: phase_name(phase).to_string(),
                command_path: Some(path.display().to_string()),
                exit_code: None,
                stdout: String::new(),
                stderr: error,
                succeeded: false,
            };
            results.push(result.clone());
            return Some((result, definition.optional));
        }
    };
    let mut phase_env = env.clone();
    phase_env.insert("HC_HOOK_PHASE".to_string(), phase_name(phase).to_string());
    phase_env.insert(
        "HANEULCHI_HOOK_PHASE".to_string(),
        phase_name(phase).to_string(),
    );
    let result = run_hook(phase, definition, &mirrored_path, session_cwd, &phase_env);
    results.push(result.clone());
    Some((result, definition.optional))
}

fn mirror_hook_path(
    repo_root: &Path,
    workspace_root: &Path,
    source_path: &Path,
) -> Result<PathBuf, String> {
    let relative_path = source_path
        .strip_prefix(repo_root)
        .map_err(|_| format!("hook path outside repo root: {}", source_path.display()))?;
    let mirrored_path = workspace_root.join(relative_path);

    if let Some(parent) = mirrored_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::copy(source_path, &mirrored_path).map_err(|error| error.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let source_mode = fs::metadata(source_path)
            .map_err(|error| error.to_string())?
            .permissions()
            .mode();
        fs::set_permissions(&mirrored_path, fs::Permissions::from_mode(source_mode))
            .map_err(|error| error.to_string())?;
    }

    Ok(mirrored_path)
}

fn render_prompt(
    request: &BootstrapRequest,
    workspace_root: &Path,
    session_cwd: &Path,
    session_id: &str,
    now_iso8601: &str,
) -> Result<String, String> {
    let workflow_name = request
        .workflow
        .effective_config
        .workflow
        .name
        .clone()
        .unwrap_or_default();
    let review_checklist_markdown = if request.workflow.effective_config.review.checklist.is_empty() {
        String::new()
    } else {
        request
            .workflow
            .effective_config
            .review
            .checklist
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let review_checklist_plain = request
        .workflow
        .effective_config
        .review
        .checklist
        .join("\n");
    let now_date = now_iso8601.chars().take(10).collect::<String>();

    let context = RenderContext::default()
        .with("task.id", request.task_id.clone())
        .with("task.key", request.task_id.clone())
        .with("task.title", request.task_title.clone())
        .with("task.description", "")
        .with("task.description_markdown", "")
        .with("task.priority", "")
        .with("task.column", "")
        .with("task.labels_csv", "")
        .with("project.id", request.project_name.clone())
        .with("project.name", request.project_name.clone())
        .with("project.repo_root", request.repo_root.display().to_string())
        .with("session.id", session_id.to_string())
        .with("session.mode", "preset")
        .with("session.workspace_root", workspace_root.display().to_string())
        .with("session.cwd", session_cwd.display().to_string())
        .with("session.branch", "")
        .with("session.adapter_name", "")
        .with("workflow.name", workflow_name)
        .with(
            "workflow.max_slots",
            request
                .workflow
                .effective_config
                .workflow
                .max_slots
                .map(|value| value.to_string())
                .unwrap_or_default(),
        )
        .with("workflow.contract_hash", request.workflow.contract_hash.clone())
        .with(
            "review.required",
            if request.workflow.effective_config.review.required {
                "true"
            } else {
                "false"
            },
        )
        .with("review.checklist_markdown", review_checklist_markdown)
        .with("review.checklist_plain", review_checklist_plain)
        .with("now.iso8601", now_iso8601.to_string())
        .with("now.date", now_date);

    render_template(&request.workflow.template_body, &context).map_err(|error| error.to_string())
}

fn render_prepare_prompt(
    request: &PrepareBootstrapRequest,
    workspace_root: &Path,
    session_cwd: &Path,
    session_id: &str,
    now_iso8601: &str,
) -> Result<String, String> {
    let workflow_name = request
        .workflow
        .effective_config
        .workflow
        .name
        .clone()
        .unwrap_or_default();
    let review_checklist_markdown = if request.workflow.effective_config.review.checklist.is_empty() {
        String::new()
    } else {
        request
            .workflow
            .effective_config
            .review
            .checklist
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let review_checklist_plain = request
        .workflow
        .effective_config
        .review
        .checklist
        .join("\n");
    let now_date = now_iso8601.chars().take(10).collect::<String>();

    let context = RenderContext::default()
        .with("task.id", request.task_id.clone())
        .with("task.key", request.task_id.clone())
        .with("task.title", request.task_title.clone())
        .with("task.description", "")
        .with("task.description_markdown", "")
        .with("task.priority", "")
        .with("task.column", "")
        .with("task.labels_csv", "")
        .with("project.id", request.project_name.clone())
        .with("project.name", request.project_name.clone())
        .with("project.repo_root", request.repo_root.display().to_string())
        .with("session.id", session_id.to_string())
        .with("session.mode", "preset")
        .with("session.workspace_root", workspace_root.display().to_string())
        .with("session.cwd", session_cwd.display().to_string())
        .with("session.branch", "")
        .with("session.adapter_name", "")
        .with("workflow.name", workflow_name)
        .with(
            "workflow.max_slots",
            request
                .workflow
                .effective_config
                .workflow
                .max_slots
                .map(|value| value.to_string())
                .unwrap_or_default(),
        )
        .with("workflow.contract_hash", request.workflow.contract_hash.clone())
        .with(
            "review.required",
            if request.workflow.effective_config.review.required {
                "true"
            } else {
                "false"
            },
        )
        .with("review.checklist_markdown", review_checklist_markdown)
        .with("review.checklist_plain", review_checklist_plain)
        .with("now.iso8601", now_iso8601.to_string())
        .with("now.date", now_date);

    render_template(&request.workflow.template_body, &context).map_err(|error| error.to_string())
}

fn bootstrap_env(
    request: &BootstrapRequest,
    workspace_root: &Path,
    session_cwd: &Path,
    session_id: &str,
    now_iso8601: &str,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("HC_TASK_ID".to_string(), request.task_id.clone()),
        ("HC_TASK_KEY".to_string(), request.task_id.clone()),
        ("HC_TASK_TITLE".to_string(), request.task_title.clone()),
        ("HC_PROJECT_ID".to_string(), request.project_name.clone()),
        ("HC_PROJECT_NAME".to_string(), request.project_name.clone()),
        (
            "HC_REPO_ROOT".to_string(),
            request.repo_root.display().to_string(),
        ),
        ("HC_WORKSPACE_ROOT".to_string(), workspace_root.display().to_string()),
        ("HC_SESSION_CWD".to_string(), session_cwd.display().to_string()),
        ("HC_SESSION_ID".to_string(), session_id.to_string()),
        (
            "HC_WORKFLOW_FILE".to_string(),
            request.workflow.discovery_path.display().to_string(),
        ),
        (
            "HC_WORKFLOW_HASH".to_string(),
            request.workflow.contract_hash.clone(),
        ),
        (
            "HC_BASE_ROOT".to_string(),
            request
                .workflow
                .effective_config
                .workspace
                .base_root
                .clone(),
        ),
        ("HC_NOW_ISO8601".to_string(), now_iso8601.to_string()),
        ("HANEULCHI_TASK_ID".to_string(), request.task_id.clone()),
        (
            "HANEULCHI_TASK_TITLE".to_string(),
            request.task_title.clone(),
        ),
        (
            "HANEULCHI_WORKSPACE_ROOT".to_string(),
            workspace_root.display().to_string(),
        ),
        (
            "HANEULCHI_SESSION_CWD".to_string(),
            session_cwd.display().to_string(),
        ),
        (
            "HANEULCHI_WORKFLOW_FILE".to_string(),
            request.workflow.discovery_path.display().to_string(),
        ),
        (
            "HANEULCHI_BASE_ROOT".to_string(),
            request
                .workflow
                .effective_config
                .workspace
                .base_root
                .clone(),
        ),
    ])
}

fn bootstrap_env_prepare(
    request: &PrepareBootstrapRequest,
    workspace_root: &Path,
    session_cwd: &Path,
    session_id: &str,
    now_iso8601: &str,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("HC_TASK_ID".to_string(), request.task_id.clone()),
        ("HC_TASK_KEY".to_string(), request.task_id.clone()),
        ("HC_TASK_TITLE".to_string(), request.task_title.clone()),
        ("HC_PROJECT_ID".to_string(), request.project_name.clone()),
        ("HC_PROJECT_NAME".to_string(), request.project_name.clone()),
        (
            "HC_REPO_ROOT".to_string(),
            request.repo_root.display().to_string(),
        ),
        ("HC_WORKSPACE_ROOT".to_string(), workspace_root.display().to_string()),
        ("HC_SESSION_CWD".to_string(), session_cwd.display().to_string()),
        ("HC_SESSION_ID".to_string(), session_id.to_string()),
        (
            "HC_WORKFLOW_FILE".to_string(),
            request.workflow.discovery_path.display().to_string(),
        ),
        (
            "HC_WORKFLOW_HASH".to_string(),
            request.workflow.contract_hash.clone(),
        ),
        (
            "HC_BASE_ROOT".to_string(),
            request
                .workflow
                .effective_config
                .workspace
                .base_root
                .clone(),
        ),
        ("HC_NOW_ISO8601".to_string(), now_iso8601.to_string()),
        ("HANEULCHI_TASK_ID".to_string(), request.task_id.clone()),
        (
            "HANEULCHI_TASK_TITLE".to_string(),
            request.task_title.clone(),
        ),
        (
            "HANEULCHI_WORKSPACE_ROOT".to_string(),
            workspace_root.display().to_string(),
        ),
        (
            "HANEULCHI_SESSION_CWD".to_string(),
            session_cwd.display().to_string(),
        ),
        (
            "HANEULCHI_WORKFLOW_FILE".to_string(),
            request.workflow.discovery_path.display().to_string(),
        ),
        (
            "HANEULCHI_BASE_ROOT".to_string(),
            request
                .workflow
                .effective_config
                .workspace
                .base_root
                .clone(),
        ),
    ])
}

fn phase_name(phase: HookPhase) -> &'static str {
    match phase {
        HookPhase::AfterCreate => "after_create",
        HookPhase::BeforeRun => "before_run",
        HookPhase::AfterRun => "after_run",
    }
}
