use hc_ffi::{
    delete_secret_ref_json, resolve_secret_env_json, terminal_settings_json,
    upsert_secret_ref_json, upsert_terminal_settings_json,
};
#[cfg(target_os = "macos")]
use hc_storage::KeychainBoundary;
use serde_json::Value;

#[test]
fn terminal_settings_json_returns_valid_json() {
    let result = terminal_settings_json();
    let raw = match result {
        Ok(json) => json,
        Err(error) => serde_json::json!({ "error": error }).to_string(),
    };
    let _value: Value = serde_json::from_str(&raw)
        .expect("terminal_settings_json must return valid JSON");
}

#[test]
fn upsert_terminal_settings_json_round_trips() {
    let settings_json = r#"{
        "shell": "/bin/zsh",
        "default_cols": 120,
        "default_rows": 40,
        "scrollback_lines": 5000,
        "font_name": "Menlo",
        "theme": "dark",
        "cursor_style": "block"
    }"#;

    // Upsert the settings.
    let upsert_result = upsert_terminal_settings_json(settings_json);
    let upsert_raw = match upsert_result {
        Ok(json) => json,
        Err(error) => serde_json::json!({ "error": error }).to_string(),
    };
    let upsert_value: Value = serde_json::from_str(&upsert_raw)
        .expect("upsert_terminal_settings_json must return valid JSON");

    // If upsert succeeded, verify it returned ok:true and then read back.
    if upsert_value.get("error").is_none() {
        assert_eq!(upsert_value["ok"], true, "upsert should return ok:true");

        // Read back and verify the fields were persisted.
        let get_result = terminal_settings_json();
        let get_raw = match get_result {
            Ok(json) => json,
            Err(error) => serde_json::json!({ "error": error }).to_string(),
        };
        let get_value: Value = serde_json::from_str(&get_raw)
            .expect("terminal_settings_json must return valid JSON after upsert");

        if get_value.get("error").is_none() {
            assert_eq!(get_value["shell"], "/bin/zsh");
            assert_eq!(get_value["default_cols"], 120);
            assert_eq!(get_value["default_rows"], 40);
            assert_eq!(get_value["scrollback_lines"], 5000);
            assert_eq!(get_value["font_name"], "Menlo");
            assert_eq!(get_value["theme"], "dark");
            assert_eq!(get_value["cursor_style"], "block");
        }
    }
}

#[cfg(target_os = "macos")]
#[test]
fn resolve_secret_env_json_reads_keychain_backed_refs() {
    let suffix = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_nanos()
    );
    let ref_id = format!("ref_{suffix}");
    let service = "haneulchi.tests.settings_bridge";
    let account = format!("account_{suffix}");
    let env_var = "OPENAI_API_KEY";

    KeychainBoundary::store(service, &account, b"sk-test-secret")
        .expect("store keychain secret");

    let ref_json = serde_json::json!({
        "ref_id": ref_id,
        "label": "OpenAI",
        "env_var_name": env_var,
        "keychain_service": service,
        "keychain_account": account,
    })
    .to_string();

    let upsert_result = upsert_secret_ref_json(&ref_json).expect("upsert secret ref");
    let upsert_value: Value = serde_json::from_str(&upsert_result).expect("valid json");
    assert_eq!(upsert_value["ok"], true);

    let resolved = resolve_secret_env_json().expect("resolve secret env");
    let resolved_value: Value = serde_json::from_str(&resolved).expect("valid resolved json");
    assert_eq!(resolved_value[env_var], "sk-test-secret");

    delete_secret_ref_json(&ref_id).expect("delete secret ref");
    let _ = security_framework::passwords::delete_generic_password(service, &account);
}
