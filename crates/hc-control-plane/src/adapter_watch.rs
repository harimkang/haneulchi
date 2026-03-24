use hc_domain::SessionSummary;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdapterWatchSummary {
    pub session_id: String,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub latest_commentary: Option<String>,
    pub active_window_title: Option<String>,
    pub dispatch_reason: Option<String>,
}

pub fn adapter_watch_for_session(session: &SessionSummary) -> Option<AdapterWatchSummary> {
    if session.adapter_kind.is_none()
        && session.provider_id.is_none()
        && session.model_id.is_none()
        && session.latest_commentary.is_none()
    {
        return None;
    }

    Some(AdapterWatchSummary {
        session_id: session.session_id.clone(),
        provider_id: session.provider_id.clone(),
        model_id: session.model_id.clone(),
        latest_commentary: session.latest_commentary.clone(),
        active_window_title: session.active_window_title.clone(),
        dispatch_reason: session.dispatch_reason.clone(),
    })
}
