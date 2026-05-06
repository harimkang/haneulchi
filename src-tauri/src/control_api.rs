use crate::{
    pty::{TerminalPtyManager, TerminalPtySnapshot},
    state_snapshot::{self, StateSnapshot},
    state_store::{
        AddTaskCommentInput, AddTaskSubtaskInput, CreateInitiativeInput, CreateTaskCycleInput,
        CreateTaskInput, CreateTaskModuleInput, PersistedRun, PersistedSession, PersistedTask,
        RecordRunStatusUpdateInput, SaveTaskWorkpadInput, StateStore, UpdateTaskContextInput,
        UpdateTaskInput, UpdateTaskPlanningInput, UpdateTaskSubtaskStatusInput,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

const MAX_REQUEST_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status_code: u16,
    pub reason: &'static str,
    pub content_type: &'static str,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ControlApiHealth {
    pub db: String,
    pub pty: String,
    pub api: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateCheckResponse {
    pub channel: String,
    pub current_version: String,
    pub latest_version: String,
    pub pub_date: Option<String>,
    pub feed_path: String,
    pub signature_state: String,
}

fn snapshot_list_response<T: Serialize>(
    store: &StateStore,
    pty: TerminalPtySnapshot,
    version: &str,
    project_id: &str,
    items: Vec<T>,
) -> Result<serde_json::Value, String> {
    build_state_snapshot_for_project_from_store(store, pty, version, project_id)
        .map(|snapshot| serde_json::json!({ "snapshot_id": snapshot.snapshot_id, "items": items }))
}

fn snapshot_object_response<T: Serialize>(
    store: &StateStore,
    pty: TerminalPtySnapshot,
    version: &str,
    project_id: &str,
    value: T,
) -> Result<serde_json::Value, String> {
    let mut response = serde_json::to_value(value)
        .map_err(|error| format!("failed to serialize response with snapshot id: {error}"))?;
    let snapshot = build_state_snapshot_for_project_from_store(store, pty, version, project_id)?;
    match response.as_object_mut() {
        Some(object) => {
            object.insert(
                "snapshot_id".to_string(),
                serde_json::Value::String(snapshot.snapshot_id),
            );
            Ok(response)
        }
        None => Ok(serde_json::json!({
            "snapshot_id": snapshot.snapshot_id,
            "value": response
        })),
    }
}

fn project_id_for_run_id(store: &StateStore, run_id: &str) -> Result<String, String> {
    store
        .get_run(run_id)?
        .map(|run| run.project_id)
        .ok_or_else(|| format!("run {run_id} not found"))
}

fn project_id_for_evidence_pack(
    store: &StateStore,
    evidence: &crate::state_store::PersistedEvidencePack,
) -> Result<String, String> {
    if let Some(run_id) = evidence.run_id.as_deref() {
        if let Some(run) = store.get_run(run_id)? {
            return Ok(run.project_id);
        }
    }
    if let Some(task_id) = evidence.task_id.as_deref() {
        if let Some(task) = store.get_task(task_id)? {
            return Ok(task.project_id);
        }
    }
    Ok("proj_local".to_string())
}

fn project_id_for_command_block(
    store: &StateStore,
    block: &crate::state_store::PersistedCommandBlock,
) -> Result<String, String> {
    if let Some(run_id) = block.run_id.as_deref() {
        if let Some(run) = store.get_run(run_id)? {
            return Ok(run.project_id);
        }
    }
    if let Some(task_id) = block.task_id.as_deref() {
        if let Some(task) = store.get_task(task_id)? {
            return Ok(task.project_id);
        }
    }
    Ok("proj_local".to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoveTaskRequest {
    status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddCommentRequest {
    body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddSubtaskRequest {
    title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateSubtaskStatusRequest {
    status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveWorkpadRequest {
    body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTaskContextRequest {
    context_pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTaskPlanningRequest {
    cycle_id: Option<String>,
    module_id: Option<String>,
    initiative_id: Option<String>,
    due_at: Option<String>,
    estimate: Option<String>,
    labels: Option<Vec<String>>,
    assignee_type: Option<String>,
    assignee_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTaskRequest {
    title: Option<String>,
    description: Option<String>,
    priority: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecordRunStatusUpdateRequest {
    body_md: String,
    lifecycle: Option<String>,
    status_detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachEvidenceRequest {
    evidence_pack_id: String,
    task_id: Option<String>,
    run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandBlockMarkRequest {
    status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandBlockMergeRequest {
    second_command_block_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerateEvidenceRequest {
    evidence_pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewDecisionRequest {
    decision: String,
    reviewer_id: Option<String>,
    body_md: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewFollowUpTaskRequest {
    title: Option<String>,
    priority: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PolicyDecisionRequest {
    decision: String,
    decision_by: Option<String>,
    decision_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateRunLifecycleRequest {
    lifecycle: String,
    status_detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReloadWorkflowRequest {
    project_id: String,
    source_path: String,
    content: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunHookRequest {
    repo_root: String,
    workspace_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeProjectRequest {
    project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionInputRequest {
    text: String,
    allow_dangerous: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalStreamChunkRequest {
    seq_start: i64,
    seq_end: i64,
    body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionAttachTaskRequest {
    task_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionAttentionRequest {
    attention_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionResizeRequest {
    cols: u16,
    rows: u16,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectLayoutRequest {
    layout_json: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectLayoutPresetRequest {
    name: String,
    layout_json: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectTabGroupRequest {
    group_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectFileWriteRequest {
    path: String,
    body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PatchImportRequest {
    body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrLandingPlanRequest {
    title: String,
    draft: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewPrLandingPlanRequest {
    title: Option<String>,
    draft: Option<bool>,
}

pub fn socket_path() -> PathBuf {
    StateStore::app_support_path()
        .parent()
        .map(|path| path.join("run").join("haneulchi.sock"))
        .unwrap_or_else(|| PathBuf::from("run").join("haneulchi.sock"))
}

pub fn start_control_api_server(
    store: StateStore,
    manager: Arc<Mutex<TerminalPtyManager>>,
    version: &'static str,
) -> Result<thread::JoinHandle<()>, String> {
    start_control_api_server_at(socket_path(), store, manager, version)
}

pub fn start_control_api_server_at(
    socket_path: PathBuf,
    store: StateStore,
    manager: Arc<Mutex<TerminalPtyManager>>,
    version: &'static str,
) -> Result<thread::JoinHandle<()>, String> {
    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create control API socket directory {}: {error}",
                parent.display()
            )
        })?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).map_err(|error| {
            format!(
                "failed to protect control API socket directory {}: {error}",
                parent.display()
            )
        })?;
    }
    if socket_path.exists() {
        fs::remove_file(&socket_path).map_err(|error| {
            format!(
                "failed to remove stale control API socket {}: {error}",
                socket_path.display()
            )
        })?;
    }

    let listener = UnixListener::bind(&socket_path).map_err(|error| {
        format!(
            "failed to bind control API socket {}: {error}",
            socket_path.display()
        )
    })?;
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600)).map_err(|error| {
        format!(
            "failed to protect control API socket {}: {error}",
            socket_path.display()
        )
    })?;

    thread::Builder::new()
        .name("haneulchi-control-api".to_string())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let _ = serve_stream(stream, &store, &manager, version);
            }
        })
        .map_err(|error| format!("failed to spawn control API thread: {error}"))
}

pub fn handle_http_request(
    request: &str,
    store: &StateStore,
    pty: TerminalPtySnapshot,
    version: &str,
) -> HttpResponse {
    let Some((method, path, query)) = parse_request_line(request) else {
        return json_response(
            400,
            "Bad Request",
            &serde_json::json!({
                "error": "invalid_request"
            }),
        );
    };

    match (method, path) {
        ("GET", "/v1/state") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match build_state_snapshot_for_project_from_store(store, pty, version, &project_id) {
                Ok(snapshot) => json_response(200, "OK", &snapshot),
                Err(error) => json_response(
                    500,
                    "Internal Server Error",
                    &serde_json::json!({ "error": error }),
                ),
            }
        }
        ("GET", "/v1/health") => json_response(200, "OK", &health_from_store(store)),
        ("GET", "/v1/update/check") => {
            let channel = query_param(query, "channel").unwrap_or_else(|| "stable".to_string());
            match update_check_response(&channel, version) {
                Ok(update) => json_response(200, "OK", &update),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/projects") => match store
            .list_projects()
            .and_then(|items| snapshot_list_response(store, pty, version, "proj_local", items))
        {
            Ok(response) => json_response(200, "OK", &response),
            Err(error) => server_error_response(error),
        },
        ("GET", path) if path.starts_with("/v1/projects/") && path.ends_with("/files") => {
            match project_id_from_files_path(path)
                .map(|project_id| crate::state_store::ProjectFileListInput {
                    project_id,
                    relative_path: query_param(query, "path"),
                })
                .and_then(|payload| store.list_project_files(payload))
            {
                Ok(files) => json_response(200, "OK", &files),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/projects/") && path.ends_with("/files/search") => {
            match project_id_from_files_search_path(path)
                .and_then(|project_id| {
                    query_param(query, "query")
                        .ok_or_else(|| "query cannot be empty".to_string())
                        .map(|search_query| crate::state_store::ProjectFileSearchInput {
                            project_id,
                            query: search_query,
                        })
                })
                .and_then(|payload| store.search_project_files(payload))
            {
                Ok(results) => json_response(200, "OK", &results),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/projects/") && path.ends_with("/file") => {
            match project_id_from_file_path(path)
                .and_then(|project_id| {
                    query_param(query, "path")
                        .ok_or_else(|| "project file path cannot be empty".to_string())
                        .map(|file_path| crate::state_store::ProjectFileReadInput {
                            project_id,
                            path: file_path,
                        })
                })
                .and_then(|payload| store.read_project_file(payload))
            {
                Ok(preview) => json_response(200, "OK", &preview),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/projects/") && path.ends_with("/file") => {
            match project_id_from_file_path(path)
                .and_then(|project_id| {
                    parse_json_body::<ProjectFileWriteRequest>(request)
                        .map(|payload| (project_id, payload))
                })
                .and_then(|(project_id, payload)| {
                    store.write_project_file(crate::state_store::ProjectFileWriteInput {
                        project_id,
                        path: payload.path,
                        body: payload.body,
                    })
                })
                .and_then(|preview| {
                    let project_id = preview.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, preview)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/projects/") && path.ends_with("/diff") => {
            match project_id_from_diff_path(path)
                .map(|project_id| crate::state_store::ProjectDiffInput {
                    project_id,
                    path: query_param(query, "path"),
                })
                .and_then(|payload| store.read_project_diff(payload))
            {
                Ok(diff) => json_response(200, "OK", &diff),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path)
            if path.starts_with("/v1/projects/") && path.ends_with("/lsp-diagnostics") =>
        {
            match project_id_from_action_path(path, "lsp-diagnostics")
                .map(
                    |project_id| crate::state_store::ProjectLspDiagnosticsInput {
                        project_id,
                        path: query_param(query, "path"),
                    },
                )
                .and_then(|payload| store.collect_project_lsp_diagnostics(payload))
            {
                Ok(diagnostics) => json_response(200, "OK", &diagnostics),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/projects/") && path.ends_with("/patch/export") => {
            match project_id_from_nested_action_path(path, "patch/export")
                .map(|project_id| crate::state_store::ProjectDiffInput {
                    project_id,
                    path: query_param(query, "path"),
                })
                .and_then(|payload| store.export_project_patch(payload))
            {
                Ok(patch) => json_response(200, "OK", &patch),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/projects/") && path.ends_with("/patch/import") => {
            match project_id_from_nested_action_path(path, "patch/import")
                .and_then(|project_id| {
                    parse_json_body::<PatchImportRequest>(request)
                        .map(|payload| (project_id, payload.body))
                })
                .and_then(|(project_id, body)| {
                    store.import_project_patch(crate::state_store::ImportPatchInput {
                        project_id,
                        body,
                    })
                })
                .and_then(|patch| {
                    let project_id = patch.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, patch)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path)
            if path.starts_with("/v1/projects/") && path.ends_with("/pr/landing-plan") =>
        {
            match project_id_from_nested_action_path(path, "pr/landing-plan")
                .and_then(|project_id| {
                    parse_json_body::<PrLandingPlanRequest>(request).map(|payload| {
                        crate::state_store::PlanPrLandingInput {
                            project_id,
                            title: payload.title,
                            draft: payload.draft.unwrap_or(false),
                        }
                    })
                })
                .and_then(|payload| store.plan_pr_landing(payload))
                .and_then(|plan| {
                    let project_id = plan.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, plan)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/projects") => {
            match parse_json_body::<crate::state_store::AddProjectInput>(request)
                .and_then(|payload| store.add_project(payload))
                .and_then(|project| {
                    let project_id = project.id.clone();
                    snapshot_object_response(store, pty, version, &project_id, project)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/projects/") && path.ends_with("/focus") => {
            match project_id_from_action_path(path, "focus")
                .and_then(|project_id| store.focus_project(&project_id))
                .and_then(|project| {
                    let project_id = project.id.clone();
                    snapshot_object_response(store, pty, version, &project_id, project)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/projects/") && path.ends_with("/detach") => {
            match project_id_from_action_path(path, "detach").and_then(|project_id| {
                store.plan_project_detach(&project_id).and_then(|plan| {
                    snapshot_object_response(store, pty, version, &project_id, plan)
                })
            }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/projects/") && path.ends_with("/layout") => {
            match project_id_from_action_path(path, "layout")
                .and_then(|project_id| {
                    parse_json_body::<ProjectLayoutRequest>(request).map(|payload| {
                        crate::state_store::UpdateProjectTabLayoutInput {
                            project_id,
                            layout_json: payload.layout_json,
                        }
                    })
                })
                .and_then(|payload| store.update_project_tab_layout(payload))
                .and_then(|tab| {
                    let project_id = tab.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, tab)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/projects/") && path.ends_with("/layout-presets") => {
            match project_id_from_action_path(path, "layout-presets").and_then(|project_id| {
                store
                    .list_project_layout_presets(&project_id)
                    .and_then(|items| {
                        snapshot_list_response(store, pty, version, &project_id, items)
                    })
            }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path)
            if path.starts_with("/v1/projects/") && path.ends_with("/layout-presets") =>
        {
            match project_id_from_action_path(path, "layout-presets")
                .and_then(|project_id| {
                    parse_json_body::<ProjectLayoutPresetRequest>(request).map(|payload| {
                        crate::state_store::SaveProjectLayoutPresetInput {
                            project_id,
                            name: payload.name,
                            layout_json: payload.layout_json,
                        }
                    })
                })
                .and_then(|payload| store.save_project_layout_preset(payload))
                .and_then(|preset| {
                    let project_id = preset.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, preset)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/projects/") && path.ends_with("/tab-group") => {
            match project_id_from_action_path(path, "tab-group")
                .and_then(|project_id| {
                    parse_json_body::<ProjectTabGroupRequest>(request)
                        .map(|payload| (project_id, payload.group_name))
                })
                .and_then(|(project_id, group_name)| {
                    store.upsert_project_tab_group(&project_id, &group_name)
                })
                .and_then(|group| {
                    let project_id = group.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, group)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/sessions") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            let state = query_param(query, "state");
            let task_id = query_param(query, "taskId");
            match store
                .list_sessions(&project_id)
                .map(|items| filter_session_list_items(items, state.as_deref(), task_id.as_deref()))
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("GET", "/v1/runtime-pool") => {
            let project_id = query_param(query, "projectId")
                .or_else(|| query_param(query, "project_id"))
                .or_else(|| query_param(query, "project"))
                .unwrap_or_else(|| "proj_local".to_string());
            match store
                .runtime_pool(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/sessions") => {
            match parse_json_body::<crate::state_store::CreateSessionInput>(request)
                .and_then(|payload| store.create_session(payload))
                .and_then(|session| {
                    let project_id = session.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, session)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/sessions/") && path.ends_with("/stream-chunks") => {
            match session_id_from_action_path(path, "stream-chunks").and_then(|session_id| {
                let limit = query_param(query, "limit")
                    .map(|value| {
                        value
                            .parse::<usize>()
                            .map_err(|_| "terminal stream chunk limit must be a number".to_string())
                    })
                    .transpose()?;
                let project_id = store
                    .get_session(&session_id)?
                    .ok_or_else(|| format!("session {session_id} not found"))?
                    .project_id;
                store
                    .list_terminal_stream_chunks(&session_id, limit)
                    .and_then(|items| {
                        snapshot_list_response(store, pty, version, &project_id, items)
                    })
            }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/sessions/") && path.ends_with("/stream-chunks") => {
            match session_id_from_action_path(path, "stream-chunks")
                .and_then(|session_id| {
                    parse_json_body::<TerminalStreamChunkRequest>(request).map(|payload| {
                        crate::state_store::RecordTerminalStreamChunkInput {
                            session_id,
                            seq_start: payload.seq_start,
                            seq_end: payload.seq_end,
                            body: payload.body,
                        }
                    })
                })
                .and_then(|payload| store.record_terminal_stream_chunk(payload))
                .and_then(|chunk| {
                    let project_id = store
                        .get_session(&chunk.session_id)?
                        .ok_or_else(|| format!("session {} not found", chunk.session_id))?
                        .project_id;
                    snapshot_object_response(store, pty, version, &project_id, chunk)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/sessions/") && path.ends_with("/focus") => {
            match session_id_from_action_path(path, "focus")
                .and_then(|session_id| store.focus_session(&session_id))
                .and_then(|session| {
                    let project_id = session.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, session)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/sessions/") && path.ends_with("/attention") => {
            match session_id_from_action_path(path, "attention")
                .and_then(|session_id| {
                    parse_json_body::<SessionAttentionRequest>(request)
                        .map(|payload| (session_id, payload.attention_state))
                })
                .and_then(|(session_id, attention_state)| {
                    store.set_session_attention(&session_id, &attention_state)
                })
                .and_then(|session| {
                    let project_id = session.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, session)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/sessions/") && path.ends_with("/resize") => {
            match session_id_from_action_path(path, "resize")
                .and_then(|session_id| {
                    parse_json_body::<SessionResizeRequest>(request)
                        .map(|payload| (session_id, payload.cols, payload.rows))
                })
                .and_then(|(session_id, cols, rows)| store.resize_session(&session_id, cols, rows))
                .and_then(|receipt| {
                    let project_id = store
                        .get_session(&receipt.session_id)?
                        .ok_or_else(|| format!("session {} not found", receipt.session_id))?
                        .project_id;
                    snapshot_object_response(store, pty, version, &project_id, receipt)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/sessions/") && path.ends_with("/input") => {
            match session_id_from_action_path(path, "input")
                .and_then(|session_id| {
                    parse_json_body::<SessionInputRequest>(request).map(|payload| {
                        crate::state_store::SessionInputInput {
                            session_id,
                            text: payload.text,
                            allow_dangerous: payload.allow_dangerous,
                        }
                    })
                })
                .and_then(|payload| store.record_session_input(payload))
                .and_then(|receipt| {
                    let project_id = store
                        .get_session(&receipt.session_id)?
                        .ok_or_else(|| format!("session {} not found", receipt.session_id))?
                        .project_id;
                    snapshot_object_response(store, pty, version, &project_id, receipt)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/sessions/") && path.ends_with("/attach-task") => {
            match session_id_from_action_path(path, "attach-task")
                .and_then(|session_id| {
                    parse_json_body::<SessionAttachTaskRequest>(request)
                        .map(|payload| (session_id, payload.task_id))
                })
                .and_then(|(session_id, task_id)| store.attach_session_task(&session_id, &task_id))
                .and_then(|session| {
                    let project_id = session.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, session)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/sessions/") && path.ends_with("/detach-task") => {
            match session_id_from_action_path(path, "detach-task")
                .and_then(|session_id| store.detach_session_task(&session_id))
                .and_then(|session| {
                    let project_id = session.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, session)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/sessions/") && path.ends_with("/takeover") => {
            match session_id_from_action_path(path, "takeover")
                .and_then(|session_id| store.takeover_session(&session_id))
                .and_then(|session| {
                    let project_id = session.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, session)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/sessions/") && path.ends_with("/release") => {
            match session_id_from_action_path(path, "release")
                .and_then(|session_id| store.release_session(&session_id))
                .and_then(|session| {
                    let project_id = session.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, session)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/sessions/") && path.ends_with("/kill") => {
            match session_id_from_action_path(path, "kill")
                .and_then(|session_id| store.kill_session(&session_id))
                .and_then(|session| {
                    let project_id = session.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, session)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/sessions/") && path.ends_with("/token-usage") => {
            match session_id_from_action_path(path, "token-usage")
                .and_then(|session_id| store.token_usage_summary_for_session(&session_id))
            {
                Ok(summary) => json_response(200, "OK", &summary),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/agents") => match store
            .list_agent_profiles()
            .and_then(|items| snapshot_list_response(store, pty, version, "proj_local", items))
        {
            Ok(response) => json_response(200, "OK", &response),
            Err(error) => server_error_response(error),
        },
        ("POST", "/v1/agents") => {
            match parse_json_body::<crate::state_store::UpsertAgentProfileInput>(request)
                .and_then(|payload| store.upsert_agent_profile(payload))
                .and_then(|agent| {
                    snapshot_object_response(store, pty, version, "proj_local", agent)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/agents/scan") => match store
            .scan_agent_profiles()
            .and_then(|items| snapshot_list_response(store, pty, version, "proj_local", items))
        {
            Ok(response) => json_response(201, "Created", &response),
            Err(error) => server_error_response(error),
        },
        ("POST", path) if path.starts_with("/v1/agents/") && path.ends_with("/pause") => {
            match agent_id_from_action_path(path, "pause")
                .and_then(|agent_id| store.update_agent_profile_status(&agent_id, "paused"))
                .and_then(|agent| {
                    snapshot_object_response(store, pty, version, "proj_local", agent)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/agents/") && path.ends_with("/resume") => {
            match agent_id_from_action_path(path, "resume")
                .and_then(|agent_id| store.update_agent_profile_status(&agent_id, "available"))
                .and_then(|agent| {
                    snapshot_object_response(store, pty, version, "proj_local", agent)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/agents/") && path.ends_with("/heartbeat") => {
            match agent_id_from_action_path(path, "heartbeat")
                .and_then(|agent_id| store.heartbeat_agent_profile(&agent_id))
                .and_then(|agent| {
                    snapshot_object_response(store, pty, version, "proj_local", agent)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/skill-packs") => {
            let project_id = query_param(query, "projectId")
                .or_else(|| query_param(query, "project_id"))
                .or_else(|| query_param(query, "project"))
                .unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_skill_packs(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/skill-packs") => {
            match parse_json_body::<crate::state_store::UpsertSkillPackInput>(request)
                .and_then(|payload| store.upsert_skill_pack(payload))
                .and_then(|pack| {
                    let project_id = pack.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, pack)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/provider-model") => match store.provider_model_settings() {
            Ok(settings) => json_response(200, "OK", &settings),
            Err(error) => server_error_response(error),
        },
        ("POST", "/v1/provider-model") => {
            match parse_json_body::<crate::state_store::ProviderModelSettingsInput>(request)
                .and_then(|payload| store.upsert_provider_model_settings(payload))
                .and_then(|settings| {
                    snapshot_object_response(store, pty, version, "proj_local", settings)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/terminal-theme") => {
            match store.terminal_theme_settings(query_param(query, "projectId").as_deref()) {
                Ok(theme) => json_response(200, "OK", &theme),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/terminal-theme") => {
            match parse_json_body::<crate::state_store::TerminalThemeSettingsInput>(request)
                .and_then(|payload| store.upsert_terminal_theme_settings(payload))
                .and_then(|theme| {
                    let project_id = theme
                        .project_id
                        .clone()
                        .unwrap_or_else(|| "proj_local".to_string());
                    snapshot_object_response(store, pty, version, &project_id, theme)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/policy/approvals") => {
            let project_id =
                query_param(query, "project").unwrap_or_else(|| "proj_local".to_string());
            let state = query_param(query, "state");
            match store
                .list_policy_approvals(&project_id, state.as_deref())
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/policy/approvals") => {
            match parse_json_body::<crate::state_store::CreatePolicyApprovalInput>(request)
                .and_then(|payload| store.create_policy_approval(payload))
                .and_then(|approval| {
                    let project_id = approval.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, approval)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path)
            if path.starts_with("/v1/policy/approvals/") && path.ends_with("/decision") =>
        {
            match policy_approval_id_from_decision_path(path)
                .and_then(|approval_id| {
                    parse_json_body::<PolicyDecisionRequest>(request)
                        .map(|payload| (approval_id, payload))
                })
                .and_then(|(approval_id, payload)| {
                    store.decide_policy_approval(crate::state_store::DecidePolicyApprovalInput {
                        approval_id,
                        decision: payload.decision,
                        decision_by: payload.decision_by,
                        decision_note: payload.decision_note,
                    })
                })
                .and_then(|approval| {
                    let project_id = approval.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, approval)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/policy/packs") => {
            let project_id =
                query_param(query, "project").unwrap_or_else(|| "proj_local".to_string());
            let active = query_param(query, "active");
            match parse_optional_bool_filter(active.as_deref(), "policy pack active")
                .and_then(|active| store.list_policy_packs(&project_id, active))
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/policy/packs") => {
            match parse_json_body::<crate::state_store::UpsertPolicyPackInput>(request)
                .and_then(|payload| store.upsert_policy_pack(payload))
                .and_then(|pack| {
                    let project_id = pack.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, pack)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/policy/audit") => {
            let project_id =
                query_param(query, "project").unwrap_or_else(|| "proj_local".to_string());
            let decision = query_param(query, "decision");
            let action_kind =
                query_param(query, "actionKind").or_else(|| query_param(query, "action"));
            let run_id = query_param(query, "run").or_else(|| query_param(query, "runId"));
            let task_id = query_param(query, "task").or_else(|| query_param(query, "taskId"));
            match store
                .list_permission_audit(
                    &project_id,
                    decision.as_deref(),
                    action_kind.as_deref(),
                    run_id.as_deref(),
                    task_id.as_deref(),
                )
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/policy/evaluate") => {
            match parse_json_body::<crate::state_store::EvaluatePolicyActionInput>(request)
                .and_then(|payload| store.evaluate_policy_action(payload))
                .and_then(|evaluation| {
                    let project_id = evaluation.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, evaluation)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/budgets") => match store.budget_summary() {
            Ok(summary) => json_response(200, "OK", &summary),
            Err(error) => server_error_response(error),
        },
        ("GET", "/v1/budgets/forecast") => match store.budget_forecast() {
            Ok(forecast) => json_response(200, "OK", &forecast),
            Err(error) => server_error_response(error),
        },
        ("POST", "/v1/budgets") => {
            match parse_json_body::<crate::state_store::UpsertBudgetInput>(request)
                .and_then(|payload| store.upsert_budget(payload))
                .and_then(|budget| {
                    let project_id = if budget.scope_type == "project" {
                        budget
                            .scope_id
                            .clone()
                            .unwrap_or_else(|| "proj_local".to_string())
                    } else {
                        "proj_local".to_string()
                    };
                    snapshot_object_response(store, pty, version, &project_id, budget)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/token-usage") => {
            match parse_json_body::<crate::state_store::TokenUsageInput>(request)
                .and_then(|payload| store.record_token_usage(payload))
                .and_then(|usage| {
                    let project_id = usage
                        .project_id
                        .clone()
                        .unwrap_or_else(|| "proj_local".to_string());
                    snapshot_object_response(store, pty, version, &project_id, usage)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/token-usage/ingest") => {
            match parse_json_body::<crate::state_store::IngestTokenUsageAdapterInput>(request)
                .and_then(|payload| store.ingest_token_usage_adapter(payload))
                .and_then(|usage| {
                    let project_id = usage
                        .project_id
                        .clone()
                        .unwrap_or_else(|| "proj_local".to_string());
                    snapshot_object_response(store, pty, version, &project_id, usage)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/agent-events/ingest") => {
            match parse_json_body::<crate::state_store::IngestAgentEventsInput>(request)
                .and_then(|payload| store.ingest_agent_events(payload))
                .and_then(|event| {
                    let project_id = event.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, event)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/provider-prices") => match store
            .list_provider_prices()
            .and_then(|items| snapshot_list_response(store, pty, version, "proj_local", items))
        {
            Ok(response) => json_response(200, "OK", &response),
            Err(error) => server_error_response(error),
        },
        ("POST", "/v1/provider-prices/update") => {
            match parse_json_body::<crate::state_store::UpdateProviderPriceTableInput>(request)
                .and_then(|payload| store.update_provider_price_table(payload))
                .and_then(|update| {
                    snapshot_object_response(store, pty, version, "proj_local", update)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/secrets") => {
            let project_id = query_param(query, "project");
            let name = query_param(query, "name");
            match store
                .list_secrets(project_id.as_deref(), name.as_deref())
                .and_then(|items| {
                    snapshot_list_response(
                        store,
                        pty,
                        version,
                        project_id.as_deref().unwrap_or("proj_local"),
                        items,
                    )
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/secrets") => {
            match parse_json_body::<crate::state_store::UpsertSecretInput>(request)
                .and_then(|payload| store.upsert_secret(payload))
                .and_then(|secret| {
                    let project_id = secret.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, secret)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/knowledge") => {
            let project_id = query_param(query, "projectId")
                .or_else(|| query_param(query, "project_id"))
                .unwrap_or_else(|| "proj_local".to_string());
            match store.search_knowledge_pages(&project_id, query_param(query, "query").as_deref())
            {
                Ok(items) => {
                    match snapshot_list_response(store, pty, version, &project_id, items) {
                        Ok(response) => json_response(200, "OK", &response),
                        Err(error) => server_error_response(error),
                    }
                }
                Err(error) => server_error_response(error),
            }
        }
        ("GET", "/v1/knowledge/sources") => {
            let project_id = query_param(query, "projectId")
                .or_else(|| query_param(query, "project_id"))
                .unwrap_or_else(|| "proj_local".to_string());
            match store.list_knowledge_sources(&project_id) {
                Ok(items) => {
                    match snapshot_list_response(store, pty, version, &project_id, items) {
                        Ok(response) => json_response(200, "OK", &response),
                        Err(error) => server_error_response(error),
                    }
                }
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/knowledge/sources") => {
            match parse_json_body::<crate::state_store::UpsertKnowledgeSourceInput>(request)
                .and_then(|payload| store.upsert_knowledge_source(payload))
                .and_then(|source| {
                    let project_id = source.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, source)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/knowledge/pages") => {
            match parse_json_body::<crate::state_store::SaveKnowledgePageInput>(request)
                .and_then(|payload| store.save_knowledge_page(payload))
                .and_then(|page| {
                    let project_id = page.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, page)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/knowledge/explorations") => {
            let project_id = query_param(query, "projectId")
                .or_else(|| query_param(query, "project_id"))
                .unwrap_or_else(|| "proj_local".to_string());
            match store.list_knowledge_explorations(&project_id) {
                Ok(items) => {
                    match snapshot_list_response(store, pty, version, &project_id, items) {
                        Ok(response) => json_response(200, "OK", &response),
                        Err(error) => server_error_response(error),
                    }
                }
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/knowledge/explorations") => {
            match parse_json_body::<crate::state_store::SaveKnowledgeExplorationInput>(request)
                .and_then(|payload| store.save_knowledge_exploration(payload))
                .and_then(|exploration| {
                    let project_id = exploration.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, exploration)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/knowledge/concepts") => {
            let project_id = query_param(query, "projectId")
                .or_else(|| query_param(query, "project_id"))
                .unwrap_or_else(|| "proj_local".to_string());
            match store.list_knowledge_concepts(&project_id) {
                Ok(items) => {
                    match snapshot_list_response(store, pty, version, &project_id, items) {
                        Ok(response) => json_response(200, "OK", &response),
                        Err(error) => server_error_response(error),
                    }
                }
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/knowledge/obsidian/export") => {
            match parse_json_body::<KnowledgeProjectRequest>(request)
                .and_then(|payload| store.export_knowledge_obsidian_markdown(&payload.project_id))
                .and_then(|export| {
                    let project_id = export.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, export)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/knowledge/chat") => {
            match parse_json_body::<crate::state_store::KnowledgeChatQuestionInput>(request)
                .and_then(|payload| store.answer_knowledge_question(payload))
                .and_then(|answer| {
                    let project_id = answer.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, answer)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/knowledge/lint") => {
            match parse_json_body::<crate::state_store::RecordKnowledgeLintReportInput>(request)
                .and_then(|payload| store.record_knowledge_lint_report(payload))
                .and_then(|report| {
                    let project_id = report.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, report)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/knowledge/automation/run") => {
            match parse_json_body::<crate::state_store::RunKnowledgeAutomationInput>(request)
                .and_then(|payload| store.run_knowledge_automation(payload))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/knowledge/ingest") => {
            match parse_json_body::<crate::state_store::IngestKnowledgeArtifactInput>(request)
                .and_then(|payload| store.ingest_knowledge_artifact(payload))
                .and_then(|result| {
                    let project_id = result.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, result)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/knowledge/") => {
            match id_from_prefix(path, "/v1/knowledge/").and_then(|page_id| {
                store.get_knowledge_page(&page_id).and_then(|page| {
                    page.ok_or_else(|| format!("knowledge page {page_id} not found"))
                })
            }) {
                Ok(page) => json_response(200, "OK", &page),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/context-packs") => {
            let project_id = query_param(query, "projectId")
                .or_else(|| query_param(query, "project_id"))
                .unwrap_or_else(|| "proj_local".to_string());
            match store.list_context_packs(&project_id) {
                Ok(items) => {
                    match snapshot_list_response(store, pty, version, &project_id, items) {
                        Ok(response) => json_response(200, "OK", &response),
                        Err(error) => server_error_response(error),
                    }
                }
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/context-packs") => {
            match parse_json_body::<crate::state_store::UpsertContextPackInput>(request)
                .and_then(|payload| store.upsert_context_pack(payload))
                .and_then(|pack| {
                    let project_id = pack.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, pack)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/context-packs/") => {
            match id_from_prefix(path, "/v1/context-packs/").and_then(|pack_id| {
                store.get_context_pack(&pack_id).and_then(|pack| {
                    pack.ok_or_else(|| format!("context pack {pack_id} not found"))
                })
            }) {
                Ok(pack) => json_response(200, "OK", &pack),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/dispatch") => {
            match parse_json_body::<crate::state_store::DispatchRunInput>(request)
                .and_then(|payload| store.dispatch_run(payload))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/runs") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            let lifecycle = query_param(query, "lifecycle");
            let task_id = query_param(query, "taskId");
            let agent_profile_id = query_param(query, "agentProfileId");
            match store
                .list_runs(&project_id)
                .map(|items| {
                    filter_run_list_items(
                        items,
                        lifecycle.as_deref(),
                        task_id.as_deref(),
                        agent_profile_id.as_deref(),
                    )
                })
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("GET", path) if is_run_resource_path(path) => {
            match run_id_from_resource_path(path).and_then(|run_id| {
                store
                    .get_run(&run_id)
                    .and_then(|run| run.ok_or_else(|| format!("run {run_id} not found")))
            }) {
                Ok(run) => json_response(200, "OK", &run),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/runs/") && path.ends_with("/token-usage") => {
            match run_id_from_action_path(path, "token-usage")
                .and_then(|run_id| store.token_usage_export_for_run(&run_id))
            {
                Ok(usage) => json_response(200, "OK", &usage),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/runs/") && path.ends_with("/replay") => {
            match run_id_from_action_path(path, "replay")
                .and_then(|run_id| store.get_run_replay_metadata(&run_id))
            {
                Ok(Some(metadata)) => json_response(200, "OK", &metadata),
                Ok(None) => json_response(
                    404,
                    "Not Found",
                    &serde_json::json!({ "error": "run_replay_not_found" }),
                ),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/runs/") && path.ends_with("/evidence/generate") => {
            match run_id_from_nested_action_path(path, "evidence/generate")
                .and_then(|run_id| {
                    parse_json_body::<GenerateEvidenceRequest>(request)
                        .map(|payload| (run_id, payload))
                })
                .and_then(|(run_id, payload)| {
                    store.generate_evidence_pack_for_run(
                        crate::state_store::GenerateEvidencePackInput {
                            run_id,
                            evidence_pack_id: payload.evidence_pack_id,
                        },
                    )
                })
                .and_then(|evidence| {
                    let project_id = project_id_for_evidence_pack(store, &evidence)?;
                    snapshot_object_response(store, pty, version, &project_id, evidence)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path)
            if path.starts_with("/v1/evidence/") && path.ends_with("/review-decision") =>
        {
            match evidence_id_from_action_path(path, "review-decision")
                .and_then(|evidence_pack_id| {
                    parse_json_body::<ReviewDecisionRequest>(request)
                        .map(|payload| (evidence_pack_id, payload))
                })
                .and_then(|(evidence_pack_id, payload)| {
                    store.record_evidence_review_decision(
                        crate::state_store::RecordEvidenceReviewDecisionInput {
                            evidence_pack_id,
                            decision: payload.decision,
                            reviewer_id: payload.reviewer_id,
                            body_md: payload.body_md,
                        },
                    )
                })
                .and_then(|evidence| {
                    let project_id = project_id_for_evidence_pack(store, &evidence)?;
                    snapshot_object_response(store, pty, version, &project_id, evidence)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/reviews") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            let state = query_param(query, "state");
            let completeness = query_param(query, "completeness");
            match build_state_snapshot_for_project_from_store(store, pty, version, &project_id) {
                Ok(snapshot) => json_response(
                    200,
                    "OK",
                    &serde_json::json!({
                        "snapshot_id": snapshot.snapshot_id,
                        "items": filter_review_list_items(
                            snapshot.reviews,
                            state.as_deref(),
                            completeness.as_deref()
                        )
                    }),
                ),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/reviews/") && path.ends_with("/follow-up-task") => {
            match review_id_from_follow_up_task_path(path)
                .and_then(|review_id| {
                    parse_json_body::<ReviewFollowUpTaskRequest>(request)
                        .map(|payload| (review_id, payload))
                })
                .and_then(|(review_id, payload)| {
                    store.create_review_follow_up_task(
                        crate::state_store::CreateReviewFollowUpTaskInput {
                            review_id,
                            title: payload.title,
                            priority: payload.priority,
                        },
                    )
                })
                .and_then(|receipt| {
                    let project_id = receipt.task.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, receipt)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path)
            if path.starts_with("/v1/reviews/") && path.ends_with("/pr/landing-plan") =>
        {
            match review_id_from_pr_landing_plan_path(path)
                .and_then(|review_id| {
                    parse_json_body::<ReviewPrLandingPlanRequest>(request)
                        .map(|payload| (review_id, payload))
                })
                .and_then(|(review_id, payload)| {
                    store.plan_review_pr_landing(crate::state_store::PlanReviewPrLandingInput {
                        review_id,
                        title: payload.title,
                        draft: payload.draft,
                    })
                })
                .and_then(|receipt| {
                    let project_id = receipt.plan.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, receipt)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/reviews/") && path.ends_with("/decision") => {
            match review_id_from_decision_path(path)
                .and_then(|evidence_pack_id| {
                    parse_json_body::<ReviewDecisionRequest>(request)
                        .map(|payload| (evidence_pack_id, payload))
                })
                .and_then(|(evidence_pack_id, payload)| {
                    store.record_evidence_review_decision(
                        crate::state_store::RecordEvidenceReviewDecisionInput {
                            evidence_pack_id,
                            decision: payload.decision,
                            reviewer_id: payload.reviewer_id,
                            body_md: payload.body_md,
                        },
                    )
                })
                .and_then(|evidence| {
                    let project_id = project_id_for_evidence_pack(store, &evidence)?;
                    snapshot_object_response(store, pty, version, &project_id, evidence)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/release-gates/runs") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_release_gate_runs(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/release-gates/run") => {
            match parse_json_body::<crate::state_store::RunReleaseGatesInput>(request)
                .and_then(|payload| store.run_release_gate_scenarios(payload))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/terminal-fidelity/smoke/runs") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_terminal_fidelity_smoke_runs(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/terminal-fidelity/smoke/run") => {
            match parse_json_body::<crate::state_store::RunTerminalFidelitySmokeInput>(request)
                .and_then(|payload| store.run_terminal_fidelity_smoke_tests(payload))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/task-lifecycle/e2e/runs") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_task_lifecycle_e2e_runs(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/task-lifecycle/e2e/run") => {
            match parse_json_body::<crate::state_store::RunTaskLifecycleE2EInput>(request)
                .and_then(|payload| store.run_task_lifecycle_e2e(payload))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/workflow/negative-tests/runs") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_workflow_negative_test_runs(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/workflow/negative-tests/run") => {
            match parse_json_body::<crate::state_store::RunWorkflowNegativeTestsInput>(request)
                .and_then(|payload| store.run_workflow_negative_tests(payload))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/distribution/dmg-smoke/runs") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_dmg_smoke_runs(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/distribution/dmg-smoke/run") => {
            match parse_json_body::<crate::state_store::RunDmgSmokeInput>(request)
                .and_then(|payload| store.run_dmg_smoke_test(payload))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/recovery/drills/runs") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_recovery_drill_runs(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/recovery/drills/run") => {
            match parse_json_body::<crate::state_store::RunRecoveryDrillsInput>(request)
                .and_then(|payload| store.run_recovery_drills(payload))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/benchmarks/runs") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_benchmark_runs(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/benchmarks/run") => {
            match parse_json_body::<crate::state_store::RunBenchmarksInput>(request)
                .and_then(|payload| store.run_benchmarks(payload))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/dogfood/telemetry-reviews") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_dogfood_telemetry_reviews(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/dogfood/telemetry-review/run") => {
            match parse_json_body::<crate::state_store::RunDogfoodTelemetryReviewInput>(request)
                .and_then(|payload| store.run_dogfood_telemetry_review(payload))
                .and_then(|review| {
                    let project_id = review.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, review)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/visual-harness/graph") => {
            match build_state_snapshot_for_project_from_store(
                store,
                pty,
                version,
                query_param(query, "projectId")
                    .as_deref()
                    .unwrap_or("proj_local"),
            ) {
                Ok(snapshot) => json_response(200, "OK", &snapshot.visual_harness),
                Err(error) => server_error_response(error),
            }
        }
        ("GET", "/v1/visual-harness/links") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_visual_harness_links(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/visual-harness/links") => {
            match parse_json_body::<crate::state_store::CreateVisualHarnessLinkInput>(request)
                .and_then(|payload| store.create_visual_harness_link(payload))
                .and_then(|link| {
                    let project_id = link.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, link)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/workflow/status") => match store.workflow_runtime_state(
            query_param(query, "projectId")
                .as_deref()
                .unwrap_or("proj_local"),
        ) {
            Ok(state) => json_response(200, "OK", &state),
            Err(error) => server_error_response(error),
        },
        ("POST", "/v1/workflow/reload") => {
            match parse_json_body::<ReloadWorkflowRequest>(request)
                .and_then(|payload| reload_workflow_from_request(store, payload))
                .and_then(|workflow| {
                    let project_id = workflow.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, workflow)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/workflow/validate") => {
            match parse_json_body::<ReloadWorkflowRequest>(request)
                .and_then(|payload| validate_workflow_from_request(store, payload))
                .and_then(|validation| {
                    let project_id = validation.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, validation)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path)
            if path.starts_with("/v1/runs/")
                && path.contains("/hooks/")
                && path.ends_with("/run") =>
        {
            match run_hook_path_parts(path)
                .and_then(|(run_id, hook_name)| {
                    parse_json_body::<RunHookRequest>(request)
                        .map(|payload| (run_id, hook_name, payload))
                })
                .and_then(|(run_id, hook_name, payload)| {
                    store.run_workflow_hook(crate::state_store::RunWorkflowHookInput {
                        run_id,
                        hook_name,
                        repo_root: payload.repo_root,
                        workspace_path: payload.workspace_path,
                    })
                })
                .and_then(|result| {
                    let project_id = project_id_for_run_id(store, &result.run_id)?;
                    snapshot_object_response(store, pty, version, &project_id, result)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/runs/") && path.ends_with("/transition") => {
            match run_id_from_action_path(path, "transition")
                .and_then(|run_id| {
                    parse_json_body::<UpdateRunLifecycleRequest>(request)
                        .map(|payload| (run_id, payload))
                })
                .and_then(|(run_id, payload)| {
                    store.update_run_lifecycle(crate::state_store::UpdateRunLifecycleInput {
                        run_id,
                        lifecycle: payload.lifecycle,
                        status_detail: payload.status_detail,
                    })
                })
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/runs/") && path.ends_with("/cancel") => {
            match run_id_from_action_path(path, "cancel")
                .and_then(|run_id| store.cancel_run(&run_id))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/runs/") && path.ends_with("/retry") => {
            match run_id_from_action_path(path, "retry")
                .and_then(|run_id| store.retry_run(&run_id))
                .and_then(|run| {
                    let project_id = run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, run)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/runs/") && path.ends_with("/status-updates") => {
            match run_id_from_action_path(path, "status-updates")
                .and_then(|run_id| {
                    parse_json_body::<RecordRunStatusUpdateRequest>(request)
                        .map(|payload| (run_id, payload))
                })
                .and_then(|(run_id, payload)| {
                    store.record_run_status_update(RecordRunStatusUpdateInput {
                        run_id,
                        body_md: payload.body_md,
                        lifecycle: payload.lifecycle,
                        status_detail: payload.status_detail,
                    })
                })
                .and_then(|comment| {
                    let run_id = comment
                        .run_id
                        .clone()
                        .ok_or_else(|| "run status update response missing run id".to_string())?;
                    let project_id = store
                        .get_run(&run_id)?
                        .ok_or_else(|| format!("run {run_id} not found"))?
                        .project_id;
                    snapshot_object_response(store, pty, version, &project_id, comment)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/command-blocks") => {
            match store.search_command_blocks_for_task(
                query_param(query, "query").as_deref(),
                query_param(query, "taskId").as_deref(),
                query_param(query, "sessionId").as_deref(),
                50,
            ) {
                Ok(items) => {
                    match snapshot_list_response(store, pty, version, "proj_local", items) {
                        Ok(response) => json_response(200, "OK", &response),
                        Err(error) => server_error_response(error),
                    }
                }
                Err(error) => server_error_response(error),
            }
        }
        ("GET", path) if is_command_block_resource_path(path) => {
            match command_block_id_from_resource_path(path).and_then(|command_block_id| {
                store
                    .get_command_block(&command_block_id)
                    .and_then(|block| {
                        block.ok_or_else(|| format!("command block {command_block_id} not found"))
                    })
            }) {
                Ok(block) => json_response(200, "OK", &block),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/command-blocks/") && path.ends_with("/mark") => {
            match command_block_id_from_command_block_action_path(path, "mark")
                .and_then(|command_block_id| {
                    parse_json_body::<CommandBlockMarkRequest>(request)
                        .map(|payload| (command_block_id, payload.status))
                })
                .and_then(|(command_block_id, status)| {
                    store.mark_command_block_status(&command_block_id, &status)
                })
                .and_then(|block| {
                    let project_id = project_id_for_command_block(store, &block)?;
                    snapshot_object_response(store, pty, version, &project_id, block)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/command-blocks/") && path.ends_with("/merge") => {
            match command_block_id_from_command_block_action_path(path, "merge")
                .and_then(|command_block_id| {
                    parse_json_body::<CommandBlockMergeRequest>(request)
                        .map(|payload| (command_block_id, payload.second_command_block_id))
                })
                .and_then(|(first_command_block_id, second_command_block_id)| {
                    store.merge_command_blocks(&first_command_block_id, &second_command_block_id)
                })
                .and_then(|block| {
                    let project_id = project_id_for_command_block(store, &block)?;
                    snapshot_object_response(store, pty, version, &project_id, block)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/command-blocks/") && path.ends_with("/split") => {
            match command_block_id_from_command_block_action_path(path, "split")
                .and_then(|command_block_id| store.split_command_block(&command_block_id))
                .and_then(|receipt| {
                    let project_id = project_id_for_command_block(store, &receipt.updated_block)?;
                    snapshot_object_response(store, pty, version, &project_id, receipt)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/command-blocks/") && path.ends_with("/explain") => {
            match command_block_id_from_command_block_action_path(path, "explain")
                .and_then(|command_block_id| {
                    parse_json_body::<crate::state_store::ExplainCommandBlockInput>(request)
                        .map(|payload| (command_block_id, payload))
                })
                .and_then(|(command_block_id, payload)| {
                    store.explain_command_block(&command_block_id, payload)
                }) {
                Ok(explanation) => json_response(200, "OK", &explanation),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/command-blocks/") && path.ends_with("/bundle") => {
            match command_block_id_from_command_block_action_path(path, "bundle")
                .and_then(|command_block_id| store.export_command_block_bundle(&command_block_id))
            {
                Ok(bundle) => json_response(200, "OK", &bundle),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path)
            if path.starts_with("/v1/command-blocks/") && path.ends_with("/attach-evidence") =>
        {
            match command_block_id_from_attach_path(path)
                .and_then(|command_block_id| {
                    parse_json_body::<AttachEvidenceRequest>(request)
                        .map(|payload| (command_block_id, payload))
                })
                .and_then(|(command_block_id, payload)| {
                    store.attach_command_block_to_evidence(
                        crate::state_store::AttachCommandBlockEvidenceInput {
                            evidence_pack_id: payload.evidence_pack_id,
                            command_block_id,
                            task_id: payload.task_id,
                            run_id: payload.run_id,
                        },
                    )
                })
                .and_then(|evidence| {
                    let project_id = project_id_for_evidence_pack(store, &evidence)?;
                    snapshot_object_response(store, pty, version, &project_id, evidence)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/cycles") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_task_cycles(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/cycles") => match parse_json_body::<CreateTaskCycleInput>(request)
            .and_then(|payload| store.create_task_cycle(payload))
            .and_then(|cycle| {
                let project_id = cycle.project_id.clone();
                snapshot_object_response(store, pty, version, &project_id, cycle)
            }) {
            Ok(response) => json_response(201, "Created", &response),
            Err(error) => bad_request_response(error),
        },
        ("GET", "/v1/modules") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_task_modules(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/modules") => match parse_json_body::<CreateTaskModuleInput>(request)
            .and_then(|payload| store.create_task_module(payload))
            .and_then(|module| {
                let project_id = module.project_id.clone();
                snapshot_object_response(store, pty, version, &project_id, module)
            }) {
            Ok(response) => json_response(201, "Created", &response),
            Err(error) => bad_request_response(error),
        },
        ("GET", "/v1/tasks") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            let status = query_param(query, "status");
            let task_query = query_param(query, "query");
            match store
                .list_tasks(&project_id)
                .map(|items| {
                    filter_task_list_items(items, status.as_deref(), task_query.as_deref())
                })
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("GET", path) if is_task_resource_path(path) => {
            match task_id_from_resource_path(path, "/v1/tasks/").and_then(|task_id| {
                store
                    .get_task(&task_id)
                    .and_then(|task| task.ok_or_else(|| format!("task {task_id} not found")))
            }) {
                Ok(task) => json_response(200, "OK", &task),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/tasks") => match parse_json_body::<CreateTaskInput>(request)
            .and_then(|payload| store.create_task(payload))
            .and_then(|task| {
                let project_id = task.project_id.clone();
                snapshot_object_response(store, pty, version, &project_id, task)
            }) {
            Ok(response) => json_response(201, "Created", &response),
            Err(error) => bad_request_response(error),
        },
        ("PATCH", path) if path.starts_with("/v1/tasks/") => {
            match task_id_from_resource_path(path, "/v1/tasks/")
                .and_then(|task_id| {
                    parse_json_body::<UpdateTaskRequest>(request).map(|payload| (task_id, payload))
                })
                .and_then(|(task_id, payload)| {
                    store.update_task(UpdateTaskInput {
                        task_id,
                        title: payload.title,
                        description: payload.description,
                        priority: payload.priority,
                    })
                })
                .and_then(|task| {
                    let project_id = task.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, task)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/tasks/") && path.ends_with("/move") => {
            match task_id_from_action_path(path, "move")
                .and_then(|task_id| {
                    parse_json_body::<MoveTaskRequest>(request)
                        .map(|payload| (task_id, payload.status))
                })
                .and_then(|(task_id, status)| store.move_task_status(&task_id, &status))
                .and_then(|task| {
                    let project_id = task.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, task)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/tasks/") && path.ends_with("/comments") => {
            match task_id_from_action_path(path, "comments")
                .and_then(|task_id| {
                    parse_json_body::<AddCommentRequest>(request)
                        .map(|payload| (task_id, payload.body))
                })
                .and_then(|(task_id, body_md)| {
                    store.add_task_comment(AddTaskCommentInput {
                        task_id,
                        author_type: "human".to_string(),
                        author_id: "local_user".to_string(),
                        body_md,
                    })
                })
                .and_then(|comment| {
                    let task_id = comment
                        .task_id
                        .clone()
                        .ok_or_else(|| "task comment response missing task id".to_string())?;
                    let project_id = store
                        .get_task(&task_id)?
                        .ok_or_else(|| format!("task {task_id} not found"))?
                        .project_id;
                    snapshot_object_response(store, pty, version, &project_id, comment)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/tasks/") && path.ends_with("/comments") => {
            match task_id_from_action_path(path, "comments").and_then(|task_id| {
                let project_id = store
                    .get_task(&task_id)?
                    .ok_or_else(|| format!("task {task_id} not found"))?
                    .project_id;
                store.list_task_comments(&task_id).and_then(|items| {
                    snapshot_list_response(store, pty, version, &project_id, items)
                })
            }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/tasks/") && path.ends_with("/subtasks") => {
            match task_id_from_action_path(path, "subtasks")
                .and_then(|task_id| {
                    parse_json_body::<AddSubtaskRequest>(request)
                        .map(|payload| (task_id, payload.title))
                })
                .and_then(|(task_id, title)| {
                    store.add_task_subtask(AddTaskSubtaskInput { task_id, title })
                })
                .and_then(|subtask| {
                    let project_id = store
                        .get_task(&subtask.task_id)?
                        .ok_or_else(|| format!("task {} not found", subtask.task_id))?
                        .project_id;
                    snapshot_object_response(store, pty, version, &project_id, subtask)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", path) if path.starts_with("/v1/tasks/") && path.ends_with("/subtasks") => {
            match task_id_from_action_path(path, "subtasks").and_then(|task_id| {
                let project_id = store
                    .get_task(&task_id)?
                    .ok_or_else(|| format!("task {task_id} not found"))?
                    .project_id;
                store.list_task_subtasks(&task_id).and_then(|items| {
                    snapshot_list_response(store, pty, version, &project_id, items)
                })
            }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/tasks/") && path.ends_with("/status") => {
            match task_subtask_status_path(path)
                .and_then(|(task_id, subtask_id)| {
                    parse_json_body::<UpdateSubtaskStatusRequest>(request)
                        .map(|payload| (task_id, subtask_id, payload.status))
                })
                .and_then(|(task_id, subtask_id, status)| {
                    store.update_task_subtask_status(UpdateTaskSubtaskStatusInput {
                        task_id,
                        subtask_id,
                        status,
                    })
                })
                .and_then(|subtask| {
                    let project_id = store
                        .get_task(&subtask.task_id)?
                        .ok_or_else(|| format!("task {} not found", subtask.task_id))?
                        .project_id;
                    snapshot_object_response(store, pty, version, &project_id, subtask)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/tasks/") && path.ends_with("/workpad") => {
            match task_id_from_action_path(path, "workpad")
                .and_then(|task_id| {
                    parse_json_body::<SaveWorkpadRequest>(request)
                        .map(|payload| (task_id, payload.body))
                })
                .and_then(|(task_id, body_md)| {
                    store.save_task_workpad(SaveTaskWorkpadInput { task_id, body_md })
                })
                .and_then(|workpad| {
                    let project_id = store
                        .get_task(&workpad.task_id)?
                        .ok_or_else(|| format!("task {} not found", workpad.task_id))?
                        .project_id;
                    snapshot_object_response(store, pty, version, &project_id, workpad)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/tasks/") && path.ends_with("/planning") => {
            match task_id_from_action_path(path, "planning")
                .and_then(|task_id| {
                    parse_json_body::<UpdateTaskPlanningRequest>(request)
                        .map(|payload| (task_id, payload))
                })
                .and_then(|(task_id, payload)| {
                    store.update_task_planning(UpdateTaskPlanningInput {
                        task_id,
                        cycle_id: payload.cycle_id,
                        module_id: payload.module_id,
                        initiative_id: payload.initiative_id,
                        due_at: payload.due_at,
                        estimate: payload.estimate,
                        labels: payload.labels,
                        assignee_type: payload.assignee_type,
                        assignee_id: payload.assignee_id,
                    })
                })
                .and_then(|task| {
                    let project_id = task.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, task)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", path) if path.starts_with("/v1/tasks/") && path.ends_with("/context") => {
            match task_id_from_action_path(path, "context")
                .and_then(|task_id| {
                    parse_json_body::<UpdateTaskContextRequest>(request)
                        .map(|payload| (task_id, payload.context_pack_id))
                })
                .and_then(|(task_id, context_pack_id)| {
                    store.update_task_context(UpdateTaskContextInput {
                        task_id,
                        context_pack_id,
                    })
                })
                .and_then(|task| {
                    let project_id = task.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, task)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("GET", "/v1/initiatives") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_initiatives(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/initiatives") => match parse_json_body::<CreateInitiativeInput>(request)
            .and_then(|payload| store.create_initiative(payload))
            .and_then(|initiative| {
                let project_id = initiative.project_id.clone();
                snapshot_object_response(store, pty, version, &project_id, initiative)
            }) {
            Ok(response) => json_response(201, "Created", &response),
            Err(error) => bad_request_response(error),
        },
        ("GET", "/v1/tracker-bindings") => {
            let project_id =
                query_param(query, "projectId").unwrap_or_else(|| "proj_local".to_string());
            match store
                .list_external_tracker_bindings(&project_id)
                .and_then(|items| snapshot_list_response(store, pty, version, &project_id, items))
            {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => server_error_response(error),
            }
        }
        ("POST", "/v1/tracker-bindings") => {
            match parse_json_body::<crate::state_store::UpsertExternalTrackerBindingInput>(request)
                .and_then(|payload| store.upsert_external_tracker_binding(payload))
                .and_then(|binding| {
                    let project_id = binding.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, binding)
                }) {
                Ok(response) => json_response(201, "Created", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/tracker-sync/linear/run") => {
            match parse_json_body::<crate::state_store::RunTrackerSyncInput>(request)
                .and_then(|payload| store.run_tracker_sync("linear", payload))
                .and_then(|sync_run| {
                    let project_id = sync_run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, sync_run)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/tracker-sync/github/run") => {
            match parse_json_body::<crate::state_store::RunTrackerSyncInput>(request)
                .and_then(|payload| store.run_tracker_sync("github", payload))
                .and_then(|sync_run| {
                    let project_id = sync_run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, sync_run)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/tracker-sync/plane/run") => {
            match parse_json_body::<crate::state_store::RunTrackerSyncInput>(request)
                .and_then(|payload| store.run_tracker_sync("plane", payload))
                .and_then(|sync_run| {
                    let project_id = sync_run.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, sync_run)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        ("POST", "/v1/browser-automation/run") => {
            match parse_json_body::<crate::state_store::RunBrowserAutomationInput>(request)
                .and_then(|payload| store.plan_browser_automation(payload))
                .and_then(|plan| {
                    let project_id = plan.project_id.clone();
                    snapshot_object_response(store, pty, version, &project_id, plan)
                }) {
                Ok(response) => json_response(200, "OK", &response),
                Err(error) => bad_request_response(error),
            }
        }
        _ => json_response(
            404,
            "Not Found",
            &serde_json::json!({
                "error": "not_found",
                "path": path
            }),
        ),
    }
}

fn bad_request_response(error: String) -> HttpResponse {
    json_response(400, "Bad Request", &serde_json::json!({ "error": error }))
}

fn server_error_response(error: String) -> HttpResponse {
    json_response(
        500,
        "Internal Server Error",
        &serde_json::json!({ "error": error }),
    )
}

fn reload_workflow_from_request(
    store: &StateStore,
    payload: ReloadWorkflowRequest,
) -> Result<crate::state_store::PersistedWorkflowVersion, String> {
    let content = match payload.content {
        Some(content) => content,
        None => fs::read_to_string(&payload.source_path)
            .map_err(|error| format!("failed to read workflow {}: {error}", payload.source_path))?,
    };
    store.reload_workflow(crate::state_store::ReloadWorkflowInput {
        project_id: payload.project_id,
        source_path: payload.source_path,
        content,
    })
}

fn validate_workflow_from_request(
    store: &StateStore,
    payload: ReloadWorkflowRequest,
) -> Result<crate::state_store::WorkflowValidationResult, String> {
    let content = match payload.content {
        Some(content) => content,
        None => fs::read_to_string(&payload.source_path)
            .map_err(|error| format!("failed to read workflow {}: {error}", payload.source_path))?,
    };
    store.validate_workflow(crate::state_store::ValidateWorkflowInput {
        project_id: payload.project_id,
        source_path: payload.source_path,
        content,
    })
}

pub fn build_state_snapshot_from_store(
    store: &StateStore,
    pty: TerminalPtySnapshot,
    version: &str,
) -> Result<StateSnapshot, String> {
    build_state_snapshot_for_project_from_store(store, pty, version, "proj_local")
}

pub fn build_state_snapshot_for_project_from_store(
    store: &StateStore,
    pty: TerminalPtySnapshot,
    version: &str,
    project_id: &str,
) -> Result<StateSnapshot, String> {
    let db_health = store.health();
    let recent_command_blocks = store
        .recent_command_blocks(10)?
        .into_iter()
        .map(|block| state_snapshot::StateCommandBlockSummary {
            id: block.id,
            session_id: block.session_id,
            command: block.command,
            status: command_block_status(block.exit_code),
        })
        .collect::<Vec<_>>();
    let persisted_sessions = store
        .list_sessions(project_id)?
        .into_iter()
        .map(|session| {
            let mut state_session = persisted_session_to_state_session(session);
            let usage = store.token_usage_summary_for_session(&state_session.id)?;
            if usage.total_tokens > 0 || usage.cost_usd > 0.0 {
                state_session.token_usage = Some(state_snapshot::StateSessionTokenUsage {
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                    total_tokens: usage.total_tokens,
                    cost_usd: usage.cost_usd,
                });
            }
            Ok(state_session)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let persisted_projects = store.list_projects()?;
    let persisted_project_tabs = store.list_project_tabs()?;
    let persisted_project_tab_groups = store.list_project_tab_groups()?;
    let persisted_initiatives = store.list_initiatives(project_id)?;
    let tasks = store
        .list_tasks(project_id)?
        .into_iter()
        .map(|task| {
            let usage = store.token_usage_summary_for_task(&task.id)?;
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
            Ok(state_snapshot::StateTaskSummary {
                id: task.id,
                title: task.title,
                status: task.status,
                priority: task.priority,
                project_id: task.project_id,
                assignee_type: task.assignee_type,
                assignee_id: task.assignee_id,
                cycle_id: task.cycle_id,
                module_id: task.module_id,
                initiative_id: task.initiative_id,
                due_at: task.due_at,
                estimate: task.estimate,
                labels: task.labels,
                context_pack_id: task.context_pack_id,
                comment_count: task.comment_count,
                has_workpad: task
                    .workpad_md
                    .as_deref()
                    .map(|workpad| !workpad.trim().is_empty())
                    .unwrap_or(false),
                subtask_count: task.subtask_count,
                open_subtask_count: task.open_subtask_count,
                token_usage,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let task_counts_by_status = store.count_tasks_by_status(project_id)?;
    let runs = store
        .list_runs(project_id)?
        .into_iter()
        .map(|run| state_snapshot::StateRunSummary {
            id: run.id,
            task_id: run.task_id,
            project_id: run.project_id,
            agent_profile_id: run.agent_profile_id,
            workflow_version_id: run.workflow_version_id,
            lifecycle: run.lifecycle,
            retry_count: run.retry_count,
            next_retry_at: run.next_retry_at,
            status_detail: run.status_detail,
            context_pack_id: run.context_pack_id,
            workspace_path: run.workspace_path,
        })
        .collect::<Vec<_>>();
    let run_counts_by_lifecycle = store.count_runs_by_lifecycle(project_id)?;
    let workflow_state = store.workflow_runtime_state(project_id)?;
    let budget_summary = store.budget_summary()?;
    let budget_forecast = store.budget_forecast()?;
    let provider_model_settings = store.provider_model_settings()?;
    let terminal_theme_settings = store.terminal_theme_settings(Some(project_id))?;
    let provider_price_summary = store.provider_price_table_summary()?;
    let secret_summary = store.secret_summary()?;
    let policy_pack_summary = store.policy_pack_summary(project_id)?;
    let permission_audit_summary = store.permission_audit_summary(project_id)?;
    let knowledge_summary = store.knowledge_summary(project_id)?;
    let latest_workflow_negative_run = store.latest_workflow_negative_test_run(project_id)?;
    let latest_task_lifecycle_e2e_run = store.latest_task_lifecycle_e2e_run(project_id)?;
    let latest_terminal_smoke_run = store.latest_terminal_fidelity_smoke_run(project_id)?;
    let latest_release_gate_run = store.latest_release_gate_run(project_id)?;
    let latest_dmg_smoke_run = store.latest_dmg_smoke_run(project_id)?;
    let latest_recovery_drill_run = store.latest_recovery_drill_run(project_id)?;
    let latest_benchmark_run = store.latest_benchmark_run(project_id)?;
    let latest_dogfood_review = store.latest_dogfood_telemetry_review(project_id)?;
    let visual_links = store.list_visual_harness_links(project_id)?;
    let current_workflow = match workflow_state.current_version_id.as_deref() {
        Some(workflow_id) => store.get_workflow_version(workflow_id)?,
        None => None,
    };
    let tracker_bindings = store.list_external_tracker_bindings(project_id)?;
    let latest_linear_sync = store.latest_external_tracker_sync_run(project_id, "linear")?;
    let latest_github_sync = store.latest_external_tracker_sync_run(project_id, "github")?;
    let latest_plane_sync = store.latest_external_tracker_sync_run(project_id, "plane")?;
    let agents = store
        .list_agent_profiles()?
        .into_iter()
        .map(|agent| {
            let usage = store.token_usage_summary_for_agent(&agent.id)?;
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
            Ok(state_snapshot::StateAgent {
                id: agent.id,
                label: agent.name,
                available: agent.status == "available",
                token_usage,
                latest_event_kind: None,
                latest_event_detail: None,
                attention_state: None,
                attention_severity: None,
                notification_count: None,
                last_heartbeat_at: agent.last_heartbeat_at,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let recent_agent_events = store.list_agent_events(project_id, 20)?;
    let reviews = store
        .list_evidence_packs_for_project(project_id)?
        .into_iter()
        .map(|pack| {
            let decision = pack
                .body_json
                .get("review_decision")
                .and_then(|review| review.get("decision"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
            state_snapshot::StateReview {
                id: format!("review_{}", pack.id),
                state: decision.unwrap_or_else(|| {
                    if pack.completeness_state == "complete" {
                        "pending".to_string()
                    } else {
                        "incomplete".to_string()
                    }
                }),
                evidence_pack_id: pack.id,
                task_id: pack.task_id,
                run_id: pack.run_id,
                completeness_state: pack.completeness_state,
                diff_summary: pack
                    .body_json
                    .get("diff_summary")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
                token_usage: pack
                    .body_json
                    .get("token_usage")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
            }
        })
        .collect::<Vec<_>>();

    let mut snapshot = state_snapshot::build_state_snapshot_with_db_state(
        version,
        &state_snapshot::current_timestamp_utc(),
        crate::readiness::collect_readiness_snapshot(),
        pty,
        &db_health.status,
        tasks,
        task_counts_by_status,
    );
    if !persisted_sessions.is_empty() {
        let mut sessions = persisted_sessions;
        for session in snapshot.sessions {
            if !sessions.iter().any(|persisted| persisted.id == session.id) {
                sessions.push(session);
            }
        }
        snapshot.sessions = sessions;
    }
    if !persisted_projects.is_empty() {
        snapshot.projects = persisted_projects
            .iter()
            .map(|project| state_snapshot::StateProject {
                id: project.id.clone(),
                name: project.name.clone(),
                state: project.status.clone(),
                token_usage: None,
            })
            .collect();
        snapshot.project_tabs = persisted_project_tabs
            .iter()
            .map(|tab| {
                let label = persisted_projects
                    .iter()
                    .find(|project| project.id == tab.project_id)
                    .map(|project| project.name.clone())
                    .unwrap_or_else(|| tab.project_id.clone());
                state_snapshot::StateProjectTab {
                    id: tab.id.clone(),
                    project_id: Some(tab.project_id.clone()),
                    label,
                    active: tab.active,
                    layout_json: Some(tab.layout_json.clone()),
                    group_name: persisted_project_tab_groups
                        .iter()
                        .find(|assignment| assignment.project_id == tab.project_id)
                        .map(|assignment| assignment.group_name.clone()),
                }
            })
            .collect();
    }
    snapshot.projects = snapshot
        .projects
        .into_iter()
        .map(|project| {
            let usage = store.token_usage_summary_for_project(&project.id)?;
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
            Ok(state_snapshot::StateProject {
                token_usage,
                ..project
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    snapshot.command_blocks.unread_count = recent_command_blocks.len();
    snapshot.command_blocks.recent = recent_command_blocks;
    snapshot.initiatives = persisted_initiatives
        .into_iter()
        .map(|initiative| {
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
        })
        .collect::<Result<Vec<_>, String>>()?;
    snapshot.runs.items = runs;
    snapshot.runs.counts_by_lifecycle = run_counts_by_lifecycle;
    snapshot.provider_model = state_snapshot::StateProviderModel {
        provider: provider_model_settings.provider,
        model: provider_model_settings.model,
        agent_profile_id: provider_model_settings.agent_profile_id,
    };
    snapshot.app.terminal_theme = state_snapshot::StateTerminalTheme {
        project_id: terminal_theme_settings.project_id,
        name: terminal_theme_settings.name,
        background: terminal_theme_settings.background,
        foreground: terminal_theme_settings.foreground,
        accent: terminal_theme_settings.accent,
    };
    snapshot.agents = agents;
    if !recent_agent_events.is_empty() {
        snapshot.agents = snapshot
            .agents
            .into_iter()
            .map(|mut agent| {
                if let Some(event) = recent_agent_events
                    .iter()
                    .find(|event| event.agent_profile_id == agent.id)
                {
                    agent.latest_event_kind = Some(event.kind.clone());
                    agent.latest_event_detail = Some(event.detail.clone());
                }
                let notification_events = recent_agent_events
                    .iter()
                    .filter(|event| event.agent_profile_id == agent.id && event.severity != "info")
                    .collect::<Vec<_>>();
                if let Some(event) = notification_events.first() {
                    agent.attention_state = Some(agent_attention_state(event).to_string());
                    agent.attention_severity = Some(event.severity.clone());
                    agent.notification_count = Some(notification_events.len() as i64);
                }
                agent
            })
            .collect();
        snapshot.attention.extend(
            recent_agent_events
                .iter()
                .filter(|event| event.severity != "info")
                .map(|event| state_snapshot::StateAttentionItem {
                    id: format!("agent_event_{}", event.agent_profile_id),
                    label: format!("Agent {} {}", event.agent_profile_id, event.kind),
                    severity: event.severity.clone(),
                    detail: event.detail.clone(),
                }),
        );
    }
    snapshot.reviews = reviews;
    snapshot.visual_harness = build_visual_harness_state(
        &snapshot,
        visual_links,
        current_workflow.as_ref(),
        &policy_pack_summary,
    );
    snapshot.budgets.workspace = budget_summary
        .get("workspace")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    snapshot.budgets.projects = budget_summary
        .get("projects")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    snapshot.budgets.goals = budget_summary
        .get("goals")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    snapshot.budgets.tasks = budget_summary
        .get("tasks")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    snapshot.budgets.runs = budget_summary
        .get("runs")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    snapshot.budgets.agents = budget_summary
        .get("agents")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    snapshot.budgets.forecasts = budget_forecast;
    snapshot.budgets.price_table = provider_price_summary;
    snapshot.security.keychain = secret_summary
        .get("keychain")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    snapshot.security.secret_count = secret_summary
        .get("secret_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as usize;
    snapshot.security.redaction = secret_summary.get("redaction").cloned().unwrap_or_else(
        || serde_json::json!({ "status": "inactive", "protected_secret_count": 0 }),
    );
    snapshot.security.policy_pack = policy_pack_summary;
    snapshot.security.permission_audit = permission_audit_summary;
    let pending_policy_approvals = store.list_policy_approvals(project_id, Some("pending"))?;
    snapshot.security.diagnostics = security_diagnostics_json(
        &snapshot.security.keychain,
        snapshot.security.secret_count,
        &snapshot.security.redaction,
        &snapshot.security.policy_pack,
        &snapshot.security.permission_audit,
        pending_policy_approvals.len(),
        &snapshot.health,
    );
    snapshot.workflow.valid = workflow_state.valid;
    snapshot.workflow.current_version_id = workflow_state.current_version_id;
    snapshot.workflow.last_known_good_version_id = workflow_state.last_known_good_version_id;
    snapshot.workflow.diagnostics = workflow_state.diagnostics;
    snapshot.workflow.invalid_projects = if snapshot.workflow.valid {
        vec![]
    } else {
        vec![project_id.to_string()]
    };
    if let Some(run) = latest_workflow_negative_run {
        snapshot.workflow_negative.last_run_id = Some(run.id);
        snapshot.workflow_negative.last_status = run.status.clone();
        snapshot.workflow_negative.last_baseline_workflow_id = Some(run.baseline_workflow_id);
        snapshot.workflow_negative.last_invalid_workflow_id = Some(run.invalid_workflow_id);
        snapshot.workflow_negative.last_known_good_workflow_id =
            Some(run.last_known_good_workflow_id);
        snapshot.workflow_negative.diagnostics = serde_json::json!({
            "status": run.status,
            "case_count": run.cases.len(),
            "dispatch_run_id": run.dispatch_run_id,
            "created_at": run.created_at
        });
    }
    snapshot.knowledge.stale_count = knowledge_summary.stale_count as usize;
    snapshot.knowledge.gap_count = knowledge_summary.gap_count as usize;
    snapshot.knowledge.recent_pages = knowledge_summary.recent_pages;
    if let Some(run) = latest_task_lifecycle_e2e_run {
        snapshot.task_lifecycle.last_run_id = Some(run.id);
        snapshot.task_lifecycle.last_status = run.status.clone();
        snapshot.task_lifecycle.last_task_id = Some(run.task_id);
        snapshot.task_lifecycle.last_agent_run_id = Some(run.run_id);
        snapshot.task_lifecycle.last_evidence_pack_id = Some(run.evidence_pack_id);
        snapshot.task_lifecycle.diagnostics = serde_json::json!({
            "status": run.status,
            "transition_count": run.transitions.len(),
            "created_at": run.created_at
        });
    }
    if let Some(run) = latest_terminal_smoke_run {
        snapshot.terminal_fidelity.last_run_id = Some(run.id);
        snapshot.terminal_fidelity.last_status = run.status.clone();
        snapshot.terminal_fidelity.last_pass_count = run.pass_count;
        snapshot.terminal_fidelity.last_fail_count = run.fail_count;
        snapshot.terminal_fidelity.last_warning_count = run.warning_count;
        snapshot.terminal_fidelity.diagnostics = serde_json::json!({
            "status": run.status,
            "case_count": run.case_count,
            "created_at": run.created_at
        });
    }
    if let Some(run) = latest_release_gate_run {
        snapshot.release_gates.last_run_id = Some(run.id);
        snapshot.release_gates.last_status = run.status.clone();
        snapshot.release_gates.last_pass_count = run.pass_count;
        snapshot.release_gates.last_fail_count = run.fail_count;
        snapshot.release_gates.last_warning_count = run.warning_count;
        snapshot.release_gates.diagnostics = serde_json::json!({
            "status": run.status,
            "scenario_count": run.scenario_count,
            "created_at": run.created_at
        });
    }
    if let Some(run) = latest_dmg_smoke_run {
        snapshot.distribution.last_dmg_smoke_run_id = Some(run.id);
        snapshot.distribution.last_status = run.status.clone();
        snapshot.distribution.explicit_blocker = run.explicit_blocker;
        snapshot.distribution.last_pass_count = run.pass_count;
        snapshot.distribution.last_fail_count = run.fail_count;
        snapshot.distribution.last_warning_count = run.warning_count;
        snapshot.distribution.diagnostics = serde_json::json!({
            "status": run.status,
            "case_count": run.case_count,
            "created_at": run.created_at
        });
    }
    if let Some(run) = latest_recovery_drill_run {
        snapshot.recovery.last_run_id = Some(run.id);
        snapshot.recovery.last_status = run.status.clone();
        snapshot.recovery.last_pass_count = run.pass_count;
        snapshot.recovery.last_fail_count = run.fail_count;
        snapshot.recovery.last_warning_count = run.warning_count;
        snapshot.recovery.diagnostics = serde_json::json!({
            "status": run.status,
            "drill_count": run.drill_count,
            "created_at": run.created_at
        });
    }
    if let Some(run) = latest_benchmark_run {
        snapshot.benchmarks.last_run_id = Some(run.id);
        snapshot.benchmarks.last_status = run.status.clone();
        snapshot.benchmarks.last_pass_count = run.pass_count;
        snapshot.benchmarks.last_fail_count = run.fail_count;
        snapshot.benchmarks.last_warning_count = run.warning_count;
        snapshot.benchmarks.suites = run
            .suites
            .into_iter()
            .map(|suite| state_snapshot::StateBenchmarkSuite {
                suite_id: suite.suite_id,
                name: suite.name,
                status: suite.status,
                metric_value: suite.metric_value,
                target_value: suite.target_value,
                unit: suite.unit,
                detail: suite.detail,
            })
            .collect();
        snapshot.benchmarks.diagnostics = serde_json::json!({
            "status": run.status,
            "suite_count": run.suite_count,
            "duration_ms": run.duration_ms,
            "created_at": run.created_at
        });
    }
    if let Some(review) = latest_dogfood_review {
        snapshot.dogfood.last_review_id = Some(review.id);
        snapshot.dogfood.last_status = review.status.clone();
        snapshot.dogfood.last_evidence_pack_id = Some(review.evidence_pack_id);
        snapshot.dogfood.last_pass_count = review.pass_count;
        snapshot.dogfood.last_warning_count = review.warning_count;
        snapshot.dogfood.last_fail_count = review.fail_count;
        snapshot.dogfood.diagnostics = serde_json::json!({
            "status": review.status,
            "finding_count": review.finding_count,
            "created_at": review.created_at
        });
    }
    let tracker_pending_count = tracker_bindings
        .iter()
        .filter(|binding| binding.sync_status == "pending")
        .count();
    let tracker_conflict_count = tracker_bindings
        .iter()
        .filter(|binding| binding.conflict_state != "none")
        .count();
    snapshot.tracker.binding_count = tracker_bindings.len();
    snapshot.tracker.bindings = tracker_bindings
        .into_iter()
        .map(|binding| state_snapshot::StateTrackerBinding {
            id: binding.id,
            local_kind: binding.local_kind,
            local_id: binding.local_id,
            provider: binding.provider,
            external_id: binding.external_id,
            external_url: binding.external_url,
            sync_mode: binding.sync_mode,
            sync_status: binding.sync_status,
            conflict_state: binding.conflict_state,
        })
        .collect();
    snapshot.tracker.diagnostics = serde_json::json!({
        "status": if tracker_conflict_count > 0 {
            "conflict"
        } else if tracker_pending_count > 0 {
            "pending"
        } else if snapshot.tracker.binding_count > 0 {
            "ok"
        } else {
            "unconfigured"
        },
        "pending_count": tracker_pending_count,
        "conflict_count": tracker_conflict_count,
        "linear": tracker_sync_diagnostics_json(latest_linear_sync),
        "github": tracker_sync_diagnostics_json(latest_github_sync),
        "plane": tracker_sync_diagnostics_json(latest_plane_sync)
    });
    let mut budget_attention = budget_attention_from_summary(&budget_summary);
    let mut policy_attention = pending_policy_approvals
        .into_iter()
        .map(|approval| state_snapshot::StateAttentionItem {
            id: approval.id.clone(),
            label: format!("Policy approval required: {}", approval.action_kind),
            severity: if matches!(approval.risk_level.as_str(), "high" | "critical") {
                "critical".to_string()
            } else {
                "warning".to_string()
            },
            detail: approval
                .command
                .unwrap_or_else(|| format!("{} action pending", approval.risk_level)),
        })
        .collect::<Vec<_>>();
    budget_attention.append(&mut policy_attention);
    budget_attention.extend(snapshot.attention);
    snapshot.attention = budget_attention;
    state_snapshot::assign_snapshot_id(&mut snapshot);
    Ok(snapshot)
}

fn agent_attention_state(event: &crate::state_store::PersistedAgentEvent) -> &'static str {
    if event.severity == "critical" || event.severity == "error" {
        "error"
    } else if event.kind == "status" {
        "needs_input"
    } else {
        "unread"
    }
}

fn budget_attention_from_summary(
    budget_summary: &serde_json::Value,
) -> Vec<state_snapshot::StateAttentionItem> {
    let mut attention = Vec::new();
    if let Some(workspace) = budget_summary.get("workspace") {
        if let Some(item) = budget_attention_item("workspace", None, workspace) {
            attention.push(item);
        }
    }
    if let Some(projects) = budget_summary
        .get("projects")
        .and_then(serde_json::Value::as_array)
    {
        for budget in projects {
            if let Some(item) = budget_attention_item(
                "project",
                budget.get("scope_id").and_then(serde_json::Value::as_str),
                budget,
            ) {
                attention.push(item);
            }
        }
    }
    if let Some(goals) = budget_summary
        .get("goals")
        .and_then(serde_json::Value::as_array)
    {
        for budget in goals {
            if let Some(item) = budget_attention_item(
                "goal",
                budget.get("scope_id").and_then(serde_json::Value::as_str),
                budget,
            ) {
                attention.push(item);
            }
        }
    }
    if let Some(tasks) = budget_summary
        .get("tasks")
        .and_then(serde_json::Value::as_array)
    {
        for budget in tasks {
            if let Some(item) = budget_attention_item(
                "task",
                budget.get("scope_id").and_then(serde_json::Value::as_str),
                budget,
            ) {
                attention.push(item);
            }
        }
    }
    if let Some(runs) = budget_summary
        .get("runs")
        .and_then(serde_json::Value::as_array)
    {
        for budget in runs {
            if let Some(item) = budget_attention_item(
                "run",
                budget.get("scope_id").and_then(serde_json::Value::as_str),
                budget,
            ) {
                attention.push(item);
            }
        }
    }
    if let Some(agents) = budget_summary
        .get("agents")
        .and_then(serde_json::Value::as_array)
    {
        for budget in agents {
            if let Some(item) = budget_attention_item(
                "agent",
                budget.get("scope_id").and_then(serde_json::Value::as_str),
                budget,
            ) {
                attention.push(item);
            }
        }
    }
    attention
}

fn security_diagnostics_json(
    keychain: &str,
    secret_count: usize,
    redaction: &serde_json::Value,
    policy_pack: &serde_json::Value,
    permission_audit: &serde_json::Value,
    pending_policy_approvals: usize,
    health: &state_snapshot::StateHealth,
) -> serde_json::Value {
    let protected_count = redaction
        .get("protected_secret_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(secret_count as u64);
    let redaction_status = redaction
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(if secret_count == 0 {
            "inactive"
        } else {
            "active"
        });
    let policy_name = policy_pack
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Default ask-before-write");
    let sandbox_mode = policy_pack
        .get("sandbox_mode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("ask-before-write");
    let network_profile = policy_pack
        .get("network_profile")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("internet");
    let keychain_check_status = if matches!(keychain, "macos" | "local") {
        "ok"
    } else {
        "warning"
    };
    let redaction_check_status = if secret_count == 0
        || (redaction_status == "active" && protected_count >= secret_count as u64)
    {
        "ok"
    } else {
        "warning"
    };
    let policy_approval_status = if pending_policy_approvals == 0 {
        "ok"
    } else {
        "warning"
    };
    let forbidden_audit_count = permission_audit
        .get("forbidden_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let audit_status = if forbidden_audit_count == 0 {
        "ok"
    } else {
        "warning"
    };
    let control_plane_status = if health.api == "ok" && health.db == "ok" && health.pty == "ok" {
        "ok"
    } else {
        "warning"
    };
    let checks = vec![
        serde_json::json!({
            "id": "keychain",
            "label": "Keychain",
            "status": keychain_check_status,
            "detail": format!("{keychain} secret storage available")
        }),
        serde_json::json!({
            "id": "redaction",
            "label": "Secret redaction",
            "status": redaction_check_status,
            "detail": format!("{protected_count} protected {}", if protected_count == 1 { "value" } else { "values" })
        }),
        serde_json::json!({
            "id": "policy-pack",
            "label": "Policy pack",
            "status": "ok",
            "detail": format!("{policy_name} · {sandbox_mode} · {network_profile} network")
        }),
        serde_json::json!({
            "id": "policy-approvals",
            "label": "Policy approvals",
            "status": policy_approval_status,
            "detail": format!("{pending_policy_approvals} pending {}", if pending_policy_approvals == 1 { "approval" } else { "approvals" })
        }),
        serde_json::json!({
            "id": "permission-audit",
            "label": "Permission audit",
            "status": audit_status,
            "detail": if forbidden_audit_count == 0 {
                "No forbidden decisions in recent audit".to_string()
            } else {
                format!("{forbidden_audit_count} forbidden {} in recent audit", if forbidden_audit_count == 1 { "decision" } else { "decisions" })
            }
        }),
        serde_json::json!({
            "id": "control-plane",
            "label": "Control plane",
            "status": control_plane_status,
            "detail": format!("api {} · db {} · pty {}", health.api, health.db, health.pty)
        }),
    ];
    let status = if checks.iter().any(|check| check["status"] == "warning") {
        "warning"
    } else {
        "ok"
    };
    serde_json::json!({
        "status": status,
        "pending_policy_approvals": pending_policy_approvals,
        "checks": checks
    })
}

fn tracker_sync_diagnostics_json(
    sync_run: Option<crate::state_store::PersistedExternalTrackerSyncRun>,
) -> serde_json::Value {
    sync_run
        .map(|sync_run| {
            serde_json::json!({
                "last_run_id": sync_run.id,
                "last_status": sync_run.status,
                "last_operation_count": sync_run.operation_count,
                "degraded_reason": sync_run.degraded_reason
            })
        })
        .unwrap_or_else(|| {
            serde_json::json!({
                "last_run_id": null,
                "last_status": "not_run",
                "last_operation_count": 0,
                "degraded_reason": null
            })
        })
}

fn budget_attention_item(
    scope_type: &str,
    scope_id: Option<&str>,
    budget: &serde_json::Value,
) -> Option<state_snapshot::StateAttentionItem> {
    let state = budget.get("state").and_then(serde_json::Value::as_str)?;
    if !matches!(state, "warn" | "exceeded") {
        return None;
    }

    let used_usd = budget
        .get("used_usd")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let max_usd = budget.get("max_usd").and_then(serde_json::Value::as_f64);
    let percent = max_usd
        .filter(|max| *max > 0.0)
        .map(|max| ((used_usd / max) * 100.0).round() as i64);
    let state_label = if state == "exceeded" {
        "exceeded"
    } else {
        "warning"
    };
    let label_scope = match scope_id {
        Some(id) => format!("{scope_type} {id}"),
        None => scope_type.to_string(),
    };
    let id_suffix = sanitize_attention_id(scope_id.unwrap_or(scope_type));

    Some(state_snapshot::StateAttentionItem {
        id: format!("budget_{scope_type}_{id_suffix}"),
        label: format!("Budget {state_label}: {label_scope}"),
        severity: if state == "exceeded" {
            "critical".to_string()
        } else {
            "warning".to_string()
        },
        detail: match (max_usd, percent) {
            (Some(max_usd), Some(percent)) => {
                format!(
                    "{} of {} used · {percent}%",
                    format_usd(used_usd),
                    format_usd(max_usd)
                )
            }
            _ => format!("{} used", format_usd(used_usd)),
        },
    })
}

fn sanitize_attention_id(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn format_usd(value: f64) -> String {
    format!("${value:.2}")
}

pub fn health_from_store(store: &StateStore) -> ControlApiHealth {
    ControlApiHealth {
        db: store.health().status,
        pty: "ok".to_string(),
        api: "ok".to_string(),
    }
}

fn update_check_response(
    channel: &str,
    current_version: &str,
) -> Result<UpdateCheckResponse, String> {
    let channel = channel.trim();
    if !matches!(channel, "stable" | "beta") {
        return Err("update channel must be stable or beta".to_string());
    }

    let feed_path = format!("/update-feed/{channel}.json");
    let feed_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "cannot resolve workspace root for update feed".to_string())?
        .join("public")
        .join("update-feed")
        .join(format!("{channel}.json"));
    let feed_raw = fs::read_to_string(&feed_file).map_err(|error| {
        format!(
            "failed to read update feed {}: {error}",
            feed_file.display()
        )
    })?;
    let feed: serde_json::Value = serde_json::from_str(&feed_raw).map_err(|error| {
        format!(
            "failed to parse update feed {}: {error}",
            feed_file.display()
        )
    })?;
    let latest_version = feed
        .get("version")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("update feed {feed_path} is missing version"))?
        .to_string();
    let pub_date = feed
        .get("pub_date")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let signature_state = update_feed_signature_state(&feed);

    Ok(UpdateCheckResponse {
        channel: channel.to_string(),
        current_version: current_version.to_string(),
        latest_version,
        pub_date,
        feed_path,
        signature_state,
    })
}

fn update_feed_signature_state(feed: &serde_json::Value) -> String {
    let signatures = feed
        .get("platforms")
        .and_then(serde_json::Value::as_object)
        .map(|platforms| {
            platforms
                .values()
                .filter_map(|platform| {
                    platform
                        .get("signature")
                        .and_then(serde_json::Value::as_str)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if signatures.is_empty()
        || signatures
            .iter()
            .any(|signature| signature.trim().is_empty())
    {
        "missing".to_string()
    } else if signatures
        .iter()
        .any(|signature| signature.contains("SIGNATURE_REQUIRED_FOR_RELEASE"))
    {
        "blocked".to_string()
    } else {
        "signed".to_string()
    }
}

fn persisted_session_to_state_session(
    session: crate::state_store::PersistedSession,
) -> state_snapshot::StateSession {
    state_snapshot::StateSession {
        id: session.id.clone(),
        project_id: session.project_id,
        pane_id: session.pane_id.unwrap_or_else(|| session.id.clone()),
        mode: session.mode,
        title: session.title,
        cwd: session.cwd.unwrap_or_default(),
        branch: session.branch.unwrap_or_default(),
        agent_profile_id: session.agent_profile_id,
        task_id: session.task_id,
        run_id: session.run_id,
        state: session.state,
        attention_state: session.attention_state,
        token_budget_state: session.token_budget_state,
        token_usage: None,
        ports: vec![],
        created_at: session.created_at,
        updated_at: session.updated_at,
    }
}

fn serve_stream(
    mut stream: UnixStream,
    store: &StateStore,
    manager: &Arc<Mutex<TerminalPtyManager>>,
    version: &str,
) -> Result<(), String> {
    let mut buffer = [0_u8; MAX_REQUEST_BYTES];
    let bytes = stream
        .read(&mut buffer)
        .map_err(|error| format!("failed to read control API request: {error}"))?;
    let request = String::from_utf8_lossy(&buffer[..bytes]);
    let pty = manager
        .lock()
        .map_err(|_| "terminal PTY manager lock was poisoned".to_string())?
        .snapshot();
    let response = handle_http_request(&request, store, pty, version);
    stream
        .write_all(response.to_http_bytes().as_bytes())
        .and_then(|_| stream.flush())
        .map_err(|error| format!("failed to write control API response: {error}"))
}

fn parse_request_line(request: &str) -> Option<(&str, &str, Option<&str>)> {
    let line = request.lines().next()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let path_with_query = parts.next()?;
    let _version = parts.next()?;
    let (path, query) = path_with_query
        .split_once('?')
        .map(|(path, query)| (path, Some(query)))
        .unwrap_or((path_with_query, None));
    Some((method, path, query))
}

fn query_param(query: Option<&str>, key: &str) -> Option<String> {
    query.and_then(|query| {
        query.split('&').find_map(|part| {
            let (candidate, value) = part.split_once('=')?;
            (candidate == key).then(|| value.replace('+', " "))
        })
    })
}

fn filter_task_list_items(
    items: Vec<PersistedTask>,
    status: Option<&str>,
    query: Option<&str>,
) -> Vec<PersistedTask> {
    let normalized_status = status.map(str::trim).filter(|value| !value.is_empty());
    let normalized_query = query
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty());

    items
        .into_iter()
        .filter(|task| {
            normalized_status
                .map(|status| task.status == status)
                .unwrap_or(true)
        })
        .filter(|task| {
            normalized_query
                .as_deref()
                .map(|query| task_matches_query(task, query))
                .unwrap_or(true)
        })
        .collect()
}

fn task_matches_query(task: &PersistedTask, query: &str) -> bool {
    let labels = task.labels.join("\n");
    [
        task.title.as_str(),
        task.status.as_str(),
        task.priority.as_str(),
        task.description.as_deref().unwrap_or_default(),
        task.assignee_id.as_deref().unwrap_or_default(),
        task.cycle_id.as_deref().unwrap_or_default(),
        task.module_id.as_deref().unwrap_or_default(),
        labels.as_str(),
    ]
    .join("\n")
    .to_lowercase()
    .contains(query)
}

fn filter_run_list_items(
    items: Vec<PersistedRun>,
    lifecycle: Option<&str>,
    task_id: Option<&str>,
    agent_profile_id: Option<&str>,
) -> Vec<PersistedRun> {
    let normalized_lifecycle = lifecycle.map(str::trim).filter(|value| !value.is_empty());
    let normalized_task_id = task_id.map(str::trim).filter(|value| !value.is_empty());
    let normalized_agent_profile_id = agent_profile_id
        .map(str::trim)
        .filter(|value| !value.is_empty());

    items
        .into_iter()
        .filter(|run| {
            normalized_lifecycle
                .map(|lifecycle| run.lifecycle == lifecycle)
                .unwrap_or(true)
        })
        .filter(|run| {
            normalized_task_id
                .map(|task_id| run.task_id == task_id)
                .unwrap_or(true)
        })
        .filter(|run| {
            normalized_agent_profile_id
                .map(|agent_profile_id| run.agent_profile_id.as_deref() == Some(agent_profile_id))
                .unwrap_or(true)
        })
        .collect()
}

fn filter_session_list_items(
    items: Vec<PersistedSession>,
    state: Option<&str>,
    task_id: Option<&str>,
) -> Vec<PersistedSession> {
    let normalized_state = state
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all");
    let normalized_task_id = task_id
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all");

    items
        .into_iter()
        .filter(|session| {
            normalized_state
                .map(|state| session.state == state)
                .unwrap_or(true)
        })
        .filter(|session| {
            normalized_task_id
                .map(|task_id| session.task_id.as_deref() == Some(task_id))
                .unwrap_or(true)
        })
        .collect()
}

fn parse_optional_bool_filter(value: Option<&str>, label: &str) -> Result<Option<bool>, String> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) if value.eq_ignore_ascii_case("all") => Ok(None),
        Some(value) if value.eq_ignore_ascii_case("true") => Ok(Some(true)),
        Some(value) if value.eq_ignore_ascii_case("false") => Ok(Some(false)),
        Some(value) => Err(format!("{label} must be true, false, or all, got {value}")),
        None => Ok(None),
    }
}

fn filter_review_list_items(
    items: Vec<state_snapshot::StateReview>,
    state: Option<&str>,
    completeness: Option<&str>,
) -> Vec<state_snapshot::StateReview> {
    let normalized_state = state
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all");
    let normalized_completeness = completeness
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all");

    items
        .into_iter()
        .filter(|review| {
            normalized_state
                .map(|state| review.state == state)
                .unwrap_or(true)
        })
        .filter(|review| {
            normalized_completeness
                .map(|completeness| review.completeness_state == completeness)
                .unwrap_or(true)
        })
        .collect()
}

fn parse_json_body<T: for<'de> Deserialize<'de>>(request: &str) -> Result<T, String> {
    let body = request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .ok_or_else(|| "missing json request body".to_string())?;
    serde_json::from_str(body).map_err(|error| format!("invalid json request body: {error}"))
}

fn task_id_from_action_path(path: &str, action: &str) -> Result<String, String> {
    let prefix = "/v1/tasks/";
    let suffix = format!("/{action}");
    path.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(&suffix))
        .filter(|task_id| !task_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("invalid task {action} path"))
}

fn task_id_from_resource_path(path: &str, prefix: &str) -> Result<String, String> {
    path.strip_prefix(prefix)
        .filter(|task_id| !task_id.is_empty() && !task_id.contains('/'))
        .map(str::to_string)
        .ok_or_else(|| "invalid task resource path".to_string())
}

fn task_subtask_status_path(path: &str) -> Result<(String, String), String> {
    let rest = path
        .strip_prefix("/v1/tasks/")
        .and_then(|rest| rest.strip_suffix("/status"))
        .ok_or_else(|| "invalid task subtask status path".to_string())?;
    let mut parts = rest.split('/');
    let task_id = parts.next().unwrap_or_default();
    let subtasks = parts.next().unwrap_or_default();
    let subtask_id = parts.next().unwrap_or_default();
    if task_id.is_empty()
        || subtasks != "subtasks"
        || subtask_id.is_empty()
        || parts.next().is_some()
    {
        return Err("invalid task subtask status path".to_string());
    }
    Ok((task_id.to_string(), subtask_id.to_string()))
}

fn is_task_resource_path(path: &str) -> bool {
    path.strip_prefix("/v1/tasks/")
        .is_some_and(|task_id| !task_id.is_empty() && !task_id.contains('/'))
}

fn session_id_from_action_path(path: &str, action: &str) -> Result<String, String> {
    let prefix = "/v1/sessions/";
    let suffix = format!("/{action}");
    path.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(&suffix))
        .filter(|session_id| !session_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("invalid session {action} path"))
}

fn project_id_from_action_path(path: &str, action: &str) -> Result<String, String> {
    let prefix = "/v1/projects/";
    let suffix = format!("/{action}");
    path.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(&suffix))
        .filter(|project_id| !project_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("invalid project {action} path"))
}

fn project_id_from_files_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/projects/")
        .and_then(|rest| rest.strip_suffix("/files"))
        .filter(|project_id| !project_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "invalid project files path".to_string())
}

fn project_id_from_files_search_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/projects/")
        .and_then(|rest| rest.strip_suffix("/files/search"))
        .filter(|project_id| !project_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "invalid project files search path".to_string())
}

fn project_id_from_file_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/projects/")
        .and_then(|rest| rest.strip_suffix("/file"))
        .filter(|project_id| !project_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "invalid project file path".to_string())
}

fn project_id_from_diff_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/projects/")
        .and_then(|rest| rest.strip_suffix("/diff"))
        .filter(|project_id| !project_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "invalid project diff path".to_string())
}

fn project_id_from_nested_action_path(path: &str, action: &str) -> Result<String, String> {
    let prefix = "/v1/projects/";
    let suffix = format!("/{action}");
    path.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(&suffix))
        .filter(|project_id| !project_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("invalid project {action} path"))
}

fn id_from_prefix(path: &str, prefix: &str) -> Result<String, String> {
    path.strip_prefix(prefix)
        .filter(|id| !id.is_empty() && !id.contains('/'))
        .map(str::to_string)
        .ok_or_else(|| format!("invalid {prefix} path"))
}

fn run_id_from_action_path(path: &str, action: &str) -> Result<String, String> {
    let prefix = "/v1/runs/";
    let suffix = format!("/{action}");
    path.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(&suffix))
        .filter(|run_id| !run_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("invalid run {action} path"))
}

fn run_id_from_resource_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/runs/")
        .filter(|run_id| !run_id.is_empty() && !run_id.contains('/'))
        .map(str::to_string)
        .ok_or_else(|| "invalid run resource path".to_string())
}

fn is_run_resource_path(path: &str) -> bool {
    path.strip_prefix("/v1/runs/")
        .is_some_and(|run_id| !run_id.is_empty() && !run_id.contains('/'))
}

fn agent_id_from_action_path(path: &str, action: &str) -> Result<String, String> {
    let prefix = "/v1/agents/";
    let suffix = format!("/{action}");
    path.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(&suffix))
        .filter(|agent_id| !agent_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("invalid agent {action} path"))
}

fn run_id_from_nested_action_path(path: &str, action: &str) -> Result<String, String> {
    let prefix = "/v1/runs/";
    let suffix = format!("/{action}");
    path.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(&suffix))
        .filter(|run_id| !run_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("invalid run {action} path"))
}

fn evidence_id_from_action_path(path: &str, action: &str) -> Result<String, String> {
    let prefix = "/v1/evidence/";
    let suffix = format!("/{action}");
    path.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(&suffix))
        .filter(|evidence_id| !evidence_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("invalid evidence {action} path"))
}

fn review_id_from_decision_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/reviews/")
        .and_then(|rest| rest.strip_suffix("/decision"))
        .and_then(|review_id| review_id.strip_prefix("review_"))
        .filter(|evidence_id| !evidence_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "invalid review decision path".to_string())
}

fn review_id_from_follow_up_task_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/reviews/")
        .and_then(|rest| rest.strip_suffix("/follow-up-task"))
        .filter(|review_id| !review_id.is_empty() && review_id.starts_with("review_"))
        .map(str::to_string)
        .ok_or_else(|| "invalid review follow-up task path".to_string())
}

fn review_id_from_pr_landing_plan_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/reviews/")
        .and_then(|rest| rest.strip_suffix("/pr/landing-plan"))
        .filter(|review_id| !review_id.is_empty() && review_id.starts_with("review_"))
        .map(str::to_string)
        .ok_or_else(|| "invalid review PR landing plan path".to_string())
}

fn policy_approval_id_from_decision_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/policy/approvals/")
        .and_then(|rest| rest.strip_suffix("/decision"))
        .filter(|approval_id| !approval_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "invalid policy approval decision path".to_string())
}

fn run_hook_path_parts(path: &str) -> Result<(String, String), String> {
    let rest = path
        .strip_prefix("/v1/runs/")
        .ok_or_else(|| "invalid run hook path".to_string())?;
    let (run_id, rest) = rest
        .split_once("/hooks/")
        .ok_or_else(|| "invalid run hook path".to_string())?;
    let hook_name = rest
        .strip_suffix("/run")
        .ok_or_else(|| "invalid run hook path".to_string())?;
    if run_id.is_empty() || hook_name.is_empty() {
        Err("invalid run hook path".to_string())
    } else {
        Ok((run_id.to_string(), hook_name.to_string()))
    }
}

fn command_block_id_from_attach_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/command-blocks/")
        .and_then(|rest| rest.strip_suffix("/attach-evidence"))
        .filter(|command_block_id| !command_block_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "invalid command block attach evidence path".to_string())
}

fn command_block_id_from_command_block_action_path(
    path: &str,
    action: &str,
) -> Result<String, String> {
    path.strip_prefix("/v1/command-blocks/")
        .and_then(|rest| rest.strip_suffix(&format!("/{action}")))
        .filter(|command_block_id| !command_block_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("invalid command block {action} path"))
}

fn command_block_id_from_resource_path(path: &str) -> Result<String, String> {
    path.strip_prefix("/v1/command-blocks/")
        .filter(|command_block_id| !command_block_id.is_empty() && !command_block_id.contains('/'))
        .map(str::to_string)
        .ok_or_else(|| "invalid command block resource path".to_string())
}

fn is_command_block_resource_path(path: &str) -> bool {
    path.strip_prefix("/v1/command-blocks/")
        .is_some_and(|command_block_id| {
            !command_block_id.is_empty() && !command_block_id.contains('/')
        })
}

fn command_block_status(exit_code: Option<i64>) -> String {
    match exit_code {
        Some(0) => "completed".to_string(),
        Some(_) => "failed".to_string(),
        None => "running".to_string(),
    }
}

fn build_visual_harness_state(
    snapshot: &StateSnapshot,
    manual_links: Vec<crate::state_store::PersistedVisualHarnessLink>,
    current_workflow: Option<&crate::state_store::PersistedWorkflowVersion>,
    policy_pack_summary: &serde_json::Value,
) -> state_snapshot::StateVisualHarness {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for task in &snapshot.tasks.items {
        nodes.push(state_snapshot::StateVisualNode {
            id: task.id.clone(),
            label: task.title.clone(),
            kind: "task".to_string(),
            status: task.status.clone(),
        });
        if let Some(context_pack_id) = task.context_pack_id.as_deref() {
            nodes.push(state_snapshot::StateVisualNode {
                id: context_pack_id.to_string(),
                label: context_pack_id.to_string(),
                kind: "context".to_string(),
                status: "linked".to_string(),
            });
            edges.push(state_snapshot::StateVisualEdge {
                id: format!("edge_{context_pack_id}_{}", task.id),
                source_id: context_pack_id.to_string(),
                target_id: task.id.clone(),
                kind: "context".to_string(),
                status: "active".to_string(),
            });
        }
    }

    for initiative in &snapshot.initiatives {
        nodes.push(state_snapshot::StateVisualNode {
            id: initiative.id.clone(),
            label: initiative.name.clone(),
            kind: "goal".to_string(),
            status: initiative.status.clone(),
        });
    }

    for task in &snapshot.tasks.items {
        if let Some(initiative_id) = task.initiative_id.as_deref() {
            edges.push(state_snapshot::StateVisualEdge {
                id: format!("edge_{initiative_id}_{}", task.id),
                source_id: initiative_id.to_string(),
                target_id: task.id.clone(),
                kind: "goal".to_string(),
                status: "active".to_string(),
            });
        }
    }

    for run in &snapshot.runs.items {
        nodes.push(state_snapshot::StateVisualNode {
            id: run.id.clone(),
            label: run.id.clone(),
            kind: "run".to_string(),
            status: run.lifecycle.clone(),
        });
        edges.push(state_snapshot::StateVisualEdge {
            id: format!("edge_{}_{}", run.task_id, run.id),
            source_id: run.task_id.clone(),
            target_id: run.id.clone(),
            kind: "dispatch".to_string(),
            status: "active".to_string(),
        });
        if let Some(agent_id) = run.agent_profile_id.as_deref() {
            edges.push(state_snapshot::StateVisualEdge {
                id: format!("edge_{agent_id}_{}", run.id),
                source_id: agent_id.to_string(),
                target_id: run.id.clone(),
                kind: "agent".to_string(),
                status: "active".to_string(),
            });
        }
        if let Some(workflow_id) = run.workflow_version_id.as_deref() {
            nodes.push(state_snapshot::StateVisualNode {
                id: workflow_id.to_string(),
                label: workflow_id.to_string(),
                kind: "workflow".to_string(),
                status: "loaded".to_string(),
            });
            edges.push(state_snapshot::StateVisualEdge {
                id: format!("edge_{workflow_id}_{}", run.id),
                source_id: workflow_id.to_string(),
                target_id: run.id.clone(),
                kind: "workflow".to_string(),
                status: "active".to_string(),
            });
        }
        if let Some(context_pack_id) = run.context_pack_id.as_deref() {
            nodes.push(state_snapshot::StateVisualNode {
                id: context_pack_id.to_string(),
                label: context_pack_id.to_string(),
                kind: "context".to_string(),
                status: "linked".to_string(),
            });
            edges.push(state_snapshot::StateVisualEdge {
                id: format!("edge_{context_pack_id}_{}", run.id),
                source_id: context_pack_id.to_string(),
                target_id: run.id.clone(),
                kind: "context".to_string(),
                status: "active".to_string(),
            });
        }
    }

    for agent in &snapshot.agents {
        nodes.push(state_snapshot::StateVisualNode {
            id: agent.id.clone(),
            label: agent.label.clone(),
            kind: "agent".to_string(),
            status: if agent.available {
                "available"
            } else {
                "paused"
            }
            .to_string(),
        });
    }

    for review in &snapshot.reviews {
        nodes.push(state_snapshot::StateVisualNode {
            id: review.id.clone(),
            label: review.evidence_pack_id.clone(),
            kind: "review_gate".to_string(),
            status: review.state.clone(),
        });
        if let Some(run_id) = review.run_id.as_deref() {
            edges.push(state_snapshot::StateVisualEdge {
                id: format!("edge_{run_id}_{}", review.id),
                source_id: run_id.to_string(),
                target_id: review.id.clone(),
                kind: "review_gate".to_string(),
                status: review.completeness_state.clone(),
            });
        } else if let Some(task_id) = review.task_id.as_deref() {
            edges.push(state_snapshot::StateVisualEdge {
                id: format!("edge_{task_id}_{}", review.id),
                source_id: task_id.to_string(),
                target_id: review.id.clone(),
                kind: "review_gate".to_string(),
                status: review.completeness_state.clone(),
            });
        }
    }

    if let Some(workflow) = current_workflow {
        nodes.push(state_snapshot::StateVisualNode {
            id: workflow.id.clone(),
            label: workflow.source_path.clone(),
            kind: "workflow".to_string(),
            status: if workflow.valid { "loaded" } else { "invalid" }.to_string(),
        });

        if let Some(context_pack_id) = workflow.parsed_json["context"]["default_pack"]
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            nodes.push(state_snapshot::StateVisualNode {
                id: context_pack_id.to_string(),
                label: context_pack_id.to_string(),
                kind: "context".to_string(),
                status: "default".to_string(),
            });
            edges.push(state_snapshot::StateVisualEdge {
                id: format!("edge_{}_{}", workflow.id, context_pack_id),
                source_id: workflow.id.clone(),
                target_id: context_pack_id.to_string(),
                kind: "context".to_string(),
                status: "configured".to_string(),
            });
        }

        if let Some(hooks) = workflow.parsed_json["hooks"].as_object() {
            for (hook_name, hook_path) in hooks {
                let hook_path = hook_path.as_str().unwrap_or("").trim();
                if hook_path.is_empty() {
                    continue;
                }
                let hook_id = format!("hook_{hook_name}");
                nodes.push(state_snapshot::StateVisualNode {
                    id: hook_id.clone(),
                    label: format!("{hook_name}: {hook_path}"),
                    kind: "hook".to_string(),
                    status: "configured".to_string(),
                });
                edges.push(state_snapshot::StateVisualEdge {
                    id: format!("edge_{}_{}", workflow.id, hook_id),
                    source_id: workflow.id.clone(),
                    target_id: hook_id,
                    kind: "trigger".to_string(),
                    status: "configured".to_string(),
                });
            }
        }

        if let Some(tools) = workflow.parsed_json["tools"].as_object() {
            for (tool_name, tool_label) in tools {
                let label = tool_label
                    .as_str()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or(tool_name);
                let tool_id = format!("tool_{tool_name}");
                nodes.push(state_snapshot::StateVisualNode {
                    id: tool_id.clone(),
                    label: label.to_string(),
                    kind: "tool".to_string(),
                    status: "configured".to_string(),
                });
                edges.push(state_snapshot::StateVisualEdge {
                    id: format!("edge_{}_{}", workflow.id, tool_id),
                    source_id: workflow.id.clone(),
                    target_id: tool_id,
                    kind: "tool".to_string(),
                    status: "configured".to_string(),
                });
            }
        }
    }

    if let Some(policy_id) = policy_pack_summary["id"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let label = policy_pack_summary["name"]
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(policy_id);
        let status = policy_pack_summary["sandbox_mode"]
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("active");
        nodes.push(state_snapshot::StateVisualNode {
            id: policy_id.to_string(),
            label: label.to_string(),
            kind: "policy_pack".to_string(),
            status: status.to_string(),
        });
        if let Some(workflow) = current_workflow {
            edges.push(state_snapshot::StateVisualEdge {
                id: format!("edge_{policy_id}_{}", workflow.id),
                source_id: policy_id.to_string(),
                target_id: workflow.id.clone(),
                kind: "policy".to_string(),
                status: "active".to_string(),
            });
        }
    }

    for link in manual_links {
        edges.push(state_snapshot::StateVisualEdge {
            id: link.id,
            source_id: link.source_id,
            target_id: link.target_id,
            kind: link.kind,
            status: "manual".to_string(),
        });
    }

    nodes.sort_by(|a, b| a.id.cmp(&b.id).then(a.kind.cmp(&b.kind)));
    nodes.dedup_by(|a, b| a.id == b.id && a.kind == b.kind);
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    edges.dedup_by(|a, b| a.id == b.id);
    state_snapshot::StateVisualHarness {
        diagnostics: serde_json::json!({
            "status": if nodes.is_empty() { "empty" } else { "ok" },
            "node_count": nodes.len(),
            "edge_count": edges.len()
        }),
        nodes,
        edges,
    }
}

fn json_response<T: Serialize>(status_code: u16, reason: &'static str, body: &T) -> HttpResponse {
    let body = serde_json::to_string_pretty(body)
        .unwrap_or_else(|_| "{\"error\":\"serialization\"}".to_string());
    HttpResponse {
        status_code,
        reason,
        content_type: "application/json",
        body,
    }
}

impl HttpResponse {
    pub fn to_http_bytes(&self) -> String {
        format!(
            "HTTP/1.1 {} {}\r\ncontent-type: {}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            self.status_code,
            self.reason,
            self.content_type,
            self.body.len(),
            self.body
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pty::TerminalPtySnapshot,
        state_store::{PersistedTaskInput, SaveTaskWorkpadInput},
    };
    use rusqlite::Connection;
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::PathBuf,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn serves_state_snapshot_json_over_http_request() {
        let db_path = unique_test_db_path("control-state");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_agent_profile(crate::state_store::UpsertAgentProfileInput {
                id: "agent_codex".to_string(),
                name: "Codex".to_string(),
                runtime: "codex".to_string(),
                command: "codex".to_string(),
                args_json: Some(serde_json::json!([])),
                env_policy_json: Some(serde_json::json!({ "inherit": false })),
                skills_json: Some(serde_json::json!(["coding"])),
                status: Some("available".to_string()),
            })
            .expect("agent profile persisted");
        let initiative = store
            .create_initiative(crate::state_store::CreateInitiativeInput {
                project_id: "proj_local".to_string(),
                name: "Platform reliability goal".to_string(),
                description: Some("Keep roadmap work tied to a visible goal".to_string()),
                budget_id: None,
                status: Some("active".to_string()),
            })
            .expect("initiative persisted");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_ready".to_string(),
                key: "HC-READY".to_string(),
                project_id: "proj_local".to_string(),
                title: "Wire control API".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: Some(initiative.id.clone()),
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_auth_state".to_string(),
                key: "AUTH-STATE".to_string(),
                project_id: "proj_auth".to_string(),
                title: "Render auth project state".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "medium".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("auth task persisted");
        store
            .save_task_workpad(SaveTaskWorkpadInput {
                task_id: "task_ready".to_string(),
                body_md: "API evidence".to_string(),
            })
            .expect("workpad persisted");
        store
            .record_token_usage(crate::state_store::TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: Some("session_task_ready".to_string()),
                task_id: Some("task_ready".to_string()),
                run_id: Some("run_task_ready".to_string()),
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 900,
                output_tokens: 600,
                cost_usd: 1.25,
                source: "adapter".to_string(),
            })
            .expect("task usage records");

        let response = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let repeated = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_response = handle_http_request(
            "GET /v1/state?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let moved = handle_http_request(
            "POST /v1/tasks/task_ready/move HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 20\r\n\r\n{\"status\":\"running\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let after_move = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&response.body).expect("json body");
        let repeated_body: serde_json::Value =
            serde_json::from_str(&repeated.body).expect("repeated state json body");
        let auth_body: serde_json::Value =
            serde_json::from_str(&auth_response.body).expect("auth state json body");
        let after_move_body: serde_json::Value =
            serde_json::from_str(&after_move.body).expect("after move state json body");

        assert_eq!(response.status_code, 200);
        assert_eq!(auth_response.status_code, 200);
        assert_eq!(moved.status_code, 200);
        assert_eq!(body["app"]["version"], "0.1.0-test");
        assert_eq!(body["tasks"]["items"][0]["id"], "task_ready");
        assert_eq!(body["tasks"]["items"][0]["has_workpad"], true);
        assert_eq!(
            body["tasks"]["items"][0]["token_usage"]["total_tokens"],
            1500
        );
        assert_eq!(body["tasks"]["items"][0]["token_usage"]["cost_usd"], 1.25);
        assert_eq!(body["projects"][0]["token_usage"]["total_tokens"], 1500);
        assert_eq!(body["projects"][0]["token_usage"]["cost_usd"], 1.25);
        assert_eq!(body["agents"][0]["token_usage"]["total_tokens"], 1500);
        assert_eq!(body["agents"][0]["token_usage"]["cost_usd"], 1.25);
        assert_eq!(body["initiatives"][0]["token_usage"]["total_tokens"], 1500);
        assert_eq!(body["initiatives"][0]["token_usage"]["cost_usd"], 1.25);
        assert_eq!(body["health"]["api"], "ok");
        assert_eq!(body["snapshot_id"], repeated_body["snapshot_id"]);
        assert_ne!(body["snapshot_id"], after_move_body["snapshot_id"]);
        assert_eq!(after_move_body["tasks"]["items"][0]["status"], "running");
        assert_eq!(auth_body["tasks"]["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_body["tasks"]["items"][0]["id"], "task_auth_state");
        assert_eq!(auth_body["tasks"]["items"][0]["project_id"], "proj_auth");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn updates_task_context_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-task-context");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_context".to_string(),
                key: "HC-CONTEXT".to_string(),
                project_id: "proj_local".to_string(),
                title: "Attach context".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");

        let updated = handle_http_request(
            "POST /v1/tasks/task_context/context HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 28\r\n\r\n{\"contextPackId\":\"ctx_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let updated_body: serde_json::Value =
            serde_json::from_str(&updated.body).expect("task json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(updated.status_code, 200);
        assert_eq!(updated_body["context_pack_id"], "ctx_auth");
        assert_eq!(
            state_body["tasks"]["items"][0]["context_pack_id"],
            "ctx_auth"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn edits_task_core_fields_through_patch_endpoint() {
        let db_path = unique_test_db_path("control-task-edit");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_edit".to_string(),
                key: "HC-EDIT".to_string(),
                project_id: "proj_local".to_string(),
                title: "Original title".to_string(),
                description: Some("Original description".to_string()),
                status: "ready".to_string(),
                priority: "medium".to_string(),
                assignee_type: Some("agent".to_string()),
                assignee_id: Some("agent_codex".to_string()),
                cycle_id: Some("Sprint 5".to_string()),
                module_id: Some("Control API".to_string()),
                initiative_id: None,
                context_pack_id: Some("ctx_auth".to_string()),
            })
            .expect("seed task");

        let patched = handle_http_request(
            "PATCH /v1/tasks/task_edit HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"title\":\" Updated title \",\"description\":\" Updated description \",\"priority\":\"urgent\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&patched.body).expect("task json");

        assert_eq!(patched.status_code, 200);
        assert_eq!(body["title"], "Updated title");
        assert_eq!(body["description"], "Updated description");
        assert_eq!(body["priority"], "urgent");
        assert_eq!(body["status"], "ready");
        assert_eq!(body["assignee_id"], "agent_codex");
        assert_eq!(body["context_pack_id"], "ctx_auth");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_single_task_for_cli_open() {
        let db_path = unique_test_db_path("control-task-open");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_open".to_string(),
                key: "HC-OPEN".to_string(),
                project_id: "proj_local".to_string(),
                title: "Open task detail".to_string(),
                description: Some("Detail body".to_string()),
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("seed task");

        let response = handle_http_request(
            "GET /v1/tasks/task_open HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&response.body).expect("task json");

        assert_eq!(response.status_code, 200);
        assert_eq!(body["id"], "task_open");
        assert_eq!(body["title"], "Open task detail");
        assert_eq!(body["description"], "Detail body");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_health_and_not_found_responses() {
        let db_path = unique_test_db_path("control-health");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let health = handle_http_request(
            "GET /v1/health HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let missing = handle_http_request(
            "GET /v1/missing HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        assert_eq!(health.status_code, 200);
        assert!(health.to_http_bytes().starts_with("HTTP/1.1 200 OK"));
        assert_eq!(missing.status_code, 404);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_update_check_metadata_from_static_feed() {
        let db_path = unique_test_db_path("control-update-check");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let response = handle_http_request(
            "GET /v1/update/check?channel=beta HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&response.body).expect("json body");

        assert_eq!(response.status_code, 200);
        assert_eq!(body["channel"], "beta");
        assert_eq!(body["current_version"], "0.1.0-test");
        assert_eq!(body["latest_version"], "0.1.0");
        assert_eq!(body["feed_path"], "/update-feed/beta.json");
        assert_eq!(body["signature_state"], "blocked");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn classifies_update_feed_signature_state_without_exposing_placeholder_status() {
        let blocked_feed = serde_json::json!({
            "platforms": {
                "darwin-aarch64": { "signature": "SIGNATURE_REQUIRED_FOR_RELEASE" }
            }
        });
        let signed_feed = serde_json::json!({
            "platforms": {
                "darwin-aarch64": { "signature": "signed-ed25519-payload" },
                "darwin-x86_64": { "signature": "another-signed-payload" }
            }
        });
        let missing_feed = serde_json::json!({
            "platforms": {
                "darwin-aarch64": { "signature": "" }
            }
        });

        assert_eq!(update_feed_signature_state(&blocked_feed), "blocked");
        assert_eq!(update_feed_signature_state(&signed_feed), "signed");
        assert_eq!(update_feed_signature_state(&missing_feed), "missing");
    }

    #[test]
    fn serves_json_over_a_unix_domain_socket() {
        let db_path = unique_test_db_path("control-socket");
        let socket_path = unique_test_socket_path("control-socket")
            .with_file_name("run")
            .join("haneulchi.sock");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let manager = Arc::new(Mutex::new(TerminalPtyManager::default()));
        let _server =
            start_control_api_server_at(socket_path.clone(), store.clone(), manager, "0.1.0-test")
                .expect("control api server starts");

        let mut stream = UnixStream::connect(&socket_path).expect("socket connects");
        stream
            .write_all(b"GET /v1/health HTTP/1.1\r\nhost: haneulchi\r\n\r\n")
            .expect("request writes");
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("response reads");

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"api\": \"ok\""));
        let socket_dir = socket_path.parent().expect("socket parent");
        assert_eq!(
            fs::metadata(socket_dir)
                .expect("socket dir metadata")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&socket_path)
                .expect("socket metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );

        cleanup_test_db(&db_path);
        if let Some(parent) = socket_path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn serves_task_list_create_move_and_comment_endpoints() {
        let db_path = unique_test_db_path("control-tasks");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let created = handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 77\r\n\r\n{\"projectId\":\"proj_local\",\"title\":\"  Create API task  \",\"priority\":\"high\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let other_project_task = handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"title\":\"Auth board task\",\"priority\":\"medium\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let other_project_filtered_task = handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"title\":\"Auth flow review\",\"priority\":\"high\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let moved_filtered_task = handle_http_request(
            "POST /v1/tasks/task_3/move HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"status\":\"ready\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let moved = handle_http_request(
            "POST /v1/tasks/task_1/move HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 18\r\n\r\n{\"status\":\"ready\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let comment = handle_http_request(
            "POST /v1/tasks/task_1/comments HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 31\r\n\r\n{\"body\":\" API comment added. \"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let comments = handle_http_request(
            "GET /v1/tasks/task_1/comments HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let subtask = handle_http_request(
            "POST /v1/tasks/task_1/subtasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"title\":\" Attach screenshots \"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let completed_subtask = handle_http_request(
            "POST /v1/tasks/task_1/subtasks/subtask_1/status HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"status\":\"done\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let subtasks = handle_http_request(
            "GET /v1/tasks/task_1/subtasks HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let cycle = handle_http_request(
            "POST /v1/cycles HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\" Sprint 13 \",\"startsAt\":\"2026-05-01\",\"endsAt\":\"2026-05-15\",\"status\":\"planned\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let cycles = handle_http_request(
            "GET /v1/cycles HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let module = handle_http_request(
            "POST /v1/modules HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\" Release \",\"description\":\"Release gate work\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let modules = handle_http_request(
            "GET /v1/modules HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let planned = handle_http_request(
            "POST /v1/tasks/task_1/planning HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"cycleId\":\"Sprint 5\",\"moduleId\":\"Control API\",\"initiativeId\":\"init_auth\",\"dueAt\":\"2026-05-15\",\"estimate\":\"3 pts\",\"labels\":[\"release\",\"evidence\"],\"assigneeType\":\"agent\",\"assigneeId\":\"agent_codex\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/tasks HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed_other_project = handle_http_request(
            "GET /v1/tasks?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let filtered_other_project = handle_http_request(
            "GET /v1/tasks?projectId=proj_auth&status=ready&query=flow HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let created_body: serde_json::Value =
            serde_json::from_str(&created.body).expect("created json");
        let other_project_task_body: serde_json::Value =
            serde_json::from_str(&other_project_task.body).expect("other task json");
        let other_project_filtered_task_body: serde_json::Value =
            serde_json::from_str(&other_project_filtered_task.body).expect("filtered task json");
        let moved_filtered_task_body: serde_json::Value =
            serde_json::from_str(&moved_filtered_task.body).expect("moved filtered task json");
        let moved_body: serde_json::Value = serde_json::from_str(&moved.body).expect("moved json");
        let comment_body: serde_json::Value =
            serde_json::from_str(&comment.body).expect("comment json");
        let comments_body: serde_json::Value =
            serde_json::from_str(&comments.body).expect("comments json");
        let subtask_body: serde_json::Value =
            serde_json::from_str(&subtask.body).expect("subtask json");
        let completed_subtask_body: serde_json::Value =
            serde_json::from_str(&completed_subtask.body).expect("completed subtask json");
        let subtasks_body: serde_json::Value =
            serde_json::from_str(&subtasks.body).expect("subtasks json");
        let cycle_body: serde_json::Value = serde_json::from_str(&cycle.body).expect("cycle json");
        let cycles_body: serde_json::Value =
            serde_json::from_str(&cycles.body).expect("cycles json");
        let module_body: serde_json::Value =
            serde_json::from_str(&module.body).expect("module json");
        let modules_body: serde_json::Value =
            serde_json::from_str(&modules.body).expect("modules json");
        let planned_body: serde_json::Value =
            serde_json::from_str(&planned.body).expect("planned json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let listed_other_project_body: serde_json::Value =
            serde_json::from_str(&listed_other_project.body).expect("other listed json");
        let filtered_other_project_body: serde_json::Value =
            serde_json::from_str(&filtered_other_project.body).expect("filtered listed json");

        assert_eq!(created.status_code, 201);
        assert_eq!(created_body["title"], "Create API task");
        assert_eq!(other_project_task.status_code, 201);
        assert_eq!(other_project_task_body["project_id"], "proj_auth");
        assert_eq!(other_project_filtered_task.status_code, 201);
        assert_eq!(
            other_project_filtered_task_body["title"],
            "Auth flow review"
        );
        assert_eq!(moved_filtered_task.status_code, 200);
        assert_eq!(moved_filtered_task_body["status"], "ready");
        assert_eq!(moved.status_code, 200);
        assert_eq!(moved_body["status"], "ready");
        assert_eq!(comment.status_code, 201);
        assert_eq!(comment_body["body_md"], "API comment added.");
        assert_eq!(comments.status_code, 200);
        assert_eq!(comments_body["items"][0]["body_md"], "API comment added.");
        assert_eq!(subtask.status_code, 201);
        assert_eq!(subtask_body["title"], "Attach screenshots");
        assert_eq!(completed_subtask.status_code, 200);
        assert_eq!(completed_subtask_body["status"], "done");
        assert_eq!(subtasks.status_code, 200);
        assert_eq!(subtasks_body["items"][0]["status"], "done");
        assert_eq!(cycle.status_code, 201);
        assert_eq!(cycle_body["name"], "Sprint 13");
        assert_eq!(cycle_body["starts_at"], "2026-05-01");
        assert!(cycle_body["snapshot_id"]
            .as_str()
            .unwrap()
            .starts_with("snap_"));
        assert_eq!(cycles.status_code, 200);
        assert_eq!(cycles_body["items"][0]["name"], "Sprint 13");
        assert_eq!(module.status_code, 201);
        assert_eq!(module_body["name"], "Release");
        assert_eq!(module_body["status"], "active");
        assert!(module_body["snapshot_id"]
            .as_str()
            .unwrap()
            .starts_with("snap_"));
        assert_eq!(modules.status_code, 200);
        assert_eq!(modules_body["items"][0]["description"], "Release gate work");
        assert_eq!(planned.status_code, 200);
        assert_eq!(planned_body["cycle_id"], "Sprint 5");
        assert_eq!(planned_body["module_id"], "Control API");
        assert_eq!(planned_body["initiative_id"], "init_auth");
        assert_eq!(planned_body["due_at"], "2026-05-15");
        assert_eq!(planned_body["estimate"], "3 pts");
        assert_eq!(planned_body["labels"][0], "release");
        assert_eq!(planned_body["labels"][1], "evidence");
        assert_eq!(planned_body["assignee_id"], "agent_codex");
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["comment_count"], 1);
        assert_eq!(listed_body["items"][0]["subtask_count"], 1);
        assert_eq!(listed_body["items"][0]["open_subtask_count"], 0);
        assert_eq!(listed_other_project.status_code, 200);
        assert_eq!(
            listed_other_project_body["items"].as_array().unwrap().len(),
            2
        );
        assert_eq!(
            listed_other_project_body["items"][0]["project_id"],
            "proj_auth"
        );
        assert_eq!(filtered_other_project.status_code, 200);
        assert_eq!(
            filtered_other_project_body["items"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            filtered_other_project_body["items"][0]["title"],
            "Auth flow review"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn task_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-task-mutation-snapshot-id");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };

        let created = handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"title\":\"Snapshot task\",\"priority\":\"high\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let patched = handle_http_request(
            "PATCH /v1/tasks/task_1 HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"title\":\"Snapshot task edited\",\"description\":\"Keep parity\",\"priority\":\"medium\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let moved = handle_http_request(
            "POST /v1/tasks/task_1/move HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"status\":\"ready\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let comment = handle_http_request(
            "POST /v1/tasks/task_1/comments HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"body\":\"Snapshot comment\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let workpad = handle_http_request(
            "POST /v1/tasks/task_1/workpad HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"body\":\"## Notes\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let planned = handle_http_request(
            "POST /v1/tasks/task_1/planning HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"cycleId\":\"Sprint 5\",\"moduleId\":\"Control API\",\"initiativeId\":\"init_auth\",\"assigneeType\":\"agent\",\"assigneeId\":\"agent_codex\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let context = handle_http_request(
            "POST /v1/tasks/task_1/context HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"contextPackId\":\"ctx_auth\"}",
            &store,
            pty,
            "0.1.0-test",
        );
        let responses = [
            ("create", 201, created),
            ("patch", 200, patched),
            ("move", 200, moved),
            ("comment", 201, comment),
            ("workpad", 200, workpad),
            ("planning", 200, planned),
            ("context", 200, context),
        ];

        for (name, expected_status, response) in responses {
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("task mutation response json");

            assert_eq!(response.status_code, expected_status, "{name}");
            assert!(
                body["snapshot_id"]
                    .as_str()
                    .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
                "{name} should include a snapshot_id: {}",
                response.body
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_initiatives_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-initiatives");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let created = handle_http_request(
            "POST /v1/initiatives HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\" Auth reliability goal \",\"description\":\"Why auth tasks matter\",\"budgetId\":\"budget_auth\",\"status\":\"active\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/initiatives HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let created_body: serde_json::Value =
            serde_json::from_str(&created.body).expect("created initiative json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed initiative json");
        let state_body: serde_json::Value =
            serde_json::from_str(&state.body).expect("state initiative json");

        assert_eq!(created.status_code, 201);
        assert_eq!(created_body["id"], "init_1");
        assert_eq!(created_body["name"], "Auth reliability goal");
        assert_eq!(created_body["status"], "active");
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["id"], "init_1");
        assert_eq!(state.status_code, 200);
        assert_eq!(state_body["initiatives"][0]["id"], "init_1");
        assert_eq!(
            state_body["initiatives"][0]["description"],
            "Why auth tasks matter"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn saves_task_workpad_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-task-workpad");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_review".to_string(),
                key: "HC-WORKPAD".to_string(),
                project_id: "proj_local".to_string(),
                title: "Review workpad".to_string(),
                description: None,
                status: "review".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");

        let saved = handle_http_request(
            "POST /v1/tasks/task_review/workpad HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"body\":\"## Notes\\n- Attach release evidence\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let saved_body: serde_json::Value =
            serde_json::from_str(&saved.body).expect("workpad json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(saved.status_code, 200);
        assert_eq!(saved_body["task_id"], "task_review");
        assert_eq!(saved_body["body_md"], "## Notes\n- Attach release evidence");
        assert_eq!(state_body["tasks"]["items"][0]["has_workpad"], true);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_external_tracker_bindings_through_control_api_and_state() {
        let db_path = unique_test_db_path("control-tracker-bindings");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let created_task = handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"title\":\"Bind tracker task\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_task = handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"title\":\"Bind auth tracker task\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let binding = handle_http_request(
            "POST /v1/tracker-bindings HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"localKind\":\"task\",\"localId\":\"task_1\",\"provider\":\"linear\",\"externalId\":\"LIN-42\",\"externalUrl\":\"https://linear.app/acme/issue/LIN-42\",\"syncMode\":\"mirror\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_binding = handle_http_request(
            "POST /v1/tracker-bindings HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"localKind\":\"task\",\"localId\":\"task_2\",\"provider\":\"github\",\"externalId\":\"octo/repo#42\",\"syncMode\":\"manual\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let invalid = handle_http_request(
            "POST /v1/tracker-bindings HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"localKind\":\"task\",\"localId\":\"missing\",\"provider\":\"jira\",\"externalId\":\"SEC-1\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/tracker-bindings HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_listed = handle_http_request(
            "GET /v1/tracker-bindings?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let created_task_body: serde_json::Value =
            serde_json::from_str(&created_task.body).expect("created task json");
        let binding_body: serde_json::Value =
            serde_json::from_str(&binding.body).expect("binding json");
        let auth_binding_body: serde_json::Value =
            serde_json::from_str(&auth_binding.body).expect("auth binding json");
        let invalid_body: serde_json::Value =
            serde_json::from_str(&invalid.body).expect("invalid json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let auth_listed_body: serde_json::Value =
            serde_json::from_str(&auth_listed.body).expect("auth listed json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(created_task.status_code, 201);
        assert_eq!(created_task_body["id"], "task_1");
        assert_eq!(auth_task.status_code, 201);
        assert_eq!(binding.status_code, 201);
        assert_eq!(binding_body["id"], "tracker_binding_1");
        assert_eq!(binding_body["provider"], "linear");
        assert_eq!(binding_body["external_id"], "LIN-42");
        assert_eq!(binding_body["sync_status"], "pending");
        assert_eq!(auth_binding.status_code, 201);
        assert_eq!(auth_binding_body["project_id"], "proj_auth");
        assert_eq!(invalid.status_code, 400);
        assert!(invalid_body["error"]
            .as_str()
            .unwrap()
            .contains("unsupported tracker provider"));
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["local_id"], "task_1");
        assert_eq!(listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_listed.status_code, 200);
        assert_eq!(auth_listed_body["items"][0]["external_id"], "octo/repo#42");
        assert_eq!(auth_listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(state_body["tracker"]["binding_count"], 1);
        assert_eq!(
            state_body["tracker"]["bindings"][0]["external_id"],
            "LIN-42"
        );
        assert_eq!(state_body["tracker"]["diagnostics"]["status"], "pending");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn plans_linear_sync_runs_without_leaking_secrets() {
        let db_path = unique_test_db_path("control-linear-sync");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"title\":\"Mirror to Linear\",\"priority\":\"urgent\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/tracker-bindings HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"localKind\":\"task\",\"localId\":\"task_1\",\"provider\":\"linear\",\"externalId\":\"LIN-42\",\"syncMode\":\"mirror\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let dry_run = handle_http_request(
            "POST /v1/tracker-sync/linear/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"dryRun\":true}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let real_run_without_token = handle_http_request(
            "POST /v1/tracker-sync/linear/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"dryRun\":false}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let dry_run_body: serde_json::Value =
            serde_json::from_str(&dry_run.body).expect("dry run json");
        let real_run_body: serde_json::Value =
            serde_json::from_str(&real_run_without_token.body).expect("real run json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(dry_run.status_code, 200);
        assert_eq!(dry_run_body["provider"], "linear");
        assert_eq!(dry_run_body["status"], "planned");
        assert_eq!(dry_run_body["operations"][0]["operation"], "issueUpdate");
        assert_eq!(dry_run_body["operations"][0]["external_id"], "LIN-42");
        assert_eq!(
            dry_run_body["operations"][0]["payload"]["title"],
            "Mirror to Linear"
        );
        assert_eq!(real_run_without_token.status_code, 200);
        assert_eq!(real_run_body["status"], "degraded");
        assert_eq!(
            real_run_body["degraded_reason"],
            "missing LINEAR_API_KEY secret"
        );
        assert!(!real_run_without_token.body.contains("secret-linear-token"));
        assert_eq!(
            state_body["tracker"]["diagnostics"]["linear"]["last_status"],
            "degraded"
        );
        assert_eq!(
            state_body["tracker"]["diagnostics"]["linear"]["last_operation_count"],
            1
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn plans_github_and_plane_sync_runs_without_network_access() {
        let db_path = unique_test_db_path("control-github-plane-sync");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"title\":\"Mirror to external trackers\",\"priority\":\"high\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/tracker-bindings HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"localKind\":\"task\",\"localId\":\"task_1\",\"provider\":\"github\",\"externalId\":\"octo/repo#123\",\"syncMode\":\"mirror\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/tracker-bindings HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"localKind\":\"task\",\"localId\":\"task_1\",\"provider\":\"plane\",\"externalId\":\"PLN-7\",\"syncMode\":\"mirror\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let github_dry_run = handle_http_request(
            "POST /v1/tracker-sync/github/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"dryRun\":true}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let plane_real_run = handle_http_request(
            "POST /v1/tracker-sync/plane/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"dryRun\":false}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let github_body: serde_json::Value =
            serde_json::from_str(&github_dry_run.body).expect("github json");
        let plane_body: serde_json::Value =
            serde_json::from_str(&plane_real_run.body).expect("plane json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(github_dry_run.status_code, 200);
        assert_eq!(github_body["provider"], "github");
        assert_eq!(github_body["status"], "planned");
        assert_eq!(github_body["operations"][0]["operation"], "issueUpdate");
        assert_eq!(github_body["operations"][0]["external_id"], "octo/repo#123");
        assert_eq!(plane_real_run.status_code, 200);
        assert_eq!(plane_body["provider"], "plane");
        assert_eq!(plane_body["status"], "degraded");
        assert_eq!(
            plane_body["degraded_reason"],
            "missing PLANE_API_KEY secret"
        );
        assert_eq!(
            state_body["tracker"]["diagnostics"]["github"]["last_status"],
            "planned"
        );
        assert_eq!(
            state_body["tracker"]["diagnostics"]["plane"]["last_status"],
            "degraded"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn workflow_integration_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-workflow-integration-snapshot-id");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(&workspace).expect("workspace dir");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(crate::state_store::AddProjectInput {
                key: "LOCAL".to_string(),
                name: "Local Project".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_integration_receipt".to_string(),
                key: "HC-INTEGRATION-RECEIPT".to_string(),
                project_id: "proj_local".to_string(),
                title: "Integration receipt task".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };
        let requests = [
            (
                "release gate",
                201,
                "POST /v1/release-gates/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            ),
            (
                "terminal smoke",
                201,
                "POST /v1/terminal-fidelity/smoke/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            ),
            (
                "task lifecycle",
                201,
                "POST /v1/task-lifecycle/e2e/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            ),
            (
                "workflow negative",
                201,
                "POST /v1/workflow/negative-tests/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            ),
            (
                "dmg smoke",
                201,
                "POST /v1/distribution/dmg-smoke/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            ),
            (
                "recovery drill",
                201,
                "POST /v1/recovery/drills/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            ),
            (
                "benchmark",
                201,
                "POST /v1/benchmarks/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            ),
            (
                "dogfood telemetry",
                201,
                "POST /v1/dogfood/telemetry-review/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            ),
            (
                "visual link",
                201,
                "POST /v1/visual-harness/links HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sourceId\":\"task_integration_receipt\",\"targetId\":\"release_gate_1\",\"kind\":\"workflow\"}",
            ),
            (
                "linear binding",
                201,
                "POST /v1/tracker-bindings HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"localKind\":\"task\",\"localId\":\"task_integration_receipt\",\"provider\":\"linear\",\"externalId\":\"LIN-1\",\"syncMode\":\"mirror\"}",
            ),
            (
                "github binding",
                201,
                "POST /v1/tracker-bindings HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"localKind\":\"task\",\"localId\":\"task_integration_receipt\",\"provider\":\"github\",\"externalId\":\"octo/repo#1\",\"syncMode\":\"mirror\"}",
            ),
            (
                "plane binding",
                201,
                "POST /v1/tracker-bindings HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"localKind\":\"task\",\"localId\":\"task_integration_receipt\",\"provider\":\"plane\",\"externalId\":\"PLN-1\",\"syncMode\":\"mirror\"}",
            ),
            (
                "linear sync",
                200,
                "POST /v1/tracker-sync/linear/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"dryRun\":true}",
            ),
            (
                "github sync",
                200,
                "POST /v1/tracker-sync/github/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"dryRun\":true}",
            ),
            (
                "plane sync",
                200,
                "POST /v1/tracker-sync/plane/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"dryRun\":true}",
            ),
            (
                "browser automation",
                200,
                "POST /v1/browser-automation/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"url\":\"http://localhost:3000\",\"scenario\":\"snapshot\"}",
            ),
        ];

        for (name, expected_status, request) in requests {
            let response = handle_http_request(request, &store, pty.clone(), "0.1.0-test");
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("mutation response json");

            assert_eq!(response.status_code, expected_status, "{name}");
            assert!(
                body["snapshot_id"]
                    .as_str()
                    .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
                "{name} should include a snapshot_id: {}",
                response.body
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_browser_lsp_patch_and_pr_boundary_plans() {
        let db_path = unique_test_db_path("control-boundary-plans");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(workspace.join("src")).expect("workspace tree");
        fs::write(
            workspace.join("src/app.ts"),
            "export function loadUser() {\n  return 1 as any;\n}\n// TODO tighten type\n",
        )
        .expect("source written");
        Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .output()
            .expect("git init runs");
        Command::new("git")
            .args(["add", "src/app.ts"])
            .current_dir(&workspace)
            .output()
            .expect("git add runs");
        Command::new("git")
            .args([
                "-c",
                "user.email=haneulchi@example.test",
                "-c",
                "user.name=Haneulchi Tests",
                "commit",
                "-m",
                "seed",
            ])
            .current_dir(&workspace)
            .output()
            .expect("git commit runs");
        fs::write(
            workspace.join("src/app.ts"),
            "export function loadUser() {\n  return 2 as any;\n}\n// TODO tighten type\n",
        )
        .expect("source updated");

        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(crate::state_store::AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let browser = handle_http_request(
            "POST /v1/browser-automation/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"url\":\"http://localhost:3000/docs\",\"scenario\":\"smoke\"}",
            &store,
            TerminalPtySnapshot { total: 0, sessions: vec![] },
            "0.1.0-test",
        );
        let lsp = handle_http_request(
            "GET /v1/projects/proj_auth/lsp-diagnostics?path=src/app.ts HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot { total: 0, sessions: vec![] },
            "0.1.0-test",
        );
        let patch = handle_http_request(
            "GET /v1/projects/proj_auth/patch/export HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let patch_body: serde_json::Value = serde_json::from_str(&patch.body).expect("patch json");
        let imported = handle_http_request(
            &format!(
                "POST /v1/projects/proj_auth/patch/import HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{}",
                serde_json::json!({ "body": patch_body["body"] })
            ),
            &store,
            TerminalPtySnapshot { total: 0, sessions: vec![] },
            "0.1.0-test",
        );
        let pr = handle_http_request(
            "POST /v1/projects/proj_auth/pr/landing-plan HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"title\":\"Ship auth\",\"draft\":true}",
            &store,
            TerminalPtySnapshot { total: 0, sessions: vec![] },
            "0.1.0-test",
        );

        let browser_body: serde_json::Value =
            serde_json::from_str(&browser.body).expect("browser json");
        let lsp_body: serde_json::Value = serde_json::from_str(&lsp.body).expect("lsp json");
        let imported_body: serde_json::Value =
            serde_json::from_str(&imported.body).expect("import json");
        let pr_body: serde_json::Value = serde_json::from_str(&pr.body).expect("pr json");

        assert_eq!(browser.status_code, 200);
        assert_eq!(browser_body["status"], "planned");
        assert_eq!(lsp.status_code, 200);
        assert!(lsp_body["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["message"].as_str().unwrap().contains("TODO")));
        assert!(lsp_body["symbols"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "loadUser" && item["kind"] == "function"));
        assert_eq!(patch.status_code, 200);
        assert_eq!(patch_body["status"], "exported");
        assert_eq!(imported.status_code, 200);
        assert_eq!(imported_body["status"], "validated");
        assert_eq!(pr.status_code, 200);
        assert_eq!(pr_body["provider"], "github");
        assert_eq!(pr_body["draft"], true);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn project_tool_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-project-tool-snapshot-id");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(workspace.join("src")).expect("workspace tree");
        fs::write(workspace.join("src/app.ts"), "export const before = 1;\n")
            .expect("source written");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(crate::state_store::AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");
        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };
        let written = handle_http_request(
            "POST /v1/projects/proj_auth/file HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"path\":\"src/app.ts\",\"body\":\"export const after = 2;\\n\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let imported = handle_http_request(
            "POST /v1/projects/proj_auth/patch/import HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"body\":\"diff --git a/src/app.ts b/src/app.ts\\n--- a/src/app.ts\\n+++ b/src/app.ts\\n@@ -1 +1 @@\\n-export const before = 1;\\n+export const after = 2;\\n\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let landing_plan = handle_http_request(
            "POST /v1/projects/proj_auth/pr/landing-plan HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"title\":\"Snapshot project tools\",\"draft\":true}",
            &store,
            pty,
            "0.1.0-test",
        );
        let responses = [
            ("file write", 200, written),
            ("patch import", 200, imported),
            ("pr landing plan", 200, landing_plan),
        ];

        for (name, expected_status, response) in responses {
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("mutation response json");

            assert_eq!(response.status_code, expected_status, "{name}");
            assert!(
                body["snapshot_id"]
                    .as_str()
                    .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
                "{name} should include a snapshot_id: {}",
                response.body
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_release_gate_scenarios_and_records_state_summary() {
        let db_path = unique_test_db_path("control-release-gates");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"title\":\"Review release evidence\",\"priority\":\"high\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/tasks/task_1/move HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"status\":\"ready\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/dispatch HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"taskId\":\"task_1\",\"agentProfileId\":\"agent_codex\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_release_test".to_string(),
                session_id: "session_1".to_string(),
                task_id: Some("task_1".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(1),
                seq_end: Some(2),
                command: "cargo test --manifest-path src-tauri/Cargo.toml".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(1200),
                summary: Some("release gate tests passed".to_string()),
            })
            .expect("command block persisted");
        handle_http_request(
            "POST /v1/runs/run_1/evidence/generate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"evidencePackId\":\"ev_release\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/reviews/review_ev_release/decision HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"decision\":\"approved\",\"reviewerId\":\"human\",\"bodyMd\":\"Release evidence reviewed.\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let run = handle_http_request(
            "POST /v1/release-gates/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_run = handle_http_request(
            "POST /v1/release-gates/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/release-gates/runs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_listed = handle_http_request(
            "GET /v1/release-gates/runs?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let run_body: serde_json::Value = serde_json::from_str(&run.body).expect("run json");
        let auth_run_body: serde_json::Value =
            serde_json::from_str(&auth_run.body).expect("auth run json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let auth_listed_body: serde_json::Value =
            serde_json::from_str(&auth_listed.body).expect("auth listed json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let rg09 = run_body["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|scenario| scenario["gate_id"] == "RG-09")
            .expect("RG-09 scenario exists");

        assert_eq!(run.status_code, 201);
        assert_eq!(run_body["id"], "release_gate_1");
        assert_eq!(auth_run.status_code, 201);
        assert_eq!(auth_run_body["id"], "release_gate_2");
        assert_eq!(run_body["scenario_count"], 14);
        assert_eq!(run_body["status"], "blocked");
        assert_eq!(rg09["status"], "pass");
        assert_eq!(rg09["evidence"][0], "ev_release");
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["id"], "release_gate_1");
        assert_eq!(listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_listed.status_code, 200);
        assert_eq!(auth_listed_body["items"][0]["id"], "release_gate_2");
        assert_eq!(auth_listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(state_body["release_gates"]["last_run_id"], "release_gate_1");
        assert_eq!(state_body["release_gates"]["last_status"], "blocked");
        assert_eq!(
            state_body["release_gates"]["last_pass_count"],
            run_body["pass_count"]
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_terminal_fidelity_smoke_tests_and_records_state_summary() {
        let db_path = unique_test_db_path("control-terminal-smoke");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let run = handle_http_request(
            "POST /v1/terminal-fidelity/smoke/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_run = handle_http_request(
            "POST /v1/terminal-fidelity/smoke/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/terminal-fidelity/smoke/runs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_listed = handle_http_request(
            "GET /v1/terminal-fidelity/smoke/runs?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let run_body: serde_json::Value = serde_json::from_str(&run.body).expect("run json");
        let auth_run_body: serde_json::Value =
            serde_json::from_str(&auth_run.body).expect("auth run json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let auth_listed_body: serde_json::Value =
            serde_json::from_str(&auth_listed.body).expect("auth listed json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let shell_case = run_body["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "shell_basic")
            .expect("shell case exists");
        let ime_case = run_body["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "ime_korean")
            .expect("ime case exists");
        let safe_link_case = run_body["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "safe_link_sanitization")
            .expect("safe link case exists");
        let osc_case = run_body["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "osc_allowlist")
            .expect("osc allowlist case exists");

        assert_eq!(run.status_code, 201);
        assert_eq!(run_body["id"], "terminal_smoke_1");
        assert_eq!(auth_run.status_code, 201);
        assert_eq!(auth_run_body["id"], "terminal_smoke_2");
        assert_eq!(run_body["status"], "warning");
        assert_eq!(run_body["case_count"], 8);
        assert_eq!(shell_case["status"], "pass");
        assert_eq!(safe_link_case["status"], "pass");
        assert_eq!(osc_case["status"], "pass");
        assert_eq!(ime_case["status"], "warning");
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["id"], "terminal_smoke_1");
        assert_eq!(listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_listed.status_code, 200);
        assert_eq!(auth_listed_body["items"][0]["id"], "terminal_smoke_2");
        assert_eq!(auth_listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(
            state_body["terminal_fidelity"]["last_run_id"],
            "terminal_smoke_1"
        );
        assert_eq!(state_body["terminal_fidelity"]["last_status"], "warning");
        assert_eq!(
            state_body["terminal_fidelity"]["last_warning_count"],
            run_body["warning_count"]
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_task_lifecycle_e2e_and_records_state_and_release_gate_evidence() {
        let db_path = unique_test_db_path("control-task-lifecycle-e2e");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let run = handle_http_request(
            "POST /v1/task-lifecycle/e2e/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_run = handle_http_request(
            "POST /v1/task-lifecycle/e2e/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/task-lifecycle/e2e/runs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_listed = handle_http_request(
            "GET /v1/task-lifecycle/e2e/runs?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let release_gate = handle_http_request(
            "POST /v1/release-gates/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let run_body: serde_json::Value = serde_json::from_str(&run.body).expect("e2e json");
        let auth_run_body: serde_json::Value =
            serde_json::from_str(&auth_run.body).expect("auth e2e json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let auth_listed_body: serde_json::Value =
            serde_json::from_str(&auth_listed.body).expect("auth listed json");
        let release_body: serde_json::Value =
            serde_json::from_str(&release_gate.body).expect("release gate json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let rg05 = release_body["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|scenario| scenario["gate_id"] == "RG-05")
            .expect("RG-05 scenario exists");

        assert_eq!(run.status_code, 201);
        assert_eq!(run_body["id"], "task_lifecycle_e2e_1");
        assert_eq!(auth_run.status_code, 201);
        assert_eq!(auth_run_body["id"], "task_lifecycle_e2e_2");
        assert_eq!(run_body["status"], "passed");
        assert_eq!(run_body["task_id"], "task_1");
        assert_eq!(run_body["run_id"], "run_1");
        assert_eq!(run_body["evidence_pack_id"], "ev_lifecycle_run_1");
        assert_eq!(run_body["transitions"][1]["task_status"], "ready");
        assert_eq!(run_body["transitions"][3]["run_lifecycle"], "running");
        assert_eq!(run_body["transitions"][4]["task_status"], "review");
        assert_eq!(run_body["transitions"][5]["task_status"], "done");
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["id"], "task_lifecycle_e2e_1");
        assert_eq!(listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_listed.status_code, 200);
        assert_eq!(auth_listed_body["items"][0]["id"], "task_lifecycle_e2e_2");
        assert_eq!(auth_listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(
            state_body["task_lifecycle"]["last_run_id"],
            "task_lifecycle_e2e_1"
        );
        assert_eq!(state_body["task_lifecycle"]["last_status"], "passed");
        assert_eq!(rg05["status"], "pass");
        assert_eq!(rg05["evidence"][0], "task_lifecycle_e2e_1");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_workflow_negative_tests_and_records_state_and_release_gate_evidence() {
        let db_path = unique_test_db_path("control-workflow-negative");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let run = handle_http_request(
            "POST /v1/workflow/negative-tests/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_run = handle_http_request(
            "POST /v1/workflow/negative-tests/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/workflow/negative-tests/runs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_listed = handle_http_request(
            "GET /v1/workflow/negative-tests/runs?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let release_gate = handle_http_request(
            "POST /v1/release-gates/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let run_body: serde_json::Value =
            serde_json::from_str(&run.body).expect("workflow negative json");
        let auth_run_body: serde_json::Value =
            serde_json::from_str(&auth_run.body).expect("auth workflow negative json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let auth_listed_body: serde_json::Value =
            serde_json::from_str(&auth_listed.body).expect("auth listed json");
        let release_body: serde_json::Value =
            serde_json::from_str(&release_gate.body).expect("release gate json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let rg07 = release_body["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|scenario| scenario["gate_id"] == "RG-07")
            .expect("RG-07 scenario exists");

        assert_eq!(run.status_code, 201);
        assert_eq!(run_body["id"], "workflow_negative_1");
        assert_eq!(auth_run.status_code, 201);
        assert_eq!(auth_run_body["id"], "workflow_negative_2");
        assert_eq!(run_body["status"], "passed");
        assert_eq!(run_body["baseline_workflow_id"], "workflow_1");
        assert_eq!(run_body["invalid_workflow_id"], "workflow_2");
        assert_eq!(run_body["last_known_good_workflow_id"], "workflow_1");
        assert_eq!(run_body["dispatch_run_id"], "run_1");
        assert_eq!(run_body["cases"][1]["status"], "pass");
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["id"], "workflow_negative_1");
        assert_eq!(listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_listed.status_code, 200);
        assert_eq!(auth_listed_body["items"][0]["id"], "workflow_negative_2");
        assert_eq!(auth_listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(
            state_body["workflow_negative"]["last_run_id"],
            "workflow_negative_1"
        );
        assert_eq!(state_body["workflow_negative"]["last_status"], "passed");
        assert_eq!(state_body["workflow"]["valid"], true);
        assert_eq!(rg07["status"], "pass");
        assert_eq!(rg07["evidence"][0], "workflow_negative_1");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_dmg_smoke_and_records_state_and_release_gate_blocker_evidence() {
        let db_path = unique_test_db_path("control-dmg-smoke");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let run = handle_http_request(
            "POST /v1/distribution/dmg-smoke/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_run = handle_http_request(
            "POST /v1/distribution/dmg-smoke/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/distribution/dmg-smoke/runs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_listed = handle_http_request(
            "GET /v1/distribution/dmg-smoke/runs?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let release_gate = handle_http_request(
            "POST /v1/release-gates/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let rejected = handle_http_request(
            "POST /v1/distribution/dmg-smoke/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"   \"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let run_body: serde_json::Value = serde_json::from_str(&run.body).expect("dmg smoke json");
        let auth_run_body: serde_json::Value =
            serde_json::from_str(&auth_run.body).expect("auth dmg smoke json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let auth_listed_body: serde_json::Value =
            serde_json::from_str(&auth_listed.body).expect("auth listed json");
        let release_body: serde_json::Value =
            serde_json::from_str(&release_gate.body).expect("release gate json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let rg13 = release_body["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|scenario| scenario["gate_id"] == "RG-13")
            .expect("RG-13 scenario exists");
        let artifact_case = run_body["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "dmg_artifact")
            .expect("DMG artifact case exists");

        assert_eq!(run.status_code, 201);
        assert_eq!(run_body["id"], "dmg_smoke_1");
        assert_eq!(auth_run.status_code, 201);
        assert_eq!(auth_run_body["id"], "dmg_smoke_2");
        assert_eq!(run_body["status"], "blocked");
        assert_eq!(run_body["explicit_blocker"], true);
        assert_eq!(run_body["case_count"], 4);
        assert_eq!(artifact_case["status"], "fail");
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["id"], "dmg_smoke_1");
        assert_eq!(listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_listed.status_code, 200);
        assert_eq!(auth_listed_body["items"][0]["id"], "dmg_smoke_2");
        assert_eq!(auth_listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(
            state_body["distribution"]["last_dmg_smoke_run_id"],
            "dmg_smoke_1"
        );
        assert_eq!(state_body["distribution"]["last_status"], "blocked");
        assert_eq!(state_body["distribution"]["explicit_blocker"], true);
        assert_eq!(rg13["status"], "pass");
        assert_eq!(rg13["evidence"][0], "dmg_smoke_1");
        assert_eq!(rejected.status_code, 400);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_recovery_drills_and_records_state_and_release_gate_evidence() {
        let db_path = unique_test_db_path("control-recovery-drills");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let run = handle_http_request(
            "POST /v1/recovery/drills/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_run = handle_http_request(
            "POST /v1/recovery/drills/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/recovery/drills/runs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_listed = handle_http_request(
            "GET /v1/recovery/drills/runs?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let release_gate = handle_http_request(
            "POST /v1/release-gates/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let rejected = handle_http_request(
            "POST /v1/recovery/drills/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let run_body: serde_json::Value =
            serde_json::from_str(&run.body).expect("recovery drills json");
        let auth_run_body: serde_json::Value =
            serde_json::from_str(&auth_run.body).expect("auth recovery drills json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let auth_listed_body: serde_json::Value =
            serde_json::from_str(&auth_listed.body).expect("auth listed json");
        let release_body: serde_json::Value =
            serde_json::from_str(&release_gate.body).expect("release gate json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let rg14 = release_body["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|scenario| scenario["gate_id"] == "RG-14")
            .expect("RG-14 scenario exists");
        let workflow_drill = run_body["drills"]
            .as_array()
            .unwrap()
            .iter()
            .find(|drill| drill["drill_id"] == "invalid_workflow_lkg")
            .expect("invalid workflow drill exists");

        assert_eq!(run.status_code, 201);
        assert_eq!(run_body["id"], "recovery_drill_1");
        assert_eq!(auth_run.status_code, 201);
        assert_eq!(auth_run_body["id"], "recovery_drill_2");
        assert_eq!(run_body["status"], "passed");
        assert_eq!(run_body["drill_count"], 4);
        assert_eq!(workflow_drill["status"], "pass");
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["id"], "recovery_drill_1");
        assert_eq!(listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_listed.status_code, 200);
        assert_eq!(auth_listed_body["items"][0]["id"], "recovery_drill_2");
        assert_eq!(auth_listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(state_body["recovery"]["last_run_id"], "recovery_drill_1");
        assert_eq!(state_body["recovery"]["last_status"], "passed");
        assert_eq!(rg14["status"], "pass");
        assert_eq!(rg14["evidence"][0], "recovery_drill_1");
        assert_eq!(rejected.status_code, 400);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_benchmarks_and_records_state_and_release_gate_evidence() {
        let db_path = unique_test_db_path("control-benchmarks");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let run = handle_http_request(
            "POST /v1/benchmarks/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_run = handle_http_request(
            "POST /v1/benchmarks/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/benchmarks/runs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_listed = handle_http_request(
            "GET /v1/benchmarks/runs?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let release_gate = handle_http_request(
            "POST /v1/release-gates/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let rejected = handle_http_request(
            "POST /v1/benchmarks/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"  \"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let run_body: serde_json::Value = serde_json::from_str(&run.body).expect("benchmark json");
        let auth_run_body: serde_json::Value =
            serde_json::from_str(&auth_run.body).expect("auth benchmark json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let auth_listed_body: serde_json::Value =
            serde_json::from_str(&auth_listed.body).expect("auth listed json");
        let release_body: serde_json::Value =
            serde_json::from_str(&release_gate.body).expect("release gate json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let rg03 = release_body["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|scenario| scenario["gate_id"] == "RG-03")
            .expect("RG-03 scenario exists");
        let rg08 = release_body["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|scenario| scenario["gate_id"] == "RG-08")
            .expect("RG-08 scenario exists");

        assert_eq!(run.status_code, 201);
        assert_eq!(run_body["id"], "benchmark_1");
        assert_eq!(auth_run.status_code, 201);
        assert_eq!(auth_run_body["id"], "benchmark_2");
        assert_eq!(run_body["status"], "passed");
        assert_eq!(run_body["suite_count"], 5);
        assert_eq!(run_body["suites"][0]["suite_id"], "state_snapshot_latency");
        assert_eq!(
            run_body["suites"][3]["suite_id"],
            "ui_cli_api_snapshot_parity"
        );
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["id"], "benchmark_1");
        assert_eq!(listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_listed.status_code, 200);
        assert_eq!(auth_listed_body["items"][0]["id"], "benchmark_2");
        assert_eq!(auth_listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(state_body["benchmarks"]["last_run_id"], "benchmark_1");
        assert_eq!(state_body["benchmarks"]["last_status"], "passed");
        assert_eq!(
            state_body["benchmarks"]["suites"][0]["suite_id"],
            "state_snapshot_latency"
        );
        assert_eq!(rg03["status"], "pass");
        assert_eq!(rg03["evidence"][0], "benchmark_1");
        assert_eq!(rg08["status"], "pass");
        assert_eq!(rg08["evidence"][0], "benchmark_1");
        assert_eq!(rejected.status_code, 400);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_dogfood_telemetry_review_and_records_release_evidence() {
        let db_path = unique_test_db_path("control-dogfood-telemetry");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_dogfood".to_string(),
                session_id: "session_dogfood".to_string(),
                task_id: None,
                run_id: None,
                seq_start: Some(1),
                seq_end: Some(2),
                command: "npm test".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(1200),
                summary: Some("dogfood telemetry command passed".to_string()),
            })
            .expect("command block persisted");

        let review = handle_http_request(
            "POST /v1/dogfood/telemetry-review/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_review = handle_http_request(
            "POST /v1/dogfood/telemetry-review/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/dogfood/telemetry-reviews HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_listed = handle_http_request(
            "GET /v1/dogfood/telemetry-reviews?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let release_gate = handle_http_request(
            "POST /v1/release-gates/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let rejected = handle_http_request(
            "POST /v1/dogfood/telemetry-review/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let review_body: serde_json::Value =
            serde_json::from_str(&review.body).expect("dogfood review json");
        let auth_review_body: serde_json::Value =
            serde_json::from_str(&auth_review.body).expect("auth dogfood review json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let auth_listed_body: serde_json::Value =
            serde_json::from_str(&auth_listed.body).expect("auth listed json");
        let release_body: serde_json::Value =
            serde_json::from_str(&release_gate.body).expect("release gate json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let rg09 = release_body["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|scenario| scenario["gate_id"] == "RG-09")
            .expect("RG-09 scenario exists");

        assert_eq!(review.status_code, 201);
        assert_eq!(review_body["id"], "dogfood_review_1");
        assert_eq!(auth_review.status_code, 201);
        assert_eq!(auth_review_body["id"], "dogfood_review_2");
        assert_eq!(review_body["status"], "passed");
        assert_eq!(review_body["evidence_pack_id"], "ev_dogfood_review_1");
        assert_eq!(
            review_body["findings"][0]["finding_id"],
            "telemetry_command_blocks"
        );
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["id"], "dogfood_review_1");
        assert_eq!(listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_listed.status_code, 200);
        assert_eq!(auth_listed_body["items"][0]["id"], "dogfood_review_2");
        assert_eq!(auth_listed_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(state_body["dogfood"]["last_review_id"], "dogfood_review_1");
        assert_eq!(
            state_body["dogfood"]["last_evidence_pack_id"],
            "ev_dogfood_review_1"
        );
        assert_eq!(rg09["status"], "pass");
        assert_eq!(rg09["evidence"], serde_json::json!(["ev_dogfood_review_1"]));
        assert_eq!(rejected.status_code, 400);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn builds_visual_harness_graph_and_persists_manual_links() {
        let db_path = unique_test_db_path("control-visual-harness");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        handle_http_request(
            "POST /v1/workflow/reload HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sourcePath\":\"/repo/WORKFLOW.md\",\"content\":\"---\\nhaneulchi: 1\\nproject:\\n  key: AUTH\\nworkspace:\\n  strategy: worktree\\ncontext:\\n  default_pack: ctx_default\\nhooks:\\n  before_run: .haneulchi/hooks/before_run.sh\\ntools:\\n  browser: localhost preview\\n  test_runner: cargo test\\n---\\nUse {task.id}.\\n\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"title\":\"Visual harness task\",\"priority\":\"high\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_task = handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"title\":\"Auth visual task\",\"priority\":\"medium\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/initiatives HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Visual runway goal\",\"description\":\"Connect graph runway evidence\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/tasks/task_1/planning HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"initiativeId\":\"init_1\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/policy/packs HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Visual safety\",\"sandboxMode\":\"ask-before-write\",\"network\":\"ask\",\"networkProfile\":\"internet\",\"fileWrite\":\"ask\",\"tools\":\"ask\",\"approvalRequired\":[\"shell_command\"],\"forbiddenOperations\":[],\"setActive\":true}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/tasks/task_1/move HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"status\":\"ready\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        handle_http_request(
            "POST /v1/dispatch HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"taskId\":\"task_1\",\"agentProfileId\":\"agent_codex\",\"contextPackId\":\"ctx_default\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let evidence = handle_http_request(
            "POST /v1/runs/run_1/evidence/generate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let link = handle_http_request(
            "POST /v1/visual-harness/links HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sourceId\":\"ctx_default\",\"targetId\":\"task_1\",\"kind\":\"context\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let workflow_link = handle_http_request(
            "POST /v1/visual-harness/links HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sourceId\":\"workflow_1\",\"targetId\":\"task_1\",\"kind\":\"workflow\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_link = handle_http_request(
            "POST /v1/visual-harness/links HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"sourceId\":\"ctx_auth\",\"targetId\":\"task_2\",\"kind\":\"context\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/visual-harness/links HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_listed = handle_http_request(
            "GET /v1/visual-harness/links?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let graph = handle_http_request(
            "GET /v1/visual-harness/graph HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_graph = handle_http_request(
            "GET /v1/visual-harness/graph?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let rejected = handle_http_request(
            "POST /v1/visual-harness/links HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sourceId\":\"\",\"targetId\":\"task_1\",\"kind\":\"context\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let link_body: serde_json::Value =
            serde_json::from_str(&link.body).expect("visual link json");
        let auth_task_body: serde_json::Value =
            serde_json::from_str(&auth_task.body).expect("auth task json");
        let auth_link_body: serde_json::Value =
            serde_json::from_str(&auth_link.body).expect("auth visual link json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed json");
        let auth_listed_body: serde_json::Value =
            serde_json::from_str(&auth_listed.body).expect("auth listed json");
        let graph_body: serde_json::Value =
            serde_json::from_str(&graph.body).expect("visual graph json");
        let auth_graph_body: serde_json::Value =
            serde_json::from_str(&auth_graph.body).expect("auth visual graph json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let nodes = state_body["visual_harness"]["nodes"].as_array().unwrap();
        let edges = state_body["visual_harness"]["edges"].as_array().unwrap();

        assert_eq!(evidence.status_code, 201);
        assert_eq!(auth_task.status_code, 201);
        assert_eq!(auth_task_body["id"], "task_2");
        assert_eq!(link.status_code, 201);
        assert_eq!(link_body["id"], "visual_link_1");
        assert_eq!(workflow_link.status_code, 201);
        assert_eq!(auth_link.status_code, 201);
        assert_eq!(auth_link_body["id"], "visual_link_3");
        assert_eq!(listed.status_code, 200);
        assert_eq!(auth_listed.status_code, 200);
        assert_eq!(graph.status_code, 200);
        assert_eq!(auth_graph.status_code, 200);
        let listed_items = listed_body["items"].as_array().unwrap();
        let auth_listed_items = auth_listed_body["items"].as_array().unwrap();
        assert!(listed_items
            .iter()
            .any(|item| item["id"] == "visual_link_1"));
        assert!(listed_items.iter().any(|item| item["kind"] == "workflow"));
        assert_eq!(listed_items.len(), 2);
        assert_eq!(auth_listed_items[0]["id"], "visual_link_3");
        assert_eq!(auth_listed_items.len(), 1);
        assert_eq!(graph_body["diagnostics"]["status"], "ok");
        assert!(graph_body["nodes"]
            .as_array()
            .expect("graph nodes")
            .iter()
            .any(|node| node["id"] == "workflow_1" && node["kind"] == "workflow"));
        assert!(auth_graph_body["nodes"]
            .as_array()
            .expect("auth graph nodes")
            .iter()
            .any(|node| node["id"] == "task_2" && node["kind"] == "task"));
        assert!(!auth_graph_body["nodes"]
            .as_array()
            .expect("auth graph nodes")
            .iter()
            .any(|node| node["id"] == "task_1" && node["kind"] == "task"));
        assert!(graph_body["edges"]
            .as_array()
            .expect("graph edges")
            .iter()
            .any(|edge| edge["id"] == "visual_link_2" && edge["kind"] == "workflow"));
        assert!(auth_graph_body["edges"]
            .as_array()
            .expect("auth graph edges")
            .iter()
            .any(|edge| edge["id"] == "visual_link_3" && edge["kind"] == "context"));
        assert!(nodes
            .iter()
            .any(|node| node["id"] == "workflow_1" && node["kind"] == "workflow"));
        assert!(nodes
            .iter()
            .any(|node| node["id"] == "hook_before_run" && node["kind"] == "hook"));
        assert!(edges.iter().any(|edge| edge["source_id"] == "workflow_1"
            && edge["target_id"] == "hook_before_run"
            && edge["kind"] == "trigger"));
        assert!(nodes
            .iter()
            .any(|node| node["id"] == "tool_browser" && node["kind"] == "tool"));
        assert!(nodes
            .iter()
            .any(|node| node["id"] == "tool_test_runner" && node["label"] == "cargo test"));
        assert!(edges.iter().any(|edge| edge["source_id"] == "workflow_1"
            && edge["target_id"] == "tool_browser"
            && edge["kind"] == "tool"));
        assert!(nodes.iter().any(|node| node["id"] == "task_1"));
        assert!(nodes
            .iter()
            .any(|node| node["id"] == "init_1" && node["kind"] == "goal"));
        assert!(edges.iter().any(|edge| edge["source_id"] == "init_1"
            && edge["target_id"] == "task_1"
            && edge["kind"] == "goal"));
        assert!(nodes
            .iter()
            .any(|node| node["id"] == "policy_pack_proj_local_Visual_safety"
                && node["kind"] == "policy_pack"));
        assert!(edges.iter().any(|edge| edge["source_id"]
            == "policy_pack_proj_local_Visual_safety"
            && edge["target_id"] == "workflow_1"
            && edge["kind"] == "policy"));
        assert!(nodes.iter().any(|node| node["id"] == "run_1"));
        assert!(nodes
            .iter()
            .any(|node| node["id"] == "review_ev_run_1" && node["kind"] == "review_gate"));
        assert!(edges
            .iter()
            .any(|edge| edge["source_id"] == "task_1" && edge["target_id"] == "run_1"));
        assert!(edges.iter().any(|edge| edge["source_id"] == "run_1"
            && edge["target_id"] == "review_ev_run_1"
            && edge["kind"] == "review_gate"));
        assert!(edges.iter().any(|edge| edge["id"] == "visual_link_1"));
        assert_eq!(rejected.status_code, 400);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_persisted_command_block_search_and_state_summaries() {
        let db_path = unique_test_db_path("control-command-blocks");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_1".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_frontend".to_string()),
                run_id: None,
                seq_start: Some(4),
                seq_end: Some(9),
                command: "npm test".to_string(),
                cwd: Some("/repo/frontend".to_string()),
                branch: Some("feature/command-search".to_string()),
                exit_code: Some(0),
                duration_ms: Some(1200),
                summary: Some("PASS frontend diagnostics".to_string()),
            })
            .expect("command block persists");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_2".to_string(),
                session_id: "pty_2".to_string(),
                task_id: Some("task_backend".to_string()),
                run_id: None,
                seq_start: Some(10),
                seq_end: Some(14),
                command: "npm test".to_string(),
                cwd: Some("/repo/backend".to_string()),
                branch: Some("feature/command-search".to_string()),
                exit_code: Some(0),
                duration_ms: Some(900),
                summary: Some("PASS frontend diagnostics".to_string()),
            })
            .expect("second command block persists");

        let search = handle_http_request(
            "GET /v1/command-blocks?query=frontend&taskId=task_frontend HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let session_search = handle_http_request(
            "GET /v1/command-blocks?query=frontend&sessionId=pty_1 HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let search_body: serde_json::Value =
            serde_json::from_str(&search.body).expect("search json");
        let session_search_body: serde_json::Value =
            serde_json::from_str(&session_search.body).expect("session search json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(search.status_code, 200);
        assert_eq!(search_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(search_body["items"][0]["id"], "cmdblk_1");
        assert_eq!(search_body["items"][0]["task_id"], "task_frontend");
        assert_eq!(
            search_body["items"][0]["summary"],
            "PASS frontend diagnostics"
        );
        assert_eq!(session_search.status_code, 200);
        assert_eq!(session_search_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(session_search_body["items"][0]["id"], "cmdblk_1");
        assert_eq!(session_search_body["items"][0]["session_id"], "pty_1");
        assert_eq!(state_body["command_blocks"]["unread_count"], 2);
        assert_eq!(
            state_body["command_blocks"]["recent"][0]["command"],
            "npm test"
        );
        assert_eq!(
            state_body["command_blocks"]["recent"][0]["status"],
            "completed"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_single_command_block_for_cli_show() {
        let db_path = unique_test_db_path("control-command-block-show");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_show".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_review".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(12),
                seq_end: Some(16),
                command: "cargo test".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(2400),
                summary: Some("PASS Rust tests".to_string()),
            })
            .expect("command block persists");

        let response = handle_http_request(
            "GET /v1/command-blocks/cmdblk_show HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value =
            serde_json::from_str(&response.body).expect("command block json");

        assert_eq!(response.status_code, 200);
        assert_eq!(body["id"], "cmdblk_show");
        assert_eq!(body["command"], "cargo test");
        assert_eq!(body["summary"], "PASS Rust tests");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn mutates_command_block_mark_merge_and_split_through_control_api() {
        let db_path = unique_test_db_path("control-command-block-actions");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_1".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_review".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(1),
                seq_end: Some(4),
                command: "npm test".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: None,
                duration_ms: Some(100),
                summary: Some("PASS frontend tests".to_string()),
            })
            .expect("first command block persists");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_2".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_review".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(5),
                seq_end: Some(8),
                command: "cargo test".to_string(),
                cwd: Some("/repo/src-tauri".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(200),
                summary: Some("PASS rust tests".to_string()),
            })
            .expect("second command block persists");

        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };
        let marked = handle_http_request(
            "POST /v1/command-blocks/cmdblk_1/mark HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"status\":\"completed\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let merged = handle_http_request(
            "POST /v1/command-blocks/cmdblk_1/merge HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"secondCommandBlockId\":\"cmdblk_2\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let split = handle_http_request(
            "POST /v1/command-blocks/cmdblk_1/split HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            pty,
            "0.1.0-test",
        );

        let marked_body: serde_json::Value =
            serde_json::from_str(&marked.body).expect("marked json");
        let merged_body: serde_json::Value =
            serde_json::from_str(&merged.body).expect("merged json");
        let split_body: serde_json::Value = serde_json::from_str(&split.body).expect("split json");

        assert_eq!(marked.status_code, 200);
        assert_eq!(marked_body["exit_code"], 0);
        assert!(marked_body["snapshot_id"].as_str().is_some());
        assert_eq!(merged.status_code, 200);
        assert_eq!(merged_body["command"], "npm test && cargo test");
        assert_eq!(merged_body["seq_start"], 1);
        assert_eq!(merged_body["seq_end"], 8);
        assert_eq!(split.status_code, 200);
        assert_eq!(
            split_body["updated_block"]["command"],
            "npm test && cargo test (part 1)"
        );
        assert_eq!(
            split_body["created_block"]["command"],
            "npm test && cargo test (part 2)"
        );
        assert!(split_body["snapshot_id"].as_str().is_some());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn explains_and_exports_command_block_bundle_through_control_api_with_redaction() {
        let db_path = unique_test_db_path("control-command-block-explain");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_secret(crate::state_store::UpsertSecretInput {
                project_id: "proj_local".to_string(),
                name: "OPENAI_API_KEY".to_string(),
                value: "haneulchi-secret-fixture-value".to_string(),
            })
            .expect("secret stored");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_explain".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_review".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(12),
                seq_end: Some(16),
                command: "OPENAI_API_KEY=haneulchi-secret-fixture-value npm test".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(2400),
                summary: Some("PASS with haneulchi-secret-fixture-value".to_string()),
            })
            .expect("command block persists");

        let explain = handle_http_request(
            "POST /v1/command-blocks/cmdblk_explain/explain HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"provider\":\"openai\",\"model\":\"gpt-5.4\",\"agentProfileId\":\"agent_codex\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let bundle = handle_http_request(
            "GET /v1/command-blocks/cmdblk_explain/bundle HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let missing = handle_http_request(
            "POST /v1/command-blocks/missing/explain HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let explain_body: serde_json::Value =
            serde_json::from_str(&explain.body).expect("command block explanation json");
        let bundle_body: serde_json::Value =
            serde_json::from_str(&bundle.body).expect("command block bundle json");

        assert_eq!(explain.status_code, 200);
        assert_eq!(explain_body["id"], "explain_cmdblk_explain");
        assert_eq!(explain_body["command_block_id"], "cmdblk_explain");
        assert_eq!(explain_body["provider"], "openai");
        assert_eq!(explain_body["model"], "gpt-5.4");
        assert_eq!(explain_body["agent_profile_id"], "agent_codex");
        assert!(explain_body["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item
                == "context: OPENAI_API_KEY=[REDACTED:OPENAI_API_KEY] npm test completed in /repo on main"));
        assert!(explain_body["prompt"]
            .as_str()
            .unwrap()
            .contains("[REDACTED:OPENAI_API_KEY]"));
        assert!(!explain.body.contains("haneulchi-secret-fixture-value"));

        assert_eq!(bundle.status_code, 200);
        assert_eq!(bundle_body["kind"], "haneulchi.command_block_bundle");
        assert_eq!(bundle_body["version"], 1);
        assert_eq!(bundle_body["command_block"]["id"], "cmdblk_explain");
        assert_eq!(
            bundle_body["explanation"]["command_block_id"],
            "cmdblk_explain"
        );
        assert!(!bundle.body.contains("haneulchi-secret-fixture-value"));
        assert_eq!(missing.status_code, 400);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn attaches_command_blocks_to_evidence_through_control_api() {
        let db_path = unique_test_db_path("control-evidence");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_1".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_review".to_string()),
                run_id: None,
                seq_start: Some(4),
                seq_end: Some(9),
                command: "npm test".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: None,
                summary: Some("PASS evidence API".to_string()),
            })
            .expect("command block persists");

        let response = handle_http_request(
            "POST /v1/command-blocks/cmdblk_1/attach-evidence HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 54\r\n\r\n{\"evidencePackId\":\"ev_local\",\"taskId\":\"task_review\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&response.body).expect("evidence json");

        assert_eq!(response.status_code, 201);
        assert_eq!(body["id"], "ev_local");
        assert_eq!(body["body_json"]["command_blocks"][0]["id"], "cmdblk_1");
        assert_eq!(
            body["body_json"]["command_blocks"][0]["summary"],
            "PASS evidence API"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn generates_evidence_and_records_review_decisions_through_control_api() {
        let db_path = unique_test_db_path("control-review-evidence");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_control_review".to_string(),
                key: "HC-CONTROL-REVIEW".to_string(),
                project_id: "proj_local".to_string(),
                title: "Control review".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        store
            .dispatch_run(crate::state_store::DispatchRunInput {
                task_id: "task_control_review".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: Some("ctx_default".to_string()),
                workspace_path: None,
            })
            .expect("run dispatches");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_incomplete_review".to_string(),
                key: "HC-INCOMPLETE-REVIEW".to_string(),
                project_id: "proj_local".to_string(),
                title: "Incomplete review evidence".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "medium".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("incomplete task persists");
        store
            .dispatch_run(crate::state_store::DispatchRunInput {
                task_id: "task_incomplete_review".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: Some("ctx_default".to_string()),
                workspace_path: None,
            })
            .expect("incomplete run dispatches");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_auth_review".to_string(),
                key: "AUTH-REVIEW".to_string(),
                project_id: "proj_auth".to_string(),
                title: "Auth project review".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("auth task persists");
        store
            .dispatch_run(crate::state_store::DispatchRunInput {
                task_id: "task_auth_review".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: Some("ctx_auth".to_string()),
                workspace_path: None,
            })
            .expect("auth run dispatches");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_control_review".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_control_review".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(1),
                seq_end: Some(6),
                command: "npm test".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(1300),
                summary: Some("control tests passed".to_string()),
            })
            .expect("command block persists");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_control_diff".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_control_review".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(7),
                seq_end: Some(10),
                command: "git diff --stat".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(180),
                summary: Some("2 files changed, 14 insertions(+), 3 deletions(-)".to_string()),
            })
            .expect("diff command block persists");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_auth_review".to_string(),
                session_id: "pty_auth".to_string(),
                task_id: Some("task_auth_review".to_string()),
                run_id: Some("run_3".to_string()),
                seq_start: Some(1),
                seq_end: Some(3),
                command: "cargo test auth".to_string(),
                cwd: Some("/repo/auth".to_string()),
                branch: Some("auth".to_string()),
                exit_code: Some(0),
                duration_ms: Some(900),
                summary: Some("auth review tests passed".to_string()),
            })
            .expect("auth command block persists");
        store
            .record_token_usage(crate::state_store::TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: Some("session_1".to_string()),
                task_id: Some("task_control_review".to_string()),
                run_id: Some("run_1".to_string()),
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 1200,
                output_tokens: 800,
                cost_usd: 8.5,
                source: "adapter:openai.responses".to_string(),
            })
            .expect("usage records");

        let generated = handle_http_request(
            "POST /v1/runs/run_1/evidence/generate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_generated = handle_http_request(
            "POST /v1/runs/run_3/evidence/generate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"evidencePackId\":\"ev_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let incomplete_generated = handle_http_request(
            "POST /v1/runs/run_2/evidence/generate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let reviews = handle_http_request(
            "GET /v1/reviews HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_reviews = handle_http_request(
            "GET /v1/reviews?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let pending_reviews = handle_http_request(
            "GET /v1/reviews?state=pending HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let incomplete_reviews = handle_http_request(
            "GET /v1/reviews?completeness=incomplete HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let complete_pending_reviews = handle_http_request(
            "GET /v1/reviews?state=pending&completeness=complete HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let approved = handle_http_request(
            "POST /v1/reviews/review_ev_run_1/decision HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"decision\":\"approved\",\"reviewerId\":\"human\",\"bodyMd\":\"Looks complete.\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let generated_body: serde_json::Value =
            serde_json::from_str(&generated.body).expect("generated evidence json");
        let auth_generated_body: serde_json::Value =
            serde_json::from_str(&auth_generated.body).expect("auth generated evidence json");
        let incomplete_generated_body: serde_json::Value =
            serde_json::from_str(&incomplete_generated.body)
                .expect("incomplete generated evidence json");
        let reviews_body: serde_json::Value =
            serde_json::from_str(&reviews.body).expect("reviews json");
        let auth_reviews_body: serde_json::Value =
            serde_json::from_str(&auth_reviews.body).expect("auth reviews json");
        let pending_reviews_body: serde_json::Value =
            serde_json::from_str(&pending_reviews.body).expect("pending reviews json");
        let incomplete_reviews_body: serde_json::Value =
            serde_json::from_str(&incomplete_reviews.body).expect("incomplete reviews json");
        let complete_pending_reviews_body: serde_json::Value =
            serde_json::from_str(&complete_pending_reviews.body)
                .expect("complete pending reviews json");
        let approved_body: serde_json::Value =
            serde_json::from_str(&approved.body).expect("approved evidence json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(generated.status_code, 201);
        assert_eq!(generated_body["id"], "ev_run_1");
        assert_eq!(auth_generated.status_code, 201);
        assert_eq!(auth_generated_body["id"], "ev_auth");
        assert_eq!(incomplete_generated.status_code, 201);
        assert_eq!(incomplete_generated_body["id"], "ev_run_2");
        assert_eq!(
            incomplete_generated_body["completeness_state"],
            "incomplete"
        );
        assert_eq!(reviews.status_code, 200);
        assert_eq!(reviews_body["items"].as_array().unwrap().len(), 2);
        let local_review_items = reviews_body["items"]
            .as_array()
            .expect("local review items");
        let local_pending_review = local_review_items
            .iter()
            .find(|review| review["id"] == "review_ev_run_1")
            .expect("pending review in local list");
        let local_incomplete_review = local_review_items
            .iter()
            .find(|review| review["id"] == "review_ev_run_2")
            .expect("incomplete review in local list");
        assert_eq!(local_pending_review["evidence_pack_id"], "ev_run_1");
        assert_eq!(local_pending_review["state"], "pending");
        assert_eq!(local_incomplete_review["completeness_state"], "incomplete");
        assert_eq!(auth_reviews.status_code, 200);
        assert_eq!(auth_reviews_body["items"][0]["id"], "review_ev_auth");
        assert_eq!(auth_reviews_body["items"][0]["evidence_pack_id"], "ev_auth");
        assert_eq!(auth_reviews_body["items"][0]["run_id"], "run_3");
        assert_eq!(auth_reviews_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(pending_reviews.status_code, 200);
        assert_eq!(pending_reviews_body["items"][0]["id"], "review_ev_run_1");
        assert_eq!(pending_reviews_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(incomplete_reviews.status_code, 200);
        assert_eq!(incomplete_reviews_body["items"][0]["id"], "review_ev_run_2");
        assert_eq!(
            incomplete_reviews_body["items"][0]["completeness_state"],
            "incomplete"
        );
        assert_eq!(
            incomplete_reviews_body["items"].as_array().unwrap().len(),
            1
        );
        assert_eq!(complete_pending_reviews.status_code, 200);
        assert_eq!(
            complete_pending_reviews_body["items"][0]["id"],
            "review_ev_run_1"
        );
        assert_eq!(
            complete_pending_reviews_body["items"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            generated_body["body_json"]["tests"][0]["command_block_id"],
            "cmdblk_control_review"
        );
        assert_eq!(
            generated_body["body_json"]["diff_summary"]["summary"],
            "2 files changed, 14 insertions(+), 3 deletions(-)"
        );
        assert_eq!(
            generated_body["body_json"]["token_usage"]["total_tokens"],
            2000
        );
        assert_eq!(generated_body["body_json"]["token_usage"]["cost_usd"], 8.5);
        assert_eq!(approved.status_code, 200);
        assert_eq!(
            approved_body["body_json"]["review_decision"]["decision"],
            "approved"
        );
        assert_eq!(
            store
                .get_task("task_control_review")
                .expect("task loads")
                .expect("task exists")
                .status,
            "done"
        );
        assert_eq!(state_body["reviews"][0]["id"], "review_ev_run_1");
        assert_eq!(state_body["reviews"][0]["evidence_pack_id"], "ev_run_1");
        assert_eq!(state_body["reviews"][0]["task_id"], "task_control_review");
        assert_eq!(state_body["reviews"][0]["run_id"], "run_1");
        assert_eq!(state_body["reviews"][0]["state"], "approved");
        assert_eq!(state_body["reviews"][0]["completeness_state"], "complete");
        assert_eq!(
            state_body["reviews"][0]["diff_summary"]["summary"],
            "2 files changed, 14 insertions(+), 3 deletions(-)"
        );
        assert_eq!(
            state_body["reviews"][0]["token_usage"]["total_tokens"],
            2000
        );
        assert_eq!(state_body["reviews"][0]["token_usage"]["cost_usd"], 8.5);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn creates_review_follow_up_task_through_control_api() {
        let db_path = unique_test_db_path("control-review-followup-task");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_control_followup".to_string(),
                key: "HC-FOLLOWUP".to_string(),
                project_id: "proj_local".to_string(),
                title: "Review follow-up source".to_string(),
                description: None,
                status: "review".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_control_followup".to_string(),
                session_id: "pty_control_followup".to_string(),
                task_id: Some("task_control_followup".to_string()),
                run_id: Some("run_control_followup".to_string()),
                seq_start: Some(1),
                seq_end: Some(2),
                command: "cargo test".to_string(),
                cwd: None,
                branch: None,
                exit_code: Some(0),
                duration_ms: Some(1100),
                summary: Some("tests passed".to_string()),
            })
            .expect("command block persisted");
        store
            .attach_command_block_to_evidence(crate::state_store::AttachCommandBlockEvidenceInput {
                evidence_pack_id: "ev_control_followup".to_string(),
                command_block_id: "cmdblk_control_followup".to_string(),
                task_id: Some("task_control_followup".to_string()),
                run_id: Some("run_control_followup".to_string()),
            })
            .expect("evidence attached");

        let response = handle_http_request(
            "POST /v1/reviews/review_ev_control_followup/follow-up-task HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"title\":\"Address control review gap\",\"priority\":\"urgent\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value =
            serde_json::from_str(&response.body).expect("follow-up receipt json");

        assert_eq!(response.status_code, 201);
        assert_eq!(body["review_id"], "review_ev_control_followup");
        assert_eq!(body["evidence_pack_id"], "ev_control_followup");
        assert_eq!(body["source_task_id"], "task_control_followup");
        assert_eq!(body["source_run_id"], "run_control_followup");
        assert_eq!(body["task"]["project_id"], "proj_local");
        assert_eq!(body["task"]["title"], "Address control review gap");
        assert_eq!(body["task"]["priority"], "urgent");
        assert!(body["comment"]["body_md"].as_str().is_some_and(|body| body
            .contains("review_ev_control_followup")
            && body.contains("ev_control_followup")
            && body.contains("task_control_followup")
            && body.contains("run_control_followup")));
        assert!(body["snapshot_id"]
            .as_str()
            .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn plans_review_pr_landing_through_control_api() {
        let db_path = unique_test_db_path("control-review-pr-landing");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(crate::state_store::AddProjectInput {
                key: "LOCAL".to_string(),
                name: "Local Project".to_string(),
                path: "/repo/local-review-pr".to_string(),
                color: None,
            })
            .expect("project persisted");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_control_review_pr".to_string(),
                key: "HC-PR".to_string(),
                project_id: "proj_local".to_string(),
                title: "Review PR source".to_string(),
                description: None,
                status: "review".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_control_review_pr".to_string(),
                session_id: "pty_control_review_pr".to_string(),
                task_id: Some("task_control_review_pr".to_string()),
                run_id: Some("run_control_review_pr".to_string()),
                seq_start: Some(1),
                seq_end: Some(2),
                command: "npm test".to_string(),
                cwd: None,
                branch: None,
                exit_code: Some(0),
                duration_ms: Some(1000),
                summary: Some("tests passed".to_string()),
            })
            .expect("command block persisted");
        store
            .attach_command_block_to_evidence(crate::state_store::AttachCommandBlockEvidenceInput {
                evidence_pack_id: "ev_control_review_pr".to_string(),
                command_block_id: "cmdblk_control_review_pr".to_string(),
                task_id: Some("task_control_review_pr".to_string()),
                run_id: Some("run_control_review_pr".to_string()),
            })
            .expect("evidence attached");

        let response = handle_http_request(
            "POST /v1/reviews/review_ev_control_review_pr/pr/landing-plan HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"title\":\"Ship control review PR\",\"draft\":true}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value =
            serde_json::from_str(&response.body).expect("review PR plan receipt json");

        assert_eq!(response.status_code, 200);
        assert_eq!(body["review_id"], "review_ev_control_review_pr");
        assert_eq!(body["evidence_pack_id"], "ev_control_review_pr");
        assert_eq!(body["source_task_id"], "task_control_review_pr");
        assert_eq!(body["source_run_id"], "run_control_review_pr");
        assert_eq!(body["plan"]["project_id"], "proj_local");
        assert_eq!(body["plan"]["title"], "Ship control review PR");
        assert_eq!(body["plan"]["draft"], true);
        assert!(body["plan"]["checklist"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item
                .as_str()
                .is_some_and(|item| item.contains("review_ev_control_review_pr")
                    && item.contains("ev_control_review_pr")))));
        assert!(body["snapshot_id"]
            .as_str()
            .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn dispatches_runs_and_lists_them_through_control_api_and_state() {
        let db_path = unique_test_db_path("control-runs");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_ready".to_string(),
                key: "HC-READY".to_string(),
                project_id: "proj_local".to_string(),
                title: "Dispatch control API run".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_auth_ready".to_string(),
                key: "AUTH-READY".to_string(),
                project_id: "proj_auth".to_string(),
                title: "Dispatch auth project run".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "medium".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("auth task persists");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_local_review".to_string(),
                key: "HC-REVIEW".to_string(),
                project_id: "proj_local".to_string(),
                title: "Review local project run".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "medium".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("local review task persists");

        let dispatched = handle_http_request(
            "POST /v1/dispatch HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 83\r\n\r\n{\"taskId\":\"task_ready\",\"agentProfileId\":\"agent_codex\",\"contextPackId\":\"ctx_default\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_dispatched = handle_http_request(
            "POST /v1/dispatch HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"taskId\":\"task_auth_ready\",\"agentProfileId\":\"agent_codex\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let local_review_dispatched = handle_http_request(
            "POST /v1/dispatch HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"taskId\":\"task_local_review\",\"agentProfileId\":\"agent_claude\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let local_review_ready = handle_http_request(
            "POST /v1/runs/run_3/transition HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"lifecycle\":\"review_ready\",\"statusDetail\":\"Evidence pack ready\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let runs = handle_http_request(
            "GET /v1/runs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_runs = handle_http_request(
            "GET /v1/runs?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let review_ready_runs = handle_http_request(
            "GET /v1/runs?projectId=proj_local&lifecycle=review_ready HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let task_runs = handle_http_request(
            "GET /v1/runs?projectId=proj_local&taskId=task_ready HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let agent_runs = handle_http_request(
            "GET /v1/runs?projectId=proj_local&agentProfileId=agent_claude HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let dispatched_body: serde_json::Value =
            serde_json::from_str(&dispatched.body).expect("dispatch json");
        let auth_dispatched_body: serde_json::Value =
            serde_json::from_str(&auth_dispatched.body).expect("auth dispatch json");
        let local_review_dispatched_body: serde_json::Value =
            serde_json::from_str(&local_review_dispatched.body)
                .expect("local review dispatch json");
        let local_review_ready_body: serde_json::Value =
            serde_json::from_str(&local_review_ready.body).expect("local review ready json");
        let runs_body: serde_json::Value = serde_json::from_str(&runs.body).expect("runs json");
        let auth_runs_body: serde_json::Value =
            serde_json::from_str(&auth_runs.body).expect("auth runs json");
        let review_ready_runs_body: serde_json::Value =
            serde_json::from_str(&review_ready_runs.body).expect("review-ready runs json");
        let task_runs_body: serde_json::Value =
            serde_json::from_str(&task_runs.body).expect("task runs json");
        let agent_runs_body: serde_json::Value =
            serde_json::from_str(&agent_runs.body).expect("agent runs json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(dispatched.status_code, 201);
        assert_eq!(dispatched_body["id"], "run_1");
        assert_eq!(dispatched_body["lifecycle"], "queued");
        assert_eq!(auth_dispatched.status_code, 201);
        assert_eq!(auth_dispatched_body["id"], "run_2");
        assert_eq!(local_review_dispatched.status_code, 201);
        assert_eq!(local_review_dispatched_body["id"], "run_3");
        assert_eq!(local_review_ready.status_code, 200);
        assert_eq!(local_review_ready_body["lifecycle"], "review_ready");
        assert_eq!(runs_body["items"][0]["task_id"], "task_ready");
        assert_eq!(runs_body["items"].as_array().unwrap().len(), 2);
        assert_eq!(auth_runs.status_code, 200);
        assert_eq!(auth_runs_body["items"][0]["task_id"], "task_auth_ready");
        assert_eq!(auth_runs_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(review_ready_runs.status_code, 200);
        assert_eq!(
            review_ready_runs_body["items"][0]["task_id"],
            "task_local_review"
        );
        assert_eq!(
            review_ready_runs_body["items"][0]["lifecycle"],
            "review_ready"
        );
        assert_eq!(review_ready_runs_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(task_runs.status_code, 200);
        assert_eq!(task_runs_body["items"][0]["id"], "run_1");
        assert_eq!(task_runs_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(agent_runs.status_code, 200);
        assert_eq!(agent_runs_body["items"][0]["id"], "run_3");
        assert_eq!(agent_runs_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(
            state_body["runs"]["items"][0]["workspace_path"],
            dispatched_body["workspace_path"]
        );
        assert!(state_body["runs"]["items"][0]["workspace_path"]
            .as_str()
            .unwrap_or("")
            .ends_with(".haneulchi/worktrees/run_1"));
        assert_eq!(state_body["runs"]["items"][0]["id"], "run_1");
        assert_eq!(state_body["runs"]["counts_by_lifecycle"]["queued"], 1);
        assert_eq!(state_body["runs"]["counts_by_lifecycle"]["review_ready"], 1);
        let state_tasks = state_body["tasks"]["items"]
            .as_array()
            .expect("state task items");
        let state_task_ready = state_tasks
            .iter()
            .find(|task| task["id"] == "task_ready")
            .expect("task_ready in state");
        let state_task_review = state_tasks
            .iter()
            .find(|task| task["id"] == "task_local_review")
            .expect("task_local_review in state");
        assert_eq!(state_task_ready["status"], "running");
        assert_eq!(state_task_review["status"], "review");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn blocks_dispatch_through_control_api_when_budget_hard_limit_is_exceeded() {
        let db_path = unique_test_db_path("control-budget-dispatch-gate");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_budget_gate".to_string(),
                key: "HC-BUDGET".to_string(),
                project_id: "proj_local".to_string(),
                title: "Budget gated dispatch".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        store
            .upsert_budget(crate::state_store::UpsertBudgetInput {
                scope_type: "project".to_string(),
                scope_id: Some("proj_local".to_string()),
                max_usd: 5.0,
                warn_pct: 0.8,
                hard_limit: true,
            })
            .expect("budget persists");
        store
            .record_token_usage(crate::state_store::TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: None,
                task_id: None,
                run_id: None,
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 1000,
                output_tokens: 1000,
                cost_usd: 5.1,
                source: "adapter".to_string(),
            })
            .expect("usage records");

        let blocked = handle_http_request(
            "POST /v1/dispatch HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"taskId\":\"task_budget_gate\",\"agentProfileId\":\"agent_codex\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let blocked_body: serde_json::Value =
            serde_json::from_str(&blocked.body).expect("blocked json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(blocked.status_code, 400);
        assert_eq!(
            blocked_body["error"],
            "project budget exceeded: used $5.10 of $5.00"
        );
        assert_eq!(
            state_body["runs"]["items"].as_array().expect("runs").len(),
            0
        );
        assert_eq!(state_body["tasks"]["items"][0]["status"], "ready");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn manages_policy_approvals_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-policy-approval");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_policy_api".to_string(),
                key: "HC-POLICY-API".to_string(),
                project_id: "proj_local".to_string(),
                title: "Policy API gate".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        store
            .dispatch_run(crate::state_store::DispatchRunInput {
                task_id: "task_policy_api".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");
        store
            .update_run_lifecycle(crate::state_store::UpdateRunLifecycleInput {
                run_id: "run_1".to_string(),
                lifecycle: "running".to_string(),
                status_detail: None,
            })
            .expect("run starts");

        let request = handle_http_request(
            "POST /v1/policy/approvals HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"taskId\":\"task_policy_api\",\"runId\":\"run_1\",\"actionKind\":\"shell_command\",\"command\":\"rm -rf build/cache\",\"riskLevel\":\"high\",\"requestedBy\":\"agent_codex\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let list = handle_http_request(
            "GET /v1/policy/approvals?state=pending HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let decision = handle_http_request(
            "POST /v1/policy/approvals/policy_approval_1/decision HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"decision\":\"denied\",\"decisionBy\":\"human\",\"decisionNote\":\"Too broad.\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let request_body: serde_json::Value =
            serde_json::from_str(&request.body).expect("request json");
        let list_body: serde_json::Value = serde_json::from_str(&list.body).expect("list json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let decision_body: serde_json::Value =
            serde_json::from_str(&decision.body).expect("decision json");

        assert_eq!(request.status_code, 201);
        assert_eq!(request_body["state"], "pending");
        assert_eq!(list.status_code, 200);
        assert_eq!(list_body["items"][0]["id"], "policy_approval_1");
        assert_eq!(
            state_body["attention"][0]["label"],
            "Policy approval required: shell_command"
        );
        assert_eq!(
            state_body["security"]["diagnostics"]["pending_policy_approvals"],
            1
        );
        assert_eq!(
            state_body["security"]["diagnostics"]["checks"][3]["status"],
            "warning"
        );
        assert_eq!(
            state_body["runs"]["counts_by_lifecycle"]["permission_requested"],
            1
        );
        assert_eq!(decision.status_code, 200);
        assert_eq!(decision_body["state"], "denied");
        assert_eq!(
            store
                .get_run("run_1")
                .expect("run loads")
                .expect("run exists")
                .lifecycle,
            "blocked"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn manages_policy_packs_and_evaluates_actions_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-policy-pack");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let upsert = handle_http_request(
            "POST /v1/policy/packs HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Ask before write\",\"sandboxMode\":\"ask-before-write\",\"network\":\"blocked\",\"networkProfile\":\"local-only\",\"fileWrite\":\"ask\",\"approvalRequired\":[\"shell_command\",\"file_write\"],\"forbiddenOperations\":[\"network\"],\"setActive\":true}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let inactive_upsert = handle_http_request(
            "POST /v1/policy/packs HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Inactive audit\",\"sandboxMode\":\"sandboxed\",\"network\":\"blocked\",\"networkProfile\":\"offline\",\"fileWrite\":\"blocked\",\"approvalRequired\":[],\"forbiddenOperations\":[\"network\"],\"setActive\":false}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let list = handle_http_request(
            "GET /v1/policy/packs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let active_list = handle_http_request(
            "GET /v1/policy/packs?project=proj_local&active=true HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let shell_eval = handle_http_request(
            "POST /v1/policy/evaluate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"actionKind\":\"shell_command\",\"command\":\"npm test\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let network_eval = handle_http_request(
            "POST /v1/policy/evaluate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"runId\":\"run_policy\",\"actionKind\":\"network\",\"command\":\"curl https://example.com\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let _other_network_eval = handle_http_request(
            "POST /v1/policy/evaluate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"runId\":\"run_other\",\"actionKind\":\"network\",\"command\":\"curl https://example.com\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let audit = handle_http_request(
            "GET /v1/policy/audit?project=proj_local&decision=forbidden&actionKind=network&run=run_policy HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let invalid = handle_http_request(
            "POST /v1/policy/packs HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Bad\",\"sandboxMode\":\"wide-open\",\"setActive\":true}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let upsert_body: serde_json::Value =
            serde_json::from_str(&upsert.body).expect("policy pack json");
        let inactive_upsert_body: serde_json::Value =
            serde_json::from_str(&inactive_upsert.body).expect("inactive policy pack json");
        let list_body: serde_json::Value =
            serde_json::from_str(&list.body).expect("policy pack list json");
        let active_list_body: serde_json::Value =
            serde_json::from_str(&active_list.body).expect("active policy pack list json");
        let shell_eval_body: serde_json::Value =
            serde_json::from_str(&shell_eval.body).expect("shell eval json");
        let network_eval_body: serde_json::Value =
            serde_json::from_str(&network_eval.body).expect("network eval json");
        let audit_body: serde_json::Value =
            serde_json::from_str(&audit.body).expect("permission audit json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(upsert.status_code, 201);
        assert_eq!(upsert_body["name"], "Ask before write");
        assert_eq!(upsert_body["sandbox_mode"], "ask-before-write");
        assert_eq!(upsert_body["network_profile"], "local-only");
        assert_eq!(upsert_body["active"], true);
        assert_eq!(inactive_upsert.status_code, 201);
        assert_eq!(inactive_upsert_body["active"], false);
        assert_eq!(list.status_code, 200);
        assert_eq!(
            list_body["items"][0]["approval_required"][0],
            "shell_command"
        );
        assert_eq!(list_body["items"].as_array().unwrap().len(), 2);
        assert_eq!(active_list.status_code, 200);
        assert_eq!(active_list_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(active_list_body["items"][0]["name"], "Ask before write");
        assert_eq!(active_list_body["items"][0]["active"], true);
        assert_eq!(shell_eval.status_code, 200);
        assert_eq!(shell_eval_body["decision"], "approval_required");
        assert_eq!(network_eval.status_code, 200);
        assert_eq!(network_eval_body["decision"], "forbidden");
        assert_eq!(audit.status_code, 200);
        assert_eq!(audit_body["items"][0]["id"], "permission_audit_2");
        assert_eq!(audit_body["items"][0]["decision"], "forbidden");
        assert_eq!(audit_body["items"][0]["action_kind"], "network");
        assert_eq!(audit_body["items"][0]["run_id"], "run_policy");
        assert_eq!(
            audit_body["items"][0]["reason"],
            "action matches forbidden operation"
        );
        assert_eq!(invalid.status_code, 400);
        assert_eq!(
            state_body["security"]["policy_pack"]["name"],
            "Ask before write"
        );
        assert_eq!(
            state_body["security"]["policy_pack"]["sandbox_mode"],
            "ask-before-write"
        );
        assert_eq!(
            state_body["security"]["policy_pack"]["network_profile"],
            "local-only"
        );
        assert_eq!(state_body["security"]["diagnostics"]["status"], "warning");
        assert_eq!(
            state_body["security"]["diagnostics"]["checks"][2]["label"],
            "Policy pack"
        );
        assert_eq!(
            state_body["security"]["diagnostics"]["checks"][2]["status"],
            "ok"
        );
        assert_eq!(
            state_body["security"]["permission_audit"]["recent_count"],
            3
        );
        assert_eq!(
            state_body["security"]["permission_audit"]["forbidden_count"],
            2
        );
        assert_eq!(
            state_body["security"]["permission_audit"]["latest_decision"],
            "forbidden"
        );
        assert!(state_body["security"]["diagnostics"]["checks"]
            .as_array()
            .expect("diagnostic checks")
            .iter()
            .any(|check| check["id"] == "permission-audit"
                && check["status"] == "warning"
                && check["detail"] == "2 forbidden decisions in recent audit"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn scans_lists_and_pauses_agents_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-agents");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let scan = handle_http_request(
            "POST /v1/agents/scan HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let pause = handle_http_request(
            "POST /v1/agents/agent_codex/pause HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let heartbeat = handle_http_request(
            "POST /v1/agents/agent_codex/heartbeat HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let list = handle_http_request(
            "GET /v1/agents HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let scan_body: serde_json::Value = serde_json::from_str(&scan.body).expect("scan json");
        let pause_body: serde_json::Value = serde_json::from_str(&pause.body).expect("pause json");
        let heartbeat_body: serde_json::Value =
            serde_json::from_str(&heartbeat.body).expect("heartbeat json");
        let list_body: serde_json::Value = serde_json::from_str(&list.body).expect("list json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(scan.status_code, 201);
        assert!(scan_body["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|agent| agent["id"] == "agent_generic_shell"));
        assert_eq!(pause.status_code, 200);
        assert_eq!(pause_body["status"], "paused");
        assert_eq!(heartbeat.status_code, 200);
        assert_eq!(heartbeat_body["status"], "available");
        assert!(heartbeat_body["last_heartbeat_at"].as_str().is_some());
        assert!(list_body["items"].as_array().unwrap().iter().any(|agent| {
            agent["id"] == "agent_codex"
                && agent["status"] == "available"
                && agent["last_heartbeat_at"].as_str().is_some()
        }));
        assert!(state_body["agents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|agent| {
                agent["id"] == "agent_codex"
                    && agent["available"] == true
                    && agent["last_heartbeat_at"].as_str().is_some()
            }));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn registers_third_party_agent_adapter_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-agent-adapter");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let registered = handle_http_request(
            "POST /v1/agents HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"id\":\"agent_acme\",\"name\":\"Acme CLI\",\"runtime\":\"generic-cli\",\"command\":\"acme-agent\",\"argsJson\":[\"--json\"],\"envPolicyJson\":{\"inherit\":false,\"allow\":[\"ACME_API_KEY\"]},\"skillsJson\":[\"code-review\"],\"status\":\"available\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let session = handle_http_request(
            "POST /v1/sessions HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"mode\":\"agent\",\"title\":\"Acme raw terminal\",\"cwd\":\"/repo\",\"agentProfileId\":\"agent_acme\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let release_gate = handle_http_request(
            "POST /v1/release-gates/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let invalid = handle_http_request(
            "POST /v1/agents HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"id\":\"agent_bad\",\"name\":\"Bad CLI\",\"runtime\":\"generic-cli\",\"command\":\"\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let registered_body: serde_json::Value =
            serde_json::from_str(&registered.body).expect("registered agent json");
        let session_body: serde_json::Value =
            serde_json::from_str(&session.body).expect("session json");
        let release_gate_body: serde_json::Value =
            serde_json::from_str(&release_gate.body).expect("release gate json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(registered.status_code, 201);
        assert_eq!(registered_body["id"], "agent_acme");
        assert_eq!(registered_body["command"], "acme-agent");
        assert_eq!(session.status_code, 201);
        assert_eq!(session_body["agent_profile_id"], "agent_acme");
        assert!(release_gate_body["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .any(|scenario| {
                scenario["gate_id"] == "RG-06"
                    && scenario["status"] == "pass"
                    && scenario["evidence"][0] == session_body["id"]
            }));
        assert!(state_body["agents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|agent| agent["id"] == "agent_acme" && agent["available"] == true));
        assert_eq!(invalid.status_code, 400);
        assert!(invalid
            .body
            .contains("agent profile command cannot be empty"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn saves_and_lists_skill_packs_through_control_api() {
        let db_path = unique_test_db_path("control-skill-packs");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let project = handle_http_request(
            "POST /v1/projects HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"key\":\"AUTH\",\"name\":\"Auth Service\",\"path\":\"/repo/auth-service\",\"color\":\"#059669\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let saved = handle_http_request(
            "POST /v1/skill-packs HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"name\":\"Auth reviewer\",\"description\":\"Review auth flows\",\"skillsJson\":[\"code-review\",\"auth\"],\"sourceContextPackId\":\"ctx_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/skill-packs?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let saved_body: serde_json::Value =
            serde_json::from_str(&saved.body).expect("saved skill pack json");
        let list_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("skill pack list json");

        assert_eq!(project.status_code, 201);
        assert_eq!(saved.status_code, 201);
        assert_eq!(saved_body["id"], "skill_pack_1");
        assert_eq!(saved_body["name"], "Auth reviewer");
        assert_eq!(saved_body["skills_json"][0], "code-review");
        assert_eq!(saved_body["source_context_pack_id"], "ctx_auth");
        assert!(saved_body["snapshot_id"].as_str().is_some());
        assert_eq!(listed.status_code, 200);
        assert_eq!(list_body["items"][0]["id"], "skill_pack_1");
        assert_eq!(list_body["items"][0]["skills_json"][1], "auth");
        assert_eq!(list_body["items"][0]["source_context_pack_id"], "ctx_auth");
        assert!(list_body["snapshot_id"].as_str().is_some());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_runtime_pool_through_control_api() {
        let db_path = unique_test_db_path("control-runtime-pool");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let project = handle_http_request(
            "POST /v1/projects HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"key\":\"AUTH\",\"name\":\"Auth Service\",\"path\":\"/repo/auth-service\",\"color\":\"#059669\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let shell = handle_http_request(
            "POST /v1/sessions HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"mode\":\"shell\",\"title\":\"Local zsh\",\"cwd\":\"/repo/auth-service\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let ssh = handle_http_request(
            "POST /v1/sessions HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"mode\":\"ssh\",\"title\":\"Deploy SSH\",\"cwd\":\"ssh://staging.example.com~/auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let pool = handle_http_request(
            "GET /v1/runtime-pool?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let body: serde_json::Value = serde_json::from_str(&pool.body).expect("runtime pool json");

        assert_eq!(project.status_code, 201);
        assert_eq!(shell.status_code, 201);
        assert_eq!(ssh.status_code, 201);
        assert_eq!(pool.status_code, 200);
        assert!(body["items"]
            .as_array()
            .expect("pool items")
            .iter()
            .any(|item| item["id"] == "shell"
                && item["label"] == "Local"
                && item["session_count"] == 1));
        assert!(body["items"]
            .as_array()
            .expect("pool items")
            .iter()
            .any(|item| item["id"] == "ssh"
                && item["label"] == "Remote SSH"
                && item["session_count"] == 1));
        assert!(body["snapshot_id"].as_str().is_some());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_provider_model_settings_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-provider-model");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let updated = handle_http_request(
            "POST /v1/provider-model HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"provider\":\"anthropic\",\"model\":\"claude-3-7-sonnet-latest\",\"agentProfileId\":\"agent_claude\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let loaded = handle_http_request(
            "GET /v1/provider-model HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let updated_body: serde_json::Value =
            serde_json::from_str(&updated.body).expect("updated provider model json");
        let loaded_body: serde_json::Value =
            serde_json::from_str(&loaded.body).expect("loaded provider model json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(updated.status_code, 200);
        assert_eq!(updated_body["provider"], "anthropic");
        assert_eq!(updated_body["model"], "claude-3-7-sonnet-latest");
        assert_eq!(updated_body["agent_profile_id"], "agent_claude");
        assert_eq!(loaded_body["provider"], "anthropic");
        assert_eq!(loaded_body["model"], "claude-3-7-sonnet-latest");
        assert_eq!(loaded_body["agent_profile_id"], "agent_claude");
        assert_eq!(state_body["provider_model"]["provider"], "anthropic");
        assert_eq!(
            state_body["provider_model"]["model"],
            "claude-3-7-sonnet-latest"
        );
        assert_eq!(
            state_body["provider_model"]["agent_profile_id"],
            "agent_claude"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_terminal_theme_settings_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-terminal-theme");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let updated = handle_http_request(
            "POST /v1/terminal-theme HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Local Focus\",\"background\":\"#09111f\",\"foreground\":\"#eaf6ff\",\"accent\":\"#19c37d\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let loaded = handle_http_request(
            "GET /v1/terminal-theme?projectId=proj_local HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let invalid = handle_http_request(
            "POST /v1/terminal-theme HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Broken\",\"background\":\"blue\",\"foreground\":\"#eaf6ff\",\"accent\":\"#19c37d\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let updated_body: serde_json::Value =
            serde_json::from_str(&updated.body).expect("updated terminal theme json");
        let loaded_body: serde_json::Value =
            serde_json::from_str(&loaded.body).expect("loaded terminal theme json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(updated.status_code, 200);
        assert_eq!(updated_body["project_id"], "proj_local");
        assert_eq!(updated_body["name"], "Local Focus");
        assert_eq!(loaded.status_code, 200);
        assert_eq!(loaded_body["background"], "#09111f");
        assert_eq!(invalid.status_code, 400);
        assert_eq!(state_body["app"]["terminal_theme"]["name"], "Local Focus");
        assert_eq!(
            state_body["app"]["terminal_theme"]["project_id"],
            "proj_local"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn operational_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-operational-mutation-snapshot-id");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };
        let requests = [
            (
                "agent upsert",
                201,
                "POST /v1/agents HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"id\":\"agent_snapshot\",\"name\":\"Snapshot Agent\",\"runtime\":\"generic-cli\",\"command\":\"snapshot-agent\",\"argsJson\":[],\"envPolicyJson\":{},\"skillsJson\":[],\"status\":\"available\"}",
            ),
            (
                "agent pause",
                200,
                "POST /v1/agents/agent_snapshot/pause HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            ),
            (
                "agent resume",
                200,
                "POST /v1/agents/agent_snapshot/resume HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            ),
            (
                "agent heartbeat",
                200,
                "POST /v1/agents/agent_snapshot/heartbeat HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            ),
            (
                "provider model",
                200,
                "POST /v1/provider-model HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"provider\":\"openai\",\"model\":\"gpt-5.4\",\"agentProfileId\":\"agent_snapshot\"}",
            ),
            (
                "terminal theme",
                200,
                "POST /v1/terminal-theme HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Snapshot Theme\",\"background\":\"#101820\",\"foreground\":\"#f2f5f8\",\"accent\":\"#19c37d\"}",
            ),
            (
                "budget",
                201,
                "POST /v1/budgets HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"scopeType\":\"project\",\"scopeId\":\"proj_local\",\"maxUsd\":20.0,\"warnPct\":0.8,\"hardLimit\":true}",
            ),
            (
                "token usage",
                201,
                "POST /v1/token-usage HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"agentProfileId\":\"agent_snapshot\",\"provider\":\"openai\",\"model\":\"gpt-5.4\",\"inputTokens\":100,\"outputTokens\":50,\"costUsd\":0.25,\"source\":\"manual\"}",
            ),
            (
                "token usage ingest",
                201,
                "POST /v1/token-usage/ingest HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"agentProfileId\":\"agent_snapshot\",\"adapter\":\"openai.responses\",\"payload\":{\"model\":\"gpt-5.4\",\"usage\":{\"input_tokens\":120,\"output_tokens\":60},\"cost_usd\":0.35}}",
            ),
            (
                "agent event ingest",
                201,
                "POST /v1/agent-events/ingest HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"agentProfileId\":\"agent_snapshot\",\"adapter\":\"raw-jsonl\",\"payload\":{\"raw\":\"{\\\"type\\\":\\\"status\\\",\\\"status\\\":\\\"working\\\",\\\"message\\\":\\\"Snapshot event\\\"}\\n\"}}",
            ),
            (
                "provider prices",
                200,
                "POST /v1/provider-prices/update HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"source\":\"snapshot-fixture\",\"prices\":[{\"provider\":\"openai\",\"model\":\"gpt-5.4\",\"inputUsdPerMillion\":2.0,\"outputUsdPerMillion\":8.0}]}",
            ),
            (
                "secret",
                201,
                "POST /v1/secrets HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"SNAPSHOT_SECRET\",\"value\":\"hidden\"}",
            ),
        ];

        for (name, expected_status, request) in requests {
            let response = handle_http_request(request, &store, pty.clone(), "0.1.0-test");
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("mutation response json");

            assert_eq!(response.status_code, expected_status, "{name}");
            assert!(
                body["snapshot_id"]
                    .as_str()
                    .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
                "{name} should include a snapshot_id: {}",
                response.body
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn manages_sessions_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-sessions");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_attached".to_string(),
                key: "HC-ATTACH".to_string(),
                project_id: "proj_local".to_string(),
                title: "Attach session task".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "medium".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("attach target task persisted");

        let created = handle_http_request(
            "POST /v1/sessions HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"mode\":\"agent\",\"title\":\"Codex AUTH-104\",\"cwd\":\"/repo/auth-service\",\"branch\":\"fix/auth\",\"agentProfileId\":\"agent_codex\",\"taskId\":\"task_auth\",\"runId\":\"run_1\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_created = handle_http_request(
            "POST /v1/sessions HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"mode\":\"agent\",\"title\":\"Auth project shell\",\"cwd\":\"/repo/auth-service\",\"agentProfileId\":\"agent_codex\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let list = handle_http_request(
            "GET /v1/sessions HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_list = handle_http_request(
            "GET /v1/sessions?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let rejected_input = handle_http_request(
            "POST /v1/sessions/session_1/input HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"text\":\"rm -rf /tmp/build\\n\",\"allowDangerous\":false}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let accepted_input = handle_http_request(
            "POST /v1/sessions/session_1/input HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"text\":\"rm -rf /tmp/build\\n\",\"allowDangerous\":true}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let attached = handle_http_request(
            "POST /v1/sessions/session_1/attach-task HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"taskId\":\"task_attached\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let detached = handle_http_request(
            "POST /v1/sessions/session_1/detach-task HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let focused = handle_http_request(
            "POST /v1/sessions/session_1/focus HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let resized = handle_http_request(
            "POST /v1/sessions/session_1/resize HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"cols\":132,\"rows\":40}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let attention_unread = handle_http_request(
            "POST /v1/sessions/session_1/attention HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"attentionState\":\"unread\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let attention_read = handle_http_request(
            "POST /v1/sessions/session_1/attention HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"attentionState\":\"none\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let takeover = handle_http_request(
            "POST /v1/sessions/session_1/takeover HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let released = handle_http_request(
            "POST /v1/sessions/session_1/release HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let killed = handle_http_request(
            "POST /v1/sessions/session_1/kill HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let active_local = handle_http_request(
            "POST /v1/sessions HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"mode\":\"shell\",\"title\":\"Local shell\",\"cwd\":\"/repo/local\",\"taskId\":\"task_attached\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let completed_list = handle_http_request(
            "GET /v1/sessions?projectId=proj_local&state=completed HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let attached_task_list = handle_http_request(
            "GET /v1/sessions?projectId=proj_local&taskId=task_attached HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let created_body: serde_json::Value =
            serde_json::from_str(&created.body).expect("created session json");
        let auth_created_body: serde_json::Value =
            serde_json::from_str(&auth_created.body).expect("auth created session json");
        let list_body: serde_json::Value = serde_json::from_str(&list.body).expect("list json");
        let auth_list_body: serde_json::Value =
            serde_json::from_str(&auth_list.body).expect("auth list json");
        let accepted_body: serde_json::Value =
            serde_json::from_str(&accepted_input.body).expect("input json");
        let attached_body: serde_json::Value =
            serde_json::from_str(&attached.body).expect("attached json");
        let detached_body: serde_json::Value =
            serde_json::from_str(&detached.body).expect("detached json");
        let focused_body: serde_json::Value =
            serde_json::from_str(&focused.body).expect("focus json");
        let resized_body: serde_json::Value =
            serde_json::from_str(&resized.body).expect("resize json");
        let attention_unread_body: serde_json::Value =
            serde_json::from_str(&attention_unread.body).expect("attention unread json");
        let attention_read_body: serde_json::Value =
            serde_json::from_str(&attention_read.body).expect("attention read json");
        let takeover_body: serde_json::Value =
            serde_json::from_str(&takeover.body).expect("takeover json");
        let released_body: serde_json::Value =
            serde_json::from_str(&released.body).expect("release json");
        let killed_body: serde_json::Value = serde_json::from_str(&killed.body).expect("kill json");
        let active_local_body: serde_json::Value =
            serde_json::from_str(&active_local.body).expect("active local session json");
        let completed_list_body: serde_json::Value =
            serde_json::from_str(&completed_list.body).expect("completed session list json");
        let attached_task_list_body: serde_json::Value =
            serde_json::from_str(&attached_task_list.body)
                .expect("attached task session list json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(created.status_code, 201);
        assert_eq!(created_body["id"], "session_1");
        assert_eq!(auth_created.status_code, 201);
        assert_eq!(auth_created_body["id"], "session_2");
        assert_eq!(list.status_code, 200);
        assert_eq!(list_body["items"][0]["title"], "Codex AUTH-104");
        assert_eq!(list_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_list.status_code, 200);
        assert_eq!(auth_list_body["items"][0]["title"], "Auth project shell");
        assert_eq!(auth_list_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(rejected_input.status_code, 400);
        assert_eq!(accepted_input.status_code, 200);
        assert_eq!(accepted_body["command_block_id"], "cmdblk_session_input_1");
        assert_eq!(attached.status_code, 200);
        assert_eq!(attached_body["task_id"], "task_attached");
        assert_eq!(detached.status_code, 200);
        assert_eq!(detached_body["task_id"], serde_json::Value::Null);
        assert_eq!(detached_body["run_id"], serde_json::Value::Null);
        assert_eq!(focused_body["attention_state"], "none");
        assert_eq!(resized.status_code, 200);
        assert_eq!(resized_body["session_id"], "session_1");
        assert_eq!(resized_body["pane_id"], "pane_session_1");
        assert_eq!(resized_body["cols"], 132);
        assert_eq!(resized_body["rows"], 40);
        assert_eq!(attention_unread.status_code, 200);
        assert_eq!(attention_unread_body["attention_state"], "unread");
        assert_eq!(attention_read.status_code, 200);
        assert_eq!(attention_read_body["attention_state"], "none");
        assert_eq!(takeover_body["attention_state"], "needs_input");
        assert_eq!(released.status_code, 200);
        assert_eq!(released_body["state"], "running");
        assert_eq!(released_body["attention_state"], "none");
        assert_eq!(killed_body["state"], "completed");
        assert_eq!(active_local.status_code, 201);
        assert_eq!(active_local_body["id"], "session_3");
        assert_eq!(active_local_body["task_id"], "task_attached");
        assert_eq!(completed_list.status_code, 200);
        assert_eq!(completed_list_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(completed_list_body["items"][0]["id"], "session_1");
        assert_eq!(completed_list_body["items"][0]["state"], "completed");
        assert_eq!(attached_task_list.status_code, 200);
        assert_eq!(
            attached_task_list_body["items"].as_array().unwrap().len(),
            1
        );
        assert_eq!(attached_task_list_body["items"][0]["id"], "session_3");
        assert_eq!(
            attached_task_list_body["items"][0]["task_id"],
            "task_attached"
        );
        assert_eq!(state_body["sessions"][0]["id"], "session_1");
        assert_eq!(state_body["sessions"][0]["state"], "completed");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn session_control_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-session-mutation-snapshot-id");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_attached".to_string(),
                key: "HC-ATTACH".to_string(),
                project_id: "proj_local".to_string(),
                title: "Attach session task".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "medium".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("attach target task persisted");

        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };
        let created = handle_http_request(
            "POST /v1/sessions HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"mode\":\"agent\",\"title\":\"Codex AUTH-104\",\"cwd\":\"/repo/auth-service\",\"branch\":\"fix/auth\",\"agentProfileId\":\"agent_codex\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let recorded_stream = handle_http_request(
            "POST /v1/sessions/session_1/stream-chunks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"seqStart\":1,\"seqEnd\":3,\"body\":\"npm test\\n\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let input = handle_http_request(
            "POST /v1/sessions/session_1/input HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"text\":\"npm test\\n\",\"allowDangerous\":false}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let focused = handle_http_request(
            "POST /v1/sessions/session_1/focus HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let resized = handle_http_request(
            "POST /v1/sessions/session_1/resize HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"cols\":132,\"rows\":40}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let attention = handle_http_request(
            "POST /v1/sessions/session_1/attention HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"attentionState\":\"unread\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let attached = handle_http_request(
            "POST /v1/sessions/session_1/attach-task HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"taskId\":\"task_attached\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let detached = handle_http_request(
            "POST /v1/sessions/session_1/detach-task HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let takeover = handle_http_request(
            "POST /v1/sessions/session_1/takeover HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let released = handle_http_request(
            "POST /v1/sessions/session_1/release HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let killed = handle_http_request(
            "POST /v1/sessions/session_1/kill HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            pty,
            "0.1.0-test",
        );
        let responses = [
            ("create", 201, created),
            ("stream", 201, recorded_stream),
            ("input", 200, input),
            ("focus", 200, focused),
            ("resize", 200, resized),
            ("attention", 200, attention),
            ("attach", 200, attached),
            ("detach", 200, detached),
            ("takeover", 200, takeover),
            ("release", 200, released),
            ("kill", 200, killed),
        ];

        for (name, expected_status, response) in responses {
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("mutation response json");

            assert_eq!(response.status_code, expected_status, "{name}");
            assert!(
                body["snapshot_id"]
                    .as_str()
                    .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
                "{name} should include a snapshot_id: {}",
                response.body
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn lists_sessions_with_project_snapshot_id_matching_state_snapshot() {
        let db_path = unique_test_db_path("control-session-list-snapshot-id");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let session = store
            .create_session(crate::state_store::CreateSessionInput {
                project_id: "proj_auth".to_string(),
                mode: "agent".to_string(),
                title: "Auth worker".to_string(),
                cwd: Some("/repo/auth".to_string()),
                branch: Some("feature/auth".to_string()),
                agent_profile_id: Some("agent_codex".to_string()),
                task_id: None,
                run_id: None,
            })
            .expect("session created");
        rusqlite::Connection::open(&db_path)
            .expect("test db opens")
            .execute(
                "UPDATE sessions SET created_at = ?1, updated_at = ?1 WHERE id = ?2",
                rusqlite::params!["2026-04-30T00:00:00Z", session.id],
            )
            .expect("session timestamps stabilized");

        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };
        let list = handle_http_request(
            "GET /v1/sessions?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            pty,
            "0.1.0-test",
        );

        let list_body: serde_json::Value = serde_json::from_str(&list.body).expect("list json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(list.status_code, 200);
        assert_eq!(state.status_code, 200);
        assert_eq!(list_body["items"][0]["id"], state_body["sessions"][0]["id"]);
        assert_eq!(list_body["snapshot_id"], state_body["snapshot_id"]);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn cli_list_endpoints_expose_state_snapshot_id() {
        let db_path = unique_test_db_path("control-cli-list-snapshot-id");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };
        let state = handle_http_request(
            "GET /v1/state?projectId=proj_local HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let endpoints = [
            "/v1/projects",
            "/v1/sessions?projectId=proj_local",
            "/v1/tasks?projectId=proj_local",
            "/v1/initiatives?projectId=proj_local",
            "/v1/tracker-bindings?projectId=proj_local",
            "/v1/runs?projectId=proj_local",
            "/v1/reviews?projectId=proj_local",
            "/v1/agents",
            "/v1/policy/approvals?project=proj_local",
            "/v1/policy/packs?project=proj_local",
            "/v1/policy/audit?project=proj_local",
            "/v1/provider-prices",
            "/v1/secrets?project=proj_local",
            "/v1/knowledge?projectId=proj_local",
            "/v1/knowledge/sources?projectId=proj_local",
            "/v1/knowledge/explorations?projectId=proj_local",
            "/v1/knowledge/concepts?projectId=proj_local",
            "/v1/context-packs?projectId=proj_local",
            "/v1/release-gates/runs?projectId=proj_local",
            "/v1/terminal-fidelity/smoke/runs?projectId=proj_local",
            "/v1/task-lifecycle/e2e/runs?projectId=proj_local",
            "/v1/workflow/negative-tests/runs?projectId=proj_local",
            "/v1/distribution/dmg-smoke/runs?projectId=proj_local",
            "/v1/recovery/drills/runs?projectId=proj_local",
            "/v1/benchmarks/runs?projectId=proj_local",
            "/v1/dogfood/telemetry-reviews?projectId=proj_local",
            "/v1/visual-harness/links?projectId=proj_local",
            "/v1/command-blocks",
        ];

        assert_eq!(state.status_code, 200);
        for endpoint in endpoints {
            let request = format!("GET {endpoint} HTTP/1.1\r\nhost: haneulchi\r\n\r\n");
            let response = handle_http_request(&request, &store, pty.clone(), "0.1.0-test");
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("list response json");

            assert_eq!(response.status_code, 200, "{endpoint}: {}", response.body);
            assert_eq!(
                body["snapshot_id"], state_body["snapshot_id"],
                "{endpoint} should expose the state snapshot id"
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn records_and_lists_terminal_stream_chunks_through_control_api() {
        let db_path = unique_test_db_path("control-terminal-stream-chunks");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_secret(crate::state_store::UpsertSecretInput {
                project_id: "proj_local".to_string(),
                name: "SESSION_TOKEN".to_string(),
                value: "terminal-secret-value".to_string(),
            })
            .expect("secret saved");
        store
            .create_session(crate::state_store::CreateSessionInput {
                project_id: "proj_local".to_string(),
                mode: "agent".to_string(),
                title: "PTY transcript".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                agent_profile_id: None,
                task_id: None,
                run_id: None,
            })
            .expect("session created");

        let recorded = handle_http_request(
            "POST /v1/sessions/session_1/stream-chunks HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"seqStart\":1,\"seqEnd\":4,\"body\":\"npm test\\nterminal-secret-value\\n\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let listed = handle_http_request(
            "GET /v1/sessions/session_1/stream-chunks HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let recorded_body: serde_json::Value =
            serde_json::from_str(&recorded.body).expect("recorded stream chunk json");
        let listed_body: serde_json::Value =
            serde_json::from_str(&listed.body).expect("listed stream chunk json");

        assert_eq!(recorded.status_code, 201);
        assert_eq!(recorded_body["id"], "terminal_stream_chunk_1");
        assert_eq!(recorded_body["session_id"], "session_1");
        assert!(recorded_body["body"]
            .as_str()
            .unwrap()
            .contains("[REDACTED:SESSION_TOKEN]"));
        assert!(!recorded.body.contains("terminal-secret-value"));
        assert_eq!(listed.status_code, 200);
        assert_eq!(listed_body["items"][0]["id"], "terminal_stream_chunk_1");
        assert!(listed_body["items"][0]["body"]
            .as_str()
            .unwrap()
            .contains("[REDACTED:SESSION_TOKEN]"));
        assert!(!listed.body.contains("terminal-secret-value"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn manages_projects_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-projects");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let created = handle_http_request(
            "POST /v1/projects HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"key\":\"AUTH\",\"name\":\"Auth Service\",\"path\":\"/repo/auth-service\",\"color\":\"#059669\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let duplicate_path = handle_http_request(
            "POST /v1/projects HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"key\":\"AUTH2\",\"name\":\"Auth Service Copy\",\"path\":\"/repo/auth-service\",\"color\":\"#0ea5e9\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let list = handle_http_request(
            "GET /v1/projects HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let focused = handle_http_request(
            "POST /v1/projects/proj_auth/focus HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let detached = handle_http_request(
            "POST /v1/projects/proj_auth/detach HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let tab_group = handle_http_request(
            "POST /v1/projects/proj_auth/tab-group HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"groupName\":\"Backend\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let layout = handle_http_request(
            "POST /v1/projects/proj_auth/layout HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"layoutJson\":{\"mode\":\"maximized\",\"focusedSessionId\":\"session_1\",\"maximizedSessionId\":\"session_1\",\"panes\":[\"session_1\"]}}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let preset = handle_http_request(
            "POST /v1/projects/proj_auth/layout-presets HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"name\":\"Review grid\",\"layoutJson\":{\"mode\":\"grid\",\"focusedSessionId\":\"session_1\",\"maximizedSessionId\":null,\"panes\":[\"session_1\"]}}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let presets = handle_http_request(
            "GET /v1/projects/proj_auth/layout-presets HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let created_body: serde_json::Value =
            serde_json::from_str(&created.body).expect("created project json");
        let duplicate_body: serde_json::Value =
            serde_json::from_str(&duplicate_path.body).expect("duplicate path error json");
        let list_body: serde_json::Value = serde_json::from_str(&list.body).expect("list json");
        let focused_body: serde_json::Value =
            serde_json::from_str(&focused.body).expect("focus json");
        let detached_body: serde_json::Value =
            serde_json::from_str(&detached.body).expect("detach json");
        let tab_group_body: serde_json::Value =
            serde_json::from_str(&tab_group.body).expect("tab group json");
        let layout_body: serde_json::Value =
            serde_json::from_str(&layout.body).expect("layout json");
        let preset_body: serde_json::Value =
            serde_json::from_str(&preset.body).expect("layout preset json");
        let presets_body: serde_json::Value =
            serde_json::from_str(&presets.body).expect("layout presets json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(created.status_code, 201);
        assert_eq!(created_body["id"], "proj_auth");
        assert_eq!(duplicate_path.status_code, 400);
        assert_eq!(
            duplicate_body["error"],
            "project path already registered by project proj_auth"
        );
        assert_eq!(list.status_code, 200);
        assert_eq!(list_body["items"][0]["name"], "Auth Service");
        assert_eq!(focused.status_code, 200);
        assert_eq!(focused_body["id"], "proj_auth");
        assert_eq!(detached.status_code, 200);
        assert_eq!(detached_body["window_id"], "win_proj_auth");
        assert_eq!(detached_body["status"], "planned");
        assert_eq!(tab_group.status_code, 200);
        assert_eq!(tab_group_body["group_name"], "Backend");
        assert_eq!(layout.status_code, 200);
        assert_eq!(layout_body["layout_json"]["mode"], "maximized");
        assert_eq!(preset.status_code, 201);
        assert_eq!(preset_body["name"], "Review grid");
        assert_eq!(preset_body["layout_json"]["mode"], "grid");
        assert_eq!(presets.status_code, 200);
        assert_eq!(presets_body["items"][0]["name"], "Review grid");
        assert_eq!(
            presets_body["items"][0]["layout_json"]["focusedSessionId"],
            "session_1"
        );
        assert_eq!(state_body["projects"][0]["id"], "proj_auth");
        assert_eq!(state_body["project_tabs"][0]["id"], "tab_proj_auth");
        assert_eq!(state_body["project_tabs"][0]["active"], true);
        assert_eq!(state_body["project_tabs"][0]["group_name"], "Backend");
        assert_eq!(
            state_body["project_tabs"][0]["layout_json"]["focusedSessionId"],
            "session_1"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn project_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-project-mutation-snapshot-id");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };

        let created = handle_http_request(
            "POST /v1/projects HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"key\":\"AUTH\",\"name\":\"Auth Service\",\"path\":\"/repo/auth-service\",\"color\":\"#059669\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let focused = handle_http_request(
            "POST /v1/projects/proj_auth/focus HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let detached = handle_http_request(
            "POST /v1/projects/proj_auth/detach HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let tab_group = handle_http_request(
            "POST /v1/projects/proj_auth/tab-group HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"groupName\":\"Backend\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let layout = handle_http_request(
            "POST /v1/projects/proj_auth/layout HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"layoutJson\":{\"mode\":\"maximized\",\"focusedSessionId\":\"session_1\",\"maximizedSessionId\":\"session_1\",\"panes\":[\"session_1\"]}}",
            &store,
            pty,
            "0.1.0-test",
        );
        let responses = [
            ("create", 201, created),
            ("focus", 200, focused),
            ("detach", 200, detached),
            ("tab_group", 200, tab_group),
            ("layout", 200, layout),
        ];

        for (name, expected_status, response) in responses {
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("project mutation response json");

            assert_eq!(response.status_code, expected_status, "{name}");
            assert!(
                body["snapshot_id"]
                    .as_str()
                    .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
                "{name} should include a snapshot_id: {}",
                response.body
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn initiative_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-initiative-mutation-snapshot-id");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let created = handle_http_request(
            "POST /v1/initiatives HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Auth reliability goal\",\"description\":\"Why auth tasks matter\",\"budgetId\":\"budget_auth\",\"status\":\"active\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value =
            serde_json::from_str(&created.body).expect("created initiative json");

        assert_eq!(created.status_code, 201);
        assert_eq!(body["id"], "init_1");
        assert_eq!(body["name"], "Auth reliability goal");
        assert!(
            body["snapshot_id"]
                .as_str()
                .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
            "initiative create should include a snapshot_id: {}",
            created.body
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_project_file_explorer_entries_with_git_status() {
        let db_path = unique_test_db_path("control-project-files");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(&workspace).expect("workspace");
        fs::write(workspace.join("README.md"), "hello\n").expect("readme");
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .output()
            .expect("git init runs");
        std::process::Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&workspace)
            .output()
            .expect("git add runs");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let create_request = format!(
            "POST /v1/projects HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{{\"key\":\"AUTH\",\"name\":\"Auth Service\",\"path\":\"{}\"}}",
            workspace.to_string_lossy()
        );
        let _ = handle_http_request(
            &create_request,
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let files = handle_http_request(
            "GET /v1/projects/proj_auth/files HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&files.body).expect("files json");

        assert_eq!(files.status_code, 200);
        assert_eq!(body["project_id"], "proj_auth");
        assert_eq!(body["entries"][0]["path"], "README.md");
        assert_eq!(body["entries"][0]["git_status"], "added");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_project_file_preview_content() {
        let db_path = unique_test_db_path("control-project-file-preview");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(workspace.join("src")).expect("workspace");
        fs::write(workspace.join("src").join("main.rs"), "fn main() {}\n").expect("main");
        fs::write(
            workspace.join("src").join("server.log"),
            "INFO boot\nWARN retry\n",
        )
        .expect("log");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let create_request = format!(
            "POST /v1/projects HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{{\"key\":\"AUTH\",\"name\":\"Auth Service\",\"path\":\"{}\"}}",
            workspace.to_string_lossy()
        );
        let _ = handle_http_request(
            &create_request,
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let preview = handle_http_request(
            "GET /v1/projects/proj_auth/file?path=src/main.rs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&preview.body).expect("preview json");

        assert_eq!(preview.status_code, 200);
        assert_eq!(body["project_id"], "proj_auth");
        assert_eq!(body["path"], "src/main.rs");
        assert_eq!(body["language"], "rust");
        assert_eq!(body["body"], "fn main() {}\n");

        let log_preview = handle_http_request(
            "GET /v1/projects/proj_auth/file?path=src/server.log HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let log_body: serde_json::Value =
            serde_json::from_str(&log_preview.body).expect("log preview json");

        assert_eq!(log_preview.status_code, 200);
        assert_eq!(log_body["path"], "src/server.log");
        assert_eq!(log_body["language"], "log");
        assert_eq!(log_body["body"], "INFO boot\nWARN retry\n");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_project_review_diff_content() {
        let db_path = unique_test_db_path("control-project-diff");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(&workspace).expect("workspace");
        fs::write(workspace.join("README.md"), "hello\n").expect("readme");
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .output()
            .expect("git init runs");
        std::process::Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&workspace)
            .output()
            .expect("git add runs");
        std::process::Command::new("git")
            .args([
                "-c",
                "user.email=haneulchi@example.test",
                "-c",
                "user.name=Haneulchi Tests",
                "commit",
                "-m",
                "seed",
            ])
            .current_dir(&workspace)
            .output()
            .expect("git commit runs");
        fs::write(workspace.join("README.md"), "hello\nreview notes\n").expect("readme update");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let create_request = format!(
            "POST /v1/projects HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{{\"key\":\"AUTH\",\"name\":\"Auth Service\",\"path\":\"{}\"}}",
            workspace.to_string_lossy()
        );
        let _ = handle_http_request(
            &create_request,
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let diff = handle_http_request(
            "GET /v1/projects/proj_auth/diff?path=README.md HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&diff.body).expect("diff json");

        assert_eq!(diff.status_code, 200);
        assert_eq!(body["project_id"], "proj_auth");
        assert_eq!(body["path"], "README.md");
        assert_eq!(body["file_count"], 1);
        assert_eq!(body["files"][0]["path"], "README.md");
        assert_eq!(body["files"][0]["status"], "modified");
        assert_eq!(body["files"][0]["additions"], 1);
        assert_eq!(body["files"][0]["deletions"], 0);
        assert!(body["body"].as_str().unwrap().contains("+review notes"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn writes_project_files_through_control_api() {
        let db_path = unique_test_db_path("control-project-file-write");
        let workspace = db_path.with_extension("workspace");
        fs::create_dir_all(workspace.join("src")).expect("workspace tree");
        fs::write(
            workspace.join("src").join("main.ts"),
            "export const oldValue = 1;\n",
        )
        .expect("main");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(crate::state_store::AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let response = handle_http_request(
            "POST /v1/projects/proj_auth/file HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"path\":\"src/main.ts\",\"body\":\"export const newValue = 2;\\n\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&response.body).expect("file json");

        assert_eq!(response.status_code, 200);
        assert_eq!(body["path"], "src/main.ts");
        assert_eq!(body["language"], "typescript");
        assert_eq!(body["body"], "export const newValue = 2;\n");
        assert_eq!(
            fs::read_to_string(workspace.join("src").join("main.ts")).expect("written file"),
            "export const newValue = 2;\n"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_project_file_search_results() {
        let db_path = unique_test_db_path("control-project-file-search");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(workspace.join("src").join("auth")).expect("workspace");
        fs::write(
            workspace.join("src").join("auth").join("login.ts"),
            "export {}\n",
        )
        .expect("login");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let create_request = format!(
            "POST /v1/projects HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{{\"key\":\"AUTH\",\"name\":\"Auth Service\",\"path\":\"{}\"}}",
            workspace.to_string_lossy()
        );
        let _ = handle_http_request(
            &create_request,
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let search = handle_http_request(
            "GET /v1/projects/proj_auth/files/search?query=login HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&search.body).expect("search json");

        assert_eq!(search.status_code, 200);
        assert_eq!(body["project_id"], "proj_auth");
        assert_eq!(body["query"], "login");
        assert_eq!(body["entries"][0]["path"], "src/auth/login.ts");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn dispatches_runs_with_workflow_and_default_context_through_control_api() {
        let db_path = unique_test_db_path("control-workflow-dispatch");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .reload_workflow(crate::state_store::ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: "/repo/WORKFLOW.md".to_string(),
                content: "---\nhaneulchi: 1\nproject:\n  key: AUTH\nworkspace:\n  strategy: worktree\ncontext:\n  default_pack: auth-default\n---\nUse {task.id}.\n".to_string(),
            })
            .expect("workflow reloads");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_workflow_dispatch".to_string(),
                key: "HC-WF-DISPATCH".to_string(),
                project_id: "proj_local".to_string(),
                title: "Dispatch with workflow context".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");

        let dispatched = handle_http_request(
            "POST /v1/dispatch HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 65\r\n\r\n{\"taskId\":\"task_workflow_dispatch\",\"agentProfileId\":\"agent_codex\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let dispatched_body: serde_json::Value =
            serde_json::from_str(&dispatched.body).expect("dispatch json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(dispatched.status_code, 201);
        assert_eq!(dispatched_body["workflow_version_id"], "workflow_1");
        assert_eq!(dispatched_body["context_pack_id"], "auth-default");
        assert_eq!(
            state_body["runs"]["items"][0]["workflow_version_id"],
            "workflow_1"
        );
        assert_eq!(
            state_body["runs"]["items"][0]["context_pack_id"],
            "auth-default"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_budget_summary_and_state_snapshot_budget_data() {
        let db_path = unique_test_db_path("control-budgets");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let budget = handle_http_request(
            "POST /v1/budgets HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"scopeType\":\"project\",\"scopeId\":\"proj_local\",\"maxUsd\":10.0,\"warnPct\":0.8,\"hardLimit\":true}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let initiative = handle_http_request(
            "POST /v1/initiatives HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Auth reliability goal\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_goal_budget".to_string(),
                key: "HC-GOAL-BUDGET".to_string(),
                project_id: "proj_local".to_string(),
                title: "Goal scoped task".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: Some("init_1".to_string()),
                context_pack_id: None,
            })
            .expect("goal budget task persists");
        let goal_budget = handle_http_request(
            "POST /v1/budgets HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"scopeType\":\"goal\",\"scopeId\":\"init_1\",\"maxUsd\":9.0,\"warnPct\":0.8,\"hardLimit\":false}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let task_budget = handle_http_request(
            "POST /v1/budgets HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"scopeType\":\"task\",\"scopeId\":\"task_goal_budget\",\"maxUsd\":9.0,\"warnPct\":0.8,\"hardLimit\":false}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let run_budget = handle_http_request(
            "POST /v1/budgets HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"scopeType\":\"run\",\"scopeId\":\"run_goal_budget\",\"maxUsd\":9.0,\"warnPct\":0.8,\"hardLimit\":false}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let usage = handle_http_request(
            "POST /v1/token-usage HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"taskId\":\"task_goal_budget\",\"runId\":\"run_goal_budget\",\"agentProfileId\":\"agent_codex\",\"provider\":\"openai\",\"model\":\"gpt-5.4\",\"inputTokens\":1200,\"outputTokens\":800,\"costUsd\":8.5,\"source\":\"adapter\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let budgets = handle_http_request(
            "GET /v1/budgets HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let budget_body: serde_json::Value =
            serde_json::from_str(&budget.body).expect("budget json");
        let usage_body: serde_json::Value = serde_json::from_str(&usage.body).expect("usage json");
        let budgets_body: serde_json::Value =
            serde_json::from_str(&budgets.body).expect("budgets json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(budget.status_code, 201);
        assert_eq!(budget_body["scope_type"], "project");
        assert_eq!(initiative.status_code, 201);
        assert_eq!(goal_budget.status_code, 201);
        assert_eq!(task_budget.status_code, 201);
        assert_eq!(run_budget.status_code, 201);
        assert_eq!(usage.status_code, 201);
        assert_eq!(usage_body["cost_usd"], 8.5);
        assert_eq!(budgets.status_code, 200);
        assert_eq!(budgets_body["projects"][0]["state"], "warn");
        assert_eq!(budgets_body["projects"][0]["used_usd"], 8.5);
        assert_eq!(budgets_body["goals"][0]["scope_id"], "init_1");
        assert_eq!(budgets_body["goals"][0]["used_usd"], 8.5);
        assert_eq!(budgets_body["tasks"][0]["scope_id"], "task_goal_budget");
        assert_eq!(budgets_body["tasks"][0]["used_usd"], 8.5);
        assert_eq!(budgets_body["runs"][0]["scope_id"], "run_goal_budget");
        assert_eq!(budgets_body["runs"][0]["used_usd"], 8.5);
        assert_eq!(
            state_body["budgets"]["projects"][0]["scope_id"],
            "proj_local"
        );
        assert_eq!(state_body["budgets"]["projects"][0]["state"], "warn");
        assert_eq!(state_body["budgets"]["goals"][0]["scope_id"], "init_1");
        assert_eq!(state_body["budgets"]["goals"][0]["state"], "warn");
        assert_eq!(
            state_body["budgets"]["tasks"][0]["scope_id"],
            "task_goal_budget"
        );
        assert_eq!(state_body["budgets"]["tasks"][0]["state"], "warn");
        assert_eq!(
            state_body["budgets"]["runs"][0]["scope_id"],
            "run_goal_budget"
        );
        assert_eq!(state_body["budgets"]["runs"][0]["state"], "warn");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn emits_budget_warning_attention_events_only_after_threshold() {
        let db_path = unique_test_db_path("control-budget-attention");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_budget(crate::state_store::UpsertBudgetInput {
                scope_type: "project".to_string(),
                scope_id: Some("proj_local".to_string()),
                max_usd: 10.0,
                warn_pct: 0.8,
                hard_limit: true,
            })
            .expect("budget persists");
        store
            .record_token_usage(crate::state_store::TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: None,
                task_id: None,
                run_id: None,
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 300,
                output_tokens: 200,
                cost_usd: 2.0,
                source: "adapter".to_string(),
            })
            .expect("usage records");

        let below_threshold = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let below_threshold_body: serde_json::Value =
            serde_json::from_str(&below_threshold.body).expect("state json");
        let below_threshold_attention = below_threshold_body["attention"]
            .as_array()
            .expect("attention array");

        assert_eq!(below_threshold.status_code, 200);
        assert!(below_threshold_attention.iter().all(|item| {
            !item["id"]
                .as_str()
                .expect("attention id")
                .starts_with("budget_")
        }));

        store
            .record_token_usage(crate::state_store::TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: None,
                task_id: None,
                run_id: None,
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 1200,
                output_tokens: 800,
                cost_usd: 6.5,
                source: "adapter".to_string(),
            })
            .expect("usage records");

        let warning = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let warning_body: serde_json::Value =
            serde_json::from_str(&warning.body).expect("state json");
        let warning_attention = warning_body["attention"]
            .as_array()
            .expect("attention array")
            .iter()
            .find(|item| item["id"] == "budget_project_proj_local")
            .expect("budget attention item");

        assert_eq!(warning.status_code, 200);
        assert_eq!(
            warning_attention["label"],
            "Budget warning: project proj_local"
        );
        assert_eq!(warning_attention["severity"], "warning");
        assert_eq!(warning_attention["detail"], "$8.50 of $10.00 used · 85%");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn ingests_token_usage_adapters_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-token-usage-adapter");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let usage = handle_http_request(
            "POST /v1/token-usage/ingest HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"agentProfileId\":\"agent_codex\",\"adapter\":\"openai.responses\",\"payload\":{\"model\":\"gpt-5.4\",\"usage\":{\"input_tokens\":1200,\"output_tokens\":800},\"cost_usd\":8.5}}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let usage_body: serde_json::Value = serde_json::from_str(&usage.body).expect("usage json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(usage.status_code, 201);
        assert_eq!(usage_body["source"], "adapter:openai.responses");
        assert_eq!(usage_body["input_tokens"], 1200);
        assert_eq!(usage_body["output_tokens"], 800);
        assert_eq!(state_body["budgets"]["workspace"]["used_usd"], 8.5);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn ingests_agent_events_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-agent-events");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store.scan_agent_profiles().expect("agents scan");

        let event = handle_http_request(
            "POST /v1/agent-events/ingest HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sessionId\":\"session_1\",\"runId\":\"run_1\",\"agentProfileId\":\"agent_codex\",\"adapter\":\"raw-jsonl\",\"payload\":{\"raw\":\"{\\\"type\\\":\\\"status\\\",\\\"status\\\":\\\"needs_input\\\",\\\"message\\\":\\\"Waiting for human review\\\"}\\n\"}}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let event_body: serde_json::Value = serde_json::from_str(&event.body).expect("event json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(event.status_code, 201);
        assert_eq!(event_body["kind"], "status");
        assert_eq!(event_body["severity"], "warning");
        assert!(state_body["agents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|agent| {
                agent["id"] == "agent_codex"
                    && agent["latest_event_kind"] == "status"
                    && agent["latest_event_detail"] == "Waiting for human review"
                    && agent["attention_state"] == "needs_input"
                    && agent["attention_severity"] == "warning"
                    && agent["notification_count"] == 1
            }));
        assert!(state_body["attention"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| {
                item["id"] == "agent_event_agent_codex" && item["severity"] == "warning"
            }));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn policy_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-policy-mutation-snapshot-id");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };

        let approval = handle_http_request(
            "POST /v1/policy/approvals HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"actionKind\":\"shell_command\",\"command\":\"rm -rf build/cache\",\"riskLevel\":\"high\",\"requestedBy\":\"agent_codex\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let decision = handle_http_request(
            "POST /v1/policy/approvals/policy_approval_1/decision HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"decision\":\"denied\",\"decisionBy\":\"human\",\"decisionNote\":\"Too broad.\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let pack = handle_http_request(
            "POST /v1/policy/packs HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"Ask before write\",\"sandboxMode\":\"ask-before-write\",\"network\":\"blocked\",\"networkProfile\":\"local-only\",\"fileWrite\":\"ask\",\"approvalRequired\":[\"shell_command\"],\"forbiddenOperations\":[\"network\"],\"setActive\":true}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let evaluation = handle_http_request(
            "POST /v1/policy/evaluate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"actionKind\":\"shell_command\",\"command\":\"npm test\"}",
            &store,
            pty,
            "0.1.0-test",
        );
        let responses = [
            ("approval", 201, approval),
            ("decision", 200, decision),
            ("pack", 201, pack),
            ("evaluation", 200, evaluation),
        ];

        for (name, expected_status, response) in responses {
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("policy mutation response json");

            assert_eq!(response.status_code, expected_status, "{name}");
            assert!(
                body["snapshot_id"]
                    .as_str()
                    .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
                "{name} should include a snapshot_id: {}",
                response.body
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn updates_provider_price_table_and_prices_adapter_usage() {
        let db_path = unique_test_db_path("control-provider-prices");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let update = handle_http_request(
            "POST /v1/provider-prices/update HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"source\":\"local-fixture\",\"prices\":[{\"provider\":\"openai\",\"model\":\"gpt-5.4\",\"inputUsdPerMillion\":2.0,\"outputUsdPerMillion\":8.0}]}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let usage = handle_http_request(
            "POST /v1/token-usage/ingest HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"agentProfileId\":\"agent_codex\",\"adapter\":\"openai.responses\",\"payload\":{\"model\":\"gpt-5.4\",\"usage\":{\"input_tokens\":1000,\"output_tokens\":500}}}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let prices = handle_http_request(
            "GET /v1/provider-prices HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let update_body: serde_json::Value =
            serde_json::from_str(&update.body).expect("price update json");
        let usage_body: serde_json::Value = serde_json::from_str(&usage.body).expect("usage json");
        let prices_body: serde_json::Value =
            serde_json::from_str(&prices.body).expect("prices json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(update.status_code, 200);
        assert_eq!(update_body["updated"], 1);
        assert_eq!(usage.status_code, 201);
        assert_eq!(usage_body["cost_usd"], 0.006);
        assert_eq!(prices.status_code, 200);
        assert_eq!(prices_body["items"][0]["source"], "local-fixture");
        assert_eq!(state_body["budgets"]["price_table"]["count"], 1);
        assert_eq!(
            state_body["budgets"]["price_table"]["source"],
            "local-fixture"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn stores_secret_metadata_without_exposing_plaintext_through_control_api_or_state() {
        let db_path = unique_test_db_path("control-secrets");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let create = handle_http_request(
            "POST /v1/secrets HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"OPENAI_API_KEY\",\"value\":\"openai-secret-fixture-value\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let second_create = handle_http_request(
            "POST /v1/secrets HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"name\":\"ZZZ_API_KEY\",\"value\":\"zzz-hidden-test-value\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let list = handle_http_request(
            "GET /v1/secrets HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let named_list = handle_http_request(
            "GET /v1/secrets?project=proj_local&name=OPENAI_API_KEY HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let create_body: serde_json::Value =
            serde_json::from_str(&create.body).expect("secret create json");
        let second_create_body: serde_json::Value =
            serde_json::from_str(&second_create.body).expect("second secret create json");
        let list_body: serde_json::Value =
            serde_json::from_str(&list.body).expect("secret list json");
        let named_list_body: serde_json::Value =
            serde_json::from_str(&named_list.body).expect("named secret list json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(create.status_code, 201);
        assert_eq!(create_body["name"], "OPENAI_API_KEY");
        assert_eq!(create_body["project_id"], "proj_local");
        assert_eq!(create_body["redacted"], true);
        assert!(create_body.get("value").is_none());
        assert!(!create.body.contains("openai-secret-fixture-value"));
        assert_eq!(second_create.status_code, 201);
        assert_eq!(second_create_body["name"], "ZZZ_API_KEY");
        assert!(!second_create.body.contains("zzz-hidden-test-value"));
        assert_eq!(list.status_code, 200);
        assert_eq!(list_body["items"].as_array().unwrap().len(), 2);
        assert_eq!(list_body["items"][0]["name"], "OPENAI_API_KEY");
        assert!(list_body["items"][0].get("value").is_none());
        assert!(!list.body.contains("openai-secret-fixture-value"));
        assert!(!list.body.contains("zzz-hidden-test-value"));
        assert_eq!(named_list.status_code, 200);
        assert_eq!(named_list_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(named_list_body["items"][0]["name"], "OPENAI_API_KEY");
        assert!(!named_list.body.contains("openai-secret-fixture-value"));
        assert!(!named_list.body.contains("zzz-hidden-test-value"));
        assert_eq!(state_body["security"]["keychain"], "local");
        assert_eq!(state_body["security"]["secret_count"], 2);
        assert_eq!(state_body["security"]["redaction"]["status"], "active");
        assert_eq!(
            state_body["security"]["redaction"]["protected_secret_count"],
            2
        );
        assert!(!state.body.contains("openai-secret-fixture-value"));
        assert!(!state.body.contains("zzz-hidden-test-value"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn exports_run_token_usage_through_control_api() {
        let db_path = unique_test_db_path("control-run-token-usage-export");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .record_token_usage(crate::state_store::TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: Some("session_1".to_string()),
                task_id: Some("task_usage_export".to_string()),
                run_id: Some("run_1".to_string()),
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 1200,
                output_tokens: 800,
                cost_usd: 8.5,
                source: "adapter:openai.responses".to_string(),
            })
            .expect("usage records");

        let usage = handle_http_request(
            "GET /v1/runs/run_1/token-usage HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let usage_body: serde_json::Value = serde_json::from_str(&usage.body).expect("usage json");

        assert_eq!(usage.status_code, 200);
        assert_eq!(usage_body["scope_type"], "run");
        assert_eq!(usage_body["scope_id"], "run_1");
        assert_eq!(usage_body["total_tokens"], 2000);
        assert_eq!(usage_body["cost_usd"], 8.5);
        assert_eq!(usage_body["records"][0]["id"], "usage_1");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn forecasts_budget_runway_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-budget-forecast");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_budget(crate::state_store::UpsertBudgetInput {
                scope_type: "project".to_string(),
                scope_id: Some("proj_local".to_string()),
                max_usd: 20.0,
                warn_pct: 0.8,
                hard_limit: true,
            })
            .expect("budget persists");
        for (run_id, cost_usd) in [("run_1", 5.0), ("run_2", 7.0)] {
            store
                .record_token_usage(crate::state_store::TokenUsageInput {
                    project_id: Some("proj_local".to_string()),
                    session_id: None,
                    task_id: None,
                    run_id: Some(run_id.to_string()),
                    agent_profile_id: Some("agent_codex".to_string()),
                    provider: "openai".to_string(),
                    model: "gpt-5.4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 1000,
                    cost_usd,
                    source: "adapter".to_string(),
                })
                .expect("usage records");
        }

        let forecast = handle_http_request(
            "GET /v1/budgets/forecast HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let forecast_body: serde_json::Value =
            serde_json::from_str(&forecast.body).expect("forecast json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(forecast.status_code, 200);
        assert_eq!(forecast_body["projects"][0]["scope_id"], "proj_local");
        assert_eq!(forecast_body["projects"][0]["used_usd"], 12.0);
        assert_eq!(forecast_body["projects"][0]["remaining_usd"], 8.0);
        assert_eq!(forecast_body["projects"][0]["average_run_cost_usd"], 6.0);
        assert_eq!(forecast_body["projects"][0]["estimated_runs_remaining"], 1);
        assert_eq!(forecast_body["projects"][0]["run_sample_count"], 2);
        assert_eq!(
            state_body["budgets"]["forecasts"]["projects"][0]["average_run_cost_usd"],
            6.0
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_single_run_for_cli_open() {
        let db_path = unique_test_db_path("control-run-open");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_ready".to_string(),
                key: "HC-READY".to_string(),
                project_id: "proj_local".to_string(),
                title: "Open control API run".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        store
            .dispatch_run(crate::state_store::DispatchRunInput {
                task_id: "task_ready".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: Some("ctx_default".to_string()),
                workspace_path: Some("/repo/.haneulchi/worktrees/run_1".to_string()),
            })
            .expect("run dispatches");

        let response = handle_http_request(
            "GET /v1/runs/run_1 HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&response.body).expect("run json");

        assert_eq!(response.status_code, 200);
        assert_eq!(body["id"], "run_1");
        assert_eq!(body["task_id"], "task_ready");
        assert_eq!(body["lifecycle"], "queued");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_per_session_token_usage_in_state_and_session_endpoint() {
        let db_path = unique_test_db_path("control-session-token-usage");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .create_session(crate::state_store::CreateSessionInput {
                project_id: "proj_local".to_string(),
                mode: "agent".to_string(),
                title: "Codex run".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                agent_profile_id: Some("agent_codex".to_string()),
                task_id: None,
                run_id: None,
            })
            .expect("session persists");
        store
            .record_token_usage(crate::state_store::TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: Some("session_1".to_string()),
                task_id: None,
                run_id: None,
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 1200,
                output_tokens: 800,
                cost_usd: 8.5,
                source: "adapter".to_string(),
            })
            .expect("usage records");

        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let session_usage = handle_http_request(
            "GET /v1/sessions/session_1/token-usage HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");
        let usage_body: serde_json::Value =
            serde_json::from_str(&session_usage.body).expect("usage json");

        assert_eq!(state.status_code, 200);
        assert_eq!(
            state_body["sessions"][0]["token_usage"]["input_tokens"],
            1200
        );
        assert_eq!(
            state_body["sessions"][0]["token_usage"]["output_tokens"],
            800
        );
        assert_eq!(state_body["sessions"][0]["token_usage"]["cost_usd"], 8.5);
        assert_eq!(session_usage.status_code, 200);
        assert_eq!(usage_body["session_id"], "session_1");
        assert_eq!(usage_body["total_tokens"], 2000);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn serves_knowledge_vault_pages_context_packs_and_snapshot_summary() {
        let db_path = unique_test_db_path("control-knowledge");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let source = handle_http_request(
            "POST /v1/knowledge/sources HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"kind\":\"file\",\"pathOrRef\":\"docs/auth.md\",\"fingerprint\":\"sha256:abc\",\"status\":\"current\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let page = handle_http_request(
            "POST /v1/knowledge/pages HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"slug\":\"auth-flow\",\"title\":\"Auth Flow\",\"bodyMd\":\"# Auth Flow\\n\\nToken rotation notes. See [[JWT rotation]].\",\"sourceIds\":[\"ks_1\"],\"freshnessState\":\"current\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_source = handle_http_request(
            "POST /v1/knowledge/sources HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"kind\":\"file\",\"pathOrRef\":\"docs/authz.md\",\"fingerprint\":\"sha256:def\",\"status\":\"current\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_page = handle_http_request(
            "POST /v1/knowledge/pages HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"slug\":\"authz-flow\",\"title\":\"AuthZ Flow\",\"bodyMd\":\"# AuthZ Flow\\n\\nToken delegation notes.\",\"sourceIds\":[\"ks_2\"],\"freshnessState\":\"current\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let pack = handle_http_request(
            "POST /v1/context-packs HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"id\":\"ctx_auth\",\"projectId\":\"proj_local\",\"name\":\"auth-default\",\"description\":\"Auth docs\",\"sourcesJson\":[{\"type\":\"knowledge_page\",\"id\":\"kp_1\"}],\"maxTokensHint\":24000}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let lint = handle_http_request(
            "POST /v1/knowledge/lint HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"staleCount\":2,\"gapCount\":1,\"contradictionCount\":0,\"bodyMd\":\"Gap: rollback\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let exploration = handle_http_request(
            "POST /v1/knowledge/explorations HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"title\":\"Token rollout investigation\",\"question\":\"How should rollback handle token rotation?\",\"answerMd\":\"Keep both issuers during rollback.\",\"pageIds\":[\"kp_1\"],\"contextPackId\":\"ctx_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let search = handle_http_request(
            "GET /v1/knowledge?query=token HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_search = handle_http_request(
            "GET /v1/knowledge?projectId=proj_auth&query=token HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let source_index = handle_http_request(
            "GET /v1/knowledge/sources HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let context_packs = handle_http_request(
            "GET /v1/context-packs HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let explorations = handle_http_request(
            "GET /v1/knowledge/explorations?projectId=proj_local HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let concepts = handle_http_request(
            "GET /v1/knowledge/concepts?projectId=proj_local HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let obsidian_export = handle_http_request(
            "POST /v1/knowledge/obsidian/export HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let chat = handle_http_request(
            "POST /v1/knowledge/chat HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"question\":\"How should token rollback work?\",\"contextPackId\":\"ctx_auth\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let opened_page = handle_http_request(
            "GET /v1/knowledge/kp_1 HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let opened_context_pack = handle_http_request(
            "GET /v1/context-packs/ctx_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let source_body: serde_json::Value =
            serde_json::from_str(&source.body).expect("source json");
        let page_body: serde_json::Value = serde_json::from_str(&page.body).expect("page json");
        let auth_source_body: serde_json::Value =
            serde_json::from_str(&auth_source.body).expect("auth source json");
        let auth_page_body: serde_json::Value =
            serde_json::from_str(&auth_page.body).expect("auth page json");
        let pack_body: serde_json::Value = serde_json::from_str(&pack.body).expect("pack json");
        let lint_body: serde_json::Value = serde_json::from_str(&lint.body).expect("lint json");
        let exploration_body: serde_json::Value =
            serde_json::from_str(&exploration.body).expect("exploration json");
        let search_body: serde_json::Value =
            serde_json::from_str(&search.body).expect("search json");
        let auth_search_body: serde_json::Value =
            serde_json::from_str(&auth_search.body).expect("auth search json");
        let source_index_body: serde_json::Value =
            serde_json::from_str(&source_index.body).expect("source index json");
        let context_packs_body: serde_json::Value =
            serde_json::from_str(&context_packs.body).expect("context packs json");
        let explorations_body: serde_json::Value =
            serde_json::from_str(&explorations.body).expect("explorations json");
        let concepts_body: serde_json::Value =
            serde_json::from_str(&concepts.body).expect("concepts json");
        let obsidian_export_body: serde_json::Value =
            serde_json::from_str(&obsidian_export.body).expect("obsidian export json");
        let chat_body: serde_json::Value = serde_json::from_str(&chat.body).expect("chat json");
        let opened_page_body: serde_json::Value =
            serde_json::from_str(&opened_page.body).expect("opened page json");
        let opened_context_pack_body: serde_json::Value =
            serde_json::from_str(&opened_context_pack.body).expect("opened context pack json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(source.status_code, 201);
        assert_eq!(source_body["id"], "ks_1");
        assert_eq!(page.status_code, 201);
        assert_eq!(page_body["slug"], "auth-flow");
        assert_eq!(auth_source.status_code, 201);
        assert_eq!(auth_source_body["id"], "ks_2");
        assert_eq!(auth_page.status_code, 201);
        assert_eq!(auth_page_body["slug"], "authz-flow");
        assert_eq!(pack.status_code, 201);
        assert_eq!(
            pack_body["sources_json"]["budget"]["max_tokens_hint"],
            24000
        );
        assert_eq!(lint.status_code, 201);
        assert_eq!(lint_body["gap_count"], 1);
        assert_eq!(exploration.status_code, 201);
        assert_eq!(exploration_body["id"], "kexp_1");
        assert_eq!(exploration_body["page_ids"][0], "kp_1");
        assert_eq!(search.status_code, 200);
        assert_eq!(search_body["items"][0]["slug"], "auth-flow");
        assert_eq!(search_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(auth_search.status_code, 200);
        assert_eq!(auth_search_body["items"][0]["slug"], "authz-flow");
        assert_eq!(auth_search_body["items"].as_array().unwrap().len(), 1);
        assert_eq!(source_index.status_code, 200);
        assert_eq!(source_index_body["items"][0]["path_or_ref"], "docs/auth.md");
        assert_eq!(context_packs.status_code, 200);
        assert_eq!(context_packs_body["items"][0]["id"], "ctx_auth");
        assert_eq!(explorations.status_code, 200);
        assert_eq!(
            explorations_body["items"][0]["question"],
            "How should rollback handle token rotation?"
        );
        assert_eq!(concepts.status_code, 200);
        assert_eq!(concepts_body["items"][0]["slug"], "auth-flow");
        assert_eq!(
            concepts_body["items"][0]["outbound_slugs"][0],
            "jwt-rotation"
        );
        assert_eq!(concepts_body["items"][1]["inbound_page_ids"][0], "kp_1");
        assert_eq!(obsidian_export.status_code, 201);
        assert_eq!(obsidian_export_body["status"], "exported");
        assert_eq!(obsidian_export_body["file_count"], 2);
        assert_eq!(obsidian_export_body["files"][0], "Auth Flow.md");
        assert_eq!(chat.status_code, 201);
        assert_eq!(chat_body["source_count"], 1);
        assert_eq!(chat_body["cited_page_ids"][0], "kp_1");
        assert!(chat_body["answer_md"]
            .as_str()
            .expect("chat answer markdown")
            .contains("Local knowledge answer draft"));
        assert_eq!(opened_page.status_code, 200);
        assert_eq!(
            opened_page_body["body_md"],
            "# Auth Flow\n\nToken rotation notes. See [[JWT rotation]]."
        );
        assert_eq!(opened_context_pack.status_code, 200);
        assert_eq!(opened_context_pack_body["id"], "ctx_auth");
        assert_eq!(state_body["knowledge"]["stale_count"], 2);
        assert_eq!(state_body["knowledge"]["gap_count"], 1);
        assert_eq!(state_body["knowledge"]["recent_pages"][0], "auth-flow");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_knowledge_automation_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-knowledge-automation");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_knowledge_source(crate::state_store::UpsertKnowledgeSourceInput {
                project_id: "proj_local".to_string(),
                kind: "file".to_string(),
                path_or_ref: "docs/stale.md".to_string(),
                fingerprint: "sha256:stale".to_string(),
                status: "stale".to_string(),
            })
            .expect("source persists");
        store
            .save_knowledge_page(crate::state_store::SaveKnowledgePageInput {
                project_id: "proj_local".to_string(),
                slug: "gap-page".to_string(),
                title: "Gap Page".to_string(),
                body_md: "# Gap Page".to_string(),
                source_ids: vec![],
                freshness_state: "current".to_string(),
            })
            .expect("page persists");

        let automation = handle_http_request(
            "POST /v1/knowledge/automation/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"watch\":true}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let automation_body: serde_json::Value =
            serde_json::from_str(&automation.body).expect("automation json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(automation.status_code, 201);
        assert_eq!(automation_body["status"], "compiled");
        assert_eq!(automation_body["watch_enabled"], true);
        assert_eq!(automation_body["source_count"], 1);
        assert_eq!(automation_body["page_count"], 1);
        assert_eq!(automation_body["lint_report_id"], "klr_1");
        assert_eq!(state_body["knowledge"]["stale_count"], 1);
        assert_eq!(state_body["knowledge"]["gap_count"], 1);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn knowledge_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-knowledge-mutation-snapshot-id");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };
        let requests = [
            (
                "knowledge source",
                201,
                "POST /v1/knowledge/sources HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"kind\":\"file\",\"pathOrRef\":\"docs/snapshot.md\",\"fingerprint\":\"sha256:snapshot\",\"status\":\"current\"}",
            ),
            (
                "knowledge page",
                201,
                "POST /v1/knowledge/pages HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"slug\":\"snapshot-page\",\"title\":\"Snapshot Page\",\"bodyMd\":\"# Snapshot Page\",\"sourceIds\":[\"ks_1\"],\"freshnessState\":\"current\"}",
            ),
            (
                "context pack",
                201,
                "POST /v1/context-packs HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"id\":\"ctx_snapshot\",\"projectId\":\"proj_local\",\"name\":\"Snapshot Context\",\"description\":\"Snapshot context pack\",\"sourcesJson\":[{\"type\":\"knowledge_page\",\"id\":\"kp_1\"}],\"maxTokensHint\":1200}",
            ),
            (
                "knowledge exploration",
                201,
                "POST /v1/knowledge/explorations HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"title\":\"Snapshot Exploration\",\"question\":\"How is snapshot parity verified?\",\"answerMd\":\"By checking snapshot receipts.\",\"pageIds\":[\"kp_1\"],\"contextPackId\":\"ctx_snapshot\"}",
            ),
            (
                "knowledge obsidian export",
                201,
                "POST /v1/knowledge/obsidian/export HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\"}",
            ),
            (
                "knowledge chat",
                201,
                "POST /v1/knowledge/chat HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"question\":\"What is snapshot parity?\",\"contextPackId\":\"ctx_snapshot\"}",
            ),
            (
                "knowledge lint",
                201,
                "POST /v1/knowledge/lint HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"staleCount\":1,\"gapCount\":0,\"contradictionCount\":0,\"bodyMd\":\"One stale page.\"}",
            ),
            (
                "knowledge automation",
                201,
                "POST /v1/knowledge/automation/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"watch\":false}",
            ),
            (
                "knowledge ingest",
                201,
                "POST /v1/knowledge/ingest HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"kind\":\"markdown\",\"pathOrRef\":\"docs/ingested.md\",\"title\":\"Ingested Snapshot\",\"bodyMd\":\"Ingested body\",\"maxChunkChars\":1000}",
            ),
        ];

        for (name, expected_status, request) in requests {
            let response = handle_http_request(request, &store, pty.clone(), "0.1.0-test");
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("mutation response json");

            assert_eq!(response.status_code, expected_status, "{name}");
            assert!(
                body["snapshot_id"]
                    .as_str()
                    .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
                "{name} should include a snapshot_id: {}",
                response.body
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn ingests_long_document_artifacts_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-knowledge-ingestion");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let ingestion = handle_http_request(
            "POST /v1/knowledge/ingest HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"kind\":\"pdf\",\"pathOrRef\":\"docs/release-runbook.pdf\",\"title\":\"Release Runbook\",\"bodyMd\":\"Release runbook body\",\"maxChunkChars\":1200}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let ingestion_body: serde_json::Value =
            serde_json::from_str(&ingestion.body).expect("ingestion json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(ingestion.status_code, 201);
        assert_eq!(ingestion_body["modality"], "pdf");
        assert_eq!(ingestion_body["source_id"], "ks_1");
        assert_eq!(ingestion_body["page_id"], "kp_1");
        assert_eq!(
            state_body["knowledge"]["recent_pages"][0],
            "release-runbook"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn mutates_run_lifecycle_cancel_and_retry_through_control_api() {
        let db_path = unique_test_db_path("control-run-lifecycle");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_run_mutation".to_string(),
                key: "HC-RUN-MUT".to_string(),
                project_id: "proj_local".to_string(),
                title: "Mutate control API run".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        let _ = handle_http_request(
            "POST /v1/dispatch HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 88\r\n\r\n{\"taskId\":\"task_run_mutation\",\"agentProfileId\":\"agent_codex\",\"contextPackId\":\"ctx_default\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let review_ready = handle_http_request(
            "POST /v1/runs/run_1/transition HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"lifecycle\":\"review_ready\",\"statusDetail\":\"Evidence pack is ready\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let cancel = handle_http_request(
            "POST /v1/runs/run_1/cancel HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let retry = handle_http_request(
            "POST /v1/runs/run_1/retry HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let review_body: serde_json::Value =
            serde_json::from_str(&review_ready.body).expect("review json");
        let cancel_body: serde_json::Value =
            serde_json::from_str(&cancel.body).expect("cancel json");
        let retry_body: serde_json::Value = serde_json::from_str(&retry.body).expect("retry json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(review_ready.status_code, 200);
        assert_eq!(review_body["lifecycle"], "review_ready");
        assert_eq!(review_body["status_detail"], "Evidence pack is ready");
        assert_eq!(cancel.status_code, 200);
        assert_eq!(cancel_body["lifecycle"], "cancelled");
        assert_eq!(retry.status_code, 200);
        assert_eq!(retry_body["lifecycle"], "queued");
        assert_eq!(retry_body["retry_count"], 1);
        assert_eq!(state_body["runs"]["counts_by_lifecycle"]["queued"], 1);
        assert_eq!(state_body["tasks"]["items"][0]["status"], "running");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn run_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-run-mutation-snapshot-id");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_run_receipt".to_string(),
                key: "HC-RUN-RECEIPT".to_string(),
                project_id: "proj_local".to_string(),
                title: "Run mutation receipt".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };

        let dispatch = handle_http_request(
            "POST /v1/dispatch HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"taskId\":\"task_run_receipt\",\"agentProfileId\":\"agent_codex\",\"contextPackId\":\"ctx_default\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let transition = handle_http_request(
            "POST /v1/runs/run_1/transition HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"lifecycle\":\"review_ready\",\"statusDetail\":\"Evidence pack is ready\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let cancel = handle_http_request(
            "POST /v1/runs/run_1/cancel HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let retry = handle_http_request(
            "POST /v1/runs/run_1/retry HTTP/1.1\r\nhost: haneulchi\r\n\r\n{}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let status = handle_http_request(
            "POST /v1/runs/run_1/status-updates HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"bodyMd\":\"Investigating fixture failure.\",\"lifecycle\":\"waiting_input\",\"statusDetail\":\"Needs fixture account\"}",
            &store,
            pty,
            "0.1.0-test",
        );
        let responses = [
            ("dispatch", 201, dispatch),
            ("transition", 200, transition),
            ("cancel", 200, cancel),
            ("retry", 200, retry),
            ("status", 201, status),
        ];

        for (name, expected_status, response) in responses {
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("run mutation response json");

            assert_eq!(response.status_code, expected_status, "{name}");
            assert!(
                body["snapshot_id"]
                    .as_str()
                    .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
                "{name} should include a snapshot_id: {}",
                response.body
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn records_agent_status_updates_through_control_api_and_state() {
        let db_path = unique_test_db_path("control-agent-status-update");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_agent_update_api".to_string(),
                key: "HC-AGENT-UPDATE-API".to_string(),
                title: "Agent API update task".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                project_id: "proj_local".to_string(),
                assignee_type: Some("agent".to_string()),
                assignee_id: Some("agent_codex".to_string()),
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        store
            .dispatch_run(crate::state_store::DispatchRunInput {
                task_id: "task_agent_update_api".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");

        let update = handle_http_request(
            "POST /v1/runs/run_1/status-updates HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"bodyMd\":\"Investigating OAuth fixture failure.\",\"lifecycle\":\"waiting_input\",\"statusDetail\":\"Needs OAuth test account\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );

        let update_body: serde_json::Value =
            serde_json::from_str(&update.body).expect("update json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(update.status_code, 201);
        assert_eq!(update_body["author_type"], "agent");
        assert_eq!(update_body["author_id"], "agent_codex");
        assert_eq!(update_body["run_id"], "run_1");
        assert_eq!(
            update_body["body_md"],
            "Investigating OAuth fixture failure."
        );
        assert_eq!(state_body["tasks"]["items"][0]["comment_count"], 1);
        assert_eq!(state_body["runs"]["items"][0]["lifecycle"], "waiting_input");
        assert_eq!(
            state_body["runs"]["items"][0]["status_detail"],
            "Needs OAuth test account"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn reloads_workflow_through_control_api_and_state_snapshot() {
        let db_path = unique_test_db_path("control-workflow");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let valid = handle_http_request(
            "POST /v1/workflow/reload HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sourcePath\":\"/repo/WORKFLOW.md\",\"content\":\"---\\nhaneulchi: 1\\nproject:\\n  key: AUTH\\n  default_branch: main\\nworkspace:\\n  strategy: worktree\\n  base_root: .haneulchi/worktrees\\nhooks:\\n  before_run: .haneulchi/hooks/before_run.sh\\n---\\nUse {task.id}.\\n\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_valid = handle_http_request(
            "POST /v1/workflow/reload HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_auth\",\"sourcePath\":\"/repo/WORKFLOW.md\",\"content\":\"---\\nhaneulchi: 1\\nproject:\\n  key: AUTH\\nworkspace:\\n  strategy: worktree\\n---\\nUse {task.id}.\\n\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let invalid = handle_http_request(
            "POST /v1/workflow/reload HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sourcePath\":\"/repo/WORKFLOW.md\",\"content\":\"---\\nhaneulchi: 1\\nproject:\\n  key: AUTH\\nworkspace:\\n  strategy: worktree\\nhooks:\\n  before_run: ../escape.sh\\n---\\nUse {secret.token}.\\n\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let validate = handle_http_request(
            "POST /v1/workflow/validate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sourcePath\":\"/repo/WORKFLOW.md\",\"content\":\"---\\nhaneulchi: 1\\nproject:\\n  key: AUTH\\nworkspace:\\n  strategy: worktree\\n---\\nUse {task.id}.\\n\"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let auth_status = handle_http_request(
            "GET /v1/workflow/status?projectId=proj_auth HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let state = handle_http_request(
            "GET /v1/state HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let valid_body: serde_json::Value =
            serde_json::from_str(&valid.body).expect("valid workflow json");
        let auth_valid_body: serde_json::Value =
            serde_json::from_str(&auth_valid.body).expect("auth valid workflow json");
        let invalid_body: serde_json::Value =
            serde_json::from_str(&invalid.body).expect("invalid workflow json");
        let validate_body: serde_json::Value =
            serde_json::from_str(&validate.body).expect("validate workflow json");
        let auth_status_body: serde_json::Value =
            serde_json::from_str(&auth_status.body).expect("auth workflow status json");
        let state_body: serde_json::Value = serde_json::from_str(&state.body).expect("state json");

        assert_eq!(valid.status_code, 201);
        assert_eq!(valid_body["valid"], true);
        assert_eq!(auth_valid.status_code, 201);
        assert_eq!(auth_valid_body["project_id"], "proj_auth");
        assert_eq!(invalid.status_code, 201);
        assert_eq!(invalid_body["valid"], false);
        assert_eq!(validate.status_code, 200);
        assert_eq!(validate_body["valid"], true);
        assert_eq!(validate_body["project_id"], "proj_local");
        assert_eq!(auth_status.status_code, 200);
        assert_eq!(auth_status_body["project_id"], "proj_auth");
        assert_eq!(auth_status_body["valid"], true);
        assert_eq!(state_body["workflow"]["valid"], false);
        assert_eq!(
            state_body["workflow"]["current_version_id"],
            invalid_body["id"]
        );
        assert_eq!(
            state_body["workflow"]["last_known_good_version_id"],
            valid_body["id"]
        );
        assert!(state_body["workflow"]["diagnostics"]["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| error["code"] == "hook_path_escapes_repo"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_workflow_hook_through_control_api() {
        let db_path = unique_test_db_path("control-hook-run");
        let root = db_path.parent().unwrap().join("repo");
        let hook_dir = root.join(".haneulchi/hooks");
        let workspace = root.join(".haneulchi/worktrees/run_1");
        fs::create_dir_all(&hook_dir).expect("hook dir");
        fs::create_dir_all(&workspace).expect("workspace dir");
        let hook_path = hook_dir.join("before_run.sh");
        fs::write(&hook_path, "#!/bin/sh\nprintf 'control-hook %s %s\\n' \"$HANEULCHI_RUN_ID\" \"$HANEULCHI_TASK_ID\"\n")
            .expect("hook writes");
        let mut permissions = fs::metadata(&hook_path)
            .expect("hook metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook_path, permissions).expect("hook executable");

        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .reload_workflow(crate::state_store::ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: root.join("WORKFLOW.md").to_string_lossy().to_string(),
                content: "---\nhaneulchi: 1\nproject:\n  key: AUTH\nworkspace:\n  strategy: worktree\ncontext:\n  default_pack: auth-default\nhooks:\n  before_run: .haneulchi/hooks/before_run.sh\n---\nUse {task.id}.\n".to_string(),
            })
            .expect("workflow reloads");
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_hook_api".to_string(),
                key: "HC-HOOK-API".to_string(),
                project_id: "proj_local".to_string(),
                title: "Control hook run".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        store
            .dispatch_run(crate::state_store::DispatchRunInput {
                task_id: "task_hook_api".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: Some(workspace.to_string_lossy().to_string()),
            })
            .expect("run dispatches");

        let response = handle_http_request(
            &format!(
                "POST /v1/runs/run_1/hooks/before_run/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{{\"repoRoot\":\"{}\"}}",
                root.to_string_lossy()
            ),
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&response.body).expect("hook json");

        assert_eq!(response.status_code, 200);
        assert_eq!(body["status"], "completed");
        assert!(body["stdout"]
            .as_str()
            .unwrap()
            .contains("control-hook run_1 task_hook_api"));
        assert_eq!(
            body["env_json"]["HANEULCHI_CONTEXT_PACK_ID"],
            "auth-default"
        );

        let replay = handle_http_request(
            "GET /v1/runs/run_1/replay HTTP/1.1\r\nhost: haneulchi\r\n\r\n",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let replay_body: serde_json::Value =
            serde_json::from_str(&replay.body).expect("replay json");

        assert_eq!(replay.status_code, 200);
        assert_eq!(replay_body["run_id"], "run_1");
        assert_eq!(
            replay_body["body_json"]["workflow_version_id"],
            "workflow_1"
        );
        assert_eq!(
            replay_body["body_json"]["hook_results"][0]["hook_name"],
            "before_run"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn evidence_and_workflow_mutations_return_snapshot_id_receipts() {
        let db_path = unique_test_db_path("control-evidence-workflow-snapshot-id");
        let root = db_path.parent().unwrap().join("repo");
        let hook_dir = root.join(".haneulchi/hooks");
        let workspace = root.join(".haneulchi/worktrees/run_1");
        fs::create_dir_all(&hook_dir).expect("hook dir");
        fs::create_dir_all(&workspace).expect("workspace dir");
        let hook_path = hook_dir.join("before_run.sh");
        fs::write(
            &hook_path,
            "#!/bin/sh\nprintf 'snapshot-hook %s\\n' \"$HANEULCHI_RUN_ID\"\n",
        )
        .expect("hook writes");
        let mut permissions = fs::metadata(&hook_path)
            .expect("hook metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook_path, permissions).expect("hook executable");

        let store = StateStore::open_at(&db_path).expect("state store opens");
        let pty = TerminalPtySnapshot {
            total: 0,
            sessions: vec![],
        };
        let reload = handle_http_request(
            "POST /v1/workflow/reload HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sourcePath\":\"/repo/WORKFLOW.md\",\"content\":\"---\\nhaneulchi: 1\\nproject:\\n  key: AUTH\\nworkspace:\\n  strategy: worktree\\nhooks:\\n  before_run: .haneulchi/hooks/before_run.sh\\n---\\nUse {task.id}.\\n\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let validate = handle_http_request(
            "POST /v1/workflow/validate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"projectId\":\"proj_local\",\"sourcePath\":\"/repo/WORKFLOW.md\",\"content\":\"---\\nhaneulchi: 1\\nproject:\\n  key: AUTH\\nworkspace:\\n  strategy: worktree\\n---\\nUse {task.id}.\\n\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        store
            .upsert_task(&crate::state_store::PersistedTaskInput {
                id: "task_evidence_receipt".to_string(),
                key: "HC-EVIDENCE-RECEIPT".to_string(),
                project_id: "proj_local".to_string(),
                title: "Evidence receipt task".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persists");
        store
            .dispatch_run(crate::state_store::DispatchRunInput {
                task_id: "task_evidence_receipt".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: Some(workspace.to_string_lossy().to_string()),
            })
            .expect("run dispatches");
        store
            .upsert_command_block(&crate::state_store::PersistedCommandBlockInput {
                id: "cmdblk_evidence_receipt".to_string(),
                session_id: "session_evidence_receipt".to_string(),
                task_id: Some("task_evidence_receipt".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(1),
                seq_end: Some(2),
                command: "cargo test".to_string(),
                cwd: Some(root.to_string_lossy().to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(1200),
                summary: Some("tests passed".to_string()),
            })
            .expect("command block persists");

        let hook = handle_http_request(
            &format!(
                "POST /v1/runs/run_1/hooks/before_run/run HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{{\"repoRoot\":\"{}\"}}",
                root.to_string_lossy()
            ),
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let generated = handle_http_request(
            "POST /v1/runs/run_1/evidence/generate HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"evidencePackId\":\"ev_receipt\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let attached = handle_http_request(
            "POST /v1/command-blocks/cmdblk_evidence_receipt/attach-evidence HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"evidencePackId\":\"ev_receipt\",\"taskId\":\"task_evidence_receipt\",\"runId\":\"run_1\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let evidence_decision = handle_http_request(
            "POST /v1/evidence/ev_receipt/review-decision HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"decision\":\"approved\",\"reviewerId\":\"reviewer_snapshot\",\"bodyMd\":\"Approved.\"}",
            &store,
            pty.clone(),
            "0.1.0-test",
        );
        let review_decision = handle_http_request(
            "POST /v1/reviews/review_ev_receipt/decision HTTP/1.1\r\ncontent-type: application/json\r\n\r\n{\"decision\":\"reopened\",\"reviewerId\":\"reviewer_snapshot\",\"bodyMd\":\"Reopened.\"}",
            &store,
            pty,
            "0.1.0-test",
        );
        let responses = [
            ("workflow reload", 201, reload),
            ("workflow validate", 200, validate),
            ("workflow hook", 200, hook),
            ("evidence generate", 201, generated),
            ("command block attach", 201, attached),
            ("evidence decision", 200, evidence_decision),
            ("review decision", 200, review_decision),
        ];

        for (name, expected_status, response) in responses {
            let body: serde_json::Value =
                serde_json::from_str(&response.body).expect("mutation response json");

            assert_eq!(response.status_code, expected_status, "{name}");
            assert!(
                body["snapshot_id"]
                    .as_str()
                    .is_some_and(|snapshot_id| snapshot_id.starts_with("snap_")),
                "{name} should include a snapshot_id: {}",
                response.body
            );
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn returns_bad_request_for_invalid_task_payloads() {
        let db_path = unique_test_db_path("control-task-validation");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let response = handle_http_request(
            "POST /v1/tasks HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 37\r\n\r\n{\"projectId\":\"proj_local\",\"title\":\"   \"}",
            &store,
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "0.1.0-test",
        );
        let body: serde_json::Value = serde_json::from_str(&response.body).expect("error json");

        assert_eq!(response.status_code, 400);
        assert_eq!(body["error"], "task title cannot be empty");

        cleanup_test_db(&db_path);
    }

    fn unique_test_db_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("haneulchi-{label}-{nanos}"))
            .join("haneulchi.sqlite")
    }

    fn unique_test_socket_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("haneulchi-{label}-{nanos}.sock"))
    }

    fn cleanup_test_db(db_path: &PathBuf) {
        let _ = Connection::open(db_path)
            .and_then(|connection| connection.close().map_err(|(_, error)| error));
        let _ = fs::remove_file(db_path);
        if let Some(parent) = db_path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
}
