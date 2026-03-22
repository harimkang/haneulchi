#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkflowWatchState {
    pub reload_pending: bool,
}

impl WorkflowWatchState {
    pub fn mark_reload_pending(&mut self) {
        self.reload_pending = true;
    }

    pub fn clear(&mut self) {
        self.reload_pending = false;
    }
}
