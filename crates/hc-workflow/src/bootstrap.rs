use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::contract::{HookPhase, LoadedWorkflow};
use crate::hooks::run_hook;

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

pub fn run_bootstrap(request: BootstrapRequest) -> Result<BootstrapResult, String> {
    let mut phases = vec!["resolve".to_string(), "normalize".to_string()];
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

    let env = bootstrap_env(&request, &workspace_root, &session_cwd);
    let mut hook_phase_results = Vec::new();
    let mut warning_codes = Vec::new();
    let last_known_good_hash = Some(request.workflow.contract_hash.clone());

    if let Some(result) = run_phase_if_present(
        &request,
        HookPhase::AfterCreate,
        &workspace_root,
        &session_cwd,
        &env,
        &mut hook_phase_results,
    ) {
        phases.push("after_create".to_string());
        if !result.succeeded {
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
    } else {
        phases.push("after_create".to_string());
    }

    let rendered_prompt_path = session_cwd.join("prompt.rendered.md");
    let rendered_prompt = render_prompt(&request, &workspace_root, &session_cwd);
    fs::write(&rendered_prompt_path, rendered_prompt).map_err(|error| error.to_string())?;
    phases.push("prompt".to_string());

    if let Some(result) = run_phase_if_present(
        &request,
        HookPhase::BeforeRun,
        &workspace_root,
        &session_cwd,
        &env,
        &mut hook_phase_results,
    ) {
        phases.push("before_run".to_string());
        if !result.succeeded {
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

    if let Some(result) = run_phase_if_present(
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
) -> Option<HookPhaseResult> {
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
            return Some(result);
        }
    };
    let result = run_hook(phase, definition, &mirrored_path, session_cwd, env);
    results.push(result.clone());
    Some(result)
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

fn render_prompt(request: &BootstrapRequest, workspace_root: &Path, session_cwd: &Path) -> String {
    let workflow_name = request
        .workflow
        .effective_config
        .workflow
        .name
        .clone()
        .unwrap_or_default();
    request
        .workflow
        .template_body
        .replace("{{task.title}}", &request.task_title)
        .replace("{{task.id}}", &request.task_id)
        .replace("{{project.name}}", &request.project_name)
        .replace(
            "{{project.repo_root}}",
            &request.repo_root.display().to_string(),
        )
        .replace("{{workflow.name}}", &workflow_name)
        .replace(
            "{{session.workspace_root}}",
            &workspace_root.display().to_string(),
        )
        .replace("{{session.cwd}}", &session_cwd.display().to_string())
}

fn bootstrap_env(
    request: &BootstrapRequest,
    workspace_root: &Path,
    session_cwd: &Path,
) -> BTreeMap<String, String> {
    BTreeMap::from([
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
