use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hc_workflow::{HookPhase, LoadWorkflowRequest, WorkflowErrorCode, WorkflowLoader};

fn temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("hc-workflow-{label}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

fn write_workflow(root: &Path, contents: &str) -> PathBuf {
    let path = root.join("WORKFLOW.md");
    fs::write(&path, contents).expect("workflow file");
    path
}

#[test]
fn discovery_prefers_explicit_path_over_repo_root_file() {
    let root = temp_dir("discovery-explicit");
    let explicit_root = temp_dir("discovery-explicit-alt");
    write_workflow(
        &root,
        "---\nworkflow:\n  name: Repo Root\n---\n{{task.title}}\n",
    );
    let explicit_path = write_workflow(
        &explicit_root,
        "---\nworkflow:\n  name: Explicit\n---\n{{task.title}}\n",
    );

    let loaded = WorkflowLoader::load(&LoadWorkflowRequest {
        repo_root: root.clone(),
        explicit_workflow_path: Some(explicit_path.clone()),
    })
    .expect("load succeeds")
    .expect("workflow discovered");

    assert_eq!(loaded.discovery_path, explicit_path);
    assert_eq!(
        loaded.effective_config.workflow.name.as_deref(),
        Some("Explicit")
    );
}

#[test]
fn valid_front_matter_is_normalized_and_shorthand_hooks_expand() {
    let root = temp_dir("normalize");
    write_workflow(
        &root,
        r#"---
workflow:
  name: Auth Service Workflow
  max_slots: 4
workspace:
  strategy: worktree
  base_root: .
hooks:
  after_create: ./scripts/after-create.sh
review:
  required: true
  checklist:
    - tests passed
agents:
  allowed: [codex, claude]
---
Task: {{task.title}}
"#,
    );

    let loaded = WorkflowLoader::load(&LoadWorkflowRequest {
        repo_root: root.clone(),
        explicit_workflow_path: None,
    })
    .expect("load succeeds")
    .expect("workflow discovered");

    let after_create = loaded
        .effective_config
        .hooks
        .hook(HookPhase::AfterCreate)
        .expect("after_create normalized");
    assert_eq!(
        after_create.run.as_deref(),
        Some("./scripts/after-create.sh")
    );
    assert_eq!(after_create.timeout_sec, 30);
    assert!(!after_create.optional);
    assert_eq!(loaded.template_body.trim(), "Task: {{task.title}}");
    assert!(loaded.contract_hash.starts_with("sha256:"));
}

#[test]
fn unsupported_future_version_is_rejected() {
    let root = temp_dir("unsupported-version");
    write_workflow(&root, "---\nworkflow:\n  version: 2\n---\n{{task.title}}\n");

    let error = WorkflowLoader::load(&LoadWorkflowRequest {
        repo_root: root,
        explicit_workflow_path: None,
    })
    .expect_err("unsupported version must fail");

    assert_eq!(error.code(), WorkflowErrorCode::UnsupportedVersion);
}

#[test]
fn unknown_template_variables_are_rejected() {
    let root = temp_dir("unknown-variable");
    write_workflow(
        &root,
        "---\nworkflow:\n  name: Unknown Variable\n---\n{{totally.invalid}}\n",
    );

    let error = WorkflowLoader::load(&LoadWorkflowRequest {
        repo_root: root,
        explicit_workflow_path: None,
    })
    .expect_err("unknown variable must fail");

    assert_eq!(error.code(), WorkflowErrorCode::UnknownTemplateVariable);
}

#[test]
fn invalid_yaml_reports_front_matter_parse_error_code() {
    let root = temp_dir("invalid-yaml");
    write_workflow(
        &root,
        "---\nworkflow:\n  name: Broken\n  max_slots: [\n---\n{{task.title}}\n",
    );

    let error = WorkflowLoader::load(&LoadWorkflowRequest {
        repo_root: root,
        explicit_workflow_path: None,
    })
    .expect_err("invalid yaml must fail");

    assert_eq!(error.code(), WorkflowErrorCode::FrontMatterParse);
}

#[test]
fn hook_path_outside_repo_root_is_rejected_during_load() {
    let root = temp_dir("outside-hook-root");
    let outside_root = temp_dir("outside-hook-external");
    let outside_hook = outside_root.join("outside-hook.sh");
    fs::write(&outside_hook, "#!/bin/sh\nexit 0\n").expect("outside hook");

    write_workflow(
        &root,
        &format!(
            "---\nhooks:\n  after_create: {}\n---\n{{{{task.title}}}}\n",
            outside_hook.display()
        ),
    );

    let error = WorkflowLoader::load(&LoadWorkflowRequest {
        repo_root: root,
        explicit_workflow_path: None,
    })
    .expect_err("hook path outside repo root must fail");

    assert!(
        error.to_string().contains("hook"),
        "expected hook-path validation error, got: {error}"
    );
}

#[test]
fn haneulchi_front_matter_fields_survive_normalization() {
    let root = temp_dir("haneulchi-front-matter");
    write_workflow(
        &root,
        r#"---
haneulchi:
  board_mapping:
    ready: Ready
    review: Review
  notification_policy:
    review_ready: true
    retry_due: true
  quick_dispatch_presets:
    - ask-for-status
---
{{task.title}}
"#,
    );

    let loaded = WorkflowLoader::load(&LoadWorkflowRequest {
        repo_root: root,
        explicit_workflow_path: None,
    })
    .expect("load succeeds")
    .expect("workflow discovered");

    let normalized = format!("{loaded:#?}");
    assert!(
        normalized.contains("ask-for-status"),
        "expected normalized config to preserve haneulchi quick dispatch presets, got:\n{normalized}"
    );
}
