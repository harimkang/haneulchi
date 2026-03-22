#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkflowWatchState {
    pub reload_pending: bool,
    observed_modified_millis: Option<u128>,
    pending_modified_millis: Option<u128>,
    pending_since_millis: Option<u128>,
}

impl WorkflowWatchState {
    pub fn mark_reload_pending(&mut self, modified_millis: Option<u128>, now_millis: u128) {
        self.pending_modified_millis = modified_millis;
        self.pending_since_millis = Some(modified_millis.unwrap_or(now_millis).min(now_millis));
        self.reload_pending = true;
    }

    pub fn observe_modified(&mut self, modified_millis: Option<u128>, now_millis: u128) {
        match modified_millis {
            Some(modified_millis)
                if self.observed_modified_millis != Some(modified_millis)
                    && self.pending_modified_millis != Some(modified_millis) =>
            {
                self.mark_reload_pending(Some(modified_millis), now_millis);
            }
            Some(_) => {}
            None => self.clear(),
        }
    }

    pub fn should_reload(&self, now_millis: u128, debounce_millis: u128) -> bool {
        self.reload_pending
            && self
                .pending_since_millis
                .map(|pending_since| now_millis.saturating_sub(pending_since) >= debounce_millis)
                .unwrap_or(false)
    }

    pub fn note_reloaded(&mut self, modified_millis: Option<u128>) {
        self.observed_modified_millis = modified_millis;
        self.pending_modified_millis = None;
        self.pending_since_millis = None;
        self.reload_pending = false;
    }

    pub fn clear(&mut self) {
        self.observed_modified_millis = None;
        self.pending_modified_millis = None;
        self.pending_since_millis = None;
        self.reload_pending = false;
    }
}
