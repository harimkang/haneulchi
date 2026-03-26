use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use hc_domain::{
    AppSnapshot, AppSnapshotMeta, AppState, ClaimState, OpsSummary, OrchestratorRuntime,
    ProjectSummary, SessionFocusState, SessionRuntimeState, SessionSummary, TrackerStatus,
    WarningSummary, WorkflowHealth, WorkflowRuntimeStatus,
    inventory::{InventoryRow, InventorySummary},
    time::now_iso8601,
};
use hc_runtime::terminal::runtime::TerminalSessionSnapshot as RuntimeSessionSnapshot;
use hc_workflow::{
    LoadWorkflowRequest, PrepareBootstrapRequest, WorkflowLoader, WorkflowRuntime,
    prepare_bootstrap,
};

use crate::attention::derive_attention;
use crate::inventory::{build_inventory_for_project, build_inventory_summary};
use crate::session_projection::{focus_session, release_takeover_session, takeover_session};
use crate::shared_store::lock_shared_store;
use crate::snapshot::{SnapshotSeed, project_snapshot};
use crate::tasks::{shared_attach_session, shared_detach_session, shared_task};
use crate::workflow_projection::{sample_tracker_status, sample_workflow_status};
use crate::worktrees::{
    shared_set_worktree_pinned as worktrees_set_pinned,
    shared_update_worktree_lifecycle as worktrees_update_lifecycle,
};

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ControlPlaneError {
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("attention not found: {0}")]
    AttentionNotFound(String),
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("task claim conflict: {0}")]
    TaskClaimConflict(String),
    #[error("task project mismatch: task={task_id} session={session_id}")]
    TaskProjectMismatch { task_id: String, session_id: String },
    #[error("storage error: {0}")]
    Storage(String),
    #[error("worktree error: {0}")]
    Worktree(String),
}

pub struct ControlPlaneState {
    snapshot: AppSnapshot,
}

impl ControlPlaneState {
    pub fn from_snapshot(snapshot: AppSnapshot) -> Self {
        Self { snapshot }
    }

    pub fn sample() -> Self {
        let snapshot = project_snapshot(SnapshotSeed {
            orchestrator_runtime: OrchestratorRuntime {
                singleton_key: "main".to_string(),
                cadence_ms: 15_000,
                last_tick_at: Some(now_iso8601()),
                last_reconcile_at: None,
                max_slots: 2,
                running_slots: 2,
                workflow_state: sample_workflow_status().state.as_str().to_string(),
                tracker_state: sample_tracker_status().health.clone(),
            },
            workflow: sample_workflow_status(),
            tracker: sample_tracker_status(),
            projects: vec![ProjectSummary::new(
                "proj_demo",
                "demo",
                "/tmp/demo",
                sample_workflow_status().state,
                2,
                0,
            )],
            sessions: vec![
                SessionSummary::new("ses_01", "proj_demo", "Primary shell")
                    .with_mode("generic")
                    .with_runtime_state(SessionRuntimeState::Running)
                    .with_claim_state(ClaimState::None)
                    .with_focus_state(SessionFocusState::Focused)
                    .with_cwd("/tmp/demo")
                    .with_workspace_root("/tmp/demo")
                    .with_base_root("."),
                SessionSummary::new("ses_02", "proj_demo", "Secondary shell")
                    .with_mode("generic")
                    .with_runtime_state(SessionRuntimeState::Running)
                    .with_claim_state(ClaimState::None)
                    .with_focus_state(SessionFocusState::Background)
                    .with_cwd("/tmp/demo")
                    .with_workspace_root("/tmp/demo")
                    .with_base_root("."),
            ],
            retry_queue: vec![],
        });
        Self { snapshot }
    }

    pub fn snapshot(&self) -> &AppSnapshot {
        &self.snapshot
    }

    pub fn snapshot_mut(&mut self) -> &mut AppSnapshot {
        &mut self.snapshot
    }

    pub fn sync_from_runtime(&mut self, runtime_sessions: &[RuntimeSessionSnapshot]) {
        let previous_focus = self.snapshot.ops.app.focused_session_id.clone();
        let previous_manual: BTreeMap<String, String> = self
            .snapshot
            .sessions
            .iter()
            .map(|session| (session.session_id.clone(), session.manual_control.clone()))
            .collect();
        let previous_task_bindings: BTreeMap<String, Option<String>> = self
            .snapshot
            .sessions
            .iter()
            .map(|session| (session.session_id.clone(), session.task_id.clone()))
            .collect();
        let previous_claim_states: BTreeMap<String, ClaimState> = self
            .snapshot
            .sessions
            .iter()
            .map(|session| (session.session_id.clone(), session.claim_state))
            .collect();

        let mut sessions: Vec<SessionSummary> = runtime_sessions
            .iter()
            .map(|session| {
                let current_directory = session
                    .shell_metadata
                    .current_directory
                    .clone()
                    .or_else(|| {
                        session
                            .launch
                            .current_directory
                            .as_ref()
                            .map(|path| path.display().to_string())
                    })
                    .unwrap_or_default();
                let manual_control = previous_manual
                    .get(&session.session_id)
                    .cloned()
                    .unwrap_or_else(|| "none".to_string());
                let title = title_for_session(session, &current_directory);

                SessionSummary {
                    session_id: session.session_id.clone(),
                    project_id: current_directory.clone(),
                    task_id: previous_task_bindings
                        .get(&session.session_id)
                        .cloned()
                        .unwrap_or(None),
                    mode: session_mode_for_launch(&session.launch.program),
                    runtime_state: if session.running {
                        SessionRuntimeState::Running
                    } else {
                        SessionRuntimeState::Exited
                    },
                    manual_control: manual_control.clone(),
                    dispatch_state: "not_dispatchable".to_string(),
                    claim_state: previous_claim_states
                        .get(&session.session_id)
                        .copied()
                        .unwrap_or(ClaimState::None),
                    adapter_kind: adapter_kind_for_launch(&session.launch.program),
                    provider_id: None,
                    model_id: None,
                    dispatch_reason: None,
                    title,
                    cwd: current_directory.clone(),
                    workspace_root: current_directory.clone(),
                    base_root: ".".to_string(),
                    branch: session.shell_metadata.branch.clone(),
                    latest_summary: session.shell_metadata.last_command.clone(),
                    latest_commentary: None,
                    commentary_updated_at: None,
                    active_window_title: None,
                    unread_count: 0,
                    last_activity_at: None,
                    focus_state: if previous_focus.as_deref() == Some(session.session_id.as_str()) {
                        SessionFocusState::Focused
                    } else {
                        SessionFocusState::Background
                    },
                    can_focus: true,
                    can_takeover: session.running && manual_control == "none",
                    can_release_takeover: manual_control == "takeover",
                }
            })
            .collect();

        if !sessions.is_empty()
            && !sessions
                .iter()
                .any(|session| session.focus_state == SessionFocusState::Focused)
            && let Some(first) = sessions.first_mut()
        {
            first.focus_state = SessionFocusState::Focused;
        }

        let focused_session_id = sessions
            .iter()
            .find(|session| session.focus_state == SessionFocusState::Focused)
            .map(|session| session.session_id.clone());

        let mut project_counts: BTreeMap<String, u32> = BTreeMap::new();
        for session in &sessions {
            if !session.project_id.is_empty() {
                *project_counts
                    .entry(session.project_id.clone())
                    .or_insert(0) += 1;
            }
        }

        let projects = project_counts
            .into_iter()
            .map(|(root, session_count)| {
                let workflow = workflow_for_root(&root);
                ProjectSummary::new(
                    root.clone(),
                    project_name_from_root(&root),
                    root,
                    workflow.state,
                    session_count,
                    0,
                )
            })
            .collect::<Vec<_>>();

        let workflow = focused_session_id
            .as_ref()
            .and_then(|focused_session_id| {
                sessions
                    .iter()
                    .find(|session| &session.session_id == focused_session_id)
            })
            .map(|session| workflow_for_root(&session.project_id))
            .unwrap_or_else(empty_workflow_status);
        let tracker = TrackerStatus {
            state: "local_only".to_string(),
            last_sync_at: None,
            health: "ok".to_string(),
        };
        let retry_queue = Vec::new();
        let attention = derive_attention(&workflow, &sessions, &retry_queue);
        let queued_claim_count = sessions
            .iter()
            .filter(|session| session.claim_state == ClaimState::Claimed)
            .count() as u32;

        let mut snapshot = AppSnapshot::new(workflow.clone(), tracker.clone())
            .with_automation(OpsSummary {
                status: "running".to_string(),
                cadence_ms: 15_000,
                last_tick_at: Some(now_iso8601()),
                last_reconcile_at: None,
                running_slots: sessions
                    .iter()
                    .filter(|session| session.runtime_state == SessionRuntimeState::Running)
                    .count() as u32,
                max_slots: sessions.len().max(1) as u32,
                retry_due_count: 0,
                queued_claim_count,
                paused: false,
            })
            .with_app_state(AppState {
                active_route: "project_focus".to_string(),
                focused_session_id,
                degraded_flags: Vec::new(),
            });
        snapshot.meta = AppSnapshotMeta {
            snapshot_rev: self.snapshot.meta.snapshot_rev.saturating_add(1),
            runtime_rev: self.snapshot.meta.runtime_rev.saturating_add(1),
            projection_rev: self.snapshot.meta.projection_rev.saturating_add(1),
            snapshot_at: Some(now_iso8601()),
        };
        snapshot.projects = projects;
        snapshot.sessions = sessions;
        snapshot.attention = attention;
        snapshot.retry_queue = retry_queue;
        snapshot.warnings = Vec::<WarningSummary>::new();

        self.snapshot = snapshot;
    }

    pub fn focus_session(&mut self, session_id: &str) -> Result<(), ControlPlaneError> {
        focus_session(&mut self.snapshot, session_id)
    }

    pub fn takeover_session(&mut self, session_id: &str) -> Result<(), ControlPlaneError> {
        takeover_session(&mut self.snapshot, session_id)
    }

    pub fn release_takeover_session(&mut self, session_id: &str) -> Result<(), ControlPlaneError> {
        release_takeover_session(&mut self.snapshot, session_id)
    }

    pub fn attach_task(
        &mut self,
        session_id: &str,
        task_id: &str,
    ) -> Result<(), ControlPlaneError> {
        let session_index = self
            .snapshot
            .sessions
            .iter()
            .position(|session| session.session_id == session_id)
            .ok_or_else(|| ControlPlaneError::SessionNotFound(session_id.to_string()))?;
        let task = shared_task(task_id)
            .map_err(|_| ControlPlaneError::TaskNotFound(task_id.to_string()))?
            .ok_or_else(|| ControlPlaneError::TaskNotFound(task_id.to_string()))?;

        if !task.project_id.is_empty()
            && !self.snapshot.sessions[session_index].project_id.is_empty()
            && self.snapshot.sessions[session_index].project_id != task.project_id
        {
            return Err(ControlPlaneError::TaskProjectMismatch {
                task_id: task_id.to_string(),
                session_id: session_id.to_string(),
            });
        }

        if self.snapshot.sessions.iter().any(|session| {
            session.session_id != session_id
                && session.task_id.as_deref() == Some(task_id)
                && is_live_session(session.runtime_state)
        }) {
            return Err(ControlPlaneError::TaskClaimConflict(task_id.to_string()));
        }

        if let Some(previous_task_id) = self.snapshot.sessions[session_index].task_id.clone()
            && previous_task_id != task_id
        {
            let _ = shared_detach_session(&previous_task_id);
        }

        shared_attach_session(task_id, session_id)
            .map_err(|_| ControlPlaneError::TaskNotFound(task_id.to_string()))?;
        self.snapshot.sessions[session_index].task_id = Some(task_id.to_string());
        self.snapshot.sessions[session_index].claim_state = ClaimState::Claimed;
        bump_projection_meta(&mut self.snapshot);

        Ok(())
    }

    pub fn detach_task(&mut self, session_id: &str) -> Result<(), ControlPlaneError> {
        let session_index = self
            .snapshot
            .sessions
            .iter()
            .position(|session| session.session_id == session_id)
            .ok_or_else(|| ControlPlaneError::SessionNotFound(session_id.to_string()))?;
        let Some(task_id) = self.snapshot.sessions[session_index].task_id.clone() else {
            return Ok(());
        };

        shared_detach_session(&task_id)
            .map_err(|_| ControlPlaneError::TaskNotFound(task_id.clone()))?;
        self.snapshot.sessions[session_index].task_id = None;
        self.snapshot.sessions[session_index].claim_state = ClaimState::None;
        bump_projection_meta(&mut self.snapshot);

        Ok(())
    }

    pub fn resolve_attention(&mut self, attention_id: &str) -> Result<(), ControlPlaneError> {
        if !crate::attention::resolve_attention(&mut self.snapshot, attention_id) {
            return Err(ControlPlaneError::AttentionNotFound(
                attention_id.to_string(),
            ));
        }
        bump_projection_meta(&mut self.snapshot);
        Ok(())
    }

    pub fn dismiss_attention(&mut self, attention_id: &str) -> Result<(), ControlPlaneError> {
        if !crate::attention::dismiss_attention(&mut self.snapshot, attention_id) {
            return Err(ControlPlaneError::AttentionNotFound(
                attention_id.to_string(),
            ));
        }
        bump_projection_meta(&mut self.snapshot);
        Ok(())
    }

    pub fn snooze_attention(&mut self, attention_id: &str) -> Result<(), ControlPlaneError> {
        if !crate::attention::snooze_attention(&mut self.snapshot, attention_id) {
            return Err(ControlPlaneError::AttentionNotFound(
                attention_id.to_string(),
            ));
        }
        bump_projection_meta(&mut self.snapshot);
        Ok(())
    }
}

impl Default for ControlPlaneState {
    fn default() -> Self {
        Self {
            snapshot: {
                let mut snapshot = AppSnapshot::new(
                    empty_workflow_status(),
                    TrackerStatus {
                        state: "local_only".to_string(),
                        last_sync_at: None,
                        health: "ok".to_string(),
                    },
                )
                .with_automation(OpsSummary {
                    status: "running".to_string(),
                    cadence_ms: 15_000,
                    last_tick_at: Some(now_iso8601()),
                    last_reconcile_at: None,
                    running_slots: 0,
                    max_slots: 1,
                    retry_due_count: 0,
                    queued_claim_count: 0,
                    paused: false,
                })
                .with_app_state(AppState {
                    active_route: "project_focus".to_string(),
                    focused_session_id: None,
                    degraded_flags: Vec::new(),
                });
                snapshot.meta = AppSnapshotMeta {
                    snapshot_rev: 1,
                    runtime_rev: 1,
                    projection_rev: 1,
                    snapshot_at: Some(now_iso8601()),
                };
                snapshot
            },
        }
    }
}

pub fn shared_inventory_for_project(
    project_id: &str,
) -> Result<Vec<InventoryRow>, ControlPlaneError> {
    let store =
        lock_shared_store().map_err(|error| ControlPlaneError::Storage(error.to_string()))?;
    build_inventory_for_project(&store, project_id)
        .map_err(|error| ControlPlaneError::Storage(error.to_string()))
}

pub fn shared_inventory_summary(project_id: &str) -> Result<InventorySummary, ControlPlaneError> {
    let rows = shared_inventory_for_project(project_id)?;
    Ok(build_inventory_summary(&rows))
}

pub fn shared_set_worktree_pinned(
    worktree_id: &str,
    is_pinned: bool,
) -> Result<(), ControlPlaneError> {
    worktrees_set_pinned(worktree_id, is_pinned)
        .map_err(|error| ControlPlaneError::Worktree(error.to_string()))
}

pub fn shared_update_worktree_lifecycle(
    worktree_id: &str,
    new_state: &str,
) -> Result<(), ControlPlaneError> {
    worktrees_update_lifecycle(worktree_id, new_state)
        .map_err(|error| ControlPlaneError::Worktree(error.to_string()))
}

pub fn validate_workflow(
    repo_root: impl Into<PathBuf>,
) -> Result<Option<hc_workflow::LoadedWorkflow>, hc_workflow::WorkflowError> {
    WorkflowLoader::load(&LoadWorkflowRequest {
        repo_root: repo_root.into(),
        explicit_workflow_path: None,
    })
}

pub fn reload_workflow(
    repo_root: impl Into<PathBuf>,
) -> Result<WorkflowRuntime, hc_workflow::WorkflowError> {
    let repo_root = repo_root.into();
    let key = repo_root.display().to_string();
    let runtimes = workflow_runtimes();
    let mut runtimes = runtimes.lock().expect("workflow runtime cache lock");
    let runtime = runtimes.entry(key).or_insert_with(|| {
        WorkflowRuntime::new(LoadWorkflowRequest {
            repo_root: repo_root.clone(),
            explicit_workflow_path: None,
        })
    });
    match runtime.reload() {
        Ok(()) => Ok(runtime.clone()),
        Err(_error) if runtime.state() == hc_workflow::WorkflowState::InvalidKeptLastGood => {
            Ok(runtime.clone())
        }
        Err(error) => Err(error),
    }
}

pub fn prepare_isolated_launch(
    repo_root: impl Into<PathBuf>,
    project_name: &str,
    task_id: &str,
    task_title: &str,
    workspace_root: impl Into<PathBuf>,
) -> Result<hc_workflow::BootstrapStatusSummary, String> {
    let repo_root = repo_root.into();
    let workspace_root = workspace_root.into();
    let key = repo_root.display().to_string();
    let runtimes = workflow_runtimes();
    let mut runtimes = runtimes.lock().expect("workflow runtime cache lock");
    let runtime = runtimes.entry(key).or_insert_with(|| {
        WorkflowRuntime::new(LoadWorkflowRequest {
            repo_root: repo_root.clone(),
            explicit_workflow_path: None,
        })
    });

    if runtime.current().is_none()
        && runtime.last_known_good().is_none()
        && runtime.last_error().is_none()
    {
        match runtime.reload() {
            Ok(()) => {}
            Err(_error) if runtime.state() == hc_workflow::WorkflowState::InvalidKeptLastGood => {}
            Err(error) => return Err(error.to_string()),
        }
    }

    let Some(workflow) = runtime.current().cloned() else {
        return Ok(hc_workflow::BootstrapStatusSummary {
            workspace_root: workspace_root.display().to_string(),
            base_root: ".".to_string(),
            session_cwd: workspace_root.display().to_string(),
            rendered_prompt_path: String::new(),
            phase_sequence: vec![
                "resolve".to_string(),
                "normalize".to_string(),
                "workspace".to_string(),
                "paths".to_string(),
            ],
            hook_phase_results: Vec::new(),
            outcome_code: "launch_prepared".to_string(),
            warning_codes: Vec::new(),
            claim_released: false,
            launch_exit_code: None,
            last_known_good_hash: runtime
                .last_known_good()
                .map(|loaded| loaded.contract_hash.clone()),
        });
    };

    let summary = prepare_bootstrap(PrepareBootstrapRequest {
        workflow,
        project_name: project_name.to_string(),
        task_id: task_id.to_string(),
        task_title: task_title.to_string(),
        repo_root,
        workspace_root,
    })?;
    runtime.record_bootstrap(summary.clone());
    Ok(summary)
}

fn workflow_runtimes() -> &'static Mutex<std::collections::HashMap<String, WorkflowRuntime>> {
    static RUNTIMES: OnceLock<Mutex<std::collections::HashMap<String, WorkflowRuntime>>> =
        OnceLock::new();
    RUNTIMES.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn session_mode_for_launch(program: &str) -> String {
    match program {
        "/bin/zsh" | "/bin/bash" => "generic".to_string(),
        _ => "preset".to_string(),
    }
}

fn adapter_kind_for_launch(program: &str) -> Option<String> {
    match program {
        "/bin/zsh" | "/bin/bash" => None,
        other => Some(other.to_string()),
    }
}

fn title_for_session(session: &RuntimeSessionSnapshot, current_directory: &str) -> String {
    if !current_directory.is_empty() {
        Path::new(current_directory)
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .unwrap_or(&session.launch.program)
            .to_string()
    } else {
        session.launch.program.clone()
    }
}

fn project_name_from_root(root: &str) -> String {
    Path::new(root)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("workspace")
        .to_string()
}

fn empty_workflow_status() -> WorkflowRuntimeStatus {
    WorkflowRuntimeStatus {
        state: WorkflowHealth::None,
        path: String::new(),
        last_good_hash: None,
        last_reload_at: None,
        last_error: None,
    }
}

fn workflow_for_root(root: &str) -> WorkflowRuntimeStatus {
    if root.is_empty() {
        return empty_workflow_status();
    }

    let runtime = workflow_runtime_for_root(root);
    let workflow_path = PathBuf::from(root).join("WORKFLOW.md");
    let path = runtime
        .current()
        .map(|loaded| loaded.discovery_path.display().to_string())
        .unwrap_or_else(|| workflow_path.display().to_string());

    let state = match runtime.state() {
        hc_workflow::WorkflowState::None if runtime.last_error().is_some() => {
            WorkflowHealth::InvalidKeptLastGood
        }
        hc_workflow::WorkflowState::None => WorkflowHealth::None,
        hc_workflow::WorkflowState::Ok => WorkflowHealth::Ok,
        hc_workflow::WorkflowState::InvalidKeptLastGood => WorkflowHealth::InvalidKeptLastGood,
        hc_workflow::WorkflowState::ReloadPending => WorkflowHealth::ReloadPending,
    };

    WorkflowRuntimeStatus {
        state,
        path,
        last_good_hash: runtime
            .last_known_good()
            .map(|loaded| loaded.contract_hash.clone()),
        last_reload_at: runtime.last_reload_at().map(str::to_string),
        last_error: runtime.last_error().map(str::to_string),
    }
}

fn is_live_session(state: SessionRuntimeState) -> bool {
    matches!(
        state,
        SessionRuntimeState::Launching
            | SessionRuntimeState::Running
            | SessionRuntimeState::WaitingInput
            | SessionRuntimeState::ReviewReady
            | SessionRuntimeState::Blocked
    )
}

fn bump_projection_meta(snapshot: &mut AppSnapshot) {
    snapshot.meta.snapshot_rev = snapshot.meta.snapshot_rev.saturating_add(1);
    snapshot.meta.projection_rev = snapshot.meta.projection_rev.saturating_add(1);
}

fn shared_control_plane() -> &'static Mutex<ControlPlaneState> {
    static CONTROL_PLANE: OnceLock<Mutex<ControlPlaneState>> = OnceLock::new();
    CONTROL_PLANE.get_or_init(|| Mutex::new(ControlPlaneState::default()))
}

pub fn lock_shared_control_plane()
-> Result<std::sync::MutexGuard<'static, ControlPlaneState>, String> {
    shared_control_plane()
        .lock()
        .map_err(|_| "control plane lock poisoned".to_string())
}

pub fn reset_shared_control_plane_for_tests() {
    if let Ok(mut control_plane) = lock_shared_control_plane() {
        *control_plane = ControlPlaneState::default();
    }
}

pub fn reset_shared_control_plane_snapshot_for_tests(snapshot: AppSnapshot) {
    if let Ok(mut control_plane) = lock_shared_control_plane() {
        *control_plane = ControlPlaneState::from_snapshot(snapshot);
    }
}

fn workflow_runtime_for_root(root: &str) -> WorkflowRuntime {
    let repo_root = PathBuf::from(root);
    let key = repo_root.display().to_string();
    let runtimes = workflow_runtimes();
    let mut runtimes = runtimes.lock().expect("workflow runtime cache lock");
    let runtime = runtimes.entry(key).or_insert_with(|| {
        WorkflowRuntime::new(LoadWorkflowRequest {
            repo_root: repo_root.clone(),
            explicit_workflow_path: None,
        })
    });

    if runtime.current().is_none()
        && runtime.last_known_good().is_none()
        && runtime.last_error().is_none()
    {
        let _ = runtime.reload();
    } else {
        let _ = runtime.poll_watch();
    }

    runtime.clone()
}
