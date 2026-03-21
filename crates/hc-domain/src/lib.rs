#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionRuntimeState {
    Launching,
    Running,
    WaitingInput,
    ReviewReady,
    Blocked,
    Done,
    Error,
    Exited,
}

impl SessionRuntimeState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Launching => "launching",
            Self::Running => "running",
            Self::WaitingInput => "waiting_input",
            Self::ReviewReady => "review_ready",
            Self::Blocked => "blocked",
            Self::Done => "done",
            Self::Error => "error",
            Self::Exited => "exited",
        }
    }

    pub const fn all() -> [&'static str; 8] {
        [
            Self::Launching.as_str(),
            Self::Running.as_str(),
            Self::WaitingInput.as_str(),
            Self::ReviewReady.as_str(),
            Self::Blocked.as_str(),
            Self::Done.as_str(),
            Self::Error.as_str(),
            Self::Exited.as_str(),
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowHealth {
    None,
    Ok,
    InvalidKeptLastGood,
    ReloadPending,
}

impl WorkflowHealth {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Ok => "ok",
            Self::InvalidKeptLastGood => "invalid_kept_last_good",
            Self::ReloadPending => "reload_pending",
        }
    }

    pub const fn all() -> [&'static str; 4] {
        [
            Self::None.as_str(),
            Self::Ok.as_str(),
            Self::InvalidKeptLastGood.as_str(),
            Self::ReloadPending.as_str(),
        ]
    }
}
