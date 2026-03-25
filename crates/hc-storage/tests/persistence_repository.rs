use hc_storage::{LayoutRow, SessionMetadataRow, SqliteStore};

#[test]
fn app_state_initially_absent() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.persistence();

    let state = repo.load_app_state().expect("load");
    assert!(state.is_none());
}

#[test]
fn app_state_save_and_load() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.persistence();

    repo.save_app_state(
        "/dashboard",
        Some("proj_01"),
        Some("sess_01"),
        "2026-03-24T10:00:00Z",
    )
    .expect("save");

    let state = repo.load_app_state().expect("load").expect("some state");
    assert_eq!(state.active_route, "/dashboard");
    assert_eq!(state.last_project_id.as_deref(), Some("proj_01"));
    assert_eq!(state.last_session_id.as_deref(), Some("sess_01"));
}

#[test]
fn app_state_save_replaces_singleton() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.persistence();

    repo.save_app_state("/home", None, None, "2026-03-24T10:00:00Z")
        .expect("first save");

    repo.save_app_state("/settings", Some("proj_02"), None, "2026-03-24T11:00:00Z")
        .expect("second save");

    let state = repo.load_app_state().expect("load").expect("some state");
    assert_eq!(state.active_route, "/settings");
    assert_eq!(state.last_project_id.as_deref(), Some("proj_02"));
    assert!(state.last_session_id.is_none());
}

#[test]
fn session_metadata_upsert_and_list_recoverable() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.persistence();

    let row = SessionMetadataRow {
        id: "sess_r01".to_string(),
        project_id: "proj_01".to_string(),
        title: "Fix auth bug".to_string(),
        cwd: "/tmp/project".to_string(),
        branch: Some("feat/fix-auth".to_string()),
        last_active_at: Some("2026-03-24T09:00:00Z".to_string()),
        is_recoverable: true,
    };

    repo.upsert_session_metadata(row).expect("upsert");

    let recoverable = repo
        .list_recoverable_sessions("proj_01")
        .expect("list recoverable");
    assert_eq!(recoverable.len(), 1);
    assert_eq!(recoverable[0].id, "sess_r01");
    assert!(recoverable[0].is_recoverable);
}

#[test]
fn non_recoverable_sessions_excluded_from_list() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.persistence();

    repo.upsert_session_metadata(SessionMetadataRow {
        id: "sess_nr01".to_string(),
        project_id: "proj_02".to_string(),
        title: "Finished task".to_string(),
        cwd: "/tmp/finished".to_string(),
        branch: None,
        last_active_at: None,
        is_recoverable: false,
    })
    .expect("upsert non-recoverable");

    let recoverable = repo
        .list_recoverable_sessions("proj_02")
        .expect("list recoverable");
    assert!(recoverable.is_empty());
}

#[test]
fn layout_upsert_and_load_latest() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.persistence();

    let row = LayoutRow {
        id: "layout_01".to_string(),
        project_id: "proj_03".to_string(),
        data_json: r#"{"panes":["terminal","editor"]}"#.to_string(),
        saved_at: Some("2026-03-24T10:00:00Z".to_string()),
    };

    repo.upsert_layout(row).expect("upsert layout");

    let loaded = repo
        .load_latest_layout("proj_03")
        .expect("load")
        .expect("some layout");
    assert_eq!(loaded.id, "layout_01");
    assert_eq!(loaded.project_id, "proj_03");
    assert!(loaded.data_json.contains("terminal"));
}

#[test]
fn layout_load_latest_absent_when_none() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.persistence();

    let loaded = repo.load_latest_layout("proj_nonexistent").expect("load");
    assert!(loaded.is_none());
}
