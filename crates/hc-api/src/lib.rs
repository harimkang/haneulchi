//! Local control API scaffold over a Unix domain socket.

mod automation;
mod tasks;

pub use automation::reconcile_now_json;
pub use tasks::{
    review_decision_json, review_queue_json, task_automation_mode_json, task_create_json,
    task_drawer_json, task_move_json, task_provision_workspace_json, tasks_list_json,
};
