use hc_domain::AppSnapshot;

pub fn current_snapshot() -> Result<AppSnapshot, String> {
    let control_plane = hc_control_plane::lock_shared_control_plane()?;
    Ok(control_plane.snapshot().clone())
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StateQuery<'a> {
    pub project_id: Option<&'a str>,
    pub compact: bool,
    pub include_attention: bool,
    pub include_retry_queue: bool,
}

pub fn state_json() -> Result<String, String> {
    state_json_for(StateQuery {
        include_attention: true,
        include_retry_queue: true,
        ..StateQuery::default()
    })
}

pub fn state_json_for(query: StateQuery<'_>) -> Result<String, String> {
    let mut snapshot = current_snapshot()?;
    if let Some(project_id) = query.project_id {
        snapshot
            .projects
            .retain(|project| project.project_id == project_id);
        snapshot
            .sessions
            .retain(|session| session.project_id == project_id);
        snapshot
            .attention
            .retain(|item| item.project_id == project_id);
        snapshot
            .retry_queue
            .retain(|item| item.project_id == project_id);
    }

    if query.compact {
        snapshot.attention.clear();
        snapshot.retry_queue.clear();
        snapshot.warnings.clear();
    } else {
        if !query.include_attention {
            snapshot.attention.clear();
        }
        if !query.include_retry_queue {
            snapshot.retry_queue.clear();
        }
    }

    serde_json::to_string(&snapshot).map_err(|error| error.to_string())
}
