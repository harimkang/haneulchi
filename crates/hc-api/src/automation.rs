pub fn reconcile_now_json() -> Result<String, String> {
    serde_json::to_string(&serde_json::json!({
        "ok": true,
        "action": "reconcile_requested"
    }))
    .map_err(|error| error.to_string())
}
