mod control_api;
mod pty;
mod readiness;
mod state_snapshot;
pub mod state_store;

use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
struct ReleaseWorkflowDiagnostic {
    id: String,
    label: String,
    script: String,
    configured: bool,
    required_env: Vec<String>,
    missing_env: Vec<String>,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
struct ReleaseWorkflowStatus {
    status: String,
    workflows: Vec<ReleaseWorkflowDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
struct AgentTerminalLaunch {
    session: state_store::PersistedSession,
    pty_session: pty::TerminalPtySession,
}

#[tauri::command]
fn get_readiness_snapshot() -> readiness::ReadinessSnapshot {
    readiness::collect_readiness_snapshot()
}

#[tauri::command]
fn get_release_workflow_status() -> Result<ReleaseWorkflowStatus, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "failed to resolve workspace root".to_string())?;
    let package_json_path = root.join("package.json");
    let package_json = fs::read_to_string(&package_json_path)
        .map_err(|error| format!("failed to read package.json: {error}"))?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json)
        .map_err(|error| format!("failed to parse package.json: {error}"))?;
    let scripts = package_json
        .get("scripts")
        .and_then(serde_json::Value::as_object)
        .map(|scripts| {
            scripts
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|value| (key.clone(), value.to_string()))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let workflow_specs = [
        (
            "macos_dmg",
            "Signed macOS DMG",
            "release:macos:dmg",
            ".github/workflows/release-macos.yml",
            [
                "APPLE_CERTIFICATE",
                "APPLE_CERTIFICATE_PASSWORD",
                "APPLE_SIGNING_IDENTITY",
                "KEYCHAIN_PASSWORD",
            ]
            .as_slice(),
        ),
        (
            "macos_notarization",
            "Apple notarization",
            "release:macos:notarize",
            "scripts/release/notarize-macos.sh",
            ["APPLE_ID", "APPLE_PASSWORD", "APPLE_TEAM_ID"].as_slice(),
        ),
        (
            "macos_artifact_verification",
            "macOS artifact verification",
            "release:macos:verify",
            "scripts/release/verify-macos-artifacts.sh",
            &[] as &[&str],
        ),
        (
            "homebrew_cask",
            "Homebrew cask",
            "release:homebrew:cask",
            "scripts/release/render-homebrew-cask.sh",
            ["DMG_URL", "HOMEBREW_TAP_REPOSITORY"].as_slice(),
        ),
        (
            "crash_symbols",
            "Crash symbols",
            "release:symbols:upload",
            "scripts/release/upload-symbols.sh",
            ["SENTRY_AUTH_TOKEN", "SENTRY_ORG", "SENTRY_PROJECT"].as_slice(),
        ),
    ];

    let workflows = workflow_specs
        .iter()
        .map(|(id, label, script, file, required_env)| {
            let script_configured = scripts.contains_key(*script);
            let file_exists = root.join(file).is_file();
            let configured = script_configured && file_exists;
            let missing_env = required_env
                .iter()
                .filter(|name| {
                    std::env::var(name)
                        .map(|value| value.trim().is_empty())
                        .unwrap_or(true)
                })
                .map(|name| (*name).to_string())
                .collect::<Vec<_>>();
            let detail = if configured {
                format!("{file} configured")
            } else if script_configured {
                format!("{file} is missing")
            } else {
                format!("package script {script} is missing")
            };

            ReleaseWorkflowDiagnostic {
                id: (*id).to_string(),
                label: (*label).to_string(),
                script: (*script).to_string(),
                configured,
                required_env: required_env
                    .iter()
                    .map(|name| (*name).to_string())
                    .collect(),
                missing_env,
                detail,
            }
        })
        .collect::<Vec<_>>();

    let status = if workflows
        .iter()
        .any(|workflow| !workflow.configured || !workflow.missing_env.is_empty())
    {
        "warning"
    } else {
        "ready"
    };

    Ok(ReleaseWorkflowStatus {
        status: status.to_string(),
        workflows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_workflow_status_covers_signed_notarized_macos_dmg_pipeline() {
        let status = get_release_workflow_status().expect("release workflow status loads");
        let workflow_ids = status
            .workflows
            .iter()
            .map(|workflow| workflow.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            workflow_ids,
            vec![
                "macos_dmg",
                "macos_notarization",
                "macos_artifact_verification",
                "homebrew_cask",
                "crash_symbols",
            ]
        );

        let dmg = status
            .workflows
            .iter()
            .find(|workflow| workflow.id == "macos_dmg")
            .expect("macOS DMG workflow is reported");
        assert_eq!(dmg.label, "Signed macOS DMG");
        assert_eq!(dmg.script, "release:macos:dmg");
        assert!(dmg.configured);
        assert_eq!(
            dmg.required_env,
            vec![
                "APPLE_CERTIFICATE",
                "APPLE_CERTIFICATE_PASSWORD",
                "APPLE_SIGNING_IDENTITY",
                "KEYCHAIN_PASSWORD",
            ]
        );

        let notarization = status
            .workflows
            .iter()
            .find(|workflow| workflow.id == "macos_notarization")
            .expect("macOS notarization workflow is reported");
        assert_eq!(notarization.label, "Apple notarization");
        assert_eq!(notarization.script, "release:macos:notarize");
        assert!(notarization.configured);
        assert_eq!(
            notarization.required_env,
            vec!["APPLE_ID", "APPLE_PASSWORD", "APPLE_TEAM_ID"]
        );

        let verification = status
            .workflows
            .iter()
            .find(|workflow| workflow.id == "macos_artifact_verification")
            .expect("macOS artifact verification workflow is reported");
        assert_eq!(verification.label, "macOS artifact verification");
        assert_eq!(verification.script, "release:macos:verify");
        assert!(verification.configured);
        assert!(verification.required_env.is_empty());
        assert!(verification.missing_env.is_empty());
    }
}

#[tauri::command]
fn get_terminal_pty_snapshot(
    manager: State<'_, Arc<Mutex<pty::TerminalPtyManager>>>,
) -> Result<pty::TerminalPtySnapshot, String> {
    Ok(manager
        .lock()
        .map_err(|_| "terminal PTY manager lock was poisoned".to_string())?
        .snapshot())
}

#[tauri::command]
fn get_state_snapshot(
    manager: State<'_, Arc<Mutex<pty::TerminalPtyManager>>>,
    store: State<'_, state_store::StateStore>,
    project_id: Option<String>,
) -> Result<state_snapshot::StateSnapshot, String> {
    let pty = manager
        .lock()
        .map_err(|_| "terminal PTY manager lock was poisoned".to_string())?
        .snapshot();
    match project_id.as_deref() {
        Some(project_id) => control_api::build_state_snapshot_for_project_from_store(
            &store,
            pty,
            env!("CARGO_PKG_VERSION"),
            project_id,
        ),
        None => {
            control_api::build_state_snapshot_from_store(&store, pty, env!("CARGO_PKG_VERSION"))
        }
    }
}

#[tauri::command]
fn add_project(
    store: State<'_, state_store::StateStore>,
    request: state_store::AddProjectInput,
) -> Result<state_store::PersistedProject, String> {
    store.add_project(request)
}

#[tauri::command]
fn list_projects(
    store: State<'_, state_store::StateStore>,
) -> Result<Vec<state_store::PersistedProject>, String> {
    store.list_projects()
}

#[tauri::command]
fn focus_project(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<state_store::PersistedProject, String> {
    store.focus_project(&project_id)
}

#[tauri::command]
fn plan_project_detach(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<state_store::PersistedProjectDetachPlan, String> {
    store.plan_project_detach(&project_id)
}

#[tauri::command]
fn upsert_project_tab_group(
    store: State<'_, state_store::StateStore>,
    project_id: String,
    group_name: String,
) -> Result<state_store::PersistedProjectTabGroup, String> {
    store.upsert_project_tab_group(&project_id, &group_name)
}

#[tauri::command]
fn update_project_tab_layout(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpdateProjectTabLayoutInput,
) -> Result<state_store::PersistedProjectTab, String> {
    store.update_project_tab_layout(request)
}

#[tauri::command]
fn save_project_layout_preset(
    store: State<'_, state_store::StateStore>,
    request: state_store::SaveProjectLayoutPresetInput,
) -> Result<state_store::PersistedProjectLayoutPreset, String> {
    store.save_project_layout_preset(request)
}

#[tauri::command]
fn list_project_layout_presets(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedProjectLayoutPreset>, String> {
    store.list_project_layout_presets(&project_id)
}

#[tauri::command]
fn list_project_files(
    store: State<'_, state_store::StateStore>,
    request: state_store::ProjectFileListInput,
) -> Result<state_store::ProjectFileList, String> {
    store.list_project_files(request)
}

#[tauri::command]
fn read_project_file(
    store: State<'_, state_store::StateStore>,
    request: state_store::ProjectFileReadInput,
) -> Result<state_store::ProjectFilePreview, String> {
    store.read_project_file(request)
}

#[tauri::command]
fn write_project_file(
    store: State<'_, state_store::StateStore>,
    request: state_store::ProjectFileWriteInput,
) -> Result<state_store::ProjectFilePreview, String> {
    store.write_project_file(request)
}

#[tauri::command]
fn read_project_diff(
    store: State<'_, state_store::StateStore>,
    request: state_store::ProjectDiffInput,
) -> Result<state_store::ProjectDiff, String> {
    store.read_project_diff(request)
}

#[tauri::command]
fn export_project_patch(
    store: State<'_, state_store::StateStore>,
    request: state_store::ProjectDiffInput,
) -> Result<state_store::PatchArtifact, String> {
    store.export_project_patch(request)
}

#[tauri::command]
fn import_project_patch(
    store: State<'_, state_store::StateStore>,
    request: state_store::ImportPatchInput,
) -> Result<state_store::PatchArtifact, String> {
    store.import_project_patch(request)
}

#[tauri::command]
fn plan_pr_landing(
    store: State<'_, state_store::StateStore>,
    request: state_store::PlanPrLandingInput,
) -> Result<state_store::PrLandingPlan, String> {
    store.plan_pr_landing(request)
}

#[tauri::command]
fn plan_review_pr_landing(
    store: State<'_, state_store::StateStore>,
    request: state_store::PlanReviewPrLandingInput,
) -> Result<state_store::ReviewPrLandingPlanReceipt, String> {
    store.plan_review_pr_landing(request)
}

#[tauri::command]
fn collect_project_lsp_diagnostics(
    store: State<'_, state_store::StateStore>,
    request: state_store::ProjectLspDiagnosticsInput,
) -> Result<state_store::ProjectLspDiagnostics, String> {
    store.collect_project_lsp_diagnostics(request)
}

#[tauri::command]
fn search_project_files(
    store: State<'_, state_store::StateStore>,
    request: state_store::ProjectFileSearchInput,
) -> Result<state_store::ProjectFileSearch, String> {
    store.search_project_files(request)
}

#[tauri::command]
fn plan_browser_automation(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunBrowserAutomationInput,
) -> Result<state_store::BrowserAutomationPlan, String> {
    store.plan_browser_automation(request)
}

#[tauri::command]
fn list_tasks(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedTask>, String> {
    store.list_tasks(&project_id)
}

#[tauri::command]
fn create_task(
    store: State<'_, state_store::StateStore>,
    request: state_store::CreateTaskInput,
) -> Result<state_store::PersistedTask, String> {
    store.create_task(request)
}

#[tauri::command]
fn create_review_follow_up_task(
    store: State<'_, state_store::StateStore>,
    request: state_store::CreateReviewFollowUpTaskInput,
) -> Result<state_store::ReviewFollowUpTaskReceipt, String> {
    store.create_review_follow_up_task(request)
}

fn initiative_to_state_initiative(
    store: &state_store::StateStore,
    initiative: state_store::PersistedInitiative,
) -> Result<state_snapshot::StateInitiative, String> {
    let usage = store.token_usage_summary_for_goal(&initiative.id)?;
    let token_usage = if usage.total_tokens > 0 || usage.cost_usd > 0.0 {
        Some(state_snapshot::StateSessionTokenUsage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            total_tokens: usage.total_tokens,
            cost_usd: usage.cost_usd,
        })
    } else {
        None
    };
    Ok(state_snapshot::StateInitiative {
        id: initiative.id,
        project_id: initiative.project_id,
        name: initiative.name,
        description: initiative.description,
        budget_id: initiative.budget_id,
        status: initiative.status,
        token_usage,
    })
}

#[tauri::command]
fn list_initiatives(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_snapshot::StateInitiative>, String> {
    store
        .list_initiatives(&project_id)?
        .into_iter()
        .map(|initiative| initiative_to_state_initiative(&store, initiative))
        .collect()
}

#[tauri::command]
fn create_initiative(
    store: State<'_, state_store::StateStore>,
    request: state_store::CreateInitiativeInput,
) -> Result<state_snapshot::StateInitiative, String> {
    let initiative = store.create_initiative(request)?;
    initiative_to_state_initiative(&store, initiative)
}

#[tauri::command]
fn list_task_cycles(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedTaskCycle>, String> {
    store.list_task_cycles(&project_id)
}

#[tauri::command]
fn create_task_cycle(
    store: State<'_, state_store::StateStore>,
    request: state_store::CreateTaskCycleInput,
) -> Result<state_store::PersistedTaskCycle, String> {
    store.create_task_cycle(request)
}

#[tauri::command]
fn list_task_modules(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedTaskModule>, String> {
    store.list_task_modules(&project_id)
}

#[tauri::command]
fn create_task_module(
    store: State<'_, state_store::StateStore>,
    request: state_store::CreateTaskModuleInput,
) -> Result<state_store::PersistedTaskModule, String> {
    store.create_task_module(request)
}

#[tauri::command]
fn move_task(
    store: State<'_, state_store::StateStore>,
    id: String,
    status: String,
) -> Result<state_store::PersistedTask, String> {
    store.move_task_status(&id, &status)
}

#[tauri::command]
fn add_task_comment(
    store: State<'_, state_store::StateStore>,
    request: state_store::AddTaskCommentInput,
) -> Result<state_store::PersistedComment, String> {
    store.add_task_comment(request)
}

#[tauri::command]
fn list_task_comments(
    store: State<'_, state_store::StateStore>,
    task_id: String,
) -> Result<Vec<state_store::PersistedComment>, String> {
    store.list_task_comments(&task_id)
}

#[tauri::command]
fn add_task_subtask(
    store: State<'_, state_store::StateStore>,
    request: state_store::AddTaskSubtaskInput,
) -> Result<state_store::PersistedSubtask, String> {
    store.add_task_subtask(request)
}

#[tauri::command]
fn list_task_subtasks(
    store: State<'_, state_store::StateStore>,
    task_id: String,
) -> Result<Vec<state_store::PersistedSubtask>, String> {
    store.list_task_subtasks(&task_id)
}

#[tauri::command]
fn update_task_subtask_status(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpdateTaskSubtaskStatusInput,
) -> Result<state_store::PersistedSubtask, String> {
    store.update_task_subtask_status(request)
}

#[tauri::command]
fn update_task_planning(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpdateTaskPlanningInput,
) -> Result<state_store::PersistedTask, String> {
    store.update_task_planning(request)
}

#[tauri::command]
fn update_task_context(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpdateTaskContextInput,
) -> Result<state_store::PersistedTask, String> {
    store.update_task_context(request)
}

#[tauri::command]
fn save_task_workpad(
    store: State<'_, state_store::StateStore>,
    request: state_store::SaveTaskWorkpadInput,
) -> Result<state_store::PersistedWorkpad, String> {
    store.save_task_workpad(request)
}

#[tauri::command]
fn upsert_command_block(
    store: State<'_, state_store::StateStore>,
    request: state_store::PersistedCommandBlockInput,
) -> Result<state_store::PersistedCommandBlock, String> {
    store.upsert_command_block(&request)
}

#[tauri::command]
fn search_command_blocks(
    store: State<'_, state_store::StateStore>,
    query: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<state_store::PersistedCommandBlock>, String> {
    store.search_command_blocks(query.as_deref(), limit.unwrap_or(50).min(100))
}

#[tauri::command]
fn mark_command_block(
    store: State<'_, state_store::StateStore>,
    command_block_id: String,
    status: String,
) -> Result<state_store::PersistedCommandBlock, String> {
    store.mark_command_block_status(&command_block_id, &status)
}

#[tauri::command]
fn merge_command_blocks(
    store: State<'_, state_store::StateStore>,
    first_command_block_id: String,
    second_command_block_id: String,
) -> Result<state_store::PersistedCommandBlock, String> {
    store.merge_command_blocks(&first_command_block_id, &second_command_block_id)
}

#[tauri::command]
fn split_command_block(
    store: State<'_, state_store::StateStore>,
    command_block_id: String,
) -> Result<state_store::CommandBlockSplitReceipt, String> {
    store.split_command_block(&command_block_id)
}

#[tauri::command]
fn explain_command_block(
    store: State<'_, state_store::StateStore>,
    command_block_id: String,
    request: state_store::ExplainCommandBlockInput,
) -> Result<state_store::CommandBlockExplanation, String> {
    store.explain_command_block(&command_block_id, request)
}

#[tauri::command]
fn export_command_block_bundle(
    store: State<'_, state_store::StateStore>,
    command_block_id: String,
) -> Result<state_store::CommandBlockBundle, String> {
    store.export_command_block_bundle(&command_block_id)
}

#[tauri::command]
fn attach_command_block_to_evidence(
    store: State<'_, state_store::StateStore>,
    request: state_store::AttachCommandBlockEvidenceInput,
) -> Result<state_store::PersistedEvidencePack, String> {
    store.attach_command_block_to_evidence(request)
}

#[tauri::command]
fn generate_evidence_pack_for_run(
    store: State<'_, state_store::StateStore>,
    request: state_store::GenerateEvidencePackInput,
) -> Result<state_store::PersistedEvidencePack, String> {
    store.generate_evidence_pack_for_run(request)
}

#[tauri::command]
fn record_evidence_review_decision(
    store: State<'_, state_store::StateStore>,
    request: state_store::RecordEvidenceReviewDecisionInput,
) -> Result<state_store::PersistedEvidencePack, String> {
    store.record_evidence_review_decision(request)
}

#[tauri::command]
fn dispatch_run(
    store: State<'_, state_store::StateStore>,
    request: state_store::DispatchRunInput,
) -> Result<state_store::PersistedRun, String> {
    store.dispatch_run(request)
}

#[tauri::command]
fn list_runs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedRun>, String> {
    store.list_runs(&project_id)
}

#[tauri::command]
fn update_run_lifecycle(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpdateRunLifecycleInput,
) -> Result<state_store::PersistedRun, String> {
    store.update_run_lifecycle(request)
}

#[tauri::command]
fn cancel_run(
    store: State<'_, state_store::StateStore>,
    run_id: String,
) -> Result<state_store::PersistedRun, String> {
    store.cancel_run(&run_id)
}

#[tauri::command]
fn retry_run(
    store: State<'_, state_store::StateStore>,
    run_id: String,
) -> Result<state_store::PersistedRun, String> {
    store.retry_run(&run_id)
}

#[tauri::command]
fn record_run_status_update(
    store: State<'_, state_store::StateStore>,
    request: state_store::RecordRunStatusUpdateInput,
) -> Result<state_store::PersistedComment, String> {
    store.record_run_status_update(request)
}

#[tauri::command]
fn reload_workflow(
    store: State<'_, state_store::StateStore>,
    request: state_store::ReloadWorkflowInput,
) -> Result<state_store::PersistedWorkflowVersion, String> {
    store.reload_workflow(request)
}

#[tauri::command]
fn validate_workflow(
    store: State<'_, state_store::StateStore>,
    request: state_store::ValidateWorkflowInput,
) -> Result<state_store::WorkflowValidationResult, String> {
    store.validate_workflow(request)
}

#[tauri::command]
fn get_workflow_runtime_state(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<state_store::WorkflowRuntimeState, String> {
    store.workflow_runtime_state(&project_id)
}

#[tauri::command]
fn run_workflow_hook(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunWorkflowHookInput,
) -> Result<state_store::WorkflowHookRunResult, String> {
    store.run_workflow_hook(request)
}

#[tauri::command]
fn get_run_replay_metadata(
    store: State<'_, state_store::StateStore>,
    run_id: String,
) -> Result<Option<state_store::PersistedRunReplayMetadata>, String> {
    store.get_run_replay_metadata(&run_id)
}

#[tauri::command]
fn create_session(
    store: State<'_, state_store::StateStore>,
    request: state_store::CreateSessionInput,
) -> Result<state_store::PersistedSession, String> {
    store.create_session(request)
}

#[tauri::command]
fn list_sessions(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedSession>, String> {
    store.list_sessions(&project_id)
}

#[tauri::command]
fn list_runtime_pool(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::RuntimePoolItem>, String> {
    store.runtime_pool(&project_id)
}

#[tauri::command]
fn focus_session(
    store: State<'_, state_store::StateStore>,
    session_id: String,
) -> Result<state_store::PersistedSession, String> {
    store.focus_session(&session_id)
}

#[tauri::command]
fn set_session_attention(
    store: State<'_, state_store::StateStore>,
    session_id: String,
    attention_state: String,
) -> Result<state_store::PersistedSession, String> {
    store.set_session_attention(&session_id, &attention_state)
}

#[tauri::command]
fn resize_session(
    store: State<'_, state_store::StateStore>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<state_store::SessionResizeReceipt, String> {
    store.resize_session(&session_id, cols, rows)
}

#[tauri::command]
fn takeover_session(
    store: State<'_, state_store::StateStore>,
    session_id: String,
) -> Result<state_store::PersistedSession, String> {
    store.takeover_session(&session_id)
}

#[tauri::command]
fn release_session(
    store: State<'_, state_store::StateStore>,
    session_id: String,
) -> Result<state_store::PersistedSession, String> {
    store.release_session(&session_id)
}

#[tauri::command]
fn kill_session(
    store: State<'_, state_store::StateStore>,
    session_id: String,
) -> Result<state_store::PersistedSession, String> {
    store.kill_session(&session_id)
}

#[tauri::command]
fn attach_session_task(
    store: State<'_, state_store::StateStore>,
    session_id: String,
    task_id: String,
) -> Result<state_store::PersistedSession, String> {
    store.attach_session_task(&session_id, &task_id)
}

#[tauri::command]
fn detach_session_task(
    store: State<'_, state_store::StateStore>,
    session_id: String,
) -> Result<state_store::PersistedSession, String> {
    store.detach_session_task(&session_id)
}

#[tauri::command]
fn record_session_input(
    store: State<'_, state_store::StateStore>,
    request: state_store::SessionInputInput,
) -> Result<state_store::SessionInputReceipt, String> {
    store.record_session_input(request)
}

#[tauri::command]
fn record_terminal_stream_chunk(
    store: State<'_, state_store::StateStore>,
    request: state_store::RecordTerminalStreamChunkInput,
) -> Result<state_store::PersistedTerminalStreamChunk, String> {
    store.record_terminal_stream_chunk(request)
}

#[tauri::command]
fn list_terminal_stream_chunks(
    store: State<'_, state_store::StateStore>,
    session_id: String,
    limit: Option<usize>,
) -> Result<Vec<state_store::PersistedTerminalStreamChunk>, String> {
    store.list_terminal_stream_chunks(&session_id, limit)
}

#[tauri::command]
fn list_agent_profiles(
    store: State<'_, state_store::StateStore>,
) -> Result<Vec<state_store::PersistedAgentProfile>, String> {
    store.list_agent_profiles()
}

#[tauri::command]
fn scan_agent_profiles(
    store: State<'_, state_store::StateStore>,
) -> Result<Vec<state_store::PersistedAgentProfile>, String> {
    store.scan_agent_profiles()
}

#[tauri::command]
fn upsert_agent_profile(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpsertAgentProfileInput,
) -> Result<state_store::PersistedAgentProfile, String> {
    store.upsert_agent_profile(request)
}

#[tauri::command]
fn update_agent_profile_status(
    store: State<'_, state_store::StateStore>,
    agent_id: String,
    status: String,
) -> Result<state_store::PersistedAgentProfile, String> {
    store.update_agent_profile_status(&agent_id, &status)
}

#[tauri::command]
fn heartbeat_agent_profile(
    store: State<'_, state_store::StateStore>,
    agent_id: String,
) -> Result<state_store::PersistedAgentProfile, String> {
    store.heartbeat_agent_profile(&agent_id)
}

#[tauri::command]
fn list_skill_packs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedSkillPack>, String> {
    store.list_skill_packs(&project_id)
}

#[tauri::command]
fn upsert_skill_pack(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpsertSkillPackInput,
) -> Result<state_store::PersistedSkillPack, String> {
    store.upsert_skill_pack(request)
}

#[tauri::command]
fn get_provider_model_settings(
    store: State<'_, state_store::StateStore>,
) -> Result<state_store::ProviderModelSettings, String> {
    store.provider_model_settings()
}

#[tauri::command]
fn upsert_provider_model_settings(
    store: State<'_, state_store::StateStore>,
    request: state_store::ProviderModelSettingsInput,
) -> Result<state_store::ProviderModelSettings, String> {
    store.upsert_provider_model_settings(request)
}

#[tauri::command]
fn get_terminal_theme_settings(
    store: State<'_, state_store::StateStore>,
    project_id: Option<String>,
) -> Result<state_store::TerminalThemeSettings, String> {
    store.terminal_theme_settings(project_id.as_deref())
}

#[tauri::command]
fn upsert_terminal_theme_settings(
    store: State<'_, state_store::StateStore>,
    request: state_store::TerminalThemeSettingsInput,
) -> Result<state_store::TerminalThemeSettings, String> {
    store.upsert_terminal_theme_settings(request)
}

#[tauri::command]
fn launch_agent_terminal(
    app: AppHandle,
    manager: State<'_, Arc<Mutex<pty::TerminalPtyManager>>>,
    store: State<'_, state_store::StateStore>,
    request: state_store::AgentTerminalLaunchInput,
) -> Result<AgentTerminalLaunch, String> {
    let plan = store.agent_terminal_launch_plan(request)?;
    let event_app = app.clone();
    let pty_session = manager
        .lock()
        .map_err(|_| "terminal PTY manager lock was poisoned".to_string())?
        .spawn_from_request_with_output_sink(
            pty::SpawnTerminalPtyRequest {
                title: plan.title.clone(),
                command: plan.command.clone(),
                args: plan.args.clone(),
                cols: plan.cols,
                rows: plan.rows,
            },
            move |event| {
                let _ = event_app.emit("terminal://pty-output", event);
            },
        )?;
    let session = store.create_session(state_store::CreateSessionInput {
        project_id: plan.project_id,
        mode: "agent".to_string(),
        title: plan.title,
        cwd: None,
        branch: None,
        agent_profile_id: Some(plan.agent_profile_id),
        task_id: None,
        run_id: None,
    })?;

    Ok(AgentTerminalLaunch {
        session,
        pty_session,
    })
}

#[tauri::command]
fn create_policy_approval(
    store: State<'_, state_store::StateStore>,
    request: state_store::CreatePolicyApprovalInput,
) -> Result<state_store::PersistedPolicyApproval, String> {
    store.create_policy_approval(request)
}

#[tauri::command]
fn list_policy_approvals(
    store: State<'_, state_store::StateStore>,
    project_id: String,
    state: Option<String>,
) -> Result<Vec<state_store::PersistedPolicyApproval>, String> {
    store.list_policy_approvals(&project_id, state.as_deref())
}

#[tauri::command]
fn decide_policy_approval(
    store: State<'_, state_store::StateStore>,
    request: state_store::DecidePolicyApprovalInput,
) -> Result<state_store::PersistedPolicyApproval, String> {
    store.decide_policy_approval(request)
}

#[tauri::command]
fn upsert_policy_pack(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpsertPolicyPackInput,
) -> Result<state_store::PersistedPolicyPack, String> {
    store.upsert_policy_pack(request)
}

#[tauri::command]
fn list_policy_packs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedPolicyPack>, String> {
    store.list_policy_packs(&project_id, None)
}

#[tauri::command]
fn list_permission_audit(
    store: State<'_, state_store::StateStore>,
    project_id: String,
    decision: Option<String>,
    action_kind: Option<String>,
    run_id: Option<String>,
    task_id: Option<String>,
) -> Result<Vec<state_store::PersistedPermissionAudit>, String> {
    store.list_permission_audit(
        &project_id,
        decision.as_deref(),
        action_kind.as_deref(),
        run_id.as_deref(),
        task_id.as_deref(),
    )
}

#[tauri::command]
fn evaluate_policy_action(
    store: State<'_, state_store::StateStore>,
    request: state_store::EvaluatePolicyActionInput,
) -> Result<state_store::PolicyActionEvaluation, String> {
    store.evaluate_policy_action(request)
}

#[tauri::command]
fn get_budget_summary(
    store: State<'_, state_store::StateStore>,
) -> Result<serde_json::Value, String> {
    store.budget_summary()
}

#[tauri::command]
fn get_budget_forecast(
    store: State<'_, state_store::StateStore>,
) -> Result<serde_json::Value, String> {
    store.budget_forecast()
}

#[tauri::command]
fn list_provider_prices(
    store: State<'_, state_store::StateStore>,
) -> Result<Vec<state_store::PersistedProviderPrice>, String> {
    store.list_provider_prices()
}

#[tauri::command]
fn update_provider_price_table(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpdateProviderPriceTableInput,
) -> Result<serde_json::Value, String> {
    store.update_provider_price_table(request)
}

#[tauri::command]
fn upsert_budget(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpsertBudgetInput,
) -> Result<state_store::PersistedBudget, String> {
    store.upsert_budget(request)
}

#[tauri::command]
fn record_token_usage(
    store: State<'_, state_store::StateStore>,
    request: state_store::TokenUsageInput,
) -> Result<state_store::PersistedTokenUsage, String> {
    store.record_token_usage(request)
}

#[tauri::command]
fn ingest_token_usage_adapter(
    store: State<'_, state_store::StateStore>,
    request: state_store::IngestTokenUsageAdapterInput,
) -> Result<state_store::PersistedTokenUsage, String> {
    store.ingest_token_usage_adapter(request)
}

#[tauri::command]
fn ingest_agent_events(
    store: State<'_, state_store::StateStore>,
    request: state_store::IngestAgentEventsInput,
) -> Result<state_store::PersistedAgentEvent, String> {
    store.ingest_agent_events(request)
}

#[tauri::command]
fn run_release_gates(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunReleaseGatesInput,
) -> Result<state_store::PersistedReleaseGateRun, String> {
    store.run_release_gate_scenarios(request)
}

#[tauri::command]
fn list_release_gate_runs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedReleaseGateRun>, String> {
    store.list_release_gate_runs(&project_id)
}

#[tauri::command]
fn run_terminal_fidelity_smoke(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunTerminalFidelitySmokeInput,
) -> Result<state_store::PersistedTerminalFidelitySmokeRun, String> {
    store.run_terminal_fidelity_smoke_tests(request)
}

#[tauri::command]
fn list_terminal_fidelity_smoke_runs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedTerminalFidelitySmokeRun>, String> {
    store.list_terminal_fidelity_smoke_runs(&project_id)
}

#[tauri::command]
fn run_task_lifecycle_e2e(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunTaskLifecycleE2EInput,
) -> Result<state_store::PersistedTaskLifecycleE2ERun, String> {
    store.run_task_lifecycle_e2e(request)
}

#[tauri::command]
fn list_task_lifecycle_e2e_runs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedTaskLifecycleE2ERun>, String> {
    store.list_task_lifecycle_e2e_runs(&project_id)
}

#[tauri::command]
fn run_workflow_negative_tests(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunWorkflowNegativeTestsInput,
) -> Result<state_store::PersistedWorkflowNegativeTestRun, String> {
    store.run_workflow_negative_tests(request)
}

#[tauri::command]
fn list_workflow_negative_test_runs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedWorkflowNegativeTestRun>, String> {
    store.list_workflow_negative_test_runs(&project_id)
}

#[tauri::command]
fn run_dmg_smoke_test(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunDmgSmokeInput,
) -> Result<state_store::PersistedDmgSmokeRun, String> {
    store.run_dmg_smoke_test(request)
}

#[tauri::command]
fn list_dmg_smoke_runs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedDmgSmokeRun>, String> {
    store.list_dmg_smoke_runs(&project_id)
}

#[tauri::command]
fn run_recovery_drills(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunRecoveryDrillsInput,
) -> Result<state_store::PersistedRecoveryDrillRun, String> {
    store.run_recovery_drills(request)
}

#[tauri::command]
fn list_recovery_drill_runs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedRecoveryDrillRun>, String> {
    store.list_recovery_drill_runs(&project_id)
}

#[tauri::command]
fn run_benchmarks(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunBenchmarksInput,
) -> Result<state_store::PersistedBenchmarkRun, String> {
    store.run_benchmarks(request)
}

#[tauri::command]
fn list_benchmark_runs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedBenchmarkRun>, String> {
    store.list_benchmark_runs(&project_id)
}

#[tauri::command]
fn run_dogfood_telemetry_review(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunDogfoodTelemetryReviewInput,
) -> Result<state_store::PersistedDogfoodTelemetryReview, String> {
    store.run_dogfood_telemetry_review(request)
}

#[tauri::command]
fn list_dogfood_telemetry_reviews(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedDogfoodTelemetryReview>, String> {
    store.list_dogfood_telemetry_reviews(&project_id)
}

#[tauri::command]
fn create_visual_harness_link(
    store: State<'_, state_store::StateStore>,
    request: state_store::CreateVisualHarnessLinkInput,
) -> Result<state_store::PersistedVisualHarnessLink, String> {
    store.create_visual_harness_link(request)
}

#[tauri::command]
fn list_visual_harness_links(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedVisualHarnessLink>, String> {
    store.list_visual_harness_links(&project_id)
}

#[tauri::command]
fn upsert_external_tracker_binding(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpsertExternalTrackerBindingInput,
) -> Result<state_store::PersistedExternalTrackerBinding, String> {
    store.upsert_external_tracker_binding(request)
}

#[tauri::command]
fn list_external_tracker_bindings(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedExternalTrackerBinding>, String> {
    store.list_external_tracker_bindings(&project_id)
}

#[tauri::command]
fn run_tracker_sync(
    store: State<'_, state_store::StateStore>,
    provider: String,
    request: state_store::RunTrackerSyncInput,
) -> Result<state_store::PersistedExternalTrackerSyncRun, String> {
    store.run_tracker_sync(&provider, request)
}

#[tauri::command]
fn upsert_secret(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpsertSecretInput,
) -> Result<state_store::PersistedSecretMetadata, String> {
    store.upsert_secret(request)
}

#[tauri::command]
fn list_secrets(
    store: State<'_, state_store::StateStore>,
    project_id: Option<String>,
    name: Option<String>,
) -> Result<Vec<state_store::PersistedSecretMetadata>, String> {
    store.list_secrets(project_id.as_deref(), name.as_deref())
}

#[tauri::command]
fn upsert_knowledge_source(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpsertKnowledgeSourceInput,
) -> Result<state_store::PersistedKnowledgeSource, String> {
    store.upsert_knowledge_source(request)
}

#[tauri::command]
fn list_knowledge_sources(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedKnowledgeSource>, String> {
    store.list_knowledge_sources(&project_id)
}

#[tauri::command]
fn save_knowledge_page(
    store: State<'_, state_store::StateStore>,
    request: state_store::SaveKnowledgePageInput,
) -> Result<state_store::PersistedKnowledgePage, String> {
    store.save_knowledge_page(request)
}

#[tauri::command]
fn search_knowledge_pages(
    store: State<'_, state_store::StateStore>,
    project_id: String,
    query: Option<String>,
) -> Result<Vec<state_store::PersistedKnowledgePage>, String> {
    store.search_knowledge_pages(&project_id, query.as_deref())
}

#[tauri::command]
fn save_knowledge_exploration(
    store: State<'_, state_store::StateStore>,
    request: state_store::SaveKnowledgeExplorationInput,
) -> Result<state_store::PersistedKnowledgeExploration, String> {
    store.save_knowledge_exploration(request)
}

#[tauri::command]
fn list_knowledge_explorations(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedKnowledgeExploration>, String> {
    store.list_knowledge_explorations(&project_id)
}

#[tauri::command]
fn list_knowledge_concepts(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::KnowledgeConcept>, String> {
    store.list_knowledge_concepts(&project_id)
}

#[tauri::command]
fn export_knowledge_obsidian_markdown(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<state_store::KnowledgeObsidianExport, String> {
    store.export_knowledge_obsidian_markdown(&project_id)
}

#[tauri::command]
fn answer_knowledge_question(
    store: State<'_, state_store::StateStore>,
    request: state_store::KnowledgeChatQuestionInput,
) -> Result<state_store::KnowledgeChatAnswer, String> {
    store.answer_knowledge_question(request)
}

#[tauri::command]
fn upsert_context_pack(
    store: State<'_, state_store::StateStore>,
    request: state_store::UpsertContextPackInput,
) -> Result<state_store::PersistedContextPack, String> {
    store.upsert_context_pack(request)
}

#[tauri::command]
fn list_context_packs(
    store: State<'_, state_store::StateStore>,
    project_id: String,
) -> Result<Vec<state_store::PersistedContextPack>, String> {
    store.list_context_packs(&project_id)
}

#[tauri::command]
fn record_knowledge_lint_report(
    store: State<'_, state_store::StateStore>,
    request: state_store::RecordKnowledgeLintReportInput,
) -> Result<state_store::PersistedKnowledgeLintReport, String> {
    store.record_knowledge_lint_report(request)
}

#[tauri::command]
fn run_knowledge_automation(
    store: State<'_, state_store::StateStore>,
    request: state_store::RunKnowledgeAutomationInput,
) -> Result<state_store::KnowledgeAutomationRun, String> {
    store.run_knowledge_automation(request)
}

#[tauri::command]
fn ingest_knowledge_artifact(
    store: State<'_, state_store::StateStore>,
    request: state_store::IngestKnowledgeArtifactInput,
) -> Result<state_store::KnowledgeIngestionResult, String> {
    store.ingest_knowledge_artifact(request)
}

#[tauri::command]
fn spawn_terminal_pty_session(
    app: AppHandle,
    manager: State<'_, Arc<Mutex<pty::TerminalPtyManager>>>,
    request: pty::SpawnTerminalPtyRequest,
) -> Result<pty::TerminalPtySession, String> {
    let event_app = app.clone();
    manager
        .lock()
        .map_err(|_| "terminal PTY manager lock was poisoned".to_string())?
        .spawn_from_request_with_output_sink(request, move |event| {
            let _ = event_app.emit("terminal://pty-output", event);
        })
}

#[tauri::command]
fn resize_terminal_pty_session(
    manager: State<'_, Arc<Mutex<pty::TerminalPtyManager>>>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    manager
        .lock()
        .map_err(|_| "terminal PTY manager lock was poisoned".to_string())?
        .resize_session(&id, cols, rows)
}

#[tauri::command]
fn close_terminal_pty_session(
    manager: State<'_, Arc<Mutex<pty::TerminalPtyManager>>>,
    id: String,
) -> Result<(), String> {
    manager
        .lock()
        .map_err(|_| "terminal PTY manager lock was poisoned".to_string())?
        .close_session(&id)
}

#[tauri::command]
fn write_terminal_pty_input(
    manager: State<'_, Arc<Mutex<pty::TerminalPtyManager>>>,
    id: String,
    input: String,
) -> Result<(), String> {
    manager
        .lock()
        .map_err(|_| "terminal PTY manager lock was poisoned".to_string())?
        .write_session_input(&id, &input)
}

#[tauri::command]
fn capture_terminal_pty_command(
    request: pty::CaptureTerminalPtyRequest,
) -> Result<pty::PtyCommandCapture, String> {
    pty::capture_from_request(request)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state_store = state_store::StateStore::open_at(state_store::StateStore::app_support_path())
        .expect("failed to initialize Haneulchi SQLite state store");
    let pty_manager = Arc::new(Mutex::new(pty::TerminalPtyManager::default()));
    let _control_api = control_api::start_control_api_server(
        state_store.clone(),
        Arc::clone(&pty_manager),
        env!("CARGO_PKG_VERSION"),
    )
    .expect("failed to initialize Haneulchi local control API");

    tauri::Builder::default()
        .manage(pty_manager)
        .manage(state_store)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_readiness_snapshot,
            get_release_workflow_status,
            get_terminal_pty_snapshot,
            get_state_snapshot,
            add_project,
            list_projects,
            focus_project,
            plan_project_detach,
            upsert_project_tab_group,
            update_project_tab_layout,
            save_project_layout_preset,
            list_project_layout_presets,
            list_project_files,
            read_project_file,
            write_project_file,
            read_project_diff,
            export_project_patch,
            import_project_patch,
            plan_pr_landing,
            plan_review_pr_landing,
            collect_project_lsp_diagnostics,
            search_project_files,
            plan_browser_automation,
            list_tasks,
            create_task,
            create_review_follow_up_task,
            list_initiatives,
            create_initiative,
            list_task_cycles,
            create_task_cycle,
            list_task_modules,
            create_task_module,
            move_task,
            add_task_comment,
            list_task_comments,
            add_task_subtask,
            list_task_subtasks,
            update_task_subtask_status,
            update_task_planning,
            update_task_context,
            save_task_workpad,
            upsert_command_block,
            search_command_blocks,
            mark_command_block,
            merge_command_blocks,
            split_command_block,
            explain_command_block,
            export_command_block_bundle,
            attach_command_block_to_evidence,
            generate_evidence_pack_for_run,
            record_evidence_review_decision,
            dispatch_run,
            list_runs,
            update_run_lifecycle,
            cancel_run,
            retry_run,
            record_run_status_update,
            reload_workflow,
            validate_workflow,
            get_workflow_runtime_state,
            run_workflow_hook,
            get_run_replay_metadata,
            create_session,
            list_sessions,
            list_runtime_pool,
            focus_session,
            set_session_attention,
            resize_session,
            takeover_session,
            release_session,
            kill_session,
            attach_session_task,
            detach_session_task,
            record_session_input,
            record_terminal_stream_chunk,
            list_terminal_stream_chunks,
            list_agent_profiles,
            scan_agent_profiles,
            upsert_agent_profile,
            update_agent_profile_status,
            heartbeat_agent_profile,
            list_skill_packs,
            upsert_skill_pack,
            get_provider_model_settings,
            upsert_provider_model_settings,
            get_terminal_theme_settings,
            upsert_terminal_theme_settings,
            launch_agent_terminal,
            create_policy_approval,
            list_policy_approvals,
            decide_policy_approval,
            upsert_policy_pack,
            list_policy_packs,
            list_permission_audit,
            evaluate_policy_action,
            get_budget_summary,
            get_budget_forecast,
            list_provider_prices,
            update_provider_price_table,
            upsert_budget,
            record_token_usage,
            ingest_token_usage_adapter,
            ingest_agent_events,
            run_release_gates,
            list_release_gate_runs,
            run_terminal_fidelity_smoke,
            list_terminal_fidelity_smoke_runs,
            run_task_lifecycle_e2e,
            list_task_lifecycle_e2e_runs,
            run_workflow_negative_tests,
            list_workflow_negative_test_runs,
            run_dmg_smoke_test,
            list_dmg_smoke_runs,
            run_recovery_drills,
            list_recovery_drill_runs,
            run_benchmarks,
            list_benchmark_runs,
            run_dogfood_telemetry_review,
            list_dogfood_telemetry_reviews,
            create_visual_harness_link,
            list_visual_harness_links,
            upsert_external_tracker_binding,
            list_external_tracker_bindings,
            run_tracker_sync,
            upsert_secret,
            list_secrets,
            upsert_knowledge_source,
            list_knowledge_sources,
            save_knowledge_page,
            search_knowledge_pages,
            save_knowledge_exploration,
            list_knowledge_explorations,
            list_knowledge_concepts,
            export_knowledge_obsidian_markdown,
            answer_knowledge_question,
            upsert_context_pack,
            list_context_packs,
            record_knowledge_lint_report,
            run_knowledge_automation,
            ingest_knowledge_artifact,
            spawn_terminal_pty_session,
            resize_terminal_pty_session,
            close_terminal_pty_session,
            write_terminal_pty_input,
            capture_terminal_pty_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
