use crate::contract::LoadedWorkflow;
use crate::loader::{LoadWorkflowRequest, WorkflowLoader};
use crate::WorkflowError;

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
}

impl WorkflowRuntime {
    pub fn new(request: LoadWorkflowRequest) -> Self {
        Self {
            request,
            state: WorkflowState::None,
            current: None,
            last_known_good: None,
            last_error: None,
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

    pub fn mark_reload_pending(&mut self) {
        self.state = WorkflowState::ReloadPending;
    }

    pub fn reload(&mut self) -> Result<(), WorkflowError> {
        match WorkflowLoader::load(&self.request) {
            Ok(Some(loaded)) => {
                self.state = WorkflowState::Ok;
                self.current = Some(loaded.clone());
                self.last_known_good = Some(loaded);
                self.last_error = None;
                Ok(())
            }
            Ok(None) => {
                self.state = WorkflowState::None;
                self.current = None;
                self.last_error = None;
                Ok(())
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
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
