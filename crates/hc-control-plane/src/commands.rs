use hc_domain::{AppSnapshot, ClaimState, ProjectSummary, SessionFocusState, SessionRuntimeState, SessionSummary};
use hc_workflow::{LoadWorkflowRequest, WorkflowLoader, WorkflowRuntime};

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

pub fn validate_workflow(
    repo_root: impl Into<std::path::PathBuf>,
) -> Result<Option<hc_workflow::LoadedWorkflow>, hc_workflow::WorkflowError> {
    WorkflowLoader::load(&LoadWorkflowRequest {
        repo_root: repo_root.into(),
        explicit_workflow_path: None,
    })
}

pub fn reload_workflow(
    repo_root: impl Into<std::path::PathBuf>,
) -> Result<WorkflowRuntime, hc_workflow::WorkflowError> {
    let request = LoadWorkflowRequest {
        repo_root: repo_root.into(),
        explicit_workflow_path: None,
    };
    let mut runtime = WorkflowRuntime::new(request);
    runtime.reload()?;
    Ok(runtime)
}
