pub fn reconcile_now_json() -> Result<String, String> {
    let result = hc_control_plane::shared_scheduler_tick(0, 1, &[])
        .map_err(|error| error.to_string())?;
    serde_json::to_string(&serde_json::json!({
        "ok": true,
        "action": "reconcile_requested",
        "result": result
    }))
    .map_err(|error| error.to_string())
}
