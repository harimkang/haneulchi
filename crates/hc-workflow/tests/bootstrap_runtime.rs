use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hc_workflow::{
    BootstrapRequest, LoadWorkflowRequest, PrepareBootstrapRequest, WorkflowLoader,
    WorkflowRuntime, WorkflowState, prepare_bootstrap, run_bootstrap,
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

fn prepare_request(root: &Path, workspace_root: &Path) -> PrepareBootstrapRequest {
    PrepareBootstrapRequest {
        workflow: load_workflow(root),
        project_name: "demo".to_string(),
        task_id: "task-104".to_string(),
        task_title: "Review auth workflow".to_string(),
        repo_root: root.to_path_buf(),
        workspace_root: workspace_root.to_path_buf(),
    }
}

fn read_string(path: &Path) -> String {
    fs::read_to_string(path).expect("read file")
}

#[test]
fn bootstrap_runtime_runs_phases_in_documented_order_and_captures_hook_io() {
    let root = temp_dir("order");
    let workspace_root = root.join("worktrees/task-104");
    let after_create = root.join("after-create.sh");
    let before_run = root.join("before-run.sh");

    fs::write(
        &after_create,
        "#!/bin/sh\nprintf '%s' \"$HANEULCHI_SESSION_CWD\" > after-create-env.txt\ni=0\nwhile [ \"$i\" -lt 40000 ]; do\n  printf 'a' >&2\n  i=$((i + 1))\ndone\n",
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

    let result =
        run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");

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
    assert!(
        result
            .hook_phase_results
            .iter()
            .any(|phase| phase.phase == "after_create")
    );
    assert!(
        result
            .hook_phase_results
            .iter()
            .any(|phase| phase.phase == "before_run")
    );
    let after_create_result = result
        .hook_phase_results
        .iter()
        .find(|phase| phase.phase == "after_create")
        .expect("after_create result");
    assert!(
        after_create_result
            .command_path
            .as_deref()
            .expect("command path")
            .starts_with(&workspace_root.display().to_string())
    );
    assert!(
        result
            .hook_phase_results
            .iter()
            .find(|phase| phase.phase == "after_create")
            .expect("after_create result")
            .stderr
            .contains("[truncated]")
    );
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

    let result =
        run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");

    assert_eq!(result.outcome_code, "workflow_setup_failed");
    assert!(result.claim_released);
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

    let result =
        run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");

    assert_eq!(result.outcome_code, "launch_succeeded");
    assert_eq!(result.launch_exit_code, Some(0));
    assert!(
        result
            .warning_codes
            .contains(&"after_run_failed".to_string())
    );
}

#[test]
fn invalid_workflow_produces_invalid_kept_last_good_health() {
    let root = temp_dir("invalid-health");
    let workflow_path = root.join("WORKFLOW.md");

    // Write a valid workflow first and load it.
    write_workflow(
        &root,
        "---\nworkflow:\n  name: Valid First Load\n---\n{{task.title}}\n",
    );
    let mut runtime = WorkflowRuntime::new(LoadWorkflowRequest {
        repo_root: root.clone(),
        explicit_workflow_path: None,
    });
    runtime.reload().expect("first load should succeed");
    assert_eq!(runtime.state(), WorkflowState::Ok);

    // Overwrite with a broken workflow (invalid YAML front-matter).
    fs::write(
        &workflow_path,
        "---\nworkflow:\n  name: Broken\n  max_slots: [\n---\n{{task.title}}\n",
    )
    .expect("write broken workflow");

    let err = runtime
        .reload()
        .expect_err("reload of broken workflow must fail");
    let _ = err; // error is expected; we care about the resulting state
    assert_eq!(
        runtime.state(),
        WorkflowState::InvalidKeptLastGood,
        "runtime must be in InvalidKeptLastGood after a failed reload"
    );
}

#[test]
fn before_run_hook_failure_is_reported_in_bootstrap() {
    let root = temp_dir("before-run-hook-failure-report");
    let workspace_root = root.join("worktrees/task-104");
    let before_run = root.join("before-run-fail.sh");

    fs::write(&before_run, "#!/bin/sh\necho 'hook failed' >&2\nexit 1\n")
        .expect("write before_run script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&before_run, fs::Permissions::from_mode(0o755))
            .expect("chmod before_run");
    }

    write_workflow(
        &root,
        &format!(
            "---\nhooks:\n  before_run: {}\n---\nTask: {{{{task.title}}}}\n",
            before_run.display()
        ),
    );

    let result =
        run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");

    // A before_run hook failure must surface via outcome_code and release the claim.
    assert_eq!(
        result.outcome_code, "workflow_setup_failed",
        "before_run hook exit 1 must set outcome_code to workflow_setup_failed"
    );
    assert!(
        result.claim_released,
        "claim must be released when before_run hook fails"
    );
}

#[test]
fn bootstrap_runtime_uses_hc_prefixed_hook_env_vars() {
    let root = temp_dir("hc-prefixed-env");
    let workspace_root = root.join("worktrees/task-104");
    let after_create = root.join("after-create-env.sh");

    fs::write(
        &after_create,
        "#!/bin/sh\nprintf '%s|%s|%s' \"$HC_SESSION_CWD\" \"$HC_WORKFLOW_HASH\" \"$HC_HOOK_PHASE\" > hc-env.txt\n",
    )
    .expect("after create");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&after_create, fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    write_workflow(
        &root,
        &format!(
            "---\nhooks:\n  after_create: {}\n---\n{{{{task.title}}}}\n",
            after_create.display()
        ),
    );

    let result =
        run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");
    let captured = read_string(&PathBuf::from(&result.session_cwd).join("hc-env.txt"));

    assert!(
        captured.contains(&result.session_cwd),
        "expected HC_SESSION_CWD to be populated, got: {captured}"
    );
    assert!(
        captured.contains("sha256:"),
        "expected HC_WORKFLOW_HASH to be populated, got: {captured}"
    );
    assert!(
        captured.contains("after_create"),
        "expected HC_HOOK_PHASE to be populated, got: {captured}"
    );
}

#[test]
fn optional_before_run_failure_records_warning_and_continues_launch() {
    let root = temp_dir("optional-before-run");
    let workspace_root = root.join("worktrees/task-104");
    let before_run = root.join("before-run-optional.sh");

    fs::write(
        &before_run,
        "#!/bin/sh\necho optional-blocked >&2\nexit 7\n",
    )
    .expect("before run optional");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&before_run, fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    write_workflow(
        &root,
        &format!(
            "---\nhooks:\n  before_run:\n    run: {}\n    optional: true\n---\nTask: {{{{task.title}}}}\n",
            before_run.display()
        ),
    );

    let result =
        run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");

    assert_eq!(
        result.outcome_code, "launch_succeeded",
        "optional before_run failure should not block launch"
    );
    assert_eq!(result.launch_exit_code, Some(0));
    assert!(
        result
            .warning_codes
            .iter()
            .any(|code| code.contains("before_run")),
        "expected optional before_run failure warning, got: {:?}",
        result.warning_codes
    );
}

#[test]
fn before_run_timeout_is_reported_as_setup_failure() {
    let root = temp_dir("before-run-timeout");
    let workspace_root = root.join("worktrees/task-104");
    let before_run = root.join("before-run-timeout.sh");

    fs::write(&before_run, "#!/bin/sh\nsleep 1\n").expect("before run timeout");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&before_run, fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    write_workflow(
        &root,
        &format!(
            "---\nhooks:\n  before_run:\n    run: {}\n    timeout_sec: 0\n---\nTask: {{{{task.title}}}}\n",
            before_run.display()
        ),
    );

    let result =
        run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");

    assert_eq!(
        result.outcome_code, "workflow_setup_failed",
        "before_run timeout should block launch"
    );
    assert!(result.claim_released);
}

#[test]
fn prompt_artifact_renders_documented_scalar_catalog() {
    let root = temp_dir("render-scalar-catalog");
    let workspace_root = root.join("worktrees/task-104");
    write_workflow(
        &root,
        r#"---
review:
  required: true
  checklist:
    - tests passed
    - diff reviewed
---
# {{task.key}} / {{task.title}}
Session: {{session.id}}
Today: {{now.date}}
Checklist:
{{review.checklist_plain}}
"#,
    );

    let result =
        run_bootstrap(bootstrap_request(&root, &workspace_root)).expect("bootstrap result");
    let rendered = read_string(&PathBuf::from(result.rendered_prompt_path));

    assert!(
        !rendered.contains("{{"),
        "expected all documented scalar placeholders to render, got:\n{rendered}"
    );
    assert!(
        rendered.contains("tests passed"),
        "expected review checklist helpers to render, got:\n{rendered}"
    );
}

#[test]
fn prepare_bootstrap_runs_setup_phases_without_launching_runtime_or_after_run() {
    let root = temp_dir("prepare-only");
    let workspace_root = root.join("worktrees/task-104");
    let after_create = root.join("after-create.sh");
    let before_run = root.join("before-run.sh");
    let after_run = root.join("after-run.sh");

    fs::write(
        &after_create,
        "#!/bin/sh\necho after-create > \"$PWD/after-create.txt\"\n",
    )
    .expect("after create");
    fs::write(
        &before_run,
        "#!/bin/sh\necho before-run > \"$PWD/before-run.txt\"\n",
    )
    .expect("before run");
    fs::write(
        &after_run,
        "#!/bin/sh\necho after-run > \"$PWD/after-run.txt\"\n",
    )
    .expect("after run");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&after_create, fs::Permissions::from_mode(0o755)).expect("chmod");
        fs::set_permissions(&before_run, fs::Permissions::from_mode(0o755)).expect("chmod");
        fs::set_permissions(&after_run, fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    write_workflow(
        &root,
        &format!(
            "---\nhooks:\n  after_create: {}\n  before_run: {}\n  after_run: {}\n---\nTask: {{{{task.title}}}}\n",
            after_create.display(),
            before_run.display(),
            after_run.display()
        ),
    );

    let result = prepare_bootstrap(prepare_request(&root, &workspace_root))
        .expect("prepare bootstrap result");

    assert_eq!(result.outcome_code, "launch_prepared");
    assert!(
        PathBuf::from(&result.session_cwd)
            .join("after-create.txt")
            .exists()
    );
    assert!(
        PathBuf::from(&result.session_cwd)
            .join("before-run.txt")
            .exists()
    );
    assert!(
        !PathBuf::from(&result.session_cwd)
            .join("after-run.txt")
            .exists()
    );
    assert!(PathBuf::from(&result.rendered_prompt_path).exists());
}
