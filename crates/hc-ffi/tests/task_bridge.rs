use std::ffi::{CStr, CString};
use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use hc_ffi::{
    hc_string_free, hc_task_board_json, hc_task_move_json, reset_test_state, task_board_json,
    task_move_json, task_prepare_isolated_launch_json, workflow_reload_json,
};
use serde_json::Value;

static TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn task_board_bridge_exports_columns_and_mutations() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();

    let projection: Value = serde_json::from_str(&task_board_json(None).expect("board json"))
        .expect("valid board json");
    assert_eq!(projection["columns"].as_array().unwrap().len(), 6);
    assert_eq!(projection["columns"][1]["column"], "ready");

    let moved: Value =
        serde_json::from_str(&task_move_json("task_ready", "review").expect("move response"))
            .expect("valid move json");
    assert_eq!(moved["task"]["id"], "task_ready");
    assert_eq!(moved["task"]["column"], "review");

    let refreshed: Value = serde_json::from_str(
        &task_board_json(Some("proj_demo".to_string())).expect("filtered board json"),
    )
    .expect("valid filtered board json");
    assert!(
        refreshed["columns"][1]["tasks"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        refreshed["columns"][3]["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|task| task["id"] == "task_ready")
    );
}

#[test]
fn c_abi_task_board_bridge_matches_json_helpers() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();

    let project = CString::new("proj_demo").unwrap();
    let board_payload = hc_task_board_json(project.as_ptr());
    let board_json = unsafe { CStr::from_ptr(board_payload.ptr) }
        .to_str()
        .unwrap()
        .to_string();
    hc_string_free(board_payload);
    let board_value: Value = serde_json::from_str(&board_json).unwrap();
    assert_eq!(board_value["selected_project_id"], "proj_demo");

    let task = CString::new("task_ready").unwrap();
    let column = CString::new("done").unwrap();
    let move_payload = hc_task_move_json(task.as_ptr(), column.as_ptr());
    let move_json = unsafe { CStr::from_ptr(move_payload.ptr) }
        .to_str()
        .unwrap()
        .to_string();
    hc_string_free(move_payload);
    let move_value: Value = serde_json::from_str(&move_json).unwrap();
    assert_eq!(move_value["task"]["column"], "done");
}

#[test]
fn isolated_launch_prep_uses_last_known_good_runtime_when_current_file_is_invalid() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("hc-task-prepare-{unique}"));
    let workspace = root.join("worktrees/task-104");
    fs::create_dir_all(&workspace).expect("workspace");
    fs::write(
        root.join("WORKFLOW.md"),
        "---\nworkflow:\n  name: Good\n---\n{{task.title}}\n",
    )
    .expect("good workflow");

    let initial_reload: Value = serde_json::from_str(
        &workflow_reload_json(root.to_str().unwrap()).expect("initial reload"),
    )
    .expect("reload json");
    let last_good_hash = initial_reload["last_good_hash"].as_str().unwrap().to_string();

    fs::write(
        root.join("WORKFLOW.md"),
        "---\nworkflow:\n  name: Broken\n  max_slots: [\n---\n{{task.title}}\n",
    )
    .expect("broken workflow");

    let prepared: Value = serde_json::from_str(
        &task_prepare_isolated_launch_json(
            root.to_str().unwrap(),
            "demo",
            "task-104",
            "task-104",
            workspace.to_str().unwrap(),
        )
        .expect("prepare launch json"),
    )
    .expect("prepare json");

    assert_eq!(prepared["outcome_code"], "launch_prepared");
    assert_eq!(prepared["last_known_good_hash"], last_good_hash);
}
