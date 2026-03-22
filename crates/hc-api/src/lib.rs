//! Local control API scaffold over a Unix domain socket.

mod tasks;

pub use tasks::{task_move_json, tasks_list_json};
