use hc_domain::AppSnapshot;

pub fn current_snapshot() -> Result<AppSnapshot, String> {
    let control_plane = hc_control_plane::lock_shared_control_plane()?;
    Ok(control_plane.snapshot().clone())
}

pub fn state_json() -> Result<String, String> {
    state_json_for(None)
}

pub fn state_json_for(project_id: Option<&str>) -> Result<String, String> {
    let mut snapshot = current_snapshot()?;
    if let Some(project_id) = project_id {
        snapshot.projects.retain(|project| project.project_id == project_id);
        snapshot.sessions.retain(|session| session.project_id == project_id);
        snapshot
            .attention
            .retain(|item| item.project_id == project_id);
        snapshot
            .retry_queue
            .retain(|item| item.project_id == project_id);
    }
    serde_json::to_string(&snapshot).map_err(|error| error.to_string())
}
