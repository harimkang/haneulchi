//! WORKFLOW.md discovery, validation, and reload scaffold.

mod contract;
mod loader;
mod runtime;
mod template;
mod watch;

pub use contract::{
    EffectiveWorkflowConfig, HookDefinition, HookPhase, HooksConfig, LoadedWorkflow,
    PolicyConfig, ResolvedPaths, ReviewConfig, WorkspaceConfig, WorkspaceStrategy,
    WorkflowConfig,
};
pub use loader::{LoadWorkflowRequest, WorkflowLoader};
pub use runtime::{WorkflowLaunchBinding, WorkflowRuntime, WorkflowState};
pub use watch::WorkflowWatchState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowErrorCode {
    FrontMatterParse,
    UnsupportedVersion,
    UnknownTemplateVariable,
    InvalidBaseRoot,
    Io,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("front matter parse error: {0}")]
    FrontMatterParse(String),
    #[error("unsupported workflow version: {0}")]
    UnsupportedVersion(u64),
    #[error("unknown template variable: {0}")]
    UnknownTemplateVariable(String),
    #[error("invalid base root: {0}")]
    InvalidBaseRoot(String),
    #[error("io error: {0}")]
    Io(String),
}

impl WorkflowError {
    pub const fn code(&self) -> WorkflowErrorCode {
        match self {
            Self::FrontMatterParse(_) => WorkflowErrorCode::FrontMatterParse,
            Self::UnsupportedVersion(_) => WorkflowErrorCode::UnsupportedVersion,
            Self::UnknownTemplateVariable(_) => WorkflowErrorCode::UnknownTemplateVariable,
            Self::InvalidBaseRoot(_) => WorkflowErrorCode::InvalidBaseRoot,
            Self::Io(_) => WorkflowErrorCode::Io,
        }
    }
}
