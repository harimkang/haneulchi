use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use hc_domain::{
    AppSnapshot, AppSnapshotMeta, AppState, ClaimState, OpsSummary, ProjectSummary,
    SessionFocusState, SessionRuntimeState, SessionSummary, TrackerStatus, WarningSummary,
    WorkflowHealth, WorkflowRuntimeStatus,
};
use hc_runtime::terminal::runtime::TerminalSessionSnapshot as RuntimeSessionSnapshot;
use hc_workflow::{LoadWorkflowRequest, WorkflowLoader, WorkflowRuntime};

use crate::attention::derive_attention;
use crate::session_projection::{focus_session, release_takeover_session, takeover_session};
use crate::snapshot::{project_snapshot, SnapshotSeed};
use crate::workflow_projection::{sample_tracker_status, sample_workflow_status};

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ControlPlaneError {
    #[error("session not found: {0}")]
    SessionNotFound(String),
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
        });
        Self { snapshot }
    }

    pub fn snapshot(&self) -> &AppSnapshot {
        &self.snapshot
    }

    pub fn sync_from_runtime(&mut self, runtime_sessions: &[RuntimeSessionSnapshot]) {
        let previous_focus = self.snapshot.app.focused_session_id.clone();
        let previous_manual: BTreeMap<String, String> = self
            .snapshot
            .sessions
            .iter()
            .map(|session| (session.session_id.clone(), session.manual_control.clone()))
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
                    task_id: None,
                    mode: session_mode_for_launch(&session.launch.program),
                    runtime_state: if session.running {
                        SessionRuntimeState::Running
                    } else {
                        SessionRuntimeState::Exited
                    },
                    manual_control: manual_control.clone(),
                    dispatch_state: "not_dispatchable".to_string(),
                    claim_state: ClaimState::None,
                    adapter_kind: adapter_kind_for_launch(&session.launch.program),
                    title,
                    cwd: current_directory.clone(),
                    workspace_root: current_directory.clone(),
                    base_root: ".".to_string(),
                    branch: session.shell_metadata.branch.clone(),
                    latest_summary: session.shell_metadata.last_command.clone(),
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

        if !sessions.is_empty() && !sessions.iter().any(|session| session.focus_state == SessionFocusState::Focused) {
            if let Some(first) = sessions.first_mut() {
                first.focus_state = SessionFocusState::Focused;
            }
        }

        let focused_session_id = sessions
            .iter()
            .find(|session| session.focus_state == SessionFocusState::Focused)
            .map(|session| session.session_id.clone());

        let mut project_counts: BTreeMap<String, u32> = BTreeMap::new();
        for session in &sessions {
            if !session.project_id.is_empty() {
                *project_counts.entry(session.project_id.clone()).or_insert(0) += 1;
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
            .and_then(|focused_session_id| sessions.iter().find(|session| &session.session_id == focused_session_id))
            .map(|session| workflow_for_root(&session.project_id))
            .unwrap_or_else(empty_workflow_status);
        let tracker = TrackerStatus {
            state: "local_only".to_string(),
            last_sync_at: None,
            health: "ok".to_string(),
        };
        let attention = derive_attention(&workflow, &sessions);

        self.snapshot = AppSnapshot {
            meta: AppSnapshotMeta {
                snapshot_rev: self.snapshot.meta.snapshot_rev.saturating_add(1),
                runtime_rev: self.snapshot.meta.runtime_rev.saturating_add(1),
                projection_rev: self.snapshot.meta.projection_rev.saturating_add(1),
                snapshot_at: Some("2026-03-22T00:00:00Z".to_string()),
            },
            ops: OpsSummary {
                running_slots: sessions
                    .iter()
                    .filter(|session| session.runtime_state == SessionRuntimeState::Running)
                    .count() as u32,
                max_slots: sessions.len().max(1) as u32,
                retry_queue_count: 0,
                workflow_health: workflow.state,
            },
            workflow,
            tracker,
            app: AppState {
                active_route: "project_focus".to_string(),
                focused_session_id,
                degraded_flags: Vec::new(),
            },
            projects,
            sessions,
            attention,
            retry_queue: Vec::new(),
            warnings: Vec::<WarningSummary>::new(),
        };
    }

    pub fn focus_session(&mut self, session_id: &str) -> Result<(), ControlPlaneError> {
        focus_session(&mut self.snapshot, session_id)
    }

    pub fn takeover_session(&mut self, session_id: &str) -> Result<(), ControlPlaneError> {
        takeover_session(&mut self.snapshot, session_id)
    }

    pub fn release_takeover_session(
        &mut self,
        session_id: &str,
    ) -> Result<(), ControlPlaneError> {
        release_takeover_session(&mut self.snapshot, session_id)
    }
}

impl Default for ControlPlaneState {
    fn default() -> Self {
        Self {
            snapshot: AppSnapshot {
                meta: AppSnapshotMeta {
                    snapshot_rev: 1,
                    runtime_rev: 1,
                    projection_rev: 1,
                    snapshot_at: Some("2026-03-22T00:00:00Z".to_string()),
                },
                ops: OpsSummary {
                    running_slots: 0,
                    max_slots: 1,
                    retry_queue_count: 0,
                    workflow_health: WorkflowHealth::None,
                },
                workflow: empty_workflow_status(),
                tracker: TrackerStatus {
                    state: "local_only".to_string(),
                    last_sync_at: None,
                    health: "ok".to_string(),
                },
                app: AppState {
                    active_route: "project_focus".to_string(),
                    focused_session_id: None,
                    degraded_flags: Vec::new(),
                },
                projects: Vec::new(),
                sessions: Vec::new(),
                attention: Vec::new(),
                retry_queue: Vec::new(),
                warnings: Vec::new(),
            },
        }
    }
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
        Err(_error) if runtime.state() == hc_workflow::WorkflowState::InvalidKeptLastGood => Ok(runtime.clone()),
        Err(error) => Err(error),
    }
}

fn workflow_runtimes() -> &'static Mutex<std::collections::HashMap<String, WorkflowRuntime>> {
    static RUNTIMES: OnceLock<Mutex<std::collections::HashMap<String, WorkflowRuntime>>> = OnceLock::new();
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
        last_good_hash: runtime.last_known_good().map(|loaded| loaded.contract_hash.clone()),
        last_reload_at: runtime.last_reload_at().map(str::to_string),
        last_error: runtime.last_error().map(str::to_string),
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

    if runtime.current().is_none() && runtime.last_known_good().is_none() && runtime.last_error().is_none() {
        let _ = runtime.reload();
    } else {
        let _ = runtime.poll_watch();
    }

    runtime.clone()
}
