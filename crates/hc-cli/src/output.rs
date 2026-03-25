use serde_json::Value;

pub fn extract_data(body: &str) -> Result<Value, String> {
    let envelope: Value = serde_json::from_str(body).map_err(|error| error.to_string())?;
    if envelope["ok"].as_bool() != Some(true) {
        return Err(envelope["error"]["message"]
            .as_str()
            .unwrap_or("api_error")
            .to_string());
    }
    Ok(envelope["data"].clone())
}
