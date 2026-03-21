//! Orchestrator and snapshot scaffold.

mod attention;
mod commands;
mod session_projection;
mod snapshot;
mod workflow_projection;

pub use commands::{
    reload_workflow, validate_workflow, ControlPlaneError, ControlPlaneState,
};
pub use snapshot::{project_snapshot, SnapshotSeed};
