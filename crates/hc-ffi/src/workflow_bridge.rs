use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use hc_control_plane::{reload_workflow, validate_workflow};
use hc_workflow::WorkflowState;

use crate::HcString;

fn read_c_string(value: *const c_char) -> Result<String, String> {
    if value.is_null() {
        return Err("null pointer".to_string());
    }

    let text = unsafe { CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| error.to_string())?;

    Ok(text.to_string())
}

fn string_to_hcstring(value: Result<String, String>) -> HcString {
    let payload = match value {
        Ok(value) => value,
        Err(error) => serde_json::json!({ "error": error }).to_string(),
    };
    let string = CString::new(payload).expect("json payload is nul-free");

    HcString {
        ptr: string.into_raw(),
    }
}

pub fn workflow_validate_json(project_root: &str) -> Result<String, String> {
    match validate_workflow(project_root.to_string()).map_err(|error| error.to_string())? {
        Some(loaded) => {
            let hooks: Vec<&str> = [
                loaded
                    .effective_config
                    .hooks
                    .after_create
                    .as_ref()
                    .map(|_| "after_create"),
                loaded
                    .effective_config
                    .hooks
                    .before_run
                    .as_ref()
                    .map(|_| "before_run"),
                loaded
                    .effective_config
                    .hooks
                    .after_run
                    .as_ref()
                    .map(|_| "after_run"),
            ]
            .into_iter()
            .flatten()
            .collect();

            serde_json::to_string(&serde_json::json!({
                "state": "ok",
                "path": loaded.discovery_path,
                "last_good_hash": loaded.contract_hash,
                "last_reload_at": serde_json::Value::Null,
                "last_error": serde_json::Value::Null,
                "last_bootstrap": serde_json::Value::Null,
                "workflow": {
                    "name": loaded.effective_config.workflow.name,
                    "strategy": match loaded.effective_config.workspace.strategy {
                        hc_workflow::WorkspaceStrategy::Worktree => "worktree",
                        hc_workflow::WorkspaceStrategy::SharedRoot => "shared_root",
                    },
                    "base_root": loaded.effective_config.workspace.base_root,
                    "require_review": loaded.effective_config.review.required,
                    "max_runtime_minutes": loaded.effective_config.policy.max_runtime_minutes,
                    "unsafe_override_policy": loaded.effective_config.policy.unsafe_override_policy,
                    "review_checklist": loaded.effective_config.review.checklist,
                    "allowed_agents": loaded.effective_config.agents.allowed,
                    "hooks": hooks,
                    "hook_runs": {
                        "after_create": loaded.resolved_paths.after_create,
                        "before_run": loaded.resolved_paths.before_run,
                        "after_run": loaded.resolved_paths.after_run
                    },
                    "template_body": loaded.template_body
                }
            }))
            .map_err(|error| error.to_string())
        }
        None => serde_json::to_string(&serde_json::json!({ "state": "none" }))
            .map_err(|error| error.to_string()),
    }
}

pub fn workflow_reload_json(project_root: &str) -> Result<String, String> {
    let runtime = reload_workflow(project_root.to_string()).map_err(|error| error.to_string())?;
    let loaded = runtime.current();
    let hooks: Vec<&str> = loaded
        .map(|loaded| {
            [
                loaded
                    .effective_config
                    .hooks
                    .after_create
                    .as_ref()
                    .map(|_| "after_create"),
                loaded
                    .effective_config
                    .hooks
                    .before_run
                    .as_ref()
                    .map(|_| "before_run"),
                loaded
                    .effective_config
                    .hooks
                    .after_run
                    .as_ref()
                    .map(|_| "after_run"),
            ]
            .into_iter()
            .flatten()
            .collect()
        })
        .unwrap_or_default();
    serde_json::to_string(&serde_json::json!({
        "state": match runtime.state() {
            WorkflowState::None => "none",
            WorkflowState::Ok => "ok",
            WorkflowState::InvalidKeptLastGood => "invalid_kept_last_good",
            WorkflowState::ReloadPending => "reload_pending",
        },
        "path": loaded.map(|loaded| loaded.discovery_path.clone()),
        "last_good_hash": runtime
            .last_known_good()
            .map(|loaded| loaded.contract_hash.clone()),
        "last_reload_at": runtime.last_reload_at(),
        "last_error": runtime.last_error(),
        "last_bootstrap": runtime.last_bootstrap().map(|summary| serde_json::json!({
            "workspace_root": summary.workspace_root,
            "base_root": summary.base_root,
            "session_cwd": summary.session_cwd,
            "rendered_prompt_path": summary.rendered_prompt_path,
            "phase_sequence": summary.phase_sequence,
            "hook_phase_results": summary.hook_phase_results,
            "outcome_code": summary.outcome_code,
            "warning_codes": summary.warning_codes,
            "claim_released": summary.claim_released,
            "launch_exit_code": summary.launch_exit_code,
            "last_known_good_hash": summary.last_known_good_hash,
        })),
        "workflow": loaded.map(|loaded| serde_json::json!({
            "name": loaded.effective_config.workflow.name,
            "strategy": match loaded.effective_config.workspace.strategy {
                hc_workflow::WorkspaceStrategy::Worktree => "worktree",
                hc_workflow::WorkspaceStrategy::SharedRoot => "shared_root",
                },
                "base_root": loaded.effective_config.workspace.base_root,
                "require_review": loaded.effective_config.review.required,
                "max_runtime_minutes": loaded.effective_config.policy.max_runtime_minutes,
                "unsafe_override_policy": loaded.effective_config.policy.unsafe_override_policy,
                "review_checklist": loaded.effective_config.review.checklist,
                "allowed_agents": loaded.effective_config.agents.allowed,
                "hooks": hooks,
                "hook_runs": {
                    "after_create": loaded.resolved_paths.after_create,
                    "before_run": loaded.resolved_paths.before_run,
                    "after_run": loaded.resolved_paths.after_run
                },
                "template_body": loaded.template_body
        }))
    }))
    .map_err(|error| error.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_workflow_validate_json(project_root: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(project_root).and_then(|root| workflow_validate_json(&root)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_workflow_reload_json(project_root: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(project_root).and_then(|root| workflow_reload_json(&root)))
}
