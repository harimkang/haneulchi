use std::time::{SystemTime, UNIX_EPOCH};

use crate::contract::LoadedWorkflow;
use crate::loader::{LoadWorkflowRequest, WorkflowLoader};
use crate::watch::WorkflowWatchState;
use crate::WorkflowError;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const WATCH_DEBOUNCE_MILLIS: u128 = 1_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowState {
    None,
    Ok,
    InvalidKeptLastGood,
    ReloadPending,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowLaunchBinding {
    pub contract_hash: String,
}

#[derive(Clone)]
pub struct WorkflowRuntime {
    request: LoadWorkflowRequest,
    state: WorkflowState,
    current: Option<LoadedWorkflow>,
    last_known_good: Option<LoadedWorkflow>,
    last_error: Option<String>,
    last_reload_at: Option<String>,
    watch_state: WorkflowWatchState,
}

impl WorkflowRuntime {
    pub fn new(request: LoadWorkflowRequest) -> Self {
        Self {
            request,
            state: WorkflowState::None,
            current: None,
            last_known_good: None,
            last_error: None,
            last_reload_at: None,
            watch_state: WorkflowWatchState::default(),
        }
    }

    pub const fn state(&self) -> WorkflowState {
        self.state
    }

    pub fn current(&self) -> Option<&LoadedWorkflow> {
        self.current.as_ref()
    }

    pub fn last_known_good(&self) -> Option<&LoadedWorkflow> {
        self.last_known_good.as_ref()
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn last_reload_at(&self) -> Option<&str> {
        self.last_reload_at.as_deref()
    }

    pub fn mark_reload_pending(&mut self) {
        let now_millis = current_time_millis();
        let modified_millis = workflow_modified_millis(&self.request);
        self.watch_state
            .mark_reload_pending(modified_millis, now_millis);
        self.state = WorkflowState::ReloadPending;
    }

    pub fn poll_watch(&mut self) -> Result<(), WorkflowError> {
        let now_millis = current_time_millis();
        let modified_millis = workflow_modified_millis(&self.request);

        self.watch_state.observe_modified(modified_millis, now_millis);
        if self.watch_state.should_reload(now_millis, WATCH_DEBOUNCE_MILLIS) {
            return self.reload();
        }

        if self.watch_state.reload_pending {
            self.state = WorkflowState::ReloadPending;
        }

        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), WorkflowError> {
        let reload_at = current_timestamp();
        let modified_millis = workflow_modified_millis(&self.request);

        match WorkflowLoader::load(&self.request) {
            Ok(Some(loaded)) => {
                self.state = WorkflowState::Ok;
                self.current = Some(loaded.clone());
                self.last_known_good = Some(loaded);
                self.last_error = None;
                self.last_reload_at = Some(reload_at);
                self.watch_state.note_reloaded(modified_millis);
                Ok(())
            }
            Ok(None) => {
                self.state = WorkflowState::None;
                self.current = None;
                self.last_error = None;
                self.last_reload_at = Some(reload_at);
                self.watch_state.note_reloaded(modified_millis);
                Ok(())
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
                self.last_reload_at = Some(reload_at);
                self.watch_state.note_reloaded(modified_millis);
                if let Some(last_known_good) = self.last_known_good.clone() {
                    self.current = Some(last_known_good);
                    self.state = WorkflowState::InvalidKeptLastGood;
                } else {
                    self.current = None;
                    self.state = WorkflowState::None;
                }
                Err(error)
            }
        }
    }

    pub fn prepare_launch(&self) -> Option<WorkflowLaunchBinding> {
        self.current.as_ref().map(|workflow| WorkflowLaunchBinding {
            contract_hash: workflow.contract_hash.clone(),
        })
    }
}

fn workflow_modified_millis(request: &LoadWorkflowRequest) -> Option<u128> {
    let workflow_path = if let Some(explicit_path) = &request.explicit_workflow_path {
        explicit_path.clone()
    } else {
        request.repo_root.join("WORKFLOW.md")
    };

    std::fs::metadata(workflow_path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis())
}

fn current_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis()
}

fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("rfc3339 timestamp")
}
