use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::WorkflowError;
use crate::template::validate_template_variables;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HookPhase {
    AfterCreate,
    BeforeRun,
    AfterRun,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedWorkflow {
    pub discovery_path: PathBuf,
    pub template_body: String,
    pub contract_hash: String,
    pub effective_config: EffectiveWorkflowConfig,
    pub resolved_paths: ResolvedPaths,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveWorkflowConfig {
    pub workflow: WorkflowConfig,
    pub workspace: WorkspaceConfig,
    pub hooks: HooksConfig,
    pub review: ReviewConfig,
    pub agents: AgentsConfig,
    pub policy: PolicyConfig,
    pub haneulchi: HaneulchiConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowConfig {
    pub version: u64,
    pub name: Option<String>,
    pub poll_interval_ms: Option<u64>,
    pub max_slots: Option<u64>,
    pub active_task_states: Vec<String>,
    pub terminal_task_states: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceStrategy {
    Worktree,
    SharedRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceConfig {
    pub strategy: WorkspaceStrategy,
    pub base_root: String,
    pub cleanup_on_terminal: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HooksConfig {
    pub after_create: Option<HookDefinition>,
    pub before_run: Option<HookDefinition>,
    pub after_run: Option<HookDefinition>,
}

impl HooksConfig {
    pub fn hook(&self, phase: HookPhase) -> Option<&HookDefinition> {
        match phase {
            HookPhase::AfterCreate => self.after_create.as_ref(),
            HookPhase::BeforeRun => self.before_run.as_ref(),
            HookPhase::AfterRun => self.after_run.as_ref(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HookDefinition {
    pub run: Option<String>,
    pub args: Vec<String>,
    pub timeout_sec: u64,
    pub optional: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReviewConfig {
    pub required: bool,
    pub checklist: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AgentsConfig {
    pub allowed: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PolicyConfig {
    pub max_runtime_minutes: Option<u64>,
    pub unsafe_override_policy: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HaneulchiConfig {
    pub board_mapping_ready: Option<String>,
    pub board_mapping_review: Option<String>,
    pub notify_review_ready: bool,
    pub notify_retry_due: bool,
    pub quick_dispatch_presets: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedPaths {
    pub workflow_file: PathBuf,
    pub workflow_dir: PathBuf,
    pub after_create: Option<PathBuf>,
    pub before_run: Option<PathBuf>,
    pub after_run: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct WorkflowFrontMatter {
    #[serde(default)]
    workflow: WorkflowConfigRaw,
    #[serde(default)]
    workspace: WorkspaceConfigRaw,
    #[serde(default)]
    hooks: HooksConfigRaw,
    #[serde(default)]
    review: ReviewConfigRaw,
    #[serde(default)]
    agents: AgentsConfigRaw,
    #[serde(default)]
    policy: PolicyConfigRaw,
    #[serde(default)]
    haneulchi: HaneulchiConfigRaw,
}

#[derive(Debug, Default, Deserialize)]
struct WorkflowConfigRaw {
    version: Option<u64>,
    name: Option<String>,
    poll_interval_ms: Option<u64>,
    max_slots: Option<u64>,
    #[serde(default)]
    active_task_states: Vec<String>,
    #[serde(default)]
    terminal_task_states: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct WorkspaceConfigRaw {
    strategy: Option<String>,
    base_root: Option<String>,
    cleanup_on_terminal: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct HooksConfigRaw {
    after_create: Option<HookInput>,
    before_run: Option<HookInput>,
    after_run: Option<HookInput>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum HookInput {
    Shorthand(String),
    Detailed(HookInputRaw),
}

#[derive(Debug, Default, Deserialize)]
struct HookInputRaw {
    run: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    timeout_sec: Option<u64>,
    optional: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct ReviewConfigRaw {
    required: Option<bool>,
    #[serde(default)]
    checklist: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct AgentsConfigRaw {
    #[serde(default)]
    allowed: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct PolicyConfigRaw {
    max_runtime_minutes: Option<u64>,
    unsafe_override_policy: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct HaneulchiConfigRaw {
    #[serde(default)]
    board_mapping: HaneulchiBoardMappingRaw,
    #[serde(default)]
    notification_policy: HaneulchiNotificationPolicyRaw,
    #[serde(default)]
    quick_dispatch_presets: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct HaneulchiBoardMappingRaw {
    ready: Option<String>,
    review: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct HaneulchiNotificationPolicyRaw {
    review_ready: Option<bool>,
    retry_due: Option<bool>,
}

pub(crate) fn parse_workflow_file(
    repo_root: &Path,
    workflow_file: &Path,
    contents: &str,
) -> Result<LoadedWorkflow, WorkflowError> {
    let (front_matter, template_body) = split_front_matter(contents)?;
    validate_template_variables(template_body)?;

    let front_matter = match front_matter {
        Some(front_matter) => serde_yaml::from_str::<WorkflowFrontMatter>(front_matter)
            .map_err(|error| WorkflowError::FrontMatterParse(error.to_string()))?,
        None => WorkflowFrontMatter::default(),
    };

    let version = front_matter.workflow.version.unwrap_or(1);
    if version != 1 {
        return Err(WorkflowError::UnsupportedVersion(version));
    }

    let workspace = normalize_workspace(front_matter.workspace)?;
    let workflow_dir = workflow_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let hooks = normalize_hooks(repo_root, &workflow_dir, front_matter.hooks)?;

    let resolved_paths = ResolvedPaths {
        workflow_file: workflow_file.to_path_buf(),
        workflow_dir: workflow_dir.clone(),
        after_create: hooks
            .after_create
            .as_ref()
            .and_then(|hook| hook.run.as_ref())
            .map(|path| resolve_hook_path(repo_root, &workflow_dir, path))
            .transpose()?,
        before_run: hooks
            .before_run
            .as_ref()
            .and_then(|hook| hook.run.as_ref())
            .map(|path| resolve_hook_path(repo_root, &workflow_dir, path))
            .transpose()?,
        after_run: hooks
            .after_run
            .as_ref()
            .and_then(|hook| hook.run.as_ref())
            .map(|path| resolve_hook_path(repo_root, &workflow_dir, path))
            .transpose()?,
    };

    let contract_hash = format!("sha256:{:x}", Sha256::digest(contents.as_bytes()));
    Ok(LoadedWorkflow {
        discovery_path: workflow_file.to_path_buf(),
        template_body: template_body.trim().to_string(),
        contract_hash,
        effective_config: EffectiveWorkflowConfig {
            workflow: WorkflowConfig {
                version,
                name: front_matter.workflow.name,
                poll_interval_ms: front_matter.workflow.poll_interval_ms,
                max_slots: front_matter.workflow.max_slots,
                active_task_states: front_matter.workflow.active_task_states,
                terminal_task_states: front_matter.workflow.terminal_task_states,
            },
            workspace,
            hooks,
            review: ReviewConfig {
                required: front_matter.review.required.unwrap_or(false),
                checklist: front_matter.review.checklist,
            },
            agents: AgentsConfig {
                allowed: front_matter.agents.allowed,
            },
            policy: PolicyConfig {
                max_runtime_minutes: front_matter.policy.max_runtime_minutes,
                unsafe_override_policy: front_matter.policy.unsafe_override_policy,
            },
            haneulchi: HaneulchiConfig {
                board_mapping_ready: front_matter.haneulchi.board_mapping.ready,
                board_mapping_review: front_matter.haneulchi.board_mapping.review,
                notify_review_ready: front_matter
                    .haneulchi
                    .notification_policy
                    .review_ready
                    .unwrap_or(false),
                notify_retry_due: front_matter
                    .haneulchi
                    .notification_policy
                    .retry_due
                    .unwrap_or(false),
                quick_dispatch_presets: front_matter.haneulchi.quick_dispatch_presets,
            },
        },
        resolved_paths,
    })
}

fn split_front_matter(contents: &str) -> Result<(Option<&str>, &str), WorkflowError> {
    const FRONT_MATTER_PREFIX: &str = "---\n";
    const FRONT_MATTER_SUFFIX: &str = "\n---\n";

    if !contents.starts_with(FRONT_MATTER_PREFIX) {
        return Ok((None, contents));
    }

    let remaining = &contents[FRONT_MATTER_PREFIX.len()..];
    let Some(end_index) = remaining.find(FRONT_MATTER_SUFFIX) else {
        return Err(WorkflowError::FrontMatterParse(
            "unterminated front matter".to_string(),
        ));
    };

    let front_matter = &remaining[..end_index];
    let body = &remaining[end_index + FRONT_MATTER_SUFFIX.len()..];
    Ok((Some(front_matter), body))
}

fn normalize_workspace(raw: WorkspaceConfigRaw) -> Result<WorkspaceConfig, WorkflowError> {
    let base_root = raw.base_root.unwrap_or_else(|| ".".to_string());
    if Path::new(&base_root).is_absolute() || base_root.split('/').any(|segment| segment == "..") {
        return Err(WorkflowError::InvalidBaseRoot(base_root));
    }

    Ok(WorkspaceConfig {
        strategy: match raw.strategy.as_deref() {
            Some("shared_root") => WorkspaceStrategy::SharedRoot,
            _ => WorkspaceStrategy::Worktree,
        },
        base_root,
        cleanup_on_terminal: raw.cleanup_on_terminal.unwrap_or(false),
    })
}

fn normalize_hooks(
    repo_root: &Path,
    workflow_dir: &Path,
    raw: HooksConfigRaw,
) -> Result<HooksConfig, WorkflowError> {
    Ok(HooksConfig {
        after_create: normalize_hook(
            repo_root,
            workflow_dir,
            raw.after_create,
            HookPhase::AfterCreate,
        )?,
        before_run: normalize_hook(repo_root, workflow_dir, raw.before_run, HookPhase::BeforeRun)?,
        after_run: normalize_hook(repo_root, workflow_dir, raw.after_run, HookPhase::AfterRun)?,
    })
}

fn normalize_hook(
    repo_root: &Path,
    workflow_dir: &Path,
    input: Option<HookInput>,
    phase: HookPhase,
) -> Result<Option<HookDefinition>, WorkflowError> {
    let definition = input.map(|input| match input {
        HookInput::Shorthand(run) => HookDefinition {
            run: Some(run),
            args: Vec::new(),
            timeout_sec: default_timeout(phase),
            optional: false,
        },
        HookInput::Detailed(raw) => HookDefinition {
            run: raw.run,
            args: raw.args,
            timeout_sec: raw.timeout_sec.unwrap_or(default_timeout(phase)),
            optional: raw.optional.unwrap_or(false),
        },
    });

    if let Some(run) = definition.as_ref().and_then(|hook| hook.run.as_deref()) {
        let _ = resolve_hook_path(repo_root, workflow_dir, run)?;
    }

    Ok(definition)
}

fn default_timeout(phase: HookPhase) -> u64 {
    match phase {
        HookPhase::AfterRun => 60,
        HookPhase::AfterCreate | HookPhase::BeforeRun => 30,
    }
}

fn resolve_hook_path(
    repo_root: &Path,
    workflow_dir: &Path,
    hook_path: &str,
) -> Result<PathBuf, WorkflowError> {
    let path = Path::new(hook_path);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        normalize_lexical_path(workflow_dir.join(path))
    };

    let repo_root = normalize_lexical_path(repo_root.to_path_buf());
    if !resolved.starts_with(&repo_root) {
        return Err(WorkflowError::InvalidHookPath(hook_path.to_string()));
    }

    Ok(resolved)
}

fn normalize_lexical_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
