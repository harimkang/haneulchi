use hc_storage::{CacheEntryRecord, QuotaRecord, SqliteStore};

#[test]
fn cache_root_can_be_created_and_retrieved() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.cache();

    let root = repo
        .create_root("root_01", "/tmp/cache/root01", "2026-03-24T10:00:00Z")
        .expect("create root");

    assert_eq!(root.id, "root_01");
    assert_eq!(root.root_path, "/tmp/cache/root01");
    assert_eq!(root.created_at, "2026-03-24T10:00:00Z");
}

#[test]
fn cache_entry_upsert_and_list() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.cache();

    repo.create_root("root_02", "/tmp/cache/root02", "2026-03-24T10:00:00Z")
        .expect("create root");

    let entry = CacheEntryRecord {
        id: "entry_01".to_string(),
        cache_root_id: "root_02".to_string(),
        path: "/tmp/cache/root02/file.bin".to_string(),
        size_bytes: 1024,
        last_accessed_at: Some("2026-03-24T11:00:00Z".to_string()),
        content_hash: Some("abc123".to_string()),
    };

    repo.upsert_entry(entry.clone()).expect("upsert entry");

    let entries = repo.list_entries("root_02").expect("list entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, "entry_01");
    assert_eq!(entries[0].size_bytes, 1024);
    assert_eq!(entries[0].content_hash.as_deref(), Some("abc123"));
}

#[test]
fn total_size_bytes_sums_entries() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.cache();

    repo.create_root("root_03", "/tmp/cache/root03", "2026-03-24T10:00:00Z")
        .expect("create root");

    repo.upsert_entry(CacheEntryRecord {
        id: "entry_a".to_string(),
        cache_root_id: "root_03".to_string(),
        path: "/tmp/cache/root03/a".to_string(),
        size_bytes: 500,
        last_accessed_at: None,
        content_hash: None,
    })
    .expect("upsert a");

    repo.upsert_entry(CacheEntryRecord {
        id: "entry_b".to_string(),
        cache_root_id: "root_03".to_string(),
        path: "/tmp/cache/root03/b".to_string(),
        size_bytes: 300,
        last_accessed_at: None,
        content_hash: None,
    })
    .expect("upsert b");

    let total = repo.total_size_bytes("root_03").expect("total size");
    assert_eq!(total, 800);
}

#[test]
fn upsert_entry_replaces_existing() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.cache();

    repo.create_root("root_04", "/tmp/cache/root04", "2026-03-24T10:00:00Z")
        .expect("create root");

    repo.upsert_entry(CacheEntryRecord {
        id: "entry_x".to_string(),
        cache_root_id: "root_04".to_string(),
        path: "/tmp/cache/root04/x".to_string(),
        size_bytes: 100,
        last_accessed_at: None,
        content_hash: None,
    })
    .expect("initial upsert");

    repo.upsert_entry(CacheEntryRecord {
        id: "entry_x".to_string(),
        cache_root_id: "root_04".to_string(),
        path: "/tmp/cache/root04/x".to_string(),
        size_bytes: 200,
        last_accessed_at: Some("2026-03-24T12:00:00Z".to_string()),
        content_hash: Some("newHash".to_string()),
    })
    .expect("update upsert");

    let entries = repo.list_entries("root_04").expect("list entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].size_bytes, 200);
}

// --- Quota tests ---

#[test]
fn check_quota_returns_none_when_no_quota_configured() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.cache();

    repo.create_root("root_q1", "/tmp/cache/q1", "2026-03-25T00:00:00Z")
        .expect("create root");

    let status = repo.check_quota("root_q1").expect("check quota");
    assert!(status.is_none());
}

#[test]
fn check_quota_under_limit_is_not_over() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.cache();

    repo.create_root("root_q2", "/tmp/cache/q2", "2026-03-25T00:00:00Z")
        .expect("create root");

    repo.upsert_entry(CacheEntryRecord {
        id: "entry_q2a".to_string(),
        cache_root_id: "root_q2".to_string(),
        path: "/tmp/cache/q2/file".to_string(),
        size_bytes: 400,
        last_accessed_at: None,
        content_hash: None,
    })
    .expect("upsert entry");

    repo.upsert_quota(QuotaRecord {
        id: "quota_q2".to_string(),
        cache_root_id: "root_q2".to_string(),
        max_bytes: 1000,
        action: "warn".to_string(),
    })
    .expect("upsert quota");

    let status = repo
        .check_quota("root_q2")
        .expect("check quota")
        .expect("some status");
    assert_eq!(status.max_bytes, 1000);
    assert_eq!(status.current_bytes, 400);
    assert!(!status.is_over);
}

#[test]
fn check_quota_at_limit_is_over() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.cache();

    repo.create_root("root_q3", "/tmp/cache/q3", "2026-03-25T00:00:00Z")
        .expect("create root");

    repo.upsert_entry(CacheEntryRecord {
        id: "entry_q3a".to_string(),
        cache_root_id: "root_q3".to_string(),
        path: "/tmp/cache/q3/file".to_string(),
        size_bytes: 1000,
        last_accessed_at: None,
        content_hash: None,
    })
    .expect("upsert entry");

    repo.upsert_quota(QuotaRecord {
        id: "quota_q3".to_string(),
        cache_root_id: "root_q3".to_string(),
        max_bytes: 1000,
        action: "block".to_string(),
    })
    .expect("upsert quota");

    let status = repo
        .check_quota("root_q3")
        .expect("check quota")
        .expect("some status");
    assert_eq!(status.max_bytes, 1000);
    assert_eq!(status.current_bytes, 1000);
    assert!(status.is_over);
}

#[test]
fn check_quota_over_limit_is_over() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.cache();

    repo.create_root("root_q4", "/tmp/cache/q4", "2026-03-25T00:00:00Z")
        .expect("create root");

    repo.upsert_entry(CacheEntryRecord {
        id: "entry_q4a".to_string(),
        cache_root_id: "root_q4".to_string(),
        path: "/tmp/cache/q4/fileA".to_string(),
        size_bytes: 800,
        last_accessed_at: None,
        content_hash: None,
    })
    .expect("upsert entry a");

    repo.upsert_entry(CacheEntryRecord {
        id: "entry_q4b".to_string(),
        cache_root_id: "root_q4".to_string(),
        path: "/tmp/cache/q4/fileB".to_string(),
        size_bytes: 500,
        last_accessed_at: None,
        content_hash: None,
    })
    .expect("upsert entry b");

    repo.upsert_quota(QuotaRecord {
        id: "quota_q4".to_string(),
        cache_root_id: "root_q4".to_string(),
        max_bytes: 1000,
        action: "prune".to_string(),
    })
    .expect("upsert quota");

    let status = repo
        .check_quota("root_q4")
        .expect("check quota")
        .expect("some status");
    assert_eq!(status.max_bytes, 1000);
    assert_eq!(status.current_bytes, 1300);
    assert!(status.is_over);
}

#[test]
fn check_quota_zero_max_bytes_with_empty_cache_is_over() {
    // A quota of 0 bytes means any usage (even 0 current bytes) is considered over,
    // because is_over is defined as current_bytes >= max_bytes.
    let store = SqliteStore::in_memory().expect("sqlite store");
    let repo = store.cache();

    repo.create_root("root_q5", "/tmp/cache/q5", "2026-03-25T00:00:00Z")
        .expect("create root");

    repo.upsert_quota(QuotaRecord {
        id: "quota_q5".to_string(),
        cache_root_id: "root_q5".to_string(),
        max_bytes: 0,
        action: "block".to_string(),
    })
    .expect("upsert quota");

    // No entries inserted — current_bytes is 0, max_bytes is 0, so 0 >= 0 → is_over = true.
    let status = repo
        .check_quota("root_q5")
        .expect("check quota")
        .expect("some status");
    assert_eq!(status.max_bytes, 0);
    assert_eq!(status.current_bytes, 0);
    assert!(status.is_over, "quota of 0 bytes must always be over");
}
