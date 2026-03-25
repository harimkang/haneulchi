use hc_domain::settings::{AppStateRecord, LayoutRecord, SessionMetadataRecord};
use hc_domain::time::now_iso8601;
use hc_storage::{AppStateRow, LayoutRow, SessionMetadataRow};

use crate::commands::ControlPlaneError;
use crate::shared_store::lock_shared_store;

// ── mapping helpers ──────────────────────────────────────────────────────────

fn app_state_from_row(row: AppStateRow) -> AppStateRecord {
    AppStateRecord {
        active_route: row.active_route,
        last_project_id: row.last_project_id,
        last_session_id: row.last_session_id,
        saved_at: row.saved_at,
    }
}

fn session_metadata_to_row(r: SessionMetadataRecord) -> SessionMetadataRow {
    SessionMetadataRow {
        id: r.session_id,
        project_id: r.project_id,
        title: r.title,
        cwd: r.cwd,
        branch: r.branch,
        last_active_at: r.last_active_at,
        is_recoverable: r.is_recoverable,
    }
}

fn session_metadata_from_row(row: SessionMetadataRow) -> SessionMetadataRecord {
    SessionMetadataRecord {
        session_id: row.id,
        project_id: row.project_id,
        title: row.title,
        cwd: row.cwd,
        branch: row.branch,
        last_active_at: row.last_active_at,
        is_recoverable: row.is_recoverable,
    }
}

fn layout_to_row(r: LayoutRecord) -> LayoutRow {
    LayoutRow {
        id: r.layout_id,
        project_id: r.project_id,
        data_json: r.data_json,
        saved_at: r.saved_at,
    }
}

fn layout_from_row(row: LayoutRow) -> LayoutRecord {
    LayoutRecord {
        layout_id: row.id,
        project_id: row.project_id,
        data_json: row.data_json,
        saved_at: row.saved_at,
    }
}

// ── public service functions ─────────────────────────────────────────────────

pub fn shared_save_app_state(
    route: &str,
    last_project_id: Option<&str>,
    last_session_id: Option<&str>,
) -> Result<(), ControlPlaneError> {
    let saved_at = now_iso8601();
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .persistence()
        .save_app_state(route, last_project_id, last_session_id, &saved_at)
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}

pub fn shared_load_app_state() -> Result<Option<AppStateRecord>, ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .persistence()
        .load_app_state()
        .map(|opt| opt.map(app_state_from_row))
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}

pub fn shared_upsert_session_metadata(
    record: SessionMetadataRecord,
) -> Result<(), ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .persistence()
        .upsert_session_metadata(session_metadata_to_row(record))
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}

pub fn shared_list_recoverable_sessions(
    project_id: &str,
) -> Result<Vec<SessionMetadataRecord>, ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .persistence()
        .list_recoverable_sessions(project_id)
        .map(|rows| rows.into_iter().map(session_metadata_from_row).collect())
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}

pub fn shared_upsert_layout(record: LayoutRecord) -> Result<(), ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .persistence()
        .upsert_layout(layout_to_row(record))
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}

pub fn shared_load_latest_layout(
    project_id: &str,
) -> Result<Option<LayoutRecord>, ControlPlaneError> {
    let store = lock_shared_store().map_err(|e| ControlPlaneError::Storage(e.to_string()))?;
    store
        .persistence()
        .load_latest_layout(project_id)
        .map(|opt| opt.map(layout_from_row))
        .map_err(|e| ControlPlaneError::Storage(e.to_string()))
}
