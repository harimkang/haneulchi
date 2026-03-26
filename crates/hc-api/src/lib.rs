//! Local control API scaffold over a Unix domain socket.

mod automation;
mod dispatch;
pub mod envelope;
pub mod server;
pub mod sessions;
pub mod state;
mod tasks;
pub mod workflow;

pub use automation::reconcile_now_json;
pub use dispatch::dispatch_send_json;
pub use tasks::{
    review_decision_json, review_queue_json, task_automation_mode_json, task_create_json,
    task_drawer_json, task_move_json, task_prepare_isolated_launch_json,
    task_provision_workspace_json, tasks_list_json,
};
