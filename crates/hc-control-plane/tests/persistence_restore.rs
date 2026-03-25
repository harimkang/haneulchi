/// Integration tests for persistence save/restore round-trips.
///
/// These tests operate directly against `SqliteStore::in_memory()` via the
/// `PersistenceRepository` so they are hermetic and do not touch the
/// shared global store.
use hc_storage::{AppStateRow, LayoutRow, SessionMetadataRow, SqliteStore};

// ── helpers ──────────────────────────────────────────────────────────────────

fn store() -> SqliteStore {
    SqliteStore::in_memory().expect("in-memory store")
}

// ── tests ────────────────────────────────────────────────────────────────────

#[test]
fn app_state_round_trips_through_persistence() {
    let store = store();
    let persistence = store.persistence();

    persistence
        .save_app_state(
            "project_focus",
            Some("proj_alpha"),
            Some("ses_001"),
            "2026-03-25T10:00:00Z",
        )
        .expect("save_app_state");

    let loaded = persistence
        .load_app_state()
        .expect("load_app_state")
        .expect("row should be present");

    assert_eq!(loaded.active_route, "project_focus");
    assert_eq!(loaded.last_project_id.as_deref(), Some("proj_alpha"));
    assert_eq!(loaded.last_session_id.as_deref(), Some("ses_001"));
    assert_eq!(loaded.saved_at.as_deref(), Some("2026-03-25T10:00:00Z"));
}

#[test]
fn session_metadata_round_trips_through_persistence() {
    let store = store();
    let persistence = store.persistence();

    // Insert a recoverable session.
    persistence
        .upsert_session_metadata(SessionMetadataRow {
            id: "ses_recover_01".to_string(),
            project_id: "proj_beta".to_string(),
            title: "Recovery session".to_string(),
            cwd: "/home/user/beta".to_string(),
            branch: Some("main".to_string()),
            last_active_at: Some("2026-03-25T09:00:00Z".to_string()),
            is_recoverable: true,
        })
        .expect("upsert session metadata");

    // Insert a non-recoverable session (should be excluded).
    persistence
        .upsert_session_metadata(SessionMetadataRow {
            id: "ses_gone_01".to_string(),
            project_id: "proj_beta".to_string(),
            title: "Closed session".to_string(),
            cwd: "/home/user/beta".to_string(),
            branch: None,
            last_active_at: Some("2026-03-24T08:00:00Z".to_string()),
            is_recoverable: false,
        })
        .expect("upsert session metadata 2");

    let recoverable = persistence
        .list_recoverable_sessions("proj_beta")
        .expect("list_recoverable_sessions");

    assert_eq!(recoverable.len(), 1);
    assert_eq!(recoverable[0].id, "ses_recover_01");
    assert_eq!(recoverable[0].project_id, "proj_beta");
    assert_eq!(recoverable[0].title, "Recovery session");
    assert!(recoverable[0].is_recoverable);
}

#[test]
fn layout_round_trips_through_persistence() {
    let store = store();
    let persistence = store.persistence();

    let layout_json = r#"{"panes":[{"id":"p1","width":0.5}]}"#;

    persistence
        .upsert_layout(LayoutRow {
            id: "layout_v1".to_string(),
            project_id: "proj_gamma".to_string(),
            data_json: layout_json.to_string(),
            saved_at: Some("2026-03-25T11:00:00Z".to_string()),
        })
        .expect("upsert layout");

    let loaded = persistence
        .load_latest_layout("proj_gamma")
        .expect("load_latest_layout")
        .expect("layout row should be present");

    assert_eq!(loaded.id, "layout_v1");
    assert_eq!(loaded.project_id, "proj_gamma");
    assert_eq!(loaded.data_json, layout_json);
    assert_eq!(loaded.saved_at.as_deref(), Some("2026-03-25T11:00:00Z"));
}
