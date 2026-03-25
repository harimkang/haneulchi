use hc_storage::{SecretRefRow, SqliteStore, TerminalSettingsRow};

#[test]
fn terminal_settings_initially_absent() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.settings_repo();

    let result = repo.get_terminal_settings().expect("get settings");
    assert!(result.is_none());
}

#[test]
fn terminal_settings_upsert_and_retrieve() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.settings_repo();

    let row = TerminalSettingsRow {
        shell: "/bin/zsh".to_string(),
        default_cols: 120,
        default_rows: 40,
        scrollback_lines: 10000,
        font_name: "Menlo".to_string(),
        theme: "dark".to_string(),
        cursor_style: "block".to_string(),
    };

    repo.upsert_terminal_settings(row.clone()).expect("upsert");

    let retrieved = repo
        .get_terminal_settings()
        .expect("get")
        .expect("some row");
    assert_eq!(retrieved.shell, "/bin/zsh");
    assert_eq!(retrieved.default_cols, 120);
    assert_eq!(retrieved.default_rows, 40);
    assert_eq!(retrieved.scrollback_lines, 10000);
    assert_eq!(retrieved.font_name, "Menlo");
    assert_eq!(retrieved.theme, "dark");
    assert_eq!(retrieved.cursor_style, "block");
}

#[test]
fn terminal_settings_upsert_replaces_singleton() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.settings_repo();

    repo.upsert_terminal_settings(TerminalSettingsRow {
        shell: "/bin/bash".to_string(),
        default_cols: 80,
        default_rows: 24,
        scrollback_lines: 1000,
        font_name: "".to_string(),
        theme: "".to_string(),
        cursor_style: "".to_string(),
    })
    .expect("first upsert");

    repo.upsert_terminal_settings(TerminalSettingsRow {
        shell: "/bin/fish".to_string(),
        default_cols: 200,
        default_rows: 50,
        scrollback_lines: 50000,
        font_name: "Fira Code".to_string(),
        theme: "light".to_string(),
        cursor_style: "underline".to_string(),
    })
    .expect("second upsert");

    let retrieved = repo
        .get_terminal_settings()
        .expect("get")
        .expect("some row");
    assert_eq!(retrieved.shell, "/bin/fish");
    assert_eq!(retrieved.default_cols, 200);
}

#[test]
fn secret_refs_list_empty_initially() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.settings_repo();

    let refs = repo.list_secret_refs().expect("list");
    assert!(refs.is_empty());
}

#[test]
fn secret_ref_upsert_and_list() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.settings_repo();

    let row = SecretRefRow {
        id: "sref_01".to_string(),
        label: "GitHub Token".to_string(),
        env_var_name: "GITHUB_TOKEN".to_string(),
        keychain_service: "haneulchi".to_string(),
        keychain_account: "github_token".to_string(),
        scope: String::new(),
    };

    repo.upsert_secret_ref(row).expect("upsert");

    let refs = repo.list_secret_refs().expect("list");
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].id, "sref_01");
    assert_eq!(refs[0].label, "GitHub Token");
    assert_eq!(refs[0].env_var_name, "GITHUB_TOKEN");
    assert_eq!(refs[0].scope, "");
}

#[test]
fn secret_ref_delete_removes_entry() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.settings_repo();

    repo.upsert_secret_ref(SecretRefRow {
        id: "sref_02".to_string(),
        label: "Anthropic Key".to_string(),
        env_var_name: "ANTHROPIC_API_KEY".to_string(),
        keychain_service: "haneulchi".to_string(),
        keychain_account: "anthropic_key".to_string(),
        scope: String::new(),
    })
    .expect("upsert");

    repo.delete_secret_ref("sref_02").expect("delete");

    let refs = repo.list_secret_refs().expect("list after delete");
    assert!(refs.is_empty());
}
