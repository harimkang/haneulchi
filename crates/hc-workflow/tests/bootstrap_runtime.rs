use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hc_workflow::{
    BootstrapRequest, LoadWorkflowRequest, WorkflowLoader, run_bootstrap,
};

fn temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("hc-bootstrap-runtime-{label}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

fn write_workflow(root: &Path, body: &str) {
    fs::write(root.join("WORKFLOW.md"), body).expect("workflow file");
}

fn load_workflow(root: &Path) -> hc_workflow::LoadedWorkflow {
    WorkflowLoader::load(&LoadWorkflowRequest {
        repo_root: root.to_path_buf(),
        explicit_workflow_path: None,
    })
    .expect("workflow load")
    .expect("workflow exists")
}

fn bootstrap_request(root: &Path, workspace_root: &Path) -> BootstrapRequest {
    BootstrapRequest {
        workflow: load_workflow(root),
        project_name: "demo".to_string(),
        task_id: "task-104".to_string(),
        task_title: "Review auth workflow".to_string(),
        repo_root: root.to_path_buf(),
        workspace_root: workspace_root.to_path_buf(),
        launch_program: "/bin/sh".to_string(),
        launch_args: vec!["-lc".to_string(), "printf launched".to_string()],
    }
}

#[test]
fn bootstrap_runtime_runs_phases_in_documented_order_and_captures_hook_io() {
    let root = temp_dir("order");
    let workspace_root = root.join("worktrees/task-104");
    let after_create = root.join("after-create.sh");
    let before_run = root.join("before-run.sh");

    fs::write(
        &after_create,
        "#!/bin/sh\nprintf '%s' \"$HANEULCHI_SESSION_CWD\" > after-create-env.txt\nprintf 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' >&2\n",
    )
    .expect("after create");
    fs::write(
        &before_run,
        "#!/bin/sh\nprintf '%s' \"$HANEULCHI_WORKSPACE_ROOT\" > before-run-env.txt\n",
    )
    .expect("before run");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&after_create, fs::Permissions::from_mode(0o755)).expect("chmod");
        fs::set_permissions(&before_run, fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    write_workflow(
        &root,
        &format!(
            "---\nworkflow:\n  name: Bootstrap Demo\nworkspace:\n  strategy: worktree\n  base_root: .\nhooks:\n  after_create: {}\n  before_run: {}\n---\nTask: {{{{task.title}}}}\n",
            after_create.display(),
            before_run.display()
        ),
    );

    let result = run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");

    assert_eq!(
        result.phase_sequence,
        vec![
            "resolve",
            "normalize",
            "workspace",
            "paths",
            "after_create",
            "prompt",
            "before_run",
            "launch",
            "after_run",
            "evidence",
        ]
    );
    assert_eq!(result.workspace_root, workspace_root.display().to_string());
    assert!(result.rendered_prompt_path.ends_with("prompt.rendered.md"));
    assert!(result.hook_phase_results.iter().any(|phase| phase.phase == "after_create"));
    assert!(result.hook_phase_results.iter().any(|phase| phase.phase == "before_run"));
    let after_create_result = result
        .hook_phase_results
        .iter()
        .find(|phase| phase.phase == "after_create")
        .expect("after_create result");
    assert!(after_create_result
        .command_path
        .as_deref()
        .expect("command path")
        .starts_with(&workspace_root.display().to_string()));
    assert!(result
        .hook_phase_results
        .iter()
        .find(|phase| phase.phase == "after_create")
        .expect("after_create result")
        .stderr
        .contains("[truncated]"));
}

#[test]
fn before_run_failure_blocks_launch_and_releases_claim_with_typed_outcome() {
    let root = temp_dir("before-run-fail");
    let workspace_root = root.join("worktrees/task-104");
    let before_run = root.join("before-run.sh");

    fs::write(&before_run, "#!/bin/sh\necho blocked >&2\nexit 7\n").expect("before run");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&before_run, fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    write_workflow(
        &root,
        &format!(
            "---\nhooks:\n  before_run: {}\n---\nTask: {{{{task.title}}}}\n",
            before_run.display()
        ),
    );

    let result = run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");

    assert_eq!(result.outcome_code, "workflow_setup_failed");
    assert_eq!(result.claim_released, true);
    assert_eq!(result.launch_exit_code, None);
}

#[test]
fn after_run_failure_adds_warning_without_overwriting_primary_runtime_result() {
    let root = temp_dir("after-run-warning");
    let workspace_root = root.join("worktrees/task-104");
    let after_run = root.join("after-run.sh");

    fs::write(&after_run, "#!/bin/sh\necho warn >&2\nexit 9\n").expect("after run");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&after_run, fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    write_workflow(
        &root,
        &format!(
            "---\nhooks:\n  after_run:\n    run: {}\n    optional: true\n---\nTask: {{{{task.title}}}}\n",
            after_run.display()
        ),
    );

    let result = run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");

    assert_eq!(result.outcome_code, "launch_succeeded");
    assert_eq!(result.launch_exit_code, Some(0));
    assert!(result.warning_codes.contains(&"after_run_failed".to_string()));
}
