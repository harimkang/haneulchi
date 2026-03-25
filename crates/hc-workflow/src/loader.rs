use std::fs;
use std::path::PathBuf;

use crate::WorkflowError;
use crate::contract::{LoadedWorkflow, parse_workflow_file};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadWorkflowRequest {
    pub repo_root: PathBuf,
    pub explicit_workflow_path: Option<PathBuf>,
}

pub struct WorkflowLoader;

impl WorkflowLoader {
    pub fn load(request: &LoadWorkflowRequest) -> Result<Option<LoadedWorkflow>, WorkflowError> {
        let workflow_path = if let Some(explicit_path) = &request.explicit_workflow_path {
            explicit_path.clone()
        } else {
            request.repo_root.join("WORKFLOW.md")
        };

        if !workflow_path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&workflow_path)
            .map_err(|error| WorkflowError::Io(error.to_string()))?;
        parse_workflow_file(&workflow_path, &contents).map(Some)
    }
}
