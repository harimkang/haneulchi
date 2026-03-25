use std::collections::BTreeMap;

use hc_domain::settings::{SecretRef, TerminalSettings};
use hc_storage::{KeychainBoundary, SecretRefRow, TerminalSettingsRow};

use crate::commands::ControlPlaneError;
use crate::shared_store::lock_shared_store;

// ── mapping helpers ──────────────────────────────────────────────────────────

fn terminal_settings_to_row(s: TerminalSettings) -> TerminalSettingsRow {
    TerminalSettingsRow {
        shell: s.shell,
        default_cols: s.default_cols as i64,
        default_rows: s.default_rows as i64,
        scrollback_lines: s.scrollback_lines as i64,
        font_name: s.font_name,
        theme: s.theme,
        cursor_style: s.cursor_style,
    }
}

fn terminal_settings_from_row(row: TerminalSettingsRow) -> TerminalSettings {
    TerminalSettings {
        shell: row.shell,
        default_cols: row.default_cols.max(0) as u32,
        default_rows: row.default_rows.max(0) as u32,
        scrollback_lines: row.scrollback_lines.max(0) as u32,
        font_name: row.font_name,
        theme: row.theme,
        cursor_style: row.cursor_style,
    }
}

fn secret_ref_to_row(r: SecretRef) -> SecretRefRow {
    SecretRefRow {
        id: r.ref_id,
        label: r.label,
        env_var_name: r.env_var_name,
        keychain_service: r.keychain_service,
        keychain_account: r.keychain_account,
        scope: r.scope,
    }
}

fn secret_ref_from_row(row: SecretRefRow) -> SecretRef {
    SecretRef {
        ref_id: row.id,
        label: row.label,
        env_var_name: row.env_var_name,
        keychain_service: row.keychain_service,
        keychain_account: row.keychain_account,
        scope: row.scope,
    }
}

// ── public service functions ─────────────────────────────────────────────────

pub fn shared_get_terminal_settings() -> Result<Option<TerminalSettings>, ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .settings_repo()
        .get_terminal_settings()
        .map(|opt| opt.map(terminal_settings_from_row))
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}

pub fn shared_upsert_terminal_settings(
    settings: TerminalSettings,
) -> Result<(), ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .settings_repo()
        .upsert_terminal_settings(terminal_settings_to_row(settings))
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}

pub fn shared_list_secret_refs() -> Result<Vec<SecretRef>, ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .settings_repo()
        .list_secret_refs()
        .map(|rows| rows.into_iter().map(secret_ref_from_row).collect())
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}

pub fn shared_upsert_secret_ref(secret_ref: SecretRef) -> Result<(), ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .settings_repo()
        .upsert_secret_ref(secret_ref_to_row(secret_ref))
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}

pub fn shared_delete_secret_ref(ref_id: &str) -> Result<(), ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .settings_repo()
        .delete_secret_ref(ref_id)
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}

pub fn shared_resolve_secret_env() -> Result<BTreeMap<String, String>, ControlPlaneError> {
    shared_resolve_secret_env_filtered(None)
}

pub fn shared_resolve_secret_env_filtered(
    scope_filter: Option<&str>,
) -> Result<BTreeMap<String, String>, ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    let refs = store
        .settings_repo()
        .list_secret_refs()
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))?;

    let mut environment = BTreeMap::new();

    for reference in refs {
        if let Some(scope) = scope_filter {
            let passes = reference.scope.is_empty() || reference.scope == scope;
            if !passes {
                continue;
            }
        }

        let secret = match KeychainBoundary::retrieve(
            &reference.keychain_service,
            &reference.keychain_account,
        ) {
            Ok(Some(v)) => v,
            Ok(None) => continue, // not found, skip gracefully
            Err(e) => return Err(ControlPlaneError::Storage(e.to_string())), // real error
        };

        let value = String::from_utf8(secret).map_err(|_| {
            ControlPlaneError::Storage(format!("keychain_ref_invalid_utf8:{}", reference.id))
        })?;
        environment.insert(reference.env_var_name, value);
    }

    Ok(environment)
}
