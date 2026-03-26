use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use hc_workflow::{
    BootstrapStatusSummary, LoadWorkflowRequest, WorkflowErrorCode, WorkflowRuntime, WorkflowState,
};

fn temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("hc-workflow-runtime-{label}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

fn write_workflow(root: &Path, contents: &str) -> PathBuf {
    let path = root.join("WORKFLOW.md");
    fs::write(&path, contents).expect("workflow file");
    path
}

#[test]
fn runtime_starts_in_none_when_no_workflow_exists() {
    let root = temp_dir("none");

    let runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root,
        explicit_workflow_path: None,
    });

    assert_eq!(runtime.state(), WorkflowState::None);
    assert!(runtime.current().is_none());
}

#[test]
fn valid_reload_promotes_state_to_ok() {
    let root = temp_dir("valid-reload");
    write_workflow(
        &root,
        "---\nworkflow:\n  name: Valid Reload\n---\n{{task.title}}\n",
    );

    let mut runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root,
        explicit_workflow_path: None,
    });
    runtime.reload().expect("valid workflow reload");

    assert_eq!(runtime.state(), WorkflowState::Ok);
    assert!(runtime.current().is_some());
    assert!(runtime.last_known_good().is_some());
}

#[test]
fn invalid_reload_keeps_last_known_good() {
    let root = temp_dir("invalid-kept");
    let workflow_path = write_workflow(
        &root,
        "---\nworkflow:\n  name: Before Invalid Reload\n---\n{{task.title}}\n",
    );

    let mut runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root.clone(),
        explicit_workflow_path: None,
    });
    runtime.reload().expect("initial load");
    let initial_hash = runtime
        .current()
        .expect("current workflow")
        .contract_hash
        .clone();

    fs::write(
        workflow_path,
        "---\nworkflow:\n  name: Broken Reload\n  max_slots: [\n---\n{{task.title}}\n",
    )
    .expect("invalid rewrite");

    let error = runtime.reload().expect_err("invalid reload should fail");

    assert_eq!(error.code(), WorkflowErrorCode::FrontMatterParse);
    assert_eq!(runtime.state(), WorkflowState::InvalidKeptLastGood);
    assert_eq!(
        runtime
            .current()
            .expect("last known good kept")
            .contract_hash,
        initial_hash
    );
}

#[test]
fn runtime_can_mark_reload_pending_during_debounce() {
    let root = temp_dir("pending");
    let mut runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root,
        explicit_workflow_path: None,
    });

    runtime.mark_reload_pending();

    assert_eq!(runtime.state(), WorkflowState::ReloadPending);
}

#[test]
fn future_launch_hash_changes_while_running_session_binding_stays_pinned() {
    let root = temp_dir("launch-pin");
    let workflow_path = write_workflow(
        &root,
        "---\nworkflow:\n  name: Initial Hash\n---\n{{task.title}}\n",
    );

    let mut runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root.clone(),
        explicit_workflow_path: None,
    });
    runtime.reload().expect("initial load");
    let launch_binding = runtime.prepare_launch().expect("launch binding");
    let initial_hash = launch_binding.contract_hash.clone();

    fs::write(
        workflow_path,
        "---\nworkflow:\n  name: Updated Hash\n---\n{{task.title}}\n",
    )
    .expect("updated workflow");
    runtime.reload().expect("valid reload");

    let next_launch_hash = runtime
        .prepare_launch()
        .expect("future launch binding")
        .contract_hash;

    assert_ne!(next_launch_hash, initial_hash);
    assert_eq!(launch_binding.contract_hash, initial_hash);
}

#[test]
fn pending_watch_reloads_after_debounce_and_updates_last_reload_time() {
    let root = temp_dir("watch-reload");
    let workflow_path = write_workflow(
        &root,
        "---\nworkflow:\n  name: Initial Watch\n---\n{{task.title}}\n",
    );

    let mut runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root.clone(),
        explicit_workflow_path: None,
    });
    runtime.reload().expect("initial load");
    let initial_hash = runtime
        .prepare_launch()
        .expect("initial launch binding")
        .contract_hash;

    fs::write(
        workflow_path,
        "---\nworkflow:\n  name: Updated Watch\n---\n{{task.title}}\n",
    )
    .expect("updated workflow");

    runtime.mark_reload_pending();
    thread::sleep(Duration::from_millis(1100));
    runtime.poll_watch().expect("watch poll should reload");

    let updated_hash = runtime
        .prepare_launch()
        .expect("updated launch binding")
        .contract_hash;

    assert_eq!(runtime.state(), WorkflowState::Ok);
    assert_ne!(updated_hash, initial_hash);
    assert!(runtime.last_reload_at().is_some());
}

#[test]
fn runtime_tracks_last_bootstrap_summary_without_changing_launch_hash_semantics() {
    let root = temp_dir("bootstrap-summary");
    write_workflow(
        &root,
        "---\nworkflow:\n  name: Bootstrap Summary\n---\n{{task.title}}\n",
    );

    let mut runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root,
        explicit_workflow_path: None,
    });
    runtime.reload().expect("initial load");
    runtime.record_bootstrap(BootstrapStatusSummary {
        workspace_root: "/tmp/demo/worktrees/task-104".to_string(),
        base_root: ".".to_string(),
        session_cwd: "/tmp/demo/worktrees/task-104".to_string(),
        rendered_prompt_path: "/tmp/demo/worktrees/task-104/prompt.rendered.md".to_string(),
        phase_sequence: vec!["resolve".to_string(), "launch".to_string()],
        hook_phase_results: Vec::new(),
        outcome_code: "launch_succeeded".to_string(),
        warning_codes: Vec::new(),
        claim_released: false,
        launch_exit_code: Some(0),
        last_known_good_hash: runtime
            .prepare_launch()
            .map(|binding| binding.contract_hash),
    });

    assert_eq!(
        runtime
            .last_bootstrap()
            .expect("bootstrap summary")
            .outcome_code,
        "launch_succeeded"
    );
    assert!(runtime.prepare_launch().is_some());
}

#[test]
fn reload_invalid_workflow_retains_last_good() {
    let root = temp_dir("reload-invalid-retained");
    let workflow_path = write_workflow(
        &root,
        "---\nworkflow:\n  name: Good Before Invalid\n---\n{{task.title}}\n",
    );

    let mut runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root.clone(),
        explicit_workflow_path: None,
    });
    runtime.reload().expect("initial valid load");
    let good_hash = runtime
        .current()
        .expect("current after valid load")
        .contract_hash
        .clone();

    // Write a broken workflow.
    fs::write(
        &workflow_path,
        "---\nworkflow:\n  name: Broken Again\n  max_slots: [\n---\n{{task.title}}\n",
    )
    .expect("write broken workflow");
    runtime.reload().expect_err("broken reload must fail");

    assert_eq!(runtime.state(), WorkflowState::InvalidKeptLastGood);
    assert_eq!(
        runtime
            .current()
            .expect("last good kept after broken reload")
            .contract_hash,
        good_hash,
        "contract_hash must remain the last-good hash after an invalid reload"
    );
}

#[test]
fn reload_valid_workflow_after_invalid_becomes_ok() {
    let root = temp_dir("reload-recover-ok");
    let workflow_path = write_workflow(
        &root,
        "---\nworkflow:\n  name: Initial Good\n---\n{{task.title}}\n",
    );

    let mut runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root.clone(),
        explicit_workflow_path: None,
    });
    runtime.reload().expect("initial valid load");

    // Break it.
    fs::write(
        &workflow_path,
        "---\nworkflow:\n  name: Broken Middle\n  max_slots: [\n---\n{{task.title}}\n",
    )
    .expect("write broken workflow");
    runtime.reload().expect_err("broken reload must fail");
    assert_eq!(runtime.state(), WorkflowState::InvalidKeptLastGood);

    // Fix it.
    fs::write(
        &workflow_path,
        "---\nworkflow:\n  name: Recovered Good\n---\n{{task.title}}\n",
    )
    .expect("write recovered workflow");
    runtime.reload().expect("recovered reload must succeed");

    assert_eq!(
        runtime.state(),
        WorkflowState::Ok,
        "state must return to Ok after reloading a valid workflow"
    );
}

#[test]
fn invalid_hook_path_reload_keeps_last_good_and_records_last_error() {
    let root = temp_dir("invalid-hook-reload");
    let outside_root = temp_dir("invalid-hook-reload-outside");
    let outside_hook = outside_root.join("outside.sh");
    fs::write(&outside_hook, "#!/bin/sh\nexit 0\n").expect("outside hook");
    let workflow_path = write_workflow(
        &root,
        "---\nworkflow:\n  name: Initial Good\n---\n{{task.title}}\n",
    );

    let mut runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root.clone(),
        explicit_workflow_path: None,
    });
    runtime.reload().expect("initial valid load");
    let good_hash = runtime
        .current()
        .expect("current after valid load")
        .contract_hash
        .clone();

    fs::write(
        &workflow_path,
        format!(
            "---\nhooks:\n  after_create: {}\n---\n{{{{task.title}}}}\n",
            outside_hook.display()
        ),
    )
    .expect("write invalid hook workflow");

    let error = runtime
        .reload()
        .expect_err("invalid hook path reload must fail");

    assert_eq!(runtime.state(), WorkflowState::InvalidKeptLastGood);
    assert_eq!(
        runtime
            .current()
            .expect("last known good remains active")
            .contract_hash,
        good_hash
    );
    assert!(
        error.to_string().contains("hook"),
        "expected invalid hook path error, got: {error}"
    );
    assert!(
        runtime
            .last_error()
            .unwrap_or_default()
            .contains("hook"),
        "expected runtime.last_error to capture hook validation failure"
    );
}
