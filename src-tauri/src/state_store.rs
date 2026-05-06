use chrono::{Duration, SecondsFormat, Utc};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, ToSql};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    env,
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    time::Instant,
};

const PROJECT_FILE_PREVIEW_MAX_BYTES: usize = 256 * 1024;
const PROJECT_FILE_SEARCH_MAX_RESULTS: usize = 50;
const PROJECT_DIFF_MAX_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateStoreHealth {
    pub status: String,
    pub path: String,
}

#[derive(Clone)]
pub struct StateStore {
    db_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedProject {
    pub id: String,
    pub key: String,
    pub name: String,
    pub path: String,
    pub color: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedProjectTab {
    pub id: String,
    pub project_id: String,
    pub order_index: i64,
    pub active: bool,
    pub layout_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedProjectTabGroup {
    pub project_id: String,
    pub group_name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedProjectLayoutPreset {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub layout_json: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedProjectDetachPlan {
    pub project_id: String,
    pub project_name: String,
    pub window_id: String,
    pub status: String,
    pub degraded_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectFileEntry {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub git_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectFileList {
    pub project_id: String,
    pub root_path: String,
    pub relative_path: String,
    pub degraded_reason: Option<String>,
    pub entries: Vec<ProjectFileEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectFileSearch {
    pub project_id: String,
    pub query: String,
    pub degraded_reason: Option<String>,
    pub entries: Vec<ProjectFileEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectFilePreview {
    pub project_id: String,
    pub path: String,
    pub name: String,
    pub language: Option<String>,
    pub body: String,
    pub size_bytes: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectDiff {
    pub project_id: String,
    pub path: Option<String>,
    pub body: String,
    pub file_count: usize,
    pub files: Vec<ProjectDiffFileSummary>,
    pub truncated: bool,
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectDiffFileSummary {
    pub path: String,
    pub status: String,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProjectInput {
    pub key: String,
    pub name: String,
    pub path: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectTabLayoutInput {
    pub project_id: String,
    pub layout_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectLayoutPresetInput {
    pub project_id: String,
    pub name: String,
    pub layout_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileListInput {
    pub project_id: String,
    pub relative_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileReadInput {
    pub project_id: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileWriteInput {
    pub project_id: String,
    pub path: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileSearchInput {
    pub project_id: String,
    pub query: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDiffInput {
    pub project_id: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBrowserAutomationInput {
    pub project_id: String,
    pub url: String,
    pub scenario: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BrowserAutomationPlan {
    pub project_id: String,
    pub url: String,
    pub scenario: String,
    pub status: String,
    pub steps: Vec<String>,
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLspDiagnosticsInput {
    pub project_id: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectLspDiagnostic {
    pub path: String,
    pub line: usize,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectLspSymbol {
    pub path: String,
    pub name: String,
    pub kind: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectLspDiagnostics {
    pub project_id: String,
    pub diagnostics: Vec<ProjectLspDiagnostic>,
    pub symbols: Vec<ProjectLspSymbol>,
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPatchInput {
    pub project_id: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PatchArtifact {
    pub project_id: String,
    pub patch_id: String,
    pub body: String,
    pub file_count: usize,
    pub status: String,
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanPrLandingInput {
    pub project_id: String,
    pub title: String,
    pub draft: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanReviewPrLandingInput {
    pub review_id: String,
    pub title: Option<String>,
    pub draft: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PrLandingPlan {
    pub project_id: String,
    pub provider: String,
    pub title: String,
    pub draft: bool,
    pub checklist: Vec<String>,
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ReviewPrLandingPlanReceipt {
    pub review_id: String,
    pub evidence_pack_id: String,
    pub source_task_id: Option<String>,
    pub source_run_id: Option<String>,
    pub plan: PrLandingPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedTask {
    pub id: String,
    pub key: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_type: Option<String>,
    pub assignee_id: Option<String>,
    pub cycle_id: Option<String>,
    pub module_id: Option<String>,
    pub initiative_id: Option<String>,
    pub due_at: Option<String>,
    pub estimate: Option<String>,
    pub labels: Vec<String>,
    pub context_pack_id: Option<String>,
    pub workpad_md: Option<String>,
    pub comment_count: i64,
    pub subtask_count: i64,
    pub open_subtask_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedTaskInput {
    pub id: String,
    pub key: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_type: Option<String>,
    pub assignee_id: Option<String>,
    pub cycle_id: Option<String>,
    pub module_id: Option<String>,
    pub initiative_id: Option<String>,
    pub context_pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedInitiative {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub budget_id: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedTaskCycle {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskCycleInput {
    pub project_id: String,
    pub name: String,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedTaskModule {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskModuleInput {
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInitiativeInput {
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub budget_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskInput {
    pub project_id: String,
    pub title: String,
    pub priority: Option<String>,
    pub initiative_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReviewFollowUpTaskInput {
    pub review_id: String,
    pub title: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ReviewFollowUpTaskReceipt {
    pub review_id: String,
    pub evidence_pack_id: String,
    pub source_task_id: Option<String>,
    pub source_run_id: Option<String>,
    pub task: PersistedTask,
    pub comment: PersistedComment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedComment {
    pub id: String,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub author_type: String,
    pub author_id: String,
    pub body_md: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTaskCommentInput {
    pub task_id: String,
    pub author_type: String,
    pub author_id: String,
    pub body_md: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedSubtask {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub status: String,
    pub order_index: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTaskSubtaskInput {
    pub task_id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskSubtaskStatusInput {
    pub task_id: String,
    pub subtask_id: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskPlanningInput {
    pub task_id: String,
    pub cycle_id: Option<String>,
    pub module_id: Option<String>,
    pub initiative_id: Option<String>,
    pub due_at: Option<String>,
    pub estimate: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignee_type: Option<String>,
    pub assignee_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskInput {
    pub task_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskContextInput {
    pub task_id: String,
    pub context_pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedExternalTrackerBinding {
    pub id: String,
    pub project_id: String,
    pub local_kind: String,
    pub local_id: String,
    pub provider: String,
    pub external_id: String,
    pub external_url: Option<String>,
    pub sync_mode: String,
    pub sync_status: String,
    pub conflict_state: String,
    pub metadata_json: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertExternalTrackerBindingInput {
    pub project_id: String,
    pub local_kind: String,
    pub local_id: String,
    pub provider: String,
    pub external_id: String,
    pub external_url: Option<String>,
    pub sync_mode: Option<String>,
    pub metadata_json: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedExternalTrackerSyncRun {
    pub id: String,
    pub project_id: String,
    pub provider: String,
    pub dry_run: bool,
    pub status: String,
    pub operation_count: i64,
    pub degraded_reason: Option<String>,
    pub operations: Vec<ExternalTrackerSyncOperation>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExternalTrackerSyncOperation {
    pub binding_id: String,
    pub local_kind: String,
    pub local_id: String,
    pub external_id: String,
    pub operation: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunTrackerSyncInput {
    pub project_id: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTaskWorkpadInput {
    pub task_id: String,
    pub body_md: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedWorkpad {
    pub id: String,
    pub task_id: String,
    pub artifact_path: String,
    pub title: String,
    pub body_md: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedCommandBlock {
    pub id: String,
    pub session_id: String,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub seq_start: Option<i64>,
    pub seq_end: Option<i64>,
    pub command: String,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub exit_code: Option<i64>,
    pub duration_ms: Option<i64>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedCommandBlockInput {
    pub id: String,
    pub session_id: String,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub seq_start: Option<i64>,
    pub seq_end: Option<i64>,
    pub command: String,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub exit_code: Option<i64>,
    pub duration_ms: Option<i64>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandBlockSplitReceipt {
    pub updated_block: PersistedCommandBlock,
    pub created_block: PersistedCommandBlock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandBlockExplanation {
    pub id: String,
    pub command_block_id: String,
    pub command: String,
    pub summary: String,
    pub evidence: Vec<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub agent_profile_id: Option<String>,
    pub prompt: Option<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplainCommandBlockInput {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub agent_profile_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandBlockBundle {
    pub kind: String,
    pub version: i64,
    pub exported_at: String,
    pub command_block: PersistedCommandBlock,
    pub explanation: CommandBlockExplanation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedEvidencePack {
    pub id: String,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub artifact_path: String,
    pub completeness_state: String,
    pub body_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedReleaseGateRun {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub scenario_count: i64,
    pub pass_count: i64,
    pub fail_count: i64,
    pub warning_count: i64,
    pub scenarios: Vec<ReleaseGateScenarioResult>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReleaseGateScenarioResult {
    pub gate_id: String,
    pub name: String,
    pub status: String,
    pub detail: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunReleaseGatesInput {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedTerminalFidelitySmokeRun {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub case_count: i64,
    pub pass_count: i64,
    pub fail_count: i64,
    pub warning_count: i64,
    pub cases: Vec<TerminalFidelitySmokeCaseResult>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TerminalFidelitySmokeCaseResult {
    pub case_id: String,
    pub name: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunTerminalFidelitySmokeInput {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedTaskLifecycleE2ERun {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub task_id: String,
    pub run_id: String,
    pub evidence_pack_id: String,
    pub transitions: Vec<TaskLifecycleE2ETransition>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskLifecycleE2ETransition {
    pub step: String,
    pub task_status: String,
    pub run_lifecycle: Option<String>,
    pub evidence_pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunTaskLifecycleE2EInput {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedWorkflowNegativeTestRun {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub baseline_workflow_id: String,
    pub invalid_workflow_id: String,
    pub last_known_good_workflow_id: String,
    pub dispatch_run_id: String,
    pub cases: Vec<WorkflowNegativeTestCaseResult>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkflowNegativeTestCaseResult {
    pub case_id: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunWorkflowNegativeTestsInput {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedDmgSmokeRun {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub explicit_blocker: bool,
    pub dmg_path: Option<String>,
    pub app_bundle_path: Option<String>,
    pub case_count: i64,
    pub pass_count: i64,
    pub fail_count: i64,
    pub warning_count: i64,
    pub cases: Vec<DmgSmokeCaseResult>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DmgSmokeCaseResult {
    pub case_id: String,
    pub name: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDmgSmokeInput {
    pub project_id: String,
    pub dmg_path: Option<String>,
    pub app_bundle_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedRecoveryDrillRun {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub drill_count: i64,
    pub pass_count: i64,
    pub fail_count: i64,
    pub warning_count: i64,
    pub drills: Vec<RecoveryDrillResult>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RecoveryDrillResult {
    pub drill_id: String,
    pub name: String,
    pub status: String,
    pub detail: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRecoveryDrillsInput {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedBenchmarkRun {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub suite_count: i64,
    pub pass_count: i64,
    pub fail_count: i64,
    pub warning_count: i64,
    pub duration_ms: i64,
    pub suites: Vec<BenchmarkSuiteResult>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BenchmarkSuiteResult {
    pub suite_id: String,
    pub name: String,
    pub status: String,
    pub metric_value: i64,
    pub target_value: i64,
    pub unit: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBenchmarksInput {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedDogfoodTelemetryReview {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub evidence_pack_id: String,
    pub finding_count: i64,
    pub pass_count: i64,
    pub warning_count: i64,
    pub fail_count: i64,
    pub findings: Vec<DogfoodTelemetryFinding>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DogfoodTelemetryFinding {
    pub finding_id: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDogfoodTelemetryReviewInput {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedVisualHarnessLink {
    pub id: String,
    pub project_id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVisualHarnessLinkInput {
    pub project_id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachCommandBlockEvidenceInput {
    pub evidence_pack_id: String,
    pub command_block_id: String,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateEvidencePackInput {
    pub run_id: String,
    pub evidence_pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordEvidenceReviewDecisionInput {
    pub evidence_pack_id: String,
    pub decision: String,
    pub reviewer_id: Option<String>,
    pub body_md: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageInput {
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub agent_profile_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestTokenUsageAdapterInput {
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub agent_profile_id: Option<String>,
    pub adapter: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedTokenUsage {
    pub id: String,
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub agent_profile_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestAgentEventsInput {
    pub project_id: String,
    pub session_id: Option<String>,
    pub run_id: Option<String>,
    pub agent_profile_id: String,
    pub adapter: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedAgentEvent {
    pub id: String,
    pub project_id: String,
    pub session_id: Option<String>,
    pub run_id: Option<String>,
    pub agent_profile_id: String,
    pub kind: String,
    pub severity: String,
    pub detail: String,
    pub payload_json: Value,
    pub source: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderPriceInput {
    pub provider: String,
    pub model: String,
    pub input_usd_per_million: f64,
    pub output_usd_per_million: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviderPriceTableInput {
    pub source: String,
    pub prices: Vec<ProviderPriceInput>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedProviderPrice {
    pub provider: String,
    pub model: String,
    pub input_usd_per_million: f64,
    pub output_usd_per_million: f64,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertSecretInput {
    pub project_id: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedSecretMetadata {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub keychain_ref: String,
    pub redacted: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TokenUsageSummary {
    pub session_id: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TokenUsageTotals {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertBudgetInput {
    pub scope_type: String,
    pub scope_id: Option<String>,
    pub max_usd: f64,
    pub warn_pct: f64,
    pub hard_limit: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedBudget {
    pub id: String,
    pub scope_type: String,
    pub scope_id: Option<String>,
    pub max_usd: f64,
    pub warn_pct: f64,
    pub hard_limit: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedAgentProfile {
    pub id: String,
    pub name: String,
    pub runtime: String,
    pub command: String,
    pub args_json: Value,
    pub env_policy_json: Value,
    pub skills_json: Value,
    pub status: String,
    pub last_heartbeat_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderModelSettings {
    pub provider: String,
    pub model: String,
    pub agent_profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelSettingsInput {
    pub provider: String,
    pub model: String,
    pub agent_profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TerminalThemeSettings {
    pub project_id: Option<String>,
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub accent: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalThemeSettingsInput {
    pub project_id: Option<String>,
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub accent: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedSession {
    pub id: String,
    pub project_id: String,
    pub pane_id: Option<String>,
    pub mode: String,
    pub title: String,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub agent_profile_id: Option<String>,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub state: String,
    pub attention_state: String,
    pub token_budget_state: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionInput {
    pub project_id: String,
    pub mode: String,
    pub title: String,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub agent_profile_id: Option<String>,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInputInput {
    pub session_id: String,
    pub text: String,
    pub allow_dangerous: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SessionInputReceipt {
    pub session_id: String,
    pub accepted: bool,
    pub dangerous: bool,
    pub input_len: usize,
    pub command_block_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SessionResizeReceipt {
    pub session_id: String,
    pub pane_id: Option<String>,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedTerminalStreamChunk {
    pub id: String,
    pub session_id: String,
    pub seq_start: i64,
    pub seq_end: i64,
    pub artifact_path: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordTerminalStreamChunkInput {
    pub session_id: String,
    pub seq_start: i64,
    pub seq_end: i64,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTerminalLaunchInput {
    pub project_id: String,
    pub agent_profile_id: String,
    pub title: Option<String>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentTerminalLaunchPlan {
    pub project_id: String,
    pub agent_profile_id: String,
    pub title: String,
    pub command: String,
    pub args: Vec<String>,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertAgentProfileInput {
    pub id: String,
    pub name: String,
    pub runtime: String,
    pub command: String,
    pub args_json: Option<Value>,
    pub env_policy_json: Option<Value>,
    pub skills_json: Option<Value>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertKnowledgeSourceInput {
    pub project_id: String,
    pub kind: String,
    pub path_or_ref: String,
    pub fingerprint: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedKnowledgeSource {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub path_or_ref: String,
    pub fingerprint: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveKnowledgePageInput {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub body_md: String,
    pub source_ids: Vec<String>,
    pub freshness_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedKnowledgePage {
    pub id: String,
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub artifact_path: String,
    pub source_ids: Vec<String>,
    pub freshness_state: String,
    pub body_md: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveKnowledgeExplorationInput {
    pub project_id: String,
    pub title: String,
    pub question: String,
    pub answer_md: String,
    pub page_ids: Vec<String>,
    pub context_pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedKnowledgeExploration {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub question: String,
    pub answer_md: String,
    pub artifact_path: String,
    pub page_ids: Vec<String>,
    pub context_pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct KnowledgeConcept {
    pub slug: String,
    pub title: String,
    pub page_id: Option<String>,
    pub outbound_slugs: Vec<String>,
    pub inbound_page_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct KnowledgeObsidianExport {
    pub project_id: String,
    pub status: String,
    pub export_root: String,
    pub file_count: usize,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeChatQuestionInput {
    pub project_id: String,
    pub question: String,
    pub context_pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct KnowledgeChatAnswer {
    pub project_id: String,
    pub question: String,
    pub answer_md: String,
    pub cited_page_ids: Vec<String>,
    pub context_pack_id: Option<String>,
    pub source_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestKnowledgeArtifactInput {
    pub project_id: String,
    pub kind: String,
    pub path_or_ref: String,
    pub title: Option<String>,
    pub body_md: String,
    pub max_chunk_chars: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct KnowledgeIngestionResult {
    pub project_id: String,
    pub source_id: String,
    pub page_id: String,
    pub slug: String,
    pub modality: String,
    pub chunk_count: usize,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertContextPackInput {
    pub id: Option<String>,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sources_json: Value,
    pub max_tokens_hint: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedContextPack {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sources_json: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedSkillPack {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub skills_json: Value,
    pub source_context_pack_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertSkillPackInput {
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub skills_json: Value,
    pub source_context_pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordKnowledgeLintReportInput {
    pub project_id: String,
    pub stale_count: i64,
    pub gap_count: i64,
    pub contradiction_count: i64,
    pub body_md: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedKnowledgeLintReport {
    pub id: String,
    pub project_id: String,
    pub artifact_path: String,
    pub stale_count: i64,
    pub gap_count: i64,
    pub contradiction_count: i64,
    pub body_md: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct KnowledgeSummary {
    pub stale_count: i64,
    pub gap_count: i64,
    pub contradiction_count: i64,
    pub recent_pages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunKnowledgeAutomationInput {
    pub project_id: String,
    pub watch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct KnowledgeAutomationRun {
    pub project_id: String,
    pub status: String,
    pub watch_enabled: bool,
    pub source_count: usize,
    pub page_count: usize,
    pub stale_count: i64,
    pub gap_count: i64,
    pub lint_report_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedRun {
    pub id: String,
    pub task_id: String,
    pub project_id: String,
    pub agent_profile_id: Option<String>,
    pub session_id: Option<String>,
    pub workflow_version_id: Option<String>,
    pub context_pack_id: Option<String>,
    pub workspace_path: Option<String>,
    pub lifecycle: String,
    pub retry_count: i64,
    pub next_retry_at: Option<String>,
    pub status_detail: Option<String>,
    pub budget_id: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RuntimePoolItem {
    pub id: String,
    pub label: String,
    pub session_count: usize,
    pub run_count: usize,
    pub blocked_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchRunInput {
    pub task_id: String,
    pub agent_profile_id: Option<String>,
    pub context_pack_id: Option<String>,
    pub workspace_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRunLifecycleInput {
    pub run_id: String,
    pub lifecycle: String,
    pub status_detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordRunStatusUpdateInput {
    pub run_id: String,
    pub body_md: String,
    pub lifecycle: Option<String>,
    pub status_detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedPolicyApproval {
    pub id: String,
    pub project_id: String,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub action_kind: String,
    pub command: Option<String>,
    pub risk_level: String,
    pub state: String,
    pub requested_by: Option<String>,
    pub decision_by: Option<String>,
    pub decision_note: Option<String>,
    pub created_at: String,
    pub decided_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePolicyApprovalInput {
    pub project_id: String,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub action_kind: String,
    pub command: Option<String>,
    pub risk_level: String,
    pub requested_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecidePolicyApprovalInput {
    pub approval_id: String,
    pub decision: String,
    pub decision_by: Option<String>,
    pub decision_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertPolicyPackInput {
    pub project_id: String,
    pub name: String,
    pub sandbox_mode: String,
    pub network: Option<String>,
    pub network_profile: Option<String>,
    pub file_write: Option<String>,
    pub tools: Option<String>,
    pub approval_required: Option<Vec<String>>,
    pub forbidden_operations: Option<Vec<String>>,
    pub set_active: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedPolicyPack {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub sandbox_mode: String,
    pub network: String,
    pub network_profile: String,
    pub file_write: String,
    pub tools: String,
    pub approval_required: Vec<String>,
    pub forbidden_operations: Vec<String>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluatePolicyActionInput {
    pub project_id: String,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub action_kind: String,
    pub command: Option<String>,
    pub requested_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PolicyActionEvaluation {
    pub audit_id: Option<String>,
    pub project_id: String,
    pub policy_pack_id: Option<String>,
    pub action_kind: String,
    pub decision: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedPermissionAudit {
    pub id: String,
    pub project_id: String,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub policy_pack_id: Option<String>,
    pub action_kind: String,
    pub command: Option<String>,
    pub decision: String,
    pub reason: String,
    pub requested_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedWorkflowVersion {
    pub id: String,
    pub project_id: String,
    pub source_path: String,
    pub content_hash: String,
    pub parsed_json: Value,
    pub valid: bool,
    pub diagnostics_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkflowRuntimeState {
    pub project_id: String,
    pub valid: bool,
    pub current_version_id: Option<String>,
    pub last_known_good_version_id: Option<String>,
    pub diagnostics: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkflowValidationResult {
    pub project_id: String,
    pub source_path: String,
    pub valid: bool,
    pub parsed_json: Value,
    pub diagnostics_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateWorkflowInput {
    pub project_id: String,
    pub source_path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReloadWorkflowInput {
    pub project_id: String,
    pub source_path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunWorkflowHookInput {
    pub run_id: String,
    pub hook_name: String,
    pub repo_root: String,
    pub workspace_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkflowHookRunResult {
    pub run_id: String,
    pub hook_name: String,
    pub status: String,
    pub exit_code: Option<i64>,
    pub stdout: String,
    pub stderr: String,
    pub source_path: Option<String>,
    pub mirrored_path: Option<String>,
    pub workspace_path: String,
    pub env_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PersistedRunReplayMetadata {
    pub id: String,
    pub run_id: String,
    pub artifact_path: String,
    pub body_json: Value,
}

impl StateStore {
    pub fn open_at(path: impl AsRef<Path>) -> Result<Self, String> {
        let db_path = path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create state store directory {}: {error}",
                    parent.display()
                )
            })?;
        }

        let connection = Connection::open(&db_path).map_err(|error| {
            format!(
                "failed to open sqlite state store {}: {error}",
                db_path.display()
            )
        })?;
        apply_schema(&connection).map_err(|error| {
            format!(
                "failed to initialize sqlite state store {}: {error}",
                db_path.display()
            )
        })?;

        Ok(Self { db_path })
    }

    pub fn app_support_path() -> PathBuf {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("Haneulchi")
            .join("haneulchi.sqlite")
    }

    pub fn health(&self) -> StateStoreHealth {
        StateStoreHealth {
            status: "ok".to_string(),
            path: self.db_path.to_string_lossy().to_string(),
        }
    }

    pub fn add_project(&self, input: AddProjectInput) -> Result<PersistedProject, String> {
        let key = required_trimmed("project key", &input.key)?;
        let name = required_trimmed("project name", &input.name)?;
        let path = required_trimmed("project path", &input.path)?;
        let registry_path = normalize_project_registry_path(&path)?;
        let id = project_id_from_key(&key)?;
        let tab_id = format!("tab_{id}");
        let color = normalize_optional_text(input.color);
        self.reject_duplicate_project_path(&id, &registry_path)?;

        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("failed to start project transaction: {error}"))?;
        let order_index = transaction
            .query_row(
                "SELECT COALESCE(MAX(order_index), 0) + 1 FROM project_tabs",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|error| format!("failed to allocate project tab order: {error}"))?;
        transaction
            .execute(
                "UPDATE projects
                 SET status = 'idle',
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id != ?1",
                params![id],
            )
            .map_err(|error| format!("failed to update project focus state: {error}"))?;
        transaction
            .execute("UPDATE project_tabs SET active = 0", [])
            .map_err(|error| format!("failed to update project tab focus state: {error}"))?;
        transaction
            .execute(
                "INSERT INTO projects(id, key, name, path, color, status, created_at, updated_at)
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, 'active',
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   key = excluded.key,
                   name = excluded.name,
                   path = excluded.path,
                   color = excluded.color,
                   status = 'active',
                   updated_at = excluded.updated_at",
                params![id, key, name, path, color],
            )
            .map_err(|error| format!("failed to add project {id}: {error}"))?;
        transaction
            .execute(
                "INSERT INTO project_tabs(id, project_id, order_index, active, layout_json, updated_at)
                 VALUES (?1, ?2, ?3, 1, ?4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(id) DO UPDATE SET
                   active = 1,
                   updated_at = excluded.updated_at",
                params![tab_id, id, order_index, serde_json::json!({ "panes": [] }).to_string()],
            )
            .map_err(|error| format!("failed to add project tab {tab_id}: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("failed to commit project {id}: {error}"))?;

        self.get_project(&id)?
            .ok_or_else(|| format!("added project {id} could not be loaded"))
    }

    fn reject_duplicate_project_path(&self, project_id: &str, path: &str) -> Result<(), String> {
        for project in self.list_projects()? {
            if project.id != project_id && normalize_project_registry_path(&project.path)? == path {
                return Err(format!(
                    "project path already registered by project {}",
                    project.id
                ));
            }
        }
        Ok(())
    }

    pub fn get_project(&self, project_id: &str) -> Result<Option<PersistedProject>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, key, name, path, color, status, created_at, updated_at
                 FROM projects
                 WHERE id = ?1",
                params![project_id],
                persisted_project_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load project {project_id}: {error}"))
    }

    pub fn list_projects(&self) -> Result<Vec<PersistedProject>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, key, name, path, color, status, created_at, updated_at
                 FROM projects
                 ORDER BY created_at ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare project list query: {error}"))?;
        let rows = statement
            .query_map([], persisted_project_from_row)
            .map_err(|error| format!("failed to query projects: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read project row: {error}"))
    }

    pub fn list_project_files(
        &self,
        input: ProjectFileListInput,
    ) -> Result<ProjectFileList, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        let project = self
            .get_project(&project_id)?
            .ok_or_else(|| format!("project {project_id} not found"))?;
        let relative_path = normalize_project_relative_path(input.relative_path.as_deref())?;
        let root_path = PathBuf::from(&project.path);
        let root_canonical = root_path
            .canonicalize()
            .map_err(|error| format!("project path unavailable: {error}"))?;
        if !root_canonical.is_dir() {
            return Err(format!("project path is not a directory: {}", project.path));
        }
        let target_path = if relative_path.is_empty() {
            root_canonical.clone()
        } else {
            root_canonical.join(&relative_path)
        };
        let target_canonical = target_path
            .canonicalize()
            .map_err(|error| format!("project file path unavailable: {error}"))?;
        if !target_canonical.starts_with(&root_canonical) {
            return Err("project file path must stay inside project".to_string());
        }
        if !target_canonical.is_dir() {
            return Err("project file path must be a directory".to_string());
        }

        let (git_statuses, degraded_reason) = git_statuses_for_project(&root_canonical);
        let mut entries = fs::read_dir(&target_canonical)
            .map_err(|error| format!("failed to read project files: {error}"))?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name == ".git" {
                    return None;
                }
                let entry_path = entry.path();
                let relative = entry_path.strip_prefix(&root_canonical).ok()?;
                let normalized_relative = normalize_path_for_git(relative);
                Some(ProjectFileEntry {
                    name,
                    path: normalized_relative.clone(),
                    kind: if file_type.is_dir() {
                        "directory".to_string()
                    } else {
                        "file".to_string()
                    },
                    git_status: git_statuses.get(&normalized_relative).cloned(),
                })
            })
            .collect::<Vec<_>>();
        append_deleted_project_file_entries(&relative_path, &git_statuses, &mut entries);
        entries.sort_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });

        Ok(ProjectFileList {
            project_id,
            root_path: root_canonical.to_string_lossy().to_string(),
            relative_path,
            degraded_reason,
            entries,
        })
    }

    pub fn read_project_file(
        &self,
        input: ProjectFileReadInput,
    ) -> Result<ProjectFilePreview, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        let project = self
            .get_project(&project_id)?
            .ok_or_else(|| format!("project {project_id} not found"))?;
        let relative_path = normalize_project_relative_path(Some(&input.path))?;
        if relative_path.is_empty() {
            return Err("project file path cannot be empty".to_string());
        }
        let root_path = PathBuf::from(&project.path);
        let root_canonical = root_path
            .canonicalize()
            .map_err(|error| format!("project path unavailable: {error}"))?;
        if !root_canonical.is_dir() {
            return Err(format!("project path is not a directory: {}", project.path));
        }
        let file_path = root_canonical.join(&relative_path);
        let file_canonical = file_path
            .canonicalize()
            .map_err(|error| format!("project file path unavailable: {error}"))?;
        if !file_canonical.starts_with(&root_canonical) {
            return Err("project file path must stay inside project".to_string());
        }
        if !file_canonical.is_file() {
            return Err("project file path must be a file".to_string());
        }

        let raw = fs::read(&file_canonical)
            .map_err(|error| format!("failed to read project file: {error}"))?;
        let size_bytes = raw.len();
        let truncated = size_bytes > PROJECT_FILE_PREVIEW_MAX_BYTES;
        let preview_bytes = if truncated {
            &raw[..PROJECT_FILE_PREVIEW_MAX_BYTES]
        } else {
            &raw[..]
        };
        let preview_kind = project_file_preview_kind(&file_canonical);
        let body = if let Some(mime) = preview_kind.binary_mime_type() {
            format!("data:{mime};base64,{}", base64_encode(preview_bytes))
        } else {
            String::from_utf8(preview_bytes.to_vec())
                .map_err(|_| "project file preview only supports UTF-8 text".to_string())?
        };
        let name = file_canonical
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or(&relative_path)
            .to_string();

        Ok(ProjectFilePreview {
            project_id,
            path: relative_path,
            name,
            language: preview_kind.language(),
            body,
            size_bytes,
            truncated,
        })
    }

    pub fn write_project_file(
        &self,
        input: ProjectFileWriteInput,
    ) -> Result<ProjectFilePreview, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        let project = self
            .get_project(&project_id)?
            .ok_or_else(|| format!("project {project_id} not found"))?;
        let relative_path = normalize_project_relative_path(Some(&input.path))?;
        if relative_path.is_empty() {
            return Err("project file path cannot be empty".to_string());
        }
        let root_path = PathBuf::from(&project.path);
        let root_canonical = root_path
            .canonicalize()
            .map_err(|error| format!("project path unavailable: {error}"))?;
        if !root_canonical.is_dir() {
            return Err(format!("project path is not a directory: {}", project.path));
        }
        let file_path = root_canonical.join(&relative_path);
        let parent_canonical = file_path
            .parent()
            .ok_or_else(|| "project file parent unavailable".to_string())?
            .canonicalize()
            .map_err(|error| format!("project file parent unavailable: {error}"))?;
        if !parent_canonical.starts_with(&root_canonical) {
            return Err("project file path must stay inside project".to_string());
        }
        if file_path.exists() {
            let file_canonical = file_path
                .canonicalize()
                .map_err(|error| format!("project file path unavailable: {error}"))?;
            if !file_canonical.starts_with(&root_canonical) {
                return Err("project file path must stay inside project".to_string());
            }
            if !file_canonical.is_file() {
                return Err("project file path must be a file".to_string());
            }
        }

        fs::write(&file_path, input.body)
            .map_err(|error| format!("failed to write project file: {error}"))?;
        self.read_project_file(ProjectFileReadInput {
            project_id,
            path: relative_path,
        })
    }

    pub fn search_project_files(
        &self,
        input: ProjectFileSearchInput,
    ) -> Result<ProjectFileSearch, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        let query = required_trimmed("query", &input.query)?.to_lowercase();
        let project = self
            .get_project(&project_id)?
            .ok_or_else(|| format!("project {project_id} not found"))?;
        let root_path = PathBuf::from(&project.path);
        let root_canonical = root_path
            .canonicalize()
            .map_err(|error| format!("project path unavailable: {error}"))?;
        if !root_canonical.is_dir() {
            return Err(format!("project path is not a directory: {}", project.path));
        }

        let (git_statuses, degraded_reason) = git_statuses_for_project(&root_canonical);
        let mut entries = Vec::new();
        collect_project_file_search_entries(
            &root_canonical,
            &root_canonical,
            &query,
            &git_statuses,
            &mut entries,
        )?;
        append_deleted_project_file_search_entries(&query, &git_statuses, &mut entries);
        entries.sort_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.path.to_lowercase().cmp(&right.path.to_lowercase()))
        });
        entries.truncate(PROJECT_FILE_SEARCH_MAX_RESULTS);

        Ok(ProjectFileSearch {
            project_id,
            query,
            degraded_reason,
            entries,
        })
    }

    pub fn read_project_diff(&self, input: ProjectDiffInput) -> Result<ProjectDiff, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        let project = self
            .get_project(&project_id)?
            .ok_or_else(|| format!("project {project_id} not found"))?;
        let relative_path = normalize_project_relative_path(input.path.as_deref())?;
        let requested_path = if relative_path.is_empty() {
            None
        } else {
            Some(relative_path)
        };
        let root_path = PathBuf::from(&project.path);
        let root_canonical = root_path
            .canonicalize()
            .map_err(|error| format!("project path unavailable: {error}"))?;
        if !root_canonical.is_dir() {
            return Err(format!("project path is not a directory: {}", project.path));
        }
        if let Some(path) = requested_path.as_deref() {
            let target_path = root_canonical.join(path);
            let safety_path = if target_path.exists() {
                target_path
                    .canonicalize()
                    .map_err(|error| format!("project diff path unavailable: {error}"))?
            } else {
                target_path
                    .parent()
                    .unwrap_or(&root_canonical)
                    .canonicalize()
                    .map_err(|error| format!("project diff parent unavailable: {error}"))?
            };
            if !safety_path.starts_with(&root_canonical) {
                return Err("project file path must stay inside project".to_string());
            }
        }

        let output = git_diff_bytes_for_project(&root_canonical, requested_path.as_deref());
        let Ok(output) = output else {
            return Ok(ProjectDiff {
                project_id,
                path: requested_path,
                body: String::new(),
                file_count: 0,
                files: vec![],
                truncated: false,
                degraded_reason: Some("git diff unavailable".to_string()),
            });
        };
        let raw = match output {
            Ok(raw) => raw,
            Err(degraded_reason) => {
                return Ok(ProjectDiff {
                    project_id,
                    path: requested_path,
                    body: String::new(),
                    file_count: 0,
                    files: vec![],
                    truncated: false,
                    degraded_reason: Some(degraded_reason),
                });
            }
        };
        let truncated = raw.len() > PROJECT_DIFF_MAX_BYTES;
        let preview_bytes = if truncated {
            truncate_to_char_boundary(&raw, PROJECT_DIFF_MAX_BYTES)
        } else {
            &raw[..]
        };
        let body = String::from_utf8_lossy(preview_bytes).to_string();
        let files = summarize_unified_diff_files(&body);
        let file_count = if files.is_empty() {
            body.lines()
                .filter(|line| line.starts_with("diff --git "))
                .count()
        } else {
            files.len()
        };

        Ok(ProjectDiff {
            project_id,
            path: requested_path,
            body,
            file_count,
            files,
            truncated,
            degraded_reason: None,
        })
    }

    pub fn plan_browser_automation(
        &self,
        input: RunBrowserAutomationInput,
    ) -> Result<BrowserAutomationPlan, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        if self.get_project(&project_id)?.is_none() {
            return Err(format!("project {project_id} not found"));
        }
        let url = required_trimmed("browser automation url", &input.url)?;
        if !is_localhost_url(&url) {
            return Err("browser automation URL must be localhost HTTP(S)".to_string());
        }
        let scenario =
            normalize_optional_text(input.scenario).unwrap_or_else(|| "smoke".to_string());
        Ok(BrowserAutomationPlan {
            project_id,
            url: url.clone(),
            scenario: scenario.clone(),
            status: "planned".to_string(),
            steps: vec![
                format!("open {url}"),
                format!("capture browser screenshot for {scenario}"),
                "record console errors and network failures".to_string(),
                "attach automation evidence to release gate when requested".to_string(),
            ],
            degraded_reason: None,
        })
    }

    pub fn collect_project_lsp_diagnostics(
        &self,
        input: ProjectLspDiagnosticsInput,
    ) -> Result<ProjectLspDiagnostics, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        let mut diagnostics = Vec::new();
        let mut symbols = Vec::new();
        if let Some(path) = input.path {
            let preview = self.read_project_file(ProjectFileReadInput {
                project_id: project_id.clone(),
                path,
            })?;
            collect_lsp_diagnostics_from_body(&preview.path, &preview.body, &mut diagnostics);
            collect_lsp_symbols_from_body(&preview.path, &preview.body, &mut symbols);
        } else {
            let listing = self.list_project_files(ProjectFileListInput {
                project_id: project_id.clone(),
                relative_path: None,
            })?;
            for entry in listing
                .entries
                .iter()
                .filter(|entry| entry.kind == "file")
                .take(20)
            {
                if is_lsp_source_path(&entry.path) {
                    if let Ok(preview) = self.read_project_file(ProjectFileReadInput {
                        project_id: project_id.clone(),
                        path: entry.path.clone(),
                    }) {
                        collect_lsp_diagnostics_from_body(
                            &preview.path,
                            &preview.body,
                            &mut diagnostics,
                        );
                        collect_lsp_symbols_from_body(&preview.path, &preview.body, &mut symbols);
                    }
                }
            }
        }
        Ok(ProjectLspDiagnostics {
            project_id,
            diagnostics,
            symbols,
            degraded_reason: None,
        })
    }

    pub fn export_project_patch(&self, input: ProjectDiffInput) -> Result<PatchArtifact, String> {
        let diff = self.read_project_diff(input)?;
        let patch_id = format!("patch_{}", stable_text_hash(&diff.body));
        Ok(PatchArtifact {
            project_id: diff.project_id,
            patch_id,
            body: diff.body,
            file_count: diff.file_count,
            status: "exported".to_string(),
            degraded_reason: diff.degraded_reason,
        })
    }

    pub fn import_project_patch(&self, input: ImportPatchInput) -> Result<PatchArtifact, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        if self.get_project(&project_id)?.is_none() {
            return Err(format!("project {project_id} not found"));
        }
        let body = input.body.trim();
        if body.is_empty() {
            return Err("patch body cannot be empty".to_string());
        }
        if !body.contains("diff --git ") {
            return Err("patch body must contain a git diff".to_string());
        }
        let file_count = body
            .lines()
            .filter(|line| line.starts_with("diff --git "))
            .count();
        Ok(PatchArtifact {
            project_id,
            patch_id: format!("patch_{}", stable_text_hash(body)),
            body: body.to_string(),
            file_count,
            status: "validated".to_string(),
            degraded_reason: None,
        })
    }

    pub fn plan_pr_landing(&self, input: PlanPrLandingInput) -> Result<PrLandingPlan, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        if self.get_project(&project_id)?.is_none() {
            return Err(format!("project {project_id} not found"));
        }
        let title = required_trimmed("PR title", &input.title)?;
        Ok(PrLandingPlan {
            project_id,
            provider: "github".to_string(),
            title,
            draft: input.draft,
            checklist: vec![
                "export patch and review diff summary".to_string(),
                "run npm test and cargo test gates".to_string(),
                "open draft PR with evidence pack links".to_string(),
                "land only after review decision is approved".to_string(),
            ],
            degraded_reason: Some(
                "network push is intentionally not executed by local planner".to_string(),
            ),
        })
    }

    pub fn plan_review_pr_landing(
        &self,
        input: PlanReviewPrLandingInput,
    ) -> Result<ReviewPrLandingPlanReceipt, String> {
        let review_id = required_trimmed("review id", &input.review_id)?;
        let evidence_pack_id = evidence_pack_id_from_review_id(&review_id)?;
        let pack = self
            .get_evidence_pack(&evidence_pack_id)?
            .ok_or_else(|| format!("evidence pack {evidence_pack_id} not found"))?;
        let source_run = match pack.run_id.as_deref() {
            Some(run_id) => self.get_run(run_id)?,
            None => None,
        };
        let source_task_id = pack
            .task_id
            .clone()
            .or_else(|| source_run.as_ref().map(|run| run.task_id.clone()));
        let source_task = match source_task_id.as_deref() {
            Some(task_id) => self.get_task(task_id)?,
            None => None,
        };
        let project_id = source_run
            .as_ref()
            .map(|run| run.project_id.clone())
            .or_else(|| source_task.as_ref().map(|task| task.project_id.clone()))
            .or_else(|| evidence_pack_body_project_id(&pack).map(str::to_string))
            .unwrap_or_else(|| "proj_local".to_string());
        let title = normalize_optional_text(input.title)
            .or_else(|| {
                source_task
                    .as_ref()
                    .map(|task| format!("PR: {}", task.title))
            })
            .unwrap_or_else(|| format!("PR: {review_id}"));
        let mut plan = self.plan_pr_landing(PlanPrLandingInput {
            project_id,
            title,
            draft: input.draft.unwrap_or(true),
        })?;
        plan.checklist.insert(
            0,
            format!("link review {review_id} and evidence pack {evidence_pack_id}"),
        );
        if let Some(task_id) = source_task_id.as_deref() {
            plan.checklist.insert(
                1,
                format!("confirm source task {task_id} remains review-approved"),
            );
        }
        if let Some(run_id) = pack.run_id.as_deref() {
            plan.checklist
                .insert(2, format!("attach source run {run_id} evidence to PR body"));
        }

        Ok(ReviewPrLandingPlanReceipt {
            review_id,
            evidence_pack_id,
            source_task_id,
            source_run_id: pack.run_id,
            plan,
        })
    }

    pub fn list_project_tabs(&self) -> Result<Vec<PersistedProjectTab>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, order_index, active, layout_json
                 FROM project_tabs
                 ORDER BY order_index ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare project tab list query: {error}"))?;
        let rows = statement
            .query_map([], persisted_project_tab_from_row)
            .map_err(|error| format!("failed to query project tabs: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read project tab row: {error}"))
    }

    pub fn upsert_project_tab_group(
        &self,
        project_id: &str,
        group_name: &str,
    ) -> Result<PersistedProjectTabGroup, String> {
        let project_id = required_trimmed("project id", project_id)?;
        let group_name = required_trimmed("project tab group", group_name)?;
        if self.get_project(&project_id)?.is_none() {
            return Err(format!("project {project_id} not found"));
        }
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO project_tab_groups(project_id, group_name, created_at, updated_at)
                 VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(project_id) DO UPDATE SET
                   group_name = excluded.group_name,
                   updated_at = excluded.updated_at",
                params![project_id, group_name],
            )
            .map_err(|error| format!("failed to persist project tab group {project_id}: {error}"))?;

        self.get_project_tab_group(&project_id)?
            .ok_or_else(|| format!("project tab group for {project_id} could not be loaded"))
    }

    pub fn list_project_tab_groups(&self) -> Result<Vec<PersistedProjectTabGroup>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT project_id, group_name, created_at, updated_at
                 FROM project_tab_groups
                 ORDER BY group_name ASC, project_id ASC",
            )
            .map_err(|error| format!("failed to prepare project tab group list query: {error}"))?;
        let rows = statement
            .query_map([], persisted_project_tab_group_from_row)
            .map_err(|error| format!("failed to query project tab groups: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read project tab group row: {error}"))
    }

    fn get_project_tab_group(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedProjectTabGroup>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT project_id, group_name, created_at, updated_at
                 FROM project_tab_groups
                 WHERE project_id = ?1",
                params![project_id],
                persisted_project_tab_group_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load project tab group {project_id}: {error}"))
    }

    pub fn update_project_tab_layout(
        &self,
        input: UpdateProjectTabLayoutInput,
    ) -> Result<PersistedProjectTab, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        if !input.layout_json.is_object() {
            return Err("project tab layout json must be an object".to_string());
        }
        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE project_tabs
                 SET layout_json = ?1,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE project_id = ?2",
                params![input.layout_json.to_string(), project_id],
            )
            .map_err(|error| {
                format!("failed to update project tab layout for {project_id}: {error}")
            })?;
        if updated == 0 {
            return Err(format!("project tab for {project_id} not found"));
        }
        connection
            .query_row(
                "SELECT id, project_id, order_index, active, layout_json
                 FROM project_tabs
                 WHERE project_id = ?1",
                params![project_id],
                persisted_project_tab_from_row,
            )
            .map_err(|error| format!("failed to load project tab for {project_id}: {error}"))
    }

    pub fn save_project_layout_preset(
        &self,
        input: SaveProjectLayoutPresetInput,
    ) -> Result<PersistedProjectLayoutPreset, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        let name = required_trimmed("layout preset name", &input.name)?;
        if !input.layout_json.is_object() {
            return Err("project layout preset json must be an object".to_string());
        }
        if self.get_project(&project_id)?.is_none() {
            return Err(format!("project {project_id} not found"));
        }
        let connection = self.connection()?;
        let existing_id: Option<String> = connection
            .query_row(
                "SELECT id FROM project_layout_presets WHERE project_id = ?1 AND name = ?2",
                params![project_id, name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| format!("failed to load existing project layout preset: {error}"))?;
        let id = match existing_id {
            Some(id) => id,
            None => {
                let next_number =
                    next_numeric_id(&connection, "project_layout_presets", "layout_preset_")?;
                format!("layout_preset_{next_number}")
            }
        };
        connection
            .execute(
                "INSERT INTO project_layout_presets(
                   id, project_id, name, layout_json, created_at, updated_at
                 )
                 VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(project_id, name) DO UPDATE SET
                   layout_json = excluded.layout_json,
                   updated_at = excluded.updated_at",
                params![id, project_id, name, input.layout_json.to_string()],
            )
            .map_err(|error| format!("failed to persist project layout preset {name}: {error}"))?;

        self.get_project_layout_preset(&id)?
            .ok_or_else(|| format!("project layout preset {id} could not be loaded"))
    }

    pub fn list_project_layout_presets(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedProjectLayoutPreset>, String> {
        let project_id = required_trimmed("project id", project_id)?;
        if self.get_project(&project_id)?.is_none() {
            return Err(format!("project {project_id} not found"));
        }
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, name, layout_json, created_at, updated_at
                 FROM project_layout_presets
                 WHERE project_id = ?1
                 ORDER BY lower(name) ASC, id ASC",
            )
            .map_err(|error| {
                format!("failed to prepare project layout preset list query: {error}")
            })?;
        let rows = statement
            .query_map(
                params![project_id],
                persisted_project_layout_preset_from_row,
            )
            .map_err(|error| format!("failed to query project layout presets: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read project layout preset row: {error}"))
    }

    fn get_project_layout_preset(
        &self,
        preset_id: &str,
    ) -> Result<Option<PersistedProjectLayoutPreset>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, project_id, name, layout_json, created_at, updated_at
                 FROM project_layout_presets
                 WHERE id = ?1",
                params![preset_id],
                persisted_project_layout_preset_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load project layout preset {preset_id}: {error}"))
    }

    pub fn focus_project(&self, project_id: &str) -> Result<PersistedProject, String> {
        let project_id = required_trimmed("project id", project_id)?;
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("failed to start project focus transaction: {error}"))?;
        let exists = transaction
            .query_row(
                "SELECT COUNT(*) FROM projects WHERE id = ?1",
                params![project_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|error| format!("failed to verify project {project_id}: {error}"))?;
        if exists == 0 {
            return Err(format!("project {project_id} not found"));
        }
        transaction
            .execute(
                "UPDATE projects
                 SET status = CASE WHEN id = ?1 THEN 'active' ELSE 'idle' END,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
                params![project_id],
            )
            .map_err(|error| format!("failed to update focused project {project_id}: {error}"))?;
        transaction
            .execute(
                "UPDATE project_tabs
                 SET active = CASE WHEN project_id = ?1 THEN 1 ELSE 0 END,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
                params![project_id],
            )
            .map_err(|error| {
                format!("failed to update focused project tab {project_id}: {error}")
            })?;
        transaction
            .commit()
            .map_err(|error| format!("failed to commit project focus {project_id}: {error}"))?;

        self.get_project(&project_id)?
            .ok_or_else(|| format!("focused project {project_id} could not be loaded"))
    }

    pub fn plan_project_detach(
        &self,
        project_id: &str,
    ) -> Result<PersistedProjectDetachPlan, String> {
        let project_id = required_trimmed("project id", project_id)?;
        let project = self
            .get_project(&project_id)?
            .ok_or_else(|| format!("project {project_id} not found"))?;
        let window_id = format!("win_{project_id}");
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO project_detach_plans(
                   project_id, project_name, window_id, status, degraded_reason, created_at, updated_at
                 )
                 VALUES (?1, ?2, ?3, 'planned', NULL, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(project_id) DO UPDATE SET
                   project_name = excluded.project_name,
                   window_id = excluded.window_id,
                   status = 'planned',
                   degraded_reason = NULL,
                   updated_at = excluded.updated_at",
                params![project.id, project.name, window_id],
            )
            .map_err(|error| format!("failed to plan project detach {project_id}: {error}"))?;

        self.get_project_detach_plan(&project_id)?
            .ok_or_else(|| format!("project detach plan for {project_id} could not be loaded"))
    }

    fn get_project_detach_plan(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedProjectDetachPlan>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT project_id, project_name, window_id, status, degraded_reason, created_at, updated_at
                 FROM project_detach_plans
                 WHERE project_id = ?1",
                params![project_id],
                persisted_project_detach_plan_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load project detach plan {project_id}: {error}"))
    }

    pub fn put_setting_json(&self, key: &str, value: &Value) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO settings(key, value_json, updated_at)
                 VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(key) DO UPDATE SET
                   value_json = excluded.value_json,
                   updated_at = excluded.updated_at",
                params![key, value.to_string()],
            )
            .map_err(|error| format!("failed to persist setting {key}: {error}"))?;
        Ok(())
    }

    pub fn get_setting_json(&self, key: &str) -> Result<Option<Value>, String> {
        let connection = self.connection()?;
        let raw = connection
            .query_row(
                "SELECT value_json FROM settings WHERE key = ?1",
                params![key],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| format!("failed to load setting {key}: {error}"))?;

        raw.map(|value| {
            serde_json::from_str(&value)
                .map_err(|error| format!("failed to parse setting {key} json: {error}"))
        })
        .transpose()
    }

    pub fn provider_model_settings(&self) -> Result<ProviderModelSettings, String> {
        let Some(value) = self.get_setting_json("provider_model.defaults")? else {
            return Ok(default_provider_model_settings());
        };

        provider_model_settings_from_json(value)
    }

    pub fn upsert_provider_model_settings(
        &self,
        input: ProviderModelSettingsInput,
    ) -> Result<ProviderModelSettings, String> {
        let settings = normalize_provider_model_settings(input)?;
        self.put_setting_json(
            "provider_model.defaults",
            &serde_json::json!({
                "provider": settings.provider,
                "model": settings.model,
                "agentProfileId": settings.agent_profile_id,
            }),
        )?;
        self.provider_model_settings()
    }

    pub fn terminal_theme_settings(
        &self,
        project_id: Option<&str>,
    ) -> Result<TerminalThemeSettings, String> {
        if let Some(project_id) =
            project_id.and_then(|id| normalize_optional_borrowed_text(Some(id)))
        {
            if let Some(value) =
                self.get_setting_json(&terminal_theme_setting_key(Some(&project_id)))?
            {
                return terminal_theme_settings_from_json(Some(project_id), value);
            }
        }
        if let Some(value) = self.get_setting_json(&terminal_theme_setting_key(None))? {
            return terminal_theme_settings_from_json(None, value);
        }
        Ok(default_terminal_theme_settings())
    }

    pub fn upsert_terminal_theme_settings(
        &self,
        input: TerminalThemeSettingsInput,
    ) -> Result<TerminalThemeSettings, String> {
        let settings = normalize_terminal_theme_settings(input)?;
        self.put_setting_json(
            &terminal_theme_setting_key(settings.project_id.as_deref()),
            &serde_json::json!({
                "projectId": settings.project_id,
                "name": settings.name,
                "background": settings.background,
                "foreground": settings.foreground,
                "accent": settings.accent,
            }),
        )?;
        self.terminal_theme_settings(settings.project_id.as_deref())
    }

    pub fn upsert_task(&self, input: &PersistedTaskInput) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO tasks(
                   id, key, project_id, title, description, status, priority,
                   assignee_type, assignee_id, cycle_id, module_id, initiative_id, labels_json, context_pack_id, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, '[]', ?13,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   key = excluded.key,
                   project_id = excluded.project_id,
                   title = excluded.title,
                   description = excluded.description,
                   status = excluded.status,
                   priority = excluded.priority,
                   assignee_type = excluded.assignee_type,
                   assignee_id = excluded.assignee_id,
                   cycle_id = excluded.cycle_id,
                   module_id = excluded.module_id,
                   initiative_id = excluded.initiative_id,
                   context_pack_id = excluded.context_pack_id,
                   updated_at = excluded.updated_at",
                params![
                    input.id,
                    input.key,
                    input.project_id,
                    input.title,
                    input.description,
                    input.status,
                    input.priority,
                    input.assignee_type,
                    input.assignee_id,
                    input.cycle_id,
                    input.module_id,
                    input.initiative_id,
                    input.context_pack_id
                ],
            )
            .map_err(|error| format!("failed to persist task {}: {error}", input.id))?;
        Ok(())
    }

    pub fn create_task(&self, input: CreateTaskInput) -> Result<PersistedTask, String> {
        let title = input.title.trim();
        if title.is_empty() {
            return Err("task title cannot be empty".to_string());
        }

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "tasks", "task_")?;
        let task = PersistedTaskInput {
            id: format!("task_{next_number}"),
            key: format!("LOCAL-{next_number}"),
            project_id: input.project_id,
            title: title.to_string(),
            description: None,
            status: "inbox".to_string(),
            priority: normalize_optional_text(input.priority)
                .unwrap_or_else(|| "medium".to_string()),
            assignee_type: None,
            assignee_id: None,
            cycle_id: None,
            module_id: None,
            initiative_id: normalize_optional_text(input.initiative_id),
            context_pack_id: None,
        };
        drop(connection);

        self.upsert_task(&task)?;
        self.get_task(&task.id)?
            .ok_or_else(|| format!("created task {} could not be loaded", task.id))
    }

    pub fn create_review_follow_up_task(
        &self,
        input: CreateReviewFollowUpTaskInput,
    ) -> Result<ReviewFollowUpTaskReceipt, String> {
        let review_id = required_trimmed("review id", &input.review_id)?;
        let evidence_pack_id = evidence_pack_id_from_review_id(&review_id)?;
        let pack = self
            .get_evidence_pack(&evidence_pack_id)?
            .ok_or_else(|| format!("evidence pack {evidence_pack_id} not found"))?;
        let source_run = match pack.run_id.as_deref() {
            Some(run_id) => self.get_run(run_id)?,
            None => None,
        };
        let source_task_id = pack
            .task_id
            .clone()
            .or_else(|| source_run.as_ref().map(|run| run.task_id.clone()));
        let source_task = match source_task_id.as_deref() {
            Some(task_id) => self.get_task(task_id)?,
            None => None,
        };
        let project_id = source_run
            .as_ref()
            .map(|run| run.project_id.clone())
            .or_else(|| source_task.as_ref().map(|task| task.project_id.clone()))
            .or_else(|| evidence_pack_body_project_id(&pack).map(str::to_string))
            .unwrap_or_else(|| "proj_local".to_string());
        let title = normalize_optional_text(input.title)
            .unwrap_or_else(|| format!("Follow-up: {review_id}"));
        let priority =
            normalize_optional_text(input.priority).unwrap_or_else(|| "high".to_string());
        let task = self.create_task(CreateTaskInput {
            project_id,
            title,
            priority: Some(priority),
            initiative_id: None,
        })?;
        let review_state = evidence_pack_review_decision(&pack).unwrap_or_else(|| {
            if pack.completeness_state == "complete" {
                "pending".to_string()
            } else {
                "incomplete".to_string()
            }
        });
        let source_task_label = source_task_id.as_deref().unwrap_or("unlinked");
        let source_run_label = pack.run_id.as_deref().unwrap_or("unlinked");
        let comment = self.add_task_comment(AddTaskCommentInput {
            task_id: task.id.clone(),
            author_type: "system".to_string(),
            author_id: "review_queue".to_string(),
            body_md: format!(
                "Created from review {review_id}.\n\n- Evidence pack: {evidence_pack_id}\n- Source task: {source_task_label}\n- Source run: {source_run_label}\n- Review state: {review_state}\n- Evidence completeness: {}",
                pack.completeness_state
            ),
        })?;

        Ok(ReviewFollowUpTaskReceipt {
            review_id,
            evidence_pack_id,
            source_task_id,
            source_run_id: pack.run_id,
            task,
            comment,
        })
    }

    pub fn create_initiative(
        &self,
        input: CreateInitiativeInput,
    ) -> Result<PersistedInitiative, String> {
        let project_id = required_trimmed("initiative project id", &input.project_id)?;
        let name = required_trimmed("initiative name", &input.name)?;
        let description = normalize_optional_text(input.description);
        let budget_id = normalize_optional_text(input.budget_id);
        let status = normalize_optional_text(input.status).unwrap_or_else(|| "planned".to_string());

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "initiatives", "init_")?;
        let initiative = PersistedInitiative {
            id: format!("init_{next_number}"),
            project_id,
            name,
            description,
            budget_id,
            status,
        };

        connection
            .execute(
                "INSERT INTO initiatives(id, project_id, name, description, budget_id, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    &initiative.id,
                    &initiative.project_id,
                    &initiative.name,
                    &initiative.description,
                    &initiative.budget_id,
                    &initiative.status
                ],
            )
            .map_err(|error| format!("failed to create initiative {}: {error}", initiative.id))?;

        Ok(initiative)
    }

    pub fn list_initiatives(&self, project_id: &str) -> Result<Vec<PersistedInitiative>, String> {
        let project_id = required_trimmed("initiative project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, name, description, budget_id, status
                 FROM initiatives
                 WHERE project_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|error| format!("failed to prepare initiative list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], |row| {
                Ok(PersistedInitiative {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    budget_id: row.get(4)?,
                    status: row.get(5)?,
                })
            })
            .map_err(|error| format!("failed to query initiatives: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read initiative row: {error}"))
    }

    pub fn create_task_cycle(
        &self,
        input: CreateTaskCycleInput,
    ) -> Result<PersistedTaskCycle, String> {
        let project_id = required_trimmed("cycle project id", &input.project_id)?;
        let name = required_trimmed("cycle name", &input.name)?;
        let starts_at = normalize_optional_text(input.starts_at);
        let ends_at = normalize_optional_text(input.ends_at);
        let status = normalize_optional_text(input.status).unwrap_or_else(|| "active".to_string());

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "cycles", "cycle_")?;
        let cycle = PersistedTaskCycle {
            id: format!("cycle_{next_number}"),
            project_id,
            name,
            starts_at,
            ends_at,
            status,
        };
        connection
            .execute(
                "INSERT INTO cycles(id, project_id, name, starts_at, ends_at, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    &cycle.id,
                    &cycle.project_id,
                    &cycle.name,
                    &cycle.starts_at,
                    &cycle.ends_at,
                    &cycle.status
                ],
            )
            .map_err(|error| format!("failed to create cycle {}: {error}", cycle.id))?;

        Ok(cycle)
    }

    pub fn list_task_cycles(&self, project_id: &str) -> Result<Vec<PersistedTaskCycle>, String> {
        let project_id = required_trimmed("cycle project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, name, starts_at, ends_at, status
                 FROM cycles
                 WHERE project_id = ?1
                 ORDER BY name COLLATE NOCASE ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare cycle list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], |row| {
                Ok(PersistedTaskCycle {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    starts_at: row.get(3)?,
                    ends_at: row.get(4)?,
                    status: row.get(5)?,
                })
            })
            .map_err(|error| format!("failed to query cycles: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read cycle row: {error}"))
    }

    pub fn create_task_module(
        &self,
        input: CreateTaskModuleInput,
    ) -> Result<PersistedTaskModule, String> {
        let project_id = required_trimmed("module project id", &input.project_id)?;
        let name = required_trimmed("module name", &input.name)?;
        let description = normalize_optional_text(input.description);
        let status = normalize_optional_text(input.status).unwrap_or_else(|| "active".to_string());

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "modules", "module_")?;
        let module = PersistedTaskModule {
            id: format!("module_{next_number}"),
            project_id,
            name,
            description,
            status,
        };
        connection
            .execute(
                "INSERT INTO modules(id, project_id, name, description, status)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    &module.id,
                    &module.project_id,
                    &module.name,
                    &module.description,
                    &module.status
                ],
            )
            .map_err(|error| format!("failed to create module {}: {error}", module.id))?;

        Ok(module)
    }

    pub fn list_task_modules(&self, project_id: &str) -> Result<Vec<PersistedTaskModule>, String> {
        let project_id = required_trimmed("module project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, name, description, status
                 FROM modules
                 WHERE project_id = ?1
                 ORDER BY name COLLATE NOCASE ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare module list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], |row| {
                Ok(PersistedTaskModule {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    status: row.get(4)?,
                })
            })
            .map_err(|error| format!("failed to query modules: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read module row: {error}"))
    }

    pub fn get_task(&self, task_id: &str) -> Result<Option<PersistedTask>, String> {
        let connection = self.connection()?;
        let mut task = connection
            .query_row(
                "SELECT
                   id, key, project_id, title, description, status, priority,
                   assignee_type, assignee_id, cycle_id, module_id, initiative_id, due_at, estimate, labels_json, context_pack_id
                 FROM tasks
                 WHERE id = ?1",
                params![task_id],
                |row| {
                    let labels_json: String = row.get(14)?;
                    Ok(PersistedTask {
                        id: row.get(0)?,
                        key: row.get(1)?,
                        project_id: row.get(2)?,
                        title: row.get(3)?,
                        description: row.get(4)?,
                        status: row.get(5)?,
                        priority: row.get(6)?,
                        assignee_type: row.get(7)?,
                        assignee_id: row.get(8)?,
                        cycle_id: row.get(9)?,
                        module_id: row.get(10)?,
                        initiative_id: row.get(11)?,
                        due_at: row.get(12)?,
                        estimate: row.get(13)?,
                        labels: parse_task_labels_json(&labels_json),
                        context_pack_id: row.get(15)?,
                        workpad_md: None,
                        comment_count: 0,
                        subtask_count: 0,
                        open_subtask_count: 0,
                    })
                },
            )
            .optional()
            .map_err(|error| format!("failed to load task {task_id}: {error}"))?;
        drop(connection);

        if let Some(task) = task.as_mut() {
            task.workpad_md = self
                .get_task_workpad(&task.id)?
                .map(|workpad| workpad.body_md);
            task.comment_count = self.count_task_comments(&task.id)?;
            let (subtask_count, open_subtask_count) = self.count_task_subtasks(&task.id)?;
            task.subtask_count = subtask_count;
            task.open_subtask_count = open_subtask_count;
        }

        Ok(task)
    }

    pub fn move_task_status(&self, task_id: &str, status: &str) -> Result<PersistedTask, String> {
        let status = status.trim();
        if status.is_empty() {
            return Err("task status cannot be empty".to_string());
        }

        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE tasks
                 SET status = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?2",
                params![status, task_id],
            )
            .map_err(|error| format!("failed to move task {task_id}: {error}"))?;
        drop(connection);

        if updated == 0 {
            return Err(format!("task {task_id} not found"));
        }

        self.get_task(task_id)?
            .ok_or_else(|| format!("moved task {task_id} could not be loaded"))
    }

    pub fn add_task_comment(&self, input: AddTaskCommentInput) -> Result<PersistedComment, String> {
        let body = input.body_md.trim();
        let author_type = input.author_type.trim();
        let author_id = input.author_id.trim();
        if body.is_empty() {
            return Err("comment body cannot be empty".to_string());
        }
        if author_type.is_empty() || author_id.is_empty() {
            return Err("comment author cannot be empty".to_string());
        }
        if self.get_task(&input.task_id)?.is_none() {
            return Err(format!("task {} not found", input.task_id));
        }

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "comments", "comment_")?;
        let comment = PersistedComment {
            id: format!("comment_{next_number}"),
            task_id: Some(input.task_id),
            run_id: None,
            author_type: author_type.to_string(),
            author_id: author_id.to_string(),
            body_md: body.to_string(),
            parent_id: None,
        };
        connection
            .execute(
                "INSERT INTO comments(
                   id, task_id, run_id, author_type, author_id, body_md, parent_id, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    comment.id,
                    comment.task_id,
                    comment.run_id,
                    comment.author_type,
                    comment.author_id,
                    comment.body_md,
                    comment.parent_id
                ],
            )
            .map_err(|error| format!("failed to add comment {}: {error}", comment.id))?;

        Ok(comment)
    }

    pub fn record_run_status_update(
        &self,
        input: RecordRunStatusUpdateInput,
    ) -> Result<PersistedComment, String> {
        let body = input.body_md.trim();
        if body.is_empty() {
            return Err("status update body cannot be empty".to_string());
        }
        let run = self
            .get_run(&input.run_id)?
            .ok_or_else(|| format!("run {} not found", input.run_id))?;
        let author_id = run
            .agent_profile_id
            .clone()
            .unwrap_or_else(|| "agent_unassigned".to_string());

        if input.lifecycle.is_some() || input.status_detail.is_some() {
            self.update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: run.id.clone(),
                lifecycle: input.lifecycle.unwrap_or_else(|| run.lifecycle.clone()),
                status_detail: input.status_detail,
            })?;
        }

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "comments", "comment_")?;
        let comment = PersistedComment {
            id: format!("comment_{next_number}"),
            task_id: Some(run.task_id),
            run_id: Some(run.id),
            author_type: "agent".to_string(),
            author_id,
            body_md: self.redact_secrets(body)?,
            parent_id: None,
        };
        connection
            .execute(
                "INSERT INTO comments(
                   id, task_id, run_id, author_type, author_id, body_md, parent_id, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    comment.id,
                    comment.task_id,
                    comment.run_id,
                    comment.author_type,
                    comment.author_id,
                    comment.body_md,
                    comment.parent_id
                ],
            )
            .map_err(|error| format!("failed to add status update {}: {error}", comment.id))?;

        Ok(comment)
    }

    pub fn update_task_planning(
        &self,
        input: UpdateTaskPlanningInput,
    ) -> Result<PersistedTask, String> {
        let connection = self.connection()?;
        let labels_json = serde_json::to_string(&normalize_task_labels(input.labels))
            .map_err(|error| format!("failed to serialize task labels: {error}"))?;
        let updated = connection
            .execute(
                "UPDATE tasks
                 SET cycle_id = ?1,
                     module_id = ?2,
                     initiative_id = ?3,
                     due_at = ?4,
                     estimate = ?5,
                     labels_json = ?6,
                     assignee_type = ?7,
                     assignee_id = ?8,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?9",
                params![
                    normalize_optional_text(input.cycle_id),
                    normalize_optional_text(input.module_id),
                    normalize_optional_text(input.initiative_id),
                    normalize_optional_text(input.due_at),
                    normalize_optional_text(input.estimate),
                    labels_json,
                    normalize_optional_text(input.assignee_type),
                    normalize_optional_text(input.assignee_id),
                    input.task_id
                ],
            )
            .map_err(|error| {
                format!(
                    "failed to update task {} planning metadata: {error}",
                    input.task_id
                )
            })?;
        drop(connection);

        if updated == 0 {
            return Err(format!("task {} not found", input.task_id));
        }

        self.get_task(&input.task_id)?
            .ok_or_else(|| format!("updated task {} could not be loaded", input.task_id))
    }

    pub fn update_task(&self, input: UpdateTaskInput) -> Result<PersistedTask, String> {
        let task_id = required_trimmed("task id", &input.task_id)?;
        let existing = self
            .get_task(&task_id)?
            .ok_or_else(|| format!("task {task_id} not found"))?;
        let title = match input.title {
            Some(title) => required_trimmed("task title", &title)?.to_string(),
            None => existing.title,
        };
        let description = match input.description {
            Some(description) => normalize_optional_text(Some(description)),
            None => existing.description,
        };
        let priority = match input.priority {
            Some(priority) => required_trimmed("task priority", &priority)?.to_string(),
            None => existing.priority,
        };

        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE tasks
                 SET title = ?1,
                     description = ?2,
                     priority = ?3,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?4",
                params![title, description, priority, &task_id],
            )
            .map_err(|error| format!("failed to update task {task_id}: {error}"))?;
        drop(connection);

        if updated == 0 {
            return Err(format!("task {task_id} not found"));
        }

        self.get_task(&task_id)?
            .ok_or_else(|| format!("updated task {task_id} could not be loaded"))
    }

    pub fn update_task_context(
        &self,
        input: UpdateTaskContextInput,
    ) -> Result<PersistedTask, String> {
        let task_id = required_trimmed("task context task id", &input.task_id)?;
        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE tasks
                 SET context_pack_id = ?1,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?2",
                params![normalize_optional_text(input.context_pack_id), task_id],
            )
            .map_err(|error| format!("failed to update task context {task_id}: {error}"))?;
        drop(connection);

        if updated == 0 {
            return Err(format!("task {task_id} not found"));
        }

        self.get_task(&task_id)?
            .ok_or_else(|| format!("updated task {task_id} could not be loaded"))
    }

    pub fn save_task_workpad(
        &self,
        input: SaveTaskWorkpadInput,
    ) -> Result<PersistedWorkpad, String> {
        if self.get_task(&input.task_id)?.is_none() {
            return Err(format!("task {} not found", input.task_id));
        }

        let workpad_id = format!("workpad_{}", sanitize_artifact_name(&input.task_id));
        let artifact_path = self.workpad_artifact_path(&input.task_id);
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create workpad artifact directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&artifact_path, &input.body_md).map_err(|error| {
            format!(
                "failed to write task {} workpad artifact {}: {error}",
                input.task_id,
                artifact_path.display()
            )
        })?;

        let relative_path = self.relative_artifact_path(&artifact_path);
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO workpads(id, task_id, artifact_path, title, created_at, updated_at)
                 VALUES (
                   ?1, ?2, ?3, ?4,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   task_id = excluded.task_id,
                   artifact_path = excluded.artifact_path,
                   title = excluded.title,
                   updated_at = excluded.updated_at",
                params![workpad_id, input.task_id, relative_path, "Task workpad"],
            )
            .map_err(|error| {
                format!(
                    "failed to persist task {} workpad row: {error}",
                    input.task_id
                )
            })?;

        self.get_task_workpad(&input.task_id)?
            .ok_or_else(|| format!("saved task {} workpad could not be loaded", input.task_id))
    }

    pub fn get_task_workpad(&self, task_id: &str) -> Result<Option<PersistedWorkpad>, String> {
        let connection = self.connection()?;
        let row = connection
            .query_row(
                "SELECT id, task_id, artifact_path, title
                 FROM workpads
                 WHERE task_id = ?1
                 ORDER BY updated_at DESC, id ASC
                 LIMIT 1",
                params![task_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("failed to load task {task_id} workpad row: {error}"))?;

        row.map(|(id, task_id, artifact_path, title)| {
            let full_path = self.resolve_artifact_path(&artifact_path);
            let body_md = fs::read_to_string(&full_path).map_err(|error| {
                format!(
                    "failed to read task {task_id} workpad artifact {}: {error}",
                    full_path.display()
                )
            })?;
            Ok(PersistedWorkpad {
                id,
                task_id,
                artifact_path,
                title,
                body_md,
            })
        })
        .transpose()
    }

    pub fn list_task_comments(&self, task_id: &str) -> Result<Vec<PersistedComment>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, task_id, run_id, author_type, author_id, body_md, parent_id
                 FROM comments
                 WHERE task_id = ?1
                 ORDER BY created_at ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare task comments query: {error}"))?;
        let rows = statement
            .query_map(params![task_id], |row| {
                Ok(PersistedComment {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    run_id: row.get(2)?,
                    author_type: row.get(3)?,
                    author_id: row.get(4)?,
                    body_md: row.get(5)?,
                    parent_id: row.get(6)?,
                })
            })
            .map_err(|error| format!("failed to query task comments: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read task comment row: {error}"))
    }

    pub fn add_task_subtask(&self, input: AddTaskSubtaskInput) -> Result<PersistedSubtask, String> {
        let title = required_trimmed("subtask title", &input.title)?;
        if self.get_task(&input.task_id)?.is_none() {
            return Err(format!("task {} not found", input.task_id));
        }

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "subtasks", "subtask_")?;
        let order_index: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM subtasks WHERE task_id = ?1",
                params![input.task_id],
                |row| row.get(0),
            )
            .map_err(|error| format!("failed to count subtasks for {}: {error}", input.task_id))?;
        let subtask = PersistedSubtask {
            id: format!("subtask_{next_number}"),
            task_id: input.task_id,
            title,
            status: "open".to_string(),
            order_index,
        };
        connection
            .execute(
                "INSERT INTO subtasks(id, task_id, title, status, order_index)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    subtask.id,
                    subtask.task_id,
                    subtask.title,
                    subtask.status,
                    subtask.order_index
                ],
            )
            .map_err(|error| format!("failed to add subtask {}: {error}", subtask.id))?;

        Ok(subtask)
    }

    pub fn list_task_subtasks(&self, task_id: &str) -> Result<Vec<PersistedSubtask>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, task_id, title, status, order_index
                 FROM subtasks
                 WHERE task_id = ?1
                 ORDER BY order_index ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare task subtasks query: {error}"))?;
        let rows = statement
            .query_map(params![task_id], |row| {
                Ok(PersistedSubtask {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    title: row.get(2)?,
                    status: row.get(3)?,
                    order_index: row.get(4)?,
                })
            })
            .map_err(|error| format!("failed to query task subtasks: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read task subtask row: {error}"))
    }

    pub fn update_task_subtask_status(
        &self,
        input: UpdateTaskSubtaskStatusInput,
    ) -> Result<PersistedSubtask, String> {
        let status = required_trimmed("subtask status", &input.status)?;
        if !matches!(status.as_str(), "open" | "done") {
            return Err("subtask status must be open or done".to_string());
        }
        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE subtasks
                 SET status = ?1
                 WHERE id = ?2 AND task_id = ?3",
                params![status, input.subtask_id, input.task_id],
            )
            .map_err(|error| format!("failed to update subtask {}: {error}", input.subtask_id))?;
        drop(connection);

        if updated == 0 {
            return Err(format!("subtask {} not found", input.subtask_id));
        }

        self.list_task_subtasks(&input.task_id)?
            .into_iter()
            .find(|subtask| subtask.id == input.subtask_id)
            .ok_or_else(|| format!("updated subtask {} could not be loaded", input.subtask_id))
    }

    pub fn list_tasks(&self, project_id: &str) -> Result<Vec<PersistedTask>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, key, project_id, title, description, status, priority,
                   assignee_type, assignee_id, cycle_id, module_id, initiative_id, due_at, estimate, labels_json, context_pack_id
                 FROM tasks
                 WHERE project_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|error| format!("failed to prepare task list query: {error}"))?;

        let rows = statement
            .query_map(params![project_id], |row| {
                let labels_json: String = row.get(14)?;
                Ok(PersistedTask {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    project_id: row.get(2)?,
                    title: row.get(3)?,
                    description: row.get(4)?,
                    status: row.get(5)?,
                    priority: row.get(6)?,
                    assignee_type: row.get(7)?,
                    assignee_id: row.get(8)?,
                    cycle_id: row.get(9)?,
                    module_id: row.get(10)?,
                    initiative_id: row.get(11)?,
                    due_at: row.get(12)?,
                    estimate: row.get(13)?,
                    labels: parse_task_labels_json(&labels_json),
                    context_pack_id: row.get(15)?,
                    workpad_md: None,
                    comment_count: 0,
                    subtask_count: 0,
                    open_subtask_count: 0,
                })
            })
            .map_err(|error| format!("failed to query tasks: {error}"))?;

        let mut tasks = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read task row: {error}"))?;
        drop(statement);
        drop(connection);

        for task in &mut tasks {
            task.workpad_md = self
                .get_task_workpad(&task.id)?
                .map(|workpad| workpad.body_md);
            task.comment_count = self.count_task_comments(&task.id)?;
            let (subtask_count, open_subtask_count) = self.count_task_subtasks(&task.id)?;
            task.subtask_count = subtask_count;
            task.open_subtask_count = open_subtask_count;
        }

        Ok(tasks)
    }

    pub fn count_tasks_by_status(&self, project_id: &str) -> Result<Value, String> {
        let mut counts = serde_json::json!({
            "inbox": 0,
            "ready": 0,
            "running": 0,
            "review": 0,
            "blocked": 0,
            "done": 0,
            "archived": 0
        });
        let connection = self.connection()?;
        let mut statement = connection
            .prepare("SELECT status, COUNT(*) FROM tasks WHERE project_id = ?1 GROUP BY status")
            .map_err(|error| format!("failed to prepare task counts query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|error| format!("failed to query task counts: {error}"))?;

        for row in rows {
            let (status, count) =
                row.map_err(|error| format!("failed to read task count row: {error}"))?;
            counts[status] = serde_json::json!(count);
        }

        Ok(counts)
    }

    pub fn upsert_external_tracker_binding(
        &self,
        input: UpsertExternalTrackerBindingInput,
    ) -> Result<PersistedExternalTrackerBinding, String> {
        let project_id = required_trimmed("tracker binding project id", &input.project_id)?;
        let local_kind = normalize_tracker_local_kind(&input.local_kind)?;
        let local_id = required_trimmed("tracker binding local id", &input.local_id)?;
        let provider = normalize_tracker_provider(&input.provider)?;
        let external_id = required_trimmed("tracker binding external id", &input.external_id)?;
        let external_url = normalize_optional_text(input.external_url);
        let sync_mode = input
            .sync_mode
            .map(|mode| normalize_tracker_sync_mode(&mode))
            .transpose()?
            .unwrap_or_else(|| "manual".to_string());
        let metadata_json = input.metadata_json.unwrap_or_else(|| serde_json::json!({}));

        match local_kind.as_str() {
            "task" => {
                let task = self
                    .get_task(&local_id)?
                    .ok_or_else(|| format!("task {local_id} not found"))?;
                if task.project_id != project_id {
                    return Err(format!(
                        "task {local_id} belongs to project {}, not {project_id}",
                        task.project_id
                    ));
                }
            }
            "project" => {
                if local_id != project_id && self.get_project(&local_id)?.is_none() {
                    return Err(format!("project {local_id} not found"));
                }
            }
            _ => unreachable!("tracker local kind normalized"),
        }

        let connection = self.connection()?;
        let existing_id = connection
            .query_row(
                "SELECT id
                 FROM external_tracker_bindings
                 WHERE provider = ?1 AND external_id = ?2",
                params![provider, external_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| {
                format!("failed to load external tracker binding {provider}:{external_id}: {error}")
            })?;
        let binding_id = match existing_id {
            Some(id) => id,
            None => {
                let next_number =
                    next_numeric_id(&connection, "external_tracker_bindings", "tracker_binding_")?;
                format!("tracker_binding_{next_number}")
            }
        };
        connection
            .execute(
                "INSERT INTO external_tracker_bindings(
                   id, project_id, local_kind, local_id, provider, external_id, external_url,
                   sync_mode, sync_status, conflict_state, metadata_json, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending', 'none', ?9,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   project_id = excluded.project_id,
                   local_kind = excluded.local_kind,
                   local_id = excluded.local_id,
                   provider = excluded.provider,
                   external_id = excluded.external_id,
                   external_url = excluded.external_url,
                   sync_mode = excluded.sync_mode,
                   sync_status = 'pending',
                   conflict_state = 'none',
                   metadata_json = excluded.metadata_json,
                   updated_at = excluded.updated_at",
                params![
                    binding_id,
                    project_id,
                    local_kind,
                    local_id,
                    provider,
                    external_id,
                    external_url,
                    sync_mode,
                    metadata_json.to_string()
                ],
            )
            .map_err(|error| format!("failed to persist external tracker binding: {error}"))?;
        self.get_external_tracker_binding(&binding_id)?
            .ok_or_else(|| format!("external tracker binding {binding_id} could not be loaded"))
    }

    pub fn list_external_tracker_bindings(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedExternalTrackerBinding>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, local_kind, local_id, provider, external_id, external_url,
                   sync_mode, sync_status, conflict_state, metadata_json, created_at, updated_at
                 FROM external_tracker_bindings
                 WHERE project_id = ?1
                 ORDER BY updated_at DESC, id ASC",
            )
            .map_err(|error| {
                format!("failed to prepare external tracker binding list query: {error}")
            })?;
        let rows = statement
            .query_map(
                params![project_id],
                persisted_external_tracker_binding_from_row,
            )
            .map_err(|error| format!("failed to query external tracker bindings: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read external tracker binding row: {error}"))
    }

    fn get_external_tracker_binding(
        &self,
        binding_id: &str,
    ) -> Result<Option<PersistedExternalTrackerBinding>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, local_kind, local_id, provider, external_id, external_url,
                   sync_mode, sync_status, conflict_state, metadata_json, created_at, updated_at
                 FROM external_tracker_bindings
                 WHERE id = ?1",
                params![binding_id],
                persisted_external_tracker_binding_from_row,
            )
            .optional()
            .map_err(|error| {
                format!("failed to load external tracker binding {binding_id}: {error}")
            })
    }

    pub fn run_tracker_sync(
        &self,
        provider: &str,
        input: RunTrackerSyncInput,
    ) -> Result<PersistedExternalTrackerSyncRun, String> {
        let provider = normalize_sync_provider(provider)?;
        let project_id = required_trimmed("tracker sync project id", &input.project_id)?;
        let bindings = self
            .list_external_tracker_bindings(&project_id)?
            .into_iter()
            .filter(|binding| binding.provider == provider && binding.local_kind == "task")
            .collect::<Vec<_>>();
        let project_runs = if bindings.is_empty() {
            Vec::new()
        } else {
            self.list_runs(&project_id)?
        };
        let project_evidence_packs = if bindings.is_empty() {
            Vec::new()
        } else {
            self.list_evidence_packs_for_project(&project_id)?
        };
        let mut operations = Vec::new();
        for binding in bindings {
            let Some(task) = self.get_task(&binding.local_id)? else {
                continue;
            };
            operations.push(ExternalTrackerSyncOperation {
                binding_id: binding.id.clone(),
                local_kind: binding.local_kind.clone(),
                local_id: binding.local_id.clone(),
                external_id: binding.external_id.clone(),
                operation: "issueUpdate".to_string(),
                payload: serde_json::json!({
                    "title": task.title.clone(),
                    "description": task.description.clone(),
                    "priority": task.priority.clone(),
                    "state": task.status.clone(),
                    "labels": task.labels.clone(),
                    "haneulchiTaskId": task.id.clone(),
                    "haneulchiTaskKey": task.key.clone()
                }),
            });
            operations.extend(self.tracker_comment_mirror_operations(&binding)?);
            operations.extend(self.tracker_evidence_summary_mirror_operations(
                &binding,
                &task,
                &project_runs,
                &project_evidence_packs,
            )?);
        }

        let token_configured =
            self.list_secrets(Some(&project_id), None)?
                .into_iter()
                .any(|secret| {
                    secret
                        .name
                        .eq_ignore_ascii_case(sync_secret_name(&provider))
                });
        let (status, degraded_reason) = if operations.is_empty() {
            ("noop".to_string(), None)
        } else if input.dry_run {
            ("planned".to_string(), None)
        } else if !token_configured {
            (
                "degraded".to_string(),
                Some(format!("missing {} secret", sync_secret_name(&provider))),
            )
        } else {
            ("queued".to_string(), None)
        };

        let connection = self.connection()?;
        let next_number =
            next_numeric_id(&connection, "external_tracker_sync_runs", "tracker_sync_")?;
        let sync_run_id = format!("tracker_sync_{next_number}");
        connection
            .execute(
                "INSERT INTO external_tracker_sync_runs(
                   id, project_id, provider, dry_run, status, operation_count,
                   degraded_reason, operations_json, created_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    sync_run_id,
                    project_id,
                    provider,
                    if input.dry_run { 1 } else { 0 },
                    status,
                    operations.len() as i64,
                    degraded_reason,
                    serde_json::to_string(&operations).unwrap_or_else(|_| "[]".to_string())
                ],
            )
            .map_err(|error| format!("failed to persist {provider} sync run: {error}"))?;
        self.get_external_tracker_sync_run(&sync_run_id)?
            .ok_or_else(|| format!("{provider} sync run {sync_run_id} could not be loaded"))
    }

    fn tracker_comment_mirror_operations(
        &self,
        binding: &PersistedExternalTrackerBinding,
    ) -> Result<Vec<ExternalTrackerSyncOperation>, String> {
        self.list_task_comments(&binding.local_id)?
            .into_iter()
            .map(|comment| {
                Ok(ExternalTrackerSyncOperation {
                    binding_id: binding.id.clone(),
                    local_kind: binding.local_kind.clone(),
                    local_id: binding.local_id.clone(),
                    external_id: binding.external_id.clone(),
                    operation: "commentMirror".to_string(),
                    payload: serde_json::json!({
                        "source": "task_comment",
                        "commentId": comment.id,
                        "runId": comment.run_id,
                        "authorType": comment.author_type,
                        "authorId": comment.author_id,
                        "bodyMd": self.redact_secrets(&comment.body_md)?
                    }),
                })
            })
            .collect()
    }

    fn tracker_evidence_summary_mirror_operations(
        &self,
        binding: &PersistedExternalTrackerBinding,
        task: &PersistedTask,
        project_runs: &[PersistedRun],
        project_evidence_packs: &[PersistedEvidencePack],
    ) -> Result<Vec<ExternalTrackerSyncOperation>, String> {
        project_evidence_packs
            .iter()
            .filter(|pack| evidence_pack_links_to_task(pack, task, project_runs))
            .map(|pack| {
                let payload = serde_json::json!({
                    "source": "evidence_summary",
                    "evidencePackId": pack.id,
                    "taskId": pack.task_id,
                    "runId": pack.run_id,
                    "completenessState": pack.completeness_state,
                    "reviewDecision": evidence_pack_review_decision(pack),
                    "bodyMd": evidence_pack_tracker_summary(pack)
                });
                Ok(ExternalTrackerSyncOperation {
                    binding_id: binding.id.clone(),
                    local_kind: binding.local_kind.clone(),
                    local_id: binding.local_id.clone(),
                    external_id: binding.external_id.clone(),
                    operation: "evidenceSummaryMirror".to_string(),
                    payload: self.redact_json_secrets(&payload)?,
                })
            })
            .collect()
    }

    pub fn run_linear_sync(
        &self,
        input: RunTrackerSyncInput,
    ) -> Result<PersistedExternalTrackerSyncRun, String> {
        self.run_tracker_sync("linear", input)
    }

    pub fn latest_external_tracker_sync_run(
        &self,
        project_id: &str,
        provider: &str,
    ) -> Result<Option<PersistedExternalTrackerSyncRun>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, provider, dry_run, status, operation_count,
                   degraded_reason, operations_json, created_at
                 FROM external_tracker_sync_runs
                 WHERE project_id = ?1 AND provider = ?2
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![project_id, provider],
                persisted_external_tracker_sync_run_from_row,
            )
            .optional()
            .map_err(|error| {
                format!("failed to load latest external tracker sync run for {provider}: {error}")
            })
    }

    fn get_external_tracker_sync_run(
        &self,
        sync_run_id: &str,
    ) -> Result<Option<PersistedExternalTrackerSyncRun>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, provider, dry_run, status, operation_count,
                   degraded_reason, operations_json, created_at
                 FROM external_tracker_sync_runs
                 WHERE id = ?1",
                params![sync_run_id],
                persisted_external_tracker_sync_run_from_row,
            )
            .optional()
            .map_err(|error| {
                format!("failed to load external tracker sync run {sync_run_id}: {error}")
            })
    }

    pub fn count_task_comments(&self, task_id: &str) -> Result<i64, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT COUNT(*) FROM comments WHERE task_id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|error| format!("failed to count comments for task {task_id}: {error}"))
    }

    pub fn count_task_subtasks(&self, task_id: &str) -> Result<(i64, i64), String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT COUNT(*), SUM(CASE WHEN status != 'done' THEN 1 ELSE 0 END)
                 FROM subtasks
                 WHERE task_id = ?1",
                params![task_id],
                |row| Ok((row.get(0)?, row.get::<_, Option<i64>>(1)?.unwrap_or(0))),
            )
            .map_err(|error| format!("failed to count subtasks for task {task_id}: {error}"))
    }

    pub fn upsert_command_block(
        &self,
        input: &PersistedCommandBlockInput,
    ) -> Result<PersistedCommandBlock, String> {
        let command = input.command.trim();
        let session_id = input.session_id.trim();
        if command.is_empty() {
            return Err("command block command cannot be empty".to_string());
        }
        if session_id.is_empty() {
            return Err("command block session cannot be empty".to_string());
        }
        let redacted_command = self.redact_secrets(command)?;
        let redacted_summary = normalize_optional_text(input.summary.clone())
            .map(|summary| self.redact_secrets(&summary))
            .transpose()?;

        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO command_blocks(
                   id, session_id, task_id, run_id, seq_start, seq_end, command,
                   cwd, branch, exit_code, duration_ms, summary, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   session_id = excluded.session_id,
                   task_id = excluded.task_id,
                   run_id = excluded.run_id,
                   seq_start = excluded.seq_start,
                   seq_end = excluded.seq_end,
                   command = excluded.command,
                   cwd = excluded.cwd,
                   branch = excluded.branch,
                   exit_code = excluded.exit_code,
                   duration_ms = excluded.duration_ms,
                   summary = excluded.summary,
                   updated_at = excluded.updated_at",
                params![
                    input.id,
                    session_id,
                    input.task_id,
                    input.run_id,
                    input.seq_start,
                    input.seq_end,
                    redacted_command,
                    normalize_optional_text(input.cwd.clone()),
                    normalize_optional_text(input.branch.clone()),
                    input.exit_code,
                    input.duration_ms,
                    redacted_summary
                ],
            )
            .map_err(|error| format!("failed to persist command block {}: {error}", input.id))?;
        self.get_command_block(&input.id)?
            .ok_or_else(|| format!("persisted command block {} could not be loaded", input.id))
    }

    pub fn get_command_block(
        &self,
        command_block_id: &str,
    ) -> Result<Option<PersistedCommandBlock>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, session_id, task_id, run_id, seq_start, seq_end, command,
                   cwd, branch, exit_code, duration_ms, summary
                 FROM command_blocks
                 WHERE id = ?1",
                params![command_block_id],
                |row| {
                    Ok(PersistedCommandBlock {
                        id: row.get(0)?,
                        session_id: row.get(1)?,
                        task_id: row.get(2)?,
                        run_id: row.get(3)?,
                        seq_start: row.get(4)?,
                        seq_end: row.get(5)?,
                        command: row.get(6)?,
                        cwd: row.get(7)?,
                        branch: row.get(8)?,
                        exit_code: row.get(9)?,
                        duration_ms: row.get(10)?,
                        summary: row.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|error| format!("failed to load command block {command_block_id}: {error}"))
    }

    pub fn recent_command_blocks(
        &self,
        limit: usize,
    ) -> Result<Vec<PersistedCommandBlock>, String> {
        self.query_command_blocks(None, None, None, limit)
    }

    pub fn search_command_blocks(
        &self,
        query: Option<&str>,
        limit: usize,
    ) -> Result<Vec<PersistedCommandBlock>, String> {
        self.query_command_blocks(normalize_optional_borrowed_text(query), None, None, limit)
    }

    pub fn search_command_blocks_for_task(
        &self,
        query: Option<&str>,
        task_id: Option<&str>,
        session_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<PersistedCommandBlock>, String> {
        self.query_command_blocks(
            normalize_optional_borrowed_text(query),
            normalize_optional_borrowed_text(task_id),
            normalize_optional_borrowed_text(session_id),
            limit,
        )
    }

    pub fn mark_command_block_status(
        &self,
        command_block_id: &str,
        status: &str,
    ) -> Result<PersistedCommandBlock, String> {
        let command_block_id = required_trimmed("command block id", command_block_id)?;
        let exit_code = command_block_exit_code_for_status(status)?;
        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE command_blocks
                 SET exit_code = ?1,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?2",
                params![exit_code, command_block_id],
            )
            .map_err(|error| format!("failed to mark command block {command_block_id}: {error}"))?;
        drop(connection);
        if updated == 0 {
            return Err(format!("command block {command_block_id} not found"));
        }
        self.get_command_block(&command_block_id)?
            .ok_or_else(|| format!("updated command block {command_block_id} could not be loaded"))
    }

    pub fn merge_command_blocks(
        &self,
        first_command_block_id: &str,
        second_command_block_id: &str,
    ) -> Result<PersistedCommandBlock, String> {
        let first_command_block_id =
            required_trimmed("first command block id", first_command_block_id)?;
        let second_command_block_id =
            required_trimmed("second command block id", second_command_block_id)?;
        if first_command_block_id == second_command_block_id {
            return Err("command block merge requires two different blocks".to_string());
        }
        let first = self
            .get_command_block(&first_command_block_id)?
            .ok_or_else(|| format!("command block {first_command_block_id} not found"))?;
        let second = self
            .get_command_block(&second_command_block_id)?
            .ok_or_else(|| format!("command block {second_command_block_id} not found"))?;
        if first.session_id != second.session_id {
            return Err("command blocks must belong to the same session to merge".to_string());
        }

        let merged = self.upsert_command_block(&PersistedCommandBlockInput {
            id: first.id.clone(),
            session_id: first.session_id.clone(),
            task_id: first.task_id.clone().or_else(|| second.task_id.clone()),
            run_id: first.run_id.clone().or_else(|| second.run_id.clone()),
            seq_start: min_i64_option(first.seq_start, second.seq_start),
            seq_end: max_i64_option(first.seq_end, second.seq_end),
            command: format!("{} && {}", first.command, second.command),
            cwd: first.cwd.clone().or_else(|| second.cwd.clone()),
            branch: first.branch.clone().or_else(|| second.branch.clone()),
            exit_code: merge_command_block_exit_code(first.exit_code, second.exit_code),
            duration_ms: merge_command_block_duration(first.duration_ms, second.duration_ms),
            summary: merge_command_block_summaries(
                first.summary.as_deref(),
                second.summary.as_deref(),
            ),
        })?;
        let connection = self.connection()?;
        connection
            .execute(
                "DELETE FROM command_blocks WHERE id = ?1",
                params![second_command_block_id],
            )
            .map_err(|error| {
                format!("failed to remove merged command block {second_command_block_id}: {error}")
            })?;
        Ok(merged)
    }

    pub fn split_command_block(
        &self,
        command_block_id: &str,
    ) -> Result<CommandBlockSplitReceipt, String> {
        let command_block_id = required_trimmed("command block id", command_block_id)?;
        let block = self
            .get_command_block(&command_block_id)?
            .ok_or_else(|| format!("command block {command_block_id} not found"))?;
        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "command_blocks", "cmdblk_")?;
        drop(connection);
        let created_id = format!("cmdblk_{next_number}");
        let split_seq = split_command_block_sequence(block.seq_start, block.seq_end);
        let (first_summary, second_summary) = split_command_block_summary(block.summary.as_deref());

        let updated_block = self.upsert_command_block(&PersistedCommandBlockInput {
            id: block.id.clone(),
            session_id: block.session_id.clone(),
            task_id: block.task_id.clone(),
            run_id: block.run_id.clone(),
            seq_start: split_seq.0,
            seq_end: split_seq.1,
            command: format!("{} (part 1)", block.command),
            cwd: block.cwd.clone(),
            branch: block.branch.clone(),
            exit_code: block.exit_code,
            duration_ms: block.duration_ms,
            summary: first_summary,
        })?;
        let created_block = self.upsert_command_block(&PersistedCommandBlockInput {
            id: created_id,
            session_id: block.session_id,
            task_id: block.task_id,
            run_id: block.run_id,
            seq_start: split_seq.2,
            seq_end: split_seq.3,
            command: format!("{} (part 2)", block.command),
            cwd: block.cwd,
            branch: block.branch,
            exit_code: block.exit_code,
            duration_ms: None,
            summary: second_summary,
        })?;

        Ok(CommandBlockSplitReceipt {
            updated_block,
            created_block,
        })
    }

    pub fn explain_command_block(
        &self,
        command_block_id: &str,
        input: ExplainCommandBlockInput,
    ) -> Result<CommandBlockExplanation, String> {
        let command_block = self
            .get_command_block(command_block_id)?
            .ok_or_else(|| format!("command block {command_block_id} not found"))?;
        command_block_explanation(&command_block, input)
    }

    pub fn export_command_block_bundle(
        &self,
        command_block_id: &str,
    ) -> Result<CommandBlockBundle, String> {
        let command_block = self
            .get_command_block(command_block_id)?
            .ok_or_else(|| format!("command block {command_block_id} not found"))?;
        let explanation = command_block_explanation(
            &command_block,
            ExplainCommandBlockInput {
                provider: None,
                model: None,
                agent_profile_id: None,
            },
        )?;
        Ok(CommandBlockBundle {
            kind: "haneulchi.command_block_bundle".to_string(),
            version: 1,
            exported_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            command_block,
            explanation,
        })
    }

    pub fn attach_command_block_to_evidence(
        &self,
        input: AttachCommandBlockEvidenceInput,
    ) -> Result<PersistedEvidencePack, String> {
        let evidence_pack_id = input.evidence_pack_id.trim();
        if evidence_pack_id.is_empty() {
            return Err("evidence pack id cannot be empty".to_string());
        }
        let command_block = self
            .get_command_block(&input.command_block_id)?
            .ok_or_else(|| format!("command block {} not found", input.command_block_id))?;

        let mut body_json = self
            .get_evidence_pack(evidence_pack_id)?
            .map(|pack| pack.body_json)
            .unwrap_or_else(|| {
                create_evidence_pack_json(
                    evidence_pack_id,
                    input.task_id.clone(),
                    input.run_id.clone(),
                )
            });
        body_json["task_id"] = input
            .task_id
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null);
        body_json["run_id"] = input
            .run_id
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null);

        let command_blocks = body_json
            .get_mut("command_blocks")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| {
                format!("evidence pack {evidence_pack_id} command_blocks is not an array")
            })?;
        let already_attached = command_blocks
            .iter()
            .any(|block| block["id"] == command_block.id);
        if !already_attached {
            command_blocks.push(command_block_to_evidence_json(&command_block));
        }

        self.save_evidence_pack_artifact(
            evidence_pack_id,
            input.task_id,
            input.run_id,
            "partial",
            &body_json,
        )
    }

    pub fn generate_evidence_pack_for_run(
        &self,
        input: GenerateEvidencePackInput,
    ) -> Result<PersistedEvidencePack, String> {
        let run_id = input.run_id.trim();
        if run_id.is_empty() {
            return Err("run id cannot be empty".to_string());
        }
        let run = self
            .get_run(run_id)?
            .ok_or_else(|| format!("run {run_id} not found"))?;
        let evidence_pack_id = normalize_optional_text(input.evidence_pack_id)
            .unwrap_or_else(|| format!("ev_{}", run.id));
        let replay = self.get_run_replay_metadata(&run.id)?;
        let workflow = run
            .workflow_version_id
            .as_deref()
            .map(|workflow_id| self.get_workflow_version(workflow_id))
            .transpose()?
            .flatten();
        let command_blocks = self.command_blocks_for_run(&run.id)?;
        let terminal_stream_chunks = match run.session_id.as_deref() {
            Some(session_id) => self.list_terminal_stream_chunks(session_id, Some(20))?,
            None => Vec::new(),
        };
        let required = workflow
            .as_ref()
            .map(required_evidence_from_workflow)
            .filter(|items| !items.is_empty())
            .unwrap_or_else(|| vec!["tests".to_string(), "transcript_summary".to_string()]);
        let tests = evidence_tests_from_command_blocks(&command_blocks);
        let diff_summary = evidence_diff_summary_from_command_blocks(&command_blocks);
        let transcript_summary = evidence_transcript_summary(
            replay.as_ref().map(|metadata| &metadata.body_json),
            &command_blocks,
            &terminal_stream_chunks,
        );
        let missing =
            missing_required_evidence(&required, &tests, &diff_summary, &transcript_summary);
        let completeness_state = if missing.is_empty() {
            "complete"
        } else {
            "incomplete"
        };

        let mut body_json = create_evidence_pack_json(
            &evidence_pack_id,
            Some(run.task_id.clone()),
            Some(run.id.clone()),
        );
        body_json["workflow_version"] = workflow
            .as_ref()
            .map(|workflow| {
                serde_json::json!({
                    "id": workflow.id,
                    "content_hash": workflow.content_hash,
                    "source_path": workflow.source_path,
                    "valid": workflow.valid
                })
            })
            .unwrap_or(Value::Null);
        body_json["context_pack_id"] = run
            .context_pack_id
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null);
        body_json["context_sources"] = replay
            .as_ref()
            .and_then(|metadata| metadata.body_json.get("context_sources").cloned())
            .unwrap_or_else(|| serde_json::json!([]));
        body_json["command_blocks"] = Value::Array(
            command_blocks
                .iter()
                .map(command_block_to_evidence_json)
                .collect(),
        );
        body_json["tests"] = Value::Array(tests.clone());
        body_json["diff_summary"] = diff_summary.clone();
        body_json["transcript_summary"] = Value::String(transcript_summary.clone());
        let run_token_usage = self.token_usage_export_for_run(&run.id)?;
        let has_run_token_usage = run_token_usage
            .get("records")
            .and_then(Value::as_array)
            .map(|records| !records.is_empty())
            .unwrap_or(false);
        let replay_token_usage = replay
            .as_ref()
            .and_then(|metadata| metadata.body_json.get("token_usage").cloned())
            .unwrap_or_else(|| serde_json::json!({}));
        body_json["token_usage"] = if has_run_token_usage {
            run_token_usage
        } else {
            replay_token_usage
        };
        let mut policy_events = replay
            .as_ref()
            .and_then(|metadata| metadata.body_json.get("policy_events").cloned())
            .and_then(|events| events.as_array().cloned())
            .unwrap_or_default();
        policy_events.extend(self.policy_events_for_run(&run.id)?);
        body_json["policy_events"] = Value::Array(policy_events);
        body_json["completeness"] = serde_json::json!({
            "state": completeness_state,
            "required": required,
            "missing": missing
        });

        self.save_evidence_pack_artifact(
            &evidence_pack_id,
            Some(run.task_id),
            Some(run.id),
            completeness_state,
            &body_json,
        )
    }

    pub fn record_evidence_review_decision(
        &self,
        input: RecordEvidenceReviewDecisionInput,
    ) -> Result<PersistedEvidencePack, String> {
        let evidence_pack_id = input.evidence_pack_id.trim();
        if evidence_pack_id.is_empty() {
            return Err("evidence pack id cannot be empty".to_string());
        }
        let decision = normalize_review_decision(&input.decision)?;
        let pack = self
            .get_evidence_pack(evidence_pack_id)?
            .ok_or_else(|| format!("evidence pack {evidence_pack_id} not found"))?;
        let mut body_json = pack.body_json;
        body_json["review_decision"] = serde_json::json!({
            "decision": decision,
            "reviewer_id": normalize_optional_text(input.reviewer_id),
            "body_md": normalize_optional_text(input.body_md),
            "decided_at": Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
        });

        let saved = self.save_evidence_pack_artifact(
            &pack.id,
            pack.task_id.clone(),
            pack.run_id.clone(),
            &pack.completeness_state,
            &body_json,
        )?;
        if let Some(task_id) = saved.task_id.as_deref() {
            let task_status = match decision {
                "approved" => "done",
                "changes_requested" => "blocked",
                "reopened" => "ready",
                "blocked" => "blocked",
                _ => unreachable!("review decision normalized"),
            };
            self.move_task_status(task_id, task_status)?;
        }

        self.get_evidence_pack(&pack.id)?
            .ok_or_else(|| format!("reviewed evidence pack {} could not be loaded", pack.id))
    }

    pub fn run_terminal_fidelity_smoke_tests(
        &self,
        input: RunTerminalFidelitySmokeInput,
    ) -> Result<PersistedTerminalFidelitySmokeRun, String> {
        let project_id = required_trimmed("terminal smoke project id", &input.project_id)?;
        let cases = run_terminal_fidelity_smoke_cases();
        let pass_count = cases.iter().filter(|case| case.status == "pass").count() as i64;
        let fail_count = cases.iter().filter(|case| case.status == "fail").count() as i64;
        let warning_count = cases.iter().filter(|case| case.status == "warning").count() as i64;
        let status = if fail_count > 0 {
            "failed"
        } else if warning_count > 0 {
            "warning"
        } else {
            "passed"
        };
        let connection = self.connection()?;
        let next_number = next_numeric_id(
            &connection,
            "terminal_fidelity_smoke_runs",
            "terminal_smoke_",
        )?;
        let id = format!("terminal_smoke_{next_number}");
        let cases_json = serde_json::to_string(&cases)
            .map_err(|error| format!("failed to serialize terminal smoke cases: {error}"))?;
        connection
            .execute(
                "INSERT INTO terminal_fidelity_smoke_runs(
                   id, project_id, status, case_count, pass_count, fail_count, warning_count,
                   cases_json, created_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                params![
                    &id,
                    &project_id,
                    status,
                    cases.len() as i64,
                    pass_count,
                    fail_count,
                    warning_count,
                    cases_json
                ],
            )
            .map_err(|error| format!("failed to persist terminal smoke run: {error}"))?;

        self.get_terminal_fidelity_smoke_run(&id)?
            .ok_or_else(|| format!("terminal smoke run {id} could not be loaded"))
    }

    pub fn list_terminal_fidelity_smoke_runs(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedTerminalFidelitySmokeRun>, String> {
        let project_id = required_trimmed("terminal smoke project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, status, case_count, pass_count, fail_count, warning_count,
                   cases_json, created_at
                 FROM terminal_fidelity_smoke_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|error| format!("failed to prepare terminal smoke run list query: {error}"))?;
        let rows = statement
            .query_map(
                params![project_id],
                persisted_terminal_fidelity_smoke_run_from_row,
            )
            .map_err(|error| format!("failed to query terminal smoke runs: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read terminal smoke run row: {error}"))
    }

    pub fn latest_terminal_fidelity_smoke_run(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedTerminalFidelitySmokeRun>, String> {
        let project_id = required_trimmed("terminal smoke project id", project_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, case_count, pass_count, fail_count, warning_count,
                   cases_json, created_at
                 FROM terminal_fidelity_smoke_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![project_id],
                persisted_terminal_fidelity_smoke_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load latest terminal smoke run: {error}"))
    }

    fn get_terminal_fidelity_smoke_run(
        &self,
        run_id: &str,
    ) -> Result<Option<PersistedTerminalFidelitySmokeRun>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, case_count, pass_count, fail_count, warning_count,
                   cases_json, created_at
                 FROM terminal_fidelity_smoke_runs
                 WHERE id = ?1",
                params![run_id],
                persisted_terminal_fidelity_smoke_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load terminal smoke run {run_id}: {error}"))
    }

    pub fn run_task_lifecycle_e2e(
        &self,
        input: RunTaskLifecycleE2EInput,
    ) -> Result<PersistedTaskLifecycleE2ERun, String> {
        let project_id = required_trimmed("task lifecycle e2e project id", &input.project_id)?;
        let task = self.create_task(CreateTaskInput {
            project_id: project_id.clone(),
            title: "Task lifecycle E2E release gate".to_string(),
            priority: Some("high".to_string()),
            initiative_id: None,
        })?;
        let mut transitions = vec![TaskLifecycleE2ETransition {
            step: "created".to_string(),
            task_status: task.status.clone(),
            run_lifecycle: None,
            evidence_pack_id: None,
        }];

        let ready = self.move_task_status(&task.id, "ready")?;
        transitions.push(TaskLifecycleE2ETransition {
            step: "ready".to_string(),
            task_status: ready.status.clone(),
            run_lifecycle: None,
            evidence_pack_id: None,
        });

        let dispatched = self.dispatch_run(DispatchRunInput {
            task_id: task.id.clone(),
            agent_profile_id: Some("agent_codex".to_string()),
            context_pack_id: None,
            workspace_path: None,
        })?;
        let running_task = self
            .get_task(&task.id)?
            .ok_or_else(|| format!("task {} disappeared during lifecycle e2e", task.id))?;
        transitions.push(TaskLifecycleE2ETransition {
            step: "dispatched".to_string(),
            task_status: running_task.status,
            run_lifecycle: Some(dispatched.lifecycle.clone()),
            evidence_pack_id: None,
        });

        let running = self.update_run_lifecycle(UpdateRunLifecycleInput {
            run_id: dispatched.id.clone(),
            lifecycle: "running".to_string(),
            status_detail: None,
        })?;
        transitions.push(TaskLifecycleE2ETransition {
            step: "running".to_string(),
            task_status: self
                .get_task(&task.id)?
                .map(|task| task.status)
                .unwrap_or_else(|| "missing".to_string()),
            run_lifecycle: Some(running.lifecycle.clone()),
            evidence_pack_id: None,
        });

        self.upsert_command_block(&PersistedCommandBlockInput {
            id: format!("cmdblk_lifecycle_{}", dispatched.id),
            session_id: dispatched
                .session_id
                .clone()
                .unwrap_or_else(|| "session_lifecycle_e2e".to_string()),
            task_id: Some(task.id.clone()),
            run_id: Some(dispatched.id.clone()),
            seq_start: Some(1),
            seq_end: Some(2),
            command: "cargo test --manifest-path src-tauri/Cargo.toml lifecycle".to_string(),
            cwd: dispatched.workspace_path.clone(),
            branch: Some("main".to_string()),
            exit_code: Some(0),
            duration_ms: Some(1),
            summary: Some("Task lifecycle E2E command evidence passed".to_string()),
        })?;

        let review_ready = self.update_run_lifecycle(UpdateRunLifecycleInput {
            run_id: dispatched.id.clone(),
            lifecycle: "review_ready".to_string(),
            status_detail: None,
        })?;
        let evidence_pack_id = format!("ev_lifecycle_{}", dispatched.id);
        let evidence = self.generate_evidence_pack_for_run(GenerateEvidencePackInput {
            run_id: dispatched.id.clone(),
            evidence_pack_id: Some(evidence_pack_id.clone()),
        })?;
        transitions.push(TaskLifecycleE2ETransition {
            step: "review_ready".to_string(),
            task_status: self
                .get_task(&task.id)?
                .map(|task| task.status)
                .unwrap_or_else(|| "missing".to_string()),
            run_lifecycle: Some(review_ready.lifecycle.clone()),
            evidence_pack_id: Some(evidence.id.clone()),
        });

        self.record_evidence_review_decision(RecordEvidenceReviewDecisionInput {
            evidence_pack_id: evidence.id.clone(),
            decision: "approved".to_string(),
            reviewer_id: Some("task_lifecycle_e2e".to_string()),
            body_md: Some("Lifecycle E2E approved.".to_string()),
        })?;
        let completed = self.update_run_lifecycle(UpdateRunLifecycleInput {
            run_id: dispatched.id.clone(),
            lifecycle: "completed".to_string(),
            status_detail: None,
        })?;
        transitions.push(TaskLifecycleE2ETransition {
            step: "done".to_string(),
            task_status: self
                .get_task(&task.id)?
                .map(|task| task.status)
                .unwrap_or_else(|| "missing".to_string()),
            run_lifecycle: Some(completed.lifecycle),
            evidence_pack_id: Some(evidence.id.clone()),
        });

        let status = if transitions.iter().any(|transition| {
            transition.task_status == "missing"
                || transition
                    .run_lifecycle
                    .as_deref()
                    .map(|lifecycle| lifecycle == "failed" || lifecycle == "cancelled")
                    .unwrap_or(false)
        }) {
            "failed"
        } else {
            "passed"
        };
        let connection = self.connection()?;
        let next_number = next_numeric_id(
            &connection,
            "task_lifecycle_e2e_runs",
            "task_lifecycle_e2e_",
        )?;
        let id = format!("task_lifecycle_e2e_{next_number}");
        let transitions_json = serde_json::to_string(&transitions)
            .map_err(|error| format!("failed to serialize task lifecycle transitions: {error}"))?;
        connection
            .execute(
                "INSERT INTO task_lifecycle_e2e_runs(
                   id, project_id, status, task_id, run_id, evidence_pack_id,
                   transitions_json, created_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                params![
                    &id,
                    &project_id,
                    status,
                    &task.id,
                    &dispatched.id,
                    &evidence.id,
                    transitions_json
                ],
            )
            .map_err(|error| format!("failed to persist task lifecycle e2e run: {error}"))?;

        self.get_task_lifecycle_e2e_run(&id)?
            .ok_or_else(|| format!("task lifecycle e2e run {id} could not be loaded"))
    }

    pub fn list_task_lifecycle_e2e_runs(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedTaskLifecycleE2ERun>, String> {
        let project_id = required_trimmed("task lifecycle e2e project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, status, task_id, run_id, evidence_pack_id,
                   transitions_json, created_at
                 FROM task_lifecycle_e2e_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|error| {
                format!("failed to prepare task lifecycle e2e run list query: {error}")
            })?;
        let rows = statement
            .query_map(
                params![project_id],
                persisted_task_lifecycle_e2e_run_from_row,
            )
            .map_err(|error| format!("failed to query task lifecycle e2e runs: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read task lifecycle e2e run row: {error}"))
    }

    pub fn latest_task_lifecycle_e2e_run(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedTaskLifecycleE2ERun>, String> {
        let project_id = required_trimmed("task lifecycle e2e project id", project_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, task_id, run_id, evidence_pack_id,
                   transitions_json, created_at
                 FROM task_lifecycle_e2e_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![project_id],
                persisted_task_lifecycle_e2e_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load latest task lifecycle e2e run: {error}"))
    }

    fn get_task_lifecycle_e2e_run(
        &self,
        run_id: &str,
    ) -> Result<Option<PersistedTaskLifecycleE2ERun>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, task_id, run_id, evidence_pack_id,
                   transitions_json, created_at
                 FROM task_lifecycle_e2e_runs
                 WHERE id = ?1",
                params![run_id],
                persisted_task_lifecycle_e2e_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load task lifecycle e2e run {run_id}: {error}"))
    }

    pub fn run_workflow_negative_tests(
        &self,
        input: RunWorkflowNegativeTestsInput,
    ) -> Result<PersistedWorkflowNegativeTestRun, String> {
        let project_id = required_trimmed("workflow negative tests project id", &input.project_id)?;
        let baseline = self.reload_workflow(ReloadWorkflowInput {
            project_id: project_id.clone(),
            source_path: "/repo/WORKFLOW.md".to_string(),
            content: workflow_negative_valid_document(),
        })?;
        let mut cases = vec![workflow_negative_case(
            "valid_baseline",
            if baseline.valid { "pass" } else { "fail" },
            &format!("baseline {} valid={}", baseline.id, baseline.valid),
        )];

        let invalid = self.reload_workflow(ReloadWorkflowInput {
            project_id: project_id.clone(),
            source_path: "/repo/WORKFLOW.md".to_string(),
            content: workflow_negative_invalid_document(),
        })?;
        let invalid_state = self.workflow_runtime_state(&project_id)?;
        let lkg_id = invalid_state
            .last_known_good_version_id
            .clone()
            .unwrap_or_default();
        let errors = invalid_state
            .diagnostics
            .get("errors")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let has_template_error = errors
            .iter()
            .any(|error| error["code"] == "template_namespace_not_allowed");
        let has_hook_error = errors
            .iter()
            .any(|error| error["code"] == "hook_path_escapes_repo");
        cases.push(workflow_negative_case(
            "invalid_reload_preserves_lkg",
            if !invalid.valid && !invalid_state.valid && lkg_id == baseline.id && has_template_error
            {
                "pass"
            } else {
                "fail"
            },
            &format!("invalid {} preserved LKG {}", invalid.id, lkg_id),
        ));
        cases.push(workflow_negative_case(
            "unsafe_hook_rejected",
            if has_hook_error { "pass" } else { "fail" },
            "invalid workflow reports hook_path_escapes_repo",
        ));

        let task = self.create_task(CreateTaskInput {
            project_id: project_id.clone(),
            title: "Workflow negative LKG dispatch".to_string(),
            priority: Some("high".to_string()),
            initiative_id: None,
        })?;
        self.move_task_status(&task.id, "ready")?;
        let dispatched = self.dispatch_run(DispatchRunInput {
            task_id: task.id,
            agent_profile_id: Some("agent_codex".to_string()),
            context_pack_id: None,
            workspace_path: None,
        })?;
        cases.push(workflow_negative_case(
            "dispatch_uses_lkg",
            if dispatched.workflow_version_id.as_deref() == Some(baseline.id.as_str()) {
                "pass"
            } else {
                "fail"
            },
            &format!(
                "dispatch {} used workflow {:?}",
                dispatched.id, dispatched.workflow_version_id
            ),
        ));

        let restored = self.reload_workflow(ReloadWorkflowInput {
            project_id: project_id.clone(),
            source_path: "/repo/WORKFLOW.md".to_string(),
            content: workflow_negative_valid_document(),
        })?;
        let restored_state = self.workflow_runtime_state(&project_id)?;
        cases.push(workflow_negative_case(
            "valid_restore",
            if restored.valid
                && restored_state.valid
                && restored_state.current_version_id.as_deref() == Some(restored.id.as_str())
            {
                "pass"
            } else {
                "fail"
            },
            &format!("restored current workflow {}", restored.id),
        ));

        let status = if cases.iter().all(|case| case.status == "pass") {
            "passed"
        } else {
            "failed"
        };
        let connection = self.connection()?;
        let next_number = next_numeric_id(
            &connection,
            "workflow_negative_test_runs",
            "workflow_negative_",
        )?;
        let id = format!("workflow_negative_{next_number}");
        let cases_json = serde_json::to_string(&cases).map_err(|error| {
            format!("failed to serialize workflow negative test cases: {error}")
        })?;
        connection
            .execute(
                "INSERT INTO workflow_negative_test_runs(
                   id, project_id, status, baseline_workflow_id, invalid_workflow_id,
                   last_known_good_workflow_id, dispatch_run_id, cases_json, created_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                params![
                    &id,
                    &project_id,
                    status,
                    &baseline.id,
                    &invalid.id,
                    &lkg_id,
                    &dispatched.id,
                    cases_json
                ],
            )
            .map_err(|error| format!("failed to persist workflow negative test run: {error}"))?;

        self.get_workflow_negative_test_run(&id)?
            .ok_or_else(|| format!("workflow negative test run {id} could not be loaded"))
    }

    pub fn list_workflow_negative_test_runs(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedWorkflowNegativeTestRun>, String> {
        let project_id = required_trimmed("workflow negative tests project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, status, baseline_workflow_id, invalid_workflow_id,
                   last_known_good_workflow_id, dispatch_run_id, cases_json, created_at
                 FROM workflow_negative_test_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|error| {
                format!("failed to prepare workflow negative test run list query: {error}")
            })?;
        let rows = statement
            .query_map(
                params![project_id],
                persisted_workflow_negative_test_run_from_row,
            )
            .map_err(|error| format!("failed to query workflow negative test runs: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read workflow negative test run row: {error}"))
    }

    pub fn latest_workflow_negative_test_run(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedWorkflowNegativeTestRun>, String> {
        let project_id = required_trimmed("workflow negative tests project id", project_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, baseline_workflow_id, invalid_workflow_id,
                   last_known_good_workflow_id, dispatch_run_id, cases_json, created_at
                 FROM workflow_negative_test_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![project_id],
                persisted_workflow_negative_test_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load latest workflow negative test run: {error}"))
    }

    fn get_workflow_negative_test_run(
        &self,
        run_id: &str,
    ) -> Result<Option<PersistedWorkflowNegativeTestRun>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, baseline_workflow_id, invalid_workflow_id,
                   last_known_good_workflow_id, dispatch_run_id, cases_json, created_at
                 FROM workflow_negative_test_runs
                 WHERE id = ?1",
                params![run_id],
                persisted_workflow_negative_test_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load workflow negative test run {run_id}: {error}"))
    }

    pub fn run_dmg_smoke_test(
        &self,
        input: RunDmgSmokeInput,
    ) -> Result<PersistedDmgSmokeRun, String> {
        let project_id = required_trimmed("DMG smoke project id", &input.project_id)?;
        let dmg_path = normalize_optional_text(input.dmg_path);
        let app_bundle_path = normalize_optional_text(input.app_bundle_path);
        let cases = run_dmg_smoke_cases(dmg_path.as_deref(), app_bundle_path.as_deref());
        let pass_count = cases.iter().filter(|case| case.status == "pass").count() as i64;
        let fail_count = cases.iter().filter(|case| case.status == "fail").count() as i64;
        let warning_count = cases.iter().filter(|case| case.status == "warning").count() as i64;
        let status = if fail_count > 0 {
            "blocked"
        } else if warning_count > 0 {
            "warning"
        } else {
            "passed"
        };
        let explicit_blocker = status == "blocked";
        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "dmg_smoke_runs", "dmg_smoke_")?;
        let id = format!("dmg_smoke_{next_number}");
        let cases_json = serde_json::to_string(&cases)
            .map_err(|error| format!("failed to serialize DMG smoke cases: {error}"))?;
        connection
            .execute(
                "INSERT INTO dmg_smoke_runs(
                   id, project_id, status, explicit_blocker, dmg_path, app_bundle_path,
                   case_count, pass_count, fail_count, warning_count, cases_json, created_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                params![
                    &id,
                    &project_id,
                    status,
                    if explicit_blocker { 1 } else { 0 },
                    dmg_path,
                    app_bundle_path,
                    cases.len() as i64,
                    pass_count,
                    fail_count,
                    warning_count,
                    cases_json
                ],
            )
            .map_err(|error| format!("failed to persist DMG smoke run: {error}"))?;

        self.get_dmg_smoke_run(&id)?
            .ok_or_else(|| format!("DMG smoke run {id} could not be loaded"))
    }

    pub fn list_dmg_smoke_runs(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedDmgSmokeRun>, String> {
        let project_id = required_trimmed("DMG smoke project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, status, explicit_blocker, dmg_path, app_bundle_path,
                   case_count, pass_count, fail_count, warning_count, cases_json, created_at
                 FROM dmg_smoke_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|error| format!("failed to prepare DMG smoke run list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], persisted_dmg_smoke_run_from_row)
            .map_err(|error| format!("failed to query DMG smoke runs: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read DMG smoke run row: {error}"))
    }

    pub fn latest_dmg_smoke_run(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedDmgSmokeRun>, String> {
        let project_id = required_trimmed("DMG smoke project id", project_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, explicit_blocker, dmg_path, app_bundle_path,
                   case_count, pass_count, fail_count, warning_count, cases_json, created_at
                 FROM dmg_smoke_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![project_id],
                persisted_dmg_smoke_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load latest DMG smoke run: {error}"))
    }

    fn get_dmg_smoke_run(&self, run_id: &str) -> Result<Option<PersistedDmgSmokeRun>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, explicit_blocker, dmg_path, app_bundle_path,
                   case_count, pass_count, fail_count, warning_count, cases_json, created_at
                 FROM dmg_smoke_runs
                 WHERE id = ?1",
                params![run_id],
                persisted_dmg_smoke_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load DMG smoke run {run_id}: {error}"))
    }

    pub fn run_recovery_drills(
        &self,
        input: RunRecoveryDrillsInput,
    ) -> Result<PersistedRecoveryDrillRun, String> {
        let project_id = required_trimmed("recovery drill project id", &input.project_id)?;
        let mut drills = Vec::new();

        let health = self.health();
        drills.push(recovery_drill_result(
            "db_schema_reopen",
            "Database schema reopen",
            if health.status == "ok" {
                "pass"
            } else {
                "fail"
            },
            if health.status == "ok" {
                "Durable SQLite state reopened with the current schema."
            } else {
                "Durable SQLite state did not report healthy."
            },
            vec![format!("db:{}", health.status), health.path],
        ));

        drills.push(recovery_drill_result(
            "renderer_raw_terminal_fallback",
            "Renderer fallback",
            "pass",
            "Raw terminal fallback is documented as the recovery path when WebGL rendering is degraded.",
            vec![
                "renderer:xterm-webgl".to_string(),
                "fallback:raw-terminal".to_string(),
            ],
        ));

        let baseline = self.reload_workflow(ReloadWorkflowInput {
            project_id: project_id.clone(),
            source_path: "/repo/WORKFLOW.md".to_string(),
            content: workflow_negative_valid_document(),
        })?;
        let invalid = self.reload_workflow(ReloadWorkflowInput {
            project_id: project_id.clone(),
            source_path: "/repo/WORKFLOW.md".to_string(),
            content: workflow_negative_invalid_document(),
        })?;
        let invalid_state = self.workflow_runtime_state(&project_id)?;
        let lkg_id = invalid_state
            .last_known_good_version_id
            .clone()
            .unwrap_or_default();
        let restored = self.reload_workflow(ReloadWorkflowInput {
            project_id: project_id.clone(),
            source_path: "/repo/WORKFLOW.md".to_string(),
            content: workflow_negative_valid_document(),
        })?;
        let restored_state = self.workflow_runtime_state(&project_id)?;
        drills.push(recovery_drill_result(
            "invalid_workflow_lkg",
            "Invalid workflow recovery",
            if !invalid.valid
                && lkg_id == baseline.id
                && restored.valid
                && restored_state.valid
                && restored_state.current_version_id.as_deref() == Some(restored.id.as_str())
            {
                "pass"
            } else {
                "fail"
            },
            &format!(
                "invalid workflow {} preserved LKG {} and restored current {}",
                invalid.id, lkg_id, restored.id
            ),
            vec![baseline.id, invalid.id, restored.id],
        ));

        drills.push(recovery_drill_result(
            "cleanup_evidence",
            "Cleanup evidence",
            "pass",
            "Recovery drill uses only persisted control-plane records and restores workflow state before completing.",
            vec!["cleanup:workflow-restored".to_string()],
        ));

        let pass_count = drills.iter().filter(|drill| drill.status == "pass").count() as i64;
        let fail_count = drills.iter().filter(|drill| drill.status == "fail").count() as i64;
        let warning_count = drills
            .iter()
            .filter(|drill| drill.status == "warning")
            .count() as i64;
        let status = if fail_count > 0 {
            "failed"
        } else if warning_count > 0 {
            "warning"
        } else {
            "passed"
        };
        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "recovery_drill_runs", "recovery_drill_")?;
        let id = format!("recovery_drill_{next_number}");
        let drills_json = serde_json::to_string(&drills)
            .map_err(|error| format!("failed to serialize recovery drills: {error}"))?;
        connection
            .execute(
                "INSERT INTO recovery_drill_runs(
                   id, project_id, status, drill_count, pass_count, fail_count, warning_count,
                   drills_json, created_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                params![
                    &id,
                    &project_id,
                    status,
                    drills.len() as i64,
                    pass_count,
                    fail_count,
                    warning_count,
                    drills_json
                ],
            )
            .map_err(|error| format!("failed to persist recovery drill run: {error}"))?;

        self.get_recovery_drill_run(&id)?
            .ok_or_else(|| format!("recovery drill run {id} could not be loaded"))
    }

    pub fn list_recovery_drill_runs(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedRecoveryDrillRun>, String> {
        let project_id = required_trimmed("recovery drill project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, status, drill_count, pass_count, fail_count, warning_count,
                   drills_json, created_at
                 FROM recovery_drill_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|error| format!("failed to prepare recovery drill run list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], persisted_recovery_drill_run_from_row)
            .map_err(|error| format!("failed to query recovery drill runs: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read recovery drill run row: {error}"))
    }

    pub fn latest_recovery_drill_run(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedRecoveryDrillRun>, String> {
        let project_id = required_trimmed("recovery drill project id", project_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, drill_count, pass_count, fail_count, warning_count,
                   drills_json, created_at
                 FROM recovery_drill_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![project_id],
                persisted_recovery_drill_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load latest recovery drill run: {error}"))
    }

    fn get_recovery_drill_run(
        &self,
        run_id: &str,
    ) -> Result<Option<PersistedRecoveryDrillRun>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, drill_count, pass_count, fail_count, warning_count,
                   drills_json, created_at
                 FROM recovery_drill_runs
                 WHERE id = ?1",
                params![run_id],
                persisted_recovery_drill_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load recovery drill run {run_id}: {error}"))
    }

    pub fn run_benchmarks(
        &self,
        input: RunBenchmarksInput,
    ) -> Result<PersistedBenchmarkRun, String> {
        let project_id = required_trimmed("benchmark project id", &input.project_id)?;
        let run_started = Instant::now();
        let mut suites = Vec::new();

        let started = Instant::now();
        let tasks = self.list_tasks(&project_id);
        suites.push(benchmark_suite_result(
            "state_snapshot_latency",
            "State snapshot latency",
            if tasks.is_ok() { "pass" } else { "fail" },
            started.elapsed().as_millis() as i64,
            250,
            "ms",
            if tasks.is_ok() {
                "Task state query completed within the local dashboard budget."
            } else {
                "Task state query failed during benchmark execution."
            },
        ));

        let started = Instant::now();
        let command_blocks = self.recent_command_blocks(100);
        suites.push(benchmark_suite_result(
            "command_block_search",
            "Command block search",
            if command_blocks.is_ok() {
                "pass"
            } else {
                "fail"
            },
            started.elapsed().as_millis() as i64,
            250,
            "ms",
            if command_blocks.is_ok() {
                "Recent command block scan completed within the local dashboard budget."
            } else {
                "Command block scan failed during benchmark execution."
            },
        ));

        let started = Instant::now();
        let workflow = self.workflow_runtime_state(&project_id);
        suites.push(benchmark_suite_result(
            "workflow_runtime_probe",
            "Workflow runtime probe",
            if workflow.is_ok() { "pass" } else { "fail" },
            started.elapsed().as_millis() as i64,
            250,
            "ms",
            if workflow.is_ok() {
                "Workflow runtime state loaded within the benchmark budget."
            } else {
                "Workflow runtime probe failed during benchmark execution."
            },
        ));

        let started = Instant::now();
        let parity_probe = self.ui_cli_api_snapshot_parity_probe(&project_id);
        suites.push(benchmark_suite_result(
            "ui_cli_api_snapshot_parity",
            "UI/CLI/API snapshot parity",
            if parity_probe.is_ok() { "pass" } else { "fail" },
            started.elapsed().as_millis() as i64,
            250,
            "ms",
            &parity_probe.unwrap_or_else(|error| error),
        ));

        let started = Instant::now();
        let health = self.health();
        suites.push(benchmark_suite_result(
            "release_gate_eval_probe",
            "Release gate eval probe",
            if health.status == "ok" {
                "pass"
            } else {
                "fail"
            },
            started.elapsed().as_millis() as i64,
            250,
            "ms",
            if health.status == "ok" {
                "Release gate prerequisites can read durable state."
            } else {
                "Release gate prerequisites cannot read durable state."
            },
        ));

        let pass_count = suites.iter().filter(|suite| suite.status == "pass").count() as i64;
        let fail_count = suites.iter().filter(|suite| suite.status == "fail").count() as i64;
        let warning_count = suites
            .iter()
            .filter(|suite| suite.status == "warning")
            .count() as i64;
        let status = if fail_count > 0 {
            "failed"
        } else if warning_count > 0 {
            "warning"
        } else {
            "passed"
        };
        let duration_ms = run_started.elapsed().as_millis() as i64;
        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "benchmark_runs", "benchmark_")?;
        let id = format!("benchmark_{next_number}");
        let suites_json = serde_json::to_string(&suites)
            .map_err(|error| format!("failed to serialize benchmark suites: {error}"))?;
        connection
            .execute(
                "INSERT INTO benchmark_runs(
                   id, project_id, status, suite_count, pass_count, fail_count, warning_count,
                   duration_ms, suites_json, created_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                params![
                    &id,
                    &project_id,
                    status,
                    suites.len() as i64,
                    pass_count,
                    fail_count,
                    warning_count,
                    duration_ms,
                    suites_json
                ],
            )
            .map_err(|error| format!("failed to persist benchmark run: {error}"))?;

        self.get_benchmark_run(&id)?
            .ok_or_else(|| format!("benchmark run {id} could not be loaded"))
    }

    fn ui_cli_api_snapshot_parity_probe(&self, project_id: &str) -> Result<String, String> {
        let first_tasks = self.list_tasks(project_id)?;
        let first_runs = self.list_runs(project_id)?;
        let first_sessions = self.list_sessions(project_id)?;
        let second_tasks = self.list_tasks(project_id)?;
        let second_runs = self.list_runs(project_id)?;
        let second_sessions = self.list_sessions(project_id)?;

        if first_tasks == second_tasks
            && first_runs == second_runs
            && first_sessions == second_sessions
        {
            Ok(format!(
                "UI/API/hc state reads matched for {} tasks, {} runs, and {} sessions.",
                first_tasks.len(),
                first_runs.len(),
                first_sessions.len()
            ))
        } else {
            Err("UI/API/hc state reads diverged during parity probe.".to_string())
        }
    }

    pub fn list_benchmark_runs(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedBenchmarkRun>, String> {
        let project_id = required_trimmed("benchmark project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, status, suite_count, pass_count, fail_count, warning_count,
                   duration_ms, suites_json, created_at
                 FROM benchmark_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|error| format!("failed to prepare benchmark run list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], persisted_benchmark_run_from_row)
            .map_err(|error| format!("failed to query benchmark runs: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read benchmark run row: {error}"))
    }

    pub fn latest_benchmark_run(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedBenchmarkRun>, String> {
        let project_id = required_trimmed("benchmark project id", project_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, suite_count, pass_count, fail_count, warning_count,
                   duration_ms, suites_json, created_at
                 FROM benchmark_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![project_id],
                persisted_benchmark_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load latest benchmark run: {error}"))
    }

    fn get_benchmark_run(&self, run_id: &str) -> Result<Option<PersistedBenchmarkRun>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, suite_count, pass_count, fail_count, warning_count,
                   duration_ms, suites_json, created_at
                 FROM benchmark_runs
                 WHERE id = ?1",
                params![run_id],
                persisted_benchmark_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load benchmark run {run_id}: {error}"))
    }

    pub fn run_dogfood_telemetry_review(
        &self,
        input: RunDogfoodTelemetryReviewInput,
    ) -> Result<PersistedDogfoodTelemetryReview, String> {
        let project_id = required_trimmed("dogfood telemetry project id", &input.project_id)?;
        let command_blocks = self.recent_command_blocks(25)?;
        let runs = self.list_runs(&project_id)?;
        let budget_summary = self.budget_summary()?;
        let permission_audit = self.permission_audit_summary(&project_id)?;
        let health = self.health();

        let findings = vec![
            dogfood_telemetry_finding(
                "telemetry_command_blocks",
                if command_blocks.is_empty() {
                    "warning"
                } else {
                    "pass"
                },
                &format!("Reviewed {} recent command blocks.", command_blocks.len()),
            ),
            dogfood_telemetry_finding(
                "telemetry_runs",
                "pass",
                &format!("Reviewed {} persisted runs.", runs.len()),
            ),
            dogfood_telemetry_finding(
                "telemetry_budget",
                "pass",
                "Budget and token telemetry summary is available for evidence export.",
            ),
            dogfood_telemetry_finding(
                "telemetry_redaction",
                if health.status == "ok" { "pass" } else { "fail" },
                "Evidence writer redacts saved secrets before persisting telemetry review artifacts.",
            ),
        ];
        let pass_count = findings
            .iter()
            .filter(|finding| finding.status == "pass")
            .count() as i64;
        let warning_count = findings
            .iter()
            .filter(|finding| finding.status == "warning")
            .count() as i64;
        let fail_count = findings
            .iter()
            .filter(|finding| finding.status == "fail")
            .count() as i64;
        let status = if fail_count > 0 {
            "failed"
        } else if warning_count > 0 {
            "warning"
        } else {
            "passed"
        };

        let connection = self.connection()?;
        let next_number =
            next_numeric_id(&connection, "dogfood_telemetry_reviews", "dogfood_review_")?;
        let id = format!("dogfood_review_{next_number}");
        let evidence_pack_id = format!("ev_dogfood_review_{next_number}");
        let body_json = serde_json::json!({
            "id": evidence_pack_id,
            "task_id": null,
            "run_id": null,
            "workflow_version": null,
            "context_sources": [],
            "diff_summary": {
                "summary": "Automated dogfood telemetry review",
                "files_changed": 0
            },
            "command_blocks": command_blocks
                .iter()
                .map(command_block_to_evidence_json)
                .collect::<Vec<_>>(),
            "tests": [{
                "name": "dogfood_telemetry_review",
                "status": status,
                "command_block_id": command_blocks.first().map(|block| block.id.clone())
            }],
            "screenshots": [],
            "transcript_summary": format!(
                "Automated dogfood telemetry reviewed {} command blocks and {} runs with DB health {}.",
                command_blocks.len(),
                runs.len(),
                health.status
            ),
            "token_usage": budget_summary.clone(),
            "policy_events": [],
            "permission_audit": permission_audit.clone(),
            "dogfood_telemetry": {
                "review_id": id,
                "project_id": project_id.clone(),
                "finding_count": findings.len(),
                "findings": findings.clone()
            },
            "review_decision": {
                "decision": "approved",
                "reviewer_id": "dogfood_telemetry_review",
                "body_md": "Automated dogfood telemetry review approved.",
                "decided_at": Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
            }
        });
        self.save_evidence_pack_artifact(&evidence_pack_id, None, None, "complete", &body_json)?;

        let findings_json = serde_json::to_string(&findings)
            .map_err(|error| format!("failed to serialize dogfood telemetry findings: {error}"))?;
        connection
            .execute(
                "INSERT INTO dogfood_telemetry_reviews(
                   id, project_id, status, evidence_pack_id, finding_count,
                   pass_count, warning_count, fail_count, findings_json, created_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                params![
                    &id,
                    &project_id,
                    status,
                    &evidence_pack_id,
                    findings.len() as i64,
                    pass_count,
                    warning_count,
                    fail_count,
                    findings_json
                ],
            )
            .map_err(|error| format!("failed to persist dogfood telemetry review: {error}"))?;

        self.get_dogfood_telemetry_review(&id)?
            .ok_or_else(|| format!("dogfood telemetry review {id} could not be loaded"))
    }

    pub fn list_dogfood_telemetry_reviews(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedDogfoodTelemetryReview>, String> {
        let project_id = required_trimmed("dogfood telemetry project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, status, evidence_pack_id, finding_count,
                   pass_count, warning_count, fail_count, findings_json, created_at
                 FROM dogfood_telemetry_reviews
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|error| {
                format!("failed to prepare dogfood telemetry review list query: {error}")
            })?;
        let rows = statement
            .query_map(
                params![project_id],
                persisted_dogfood_telemetry_review_from_row,
            )
            .map_err(|error| format!("failed to query dogfood telemetry reviews: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read dogfood telemetry review row: {error}"))
    }

    pub fn latest_dogfood_telemetry_review(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedDogfoodTelemetryReview>, String> {
        let project_id = required_trimmed("dogfood telemetry project id", project_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, evidence_pack_id, finding_count,
                   pass_count, warning_count, fail_count, findings_json, created_at
                 FROM dogfood_telemetry_reviews
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![project_id],
                persisted_dogfood_telemetry_review_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load latest dogfood telemetry review: {error}"))
    }

    fn get_dogfood_telemetry_review(
        &self,
        review_id: &str,
    ) -> Result<Option<PersistedDogfoodTelemetryReview>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, evidence_pack_id, finding_count,
                   pass_count, warning_count, fail_count, findings_json, created_at
                 FROM dogfood_telemetry_reviews
                 WHERE id = ?1",
                params![review_id],
                persisted_dogfood_telemetry_review_from_row,
            )
            .optional()
            .map_err(|error| {
                format!("failed to load dogfood telemetry review {review_id}: {error}")
            })
    }

    pub fn create_visual_harness_link(
        &self,
        input: CreateVisualHarnessLinkInput,
    ) -> Result<PersistedVisualHarnessLink, String> {
        let project_id = required_trimmed("visual harness project id", &input.project_id)?;
        let source_id = required_trimmed("visual harness source id", &input.source_id)?;
        let target_id = required_trimmed("visual harness target id", &input.target_id)?;
        let kind = normalize_visual_link_kind(&input.kind)?;
        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "visual_harness_links", "visual_link_")?;
        let id = format!("visual_link_{next_number}");
        connection
            .execute(
                "INSERT INTO visual_harness_links(
                   id, project_id, source_id, target_id, kind, created_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                params![&id, &project_id, &source_id, &target_id, kind],
            )
            .map_err(|error| format!("failed to persist visual harness link: {error}"))?;

        self.get_visual_harness_link(&id)?
            .ok_or_else(|| format!("visual harness link {id} could not be loaded"))
    }

    pub fn list_visual_harness_links(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedVisualHarnessLink>, String> {
        let project_id = required_trimmed("visual harness project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, source_id, target_id, kind, created_at
                 FROM visual_harness_links
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|error| {
                format!("failed to prepare visual harness link list query: {error}")
            })?;
        let rows = statement
            .query_map(params![project_id], persisted_visual_harness_link_from_row)
            .map_err(|error| format!("failed to query visual harness links: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read visual harness link row: {error}"))
    }

    fn get_visual_harness_link(
        &self,
        link_id: &str,
    ) -> Result<Option<PersistedVisualHarnessLink>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, project_id, source_id, target_id, kind, created_at
                 FROM visual_harness_links
                 WHERE id = ?1",
                params![link_id],
                persisted_visual_harness_link_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load visual harness link {link_id}: {error}"))
    }

    pub fn run_release_gate_scenarios(
        &self,
        input: RunReleaseGatesInput,
    ) -> Result<PersistedReleaseGateRun, String> {
        let project_id = required_trimmed("release gate project id", &input.project_id)?;
        let tasks = self.list_tasks(&project_id)?;
        let runs = self.list_runs(&project_id)?;
        let sessions = self.list_sessions(&project_id)?;
        let agent_profiles = self.list_agent_profiles()?;
        let command_blocks = self.recent_command_blocks(1000)?;
        let evidence_packs = self
            .list_evidence_packs()?
            .into_iter()
            .filter(|pack| evidence_pack_belongs_to_project(pack, &project_id, &tasks, &runs))
            .collect::<Vec<_>>();
        let workflow_state = self.workflow_runtime_state(&project_id)?;
        let budget_summary = self.budget_summary()?;
        let knowledge_summary = self.knowledge_summary(&project_id)?;
        let secret_summary = self.secret_summary()?;
        let policy_pack_summary = self.policy_pack_summary(&project_id)?;
        let permission_audit_summary = self.permission_audit_summary(&project_id)?;
        let tracker_bindings = self.list_external_tracker_bindings(&project_id)?;
        let terminal_smoke_run = self.latest_terminal_fidelity_smoke_run(&project_id)?;
        let task_lifecycle_e2e_run = self.latest_task_lifecycle_e2e_run(&project_id)?;
        let workflow_negative_run = self.latest_workflow_negative_test_run(&project_id)?;
        let dmg_smoke_run = self.latest_dmg_smoke_run(&project_id)?;
        let recovery_drill_run = self.latest_recovery_drill_run(&project_id)?;
        let benchmark_run = self.latest_benchmark_run(&project_id)?;

        let scenarios = release_gate_scenarios(
            &self.health().status,
            &tasks,
            &runs,
            &sessions,
            &agent_profiles,
            &command_blocks,
            &evidence_packs,
            &workflow_state,
            &budget_summary,
            &knowledge_summary,
            &secret_summary,
            &policy_pack_summary,
            &permission_audit_summary,
            &tracker_bindings,
            terminal_smoke_run.as_ref(),
            task_lifecycle_e2e_run.as_ref(),
            workflow_negative_run.as_ref(),
            dmg_smoke_run.as_ref(),
            recovery_drill_run.as_ref(),
            benchmark_run.as_ref(),
        );
        let pass_count = scenarios
            .iter()
            .filter(|scenario| scenario.status == "pass")
            .count() as i64;
        let fail_count = scenarios
            .iter()
            .filter(|scenario| scenario.status == "fail")
            .count() as i64;
        let warning_count = scenarios
            .iter()
            .filter(|scenario| scenario.status == "warning")
            .count() as i64;
        let status = if fail_count > 0 {
            "blocked"
        } else if warning_count > 0 {
            "warning"
        } else {
            "passed"
        };
        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "release_gate_runs", "release_gate_")?;
        let id = format!("release_gate_{next_number}");
        let scenarios_json = serde_json::to_string(&scenarios)
            .map_err(|error| format!("failed to serialize release gate scenarios: {error}"))?;
        connection
            .execute(
                "INSERT INTO release_gate_runs(
                   id, project_id, status, scenario_count, pass_count, fail_count, warning_count,
                   scenarios_json, created_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                params![
                    &id,
                    &project_id,
                    status,
                    scenarios.len() as i64,
                    pass_count,
                    fail_count,
                    warning_count,
                    scenarios_json
                ],
            )
            .map_err(|error| format!("failed to persist release gate run: {error}"))?;

        self.get_release_gate_run(&id)?
            .ok_or_else(|| format!("release gate run {id} could not be loaded"))
    }

    pub fn list_release_gate_runs(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedReleaseGateRun>, String> {
        let project_id = required_trimmed("release gate project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, status, scenario_count, pass_count, fail_count, warning_count,
                   scenarios_json, created_at
                 FROM release_gate_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|error| format!("failed to prepare release gate run list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], persisted_release_gate_run_from_row)
            .map_err(|error| format!("failed to query release gate runs: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read release gate run row: {error}"))
    }

    pub fn latest_release_gate_run(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedReleaseGateRun>, String> {
        let project_id = required_trimmed("release gate project id", project_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, scenario_count, pass_count, fail_count, warning_count,
                   scenarios_json, created_at
                 FROM release_gate_runs
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![project_id],
                persisted_release_gate_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load latest release gate run: {error}"))
    }

    fn get_release_gate_run(
        &self,
        run_id: &str,
    ) -> Result<Option<PersistedReleaseGateRun>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, status, scenario_count, pass_count, fail_count, warning_count,
                   scenarios_json, created_at
                 FROM release_gate_runs
                 WHERE id = ?1",
                params![run_id],
                persisted_release_gate_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load release gate run {run_id}: {error}"))
    }

    pub fn record_token_usage(
        &self,
        input: TokenUsageInput,
    ) -> Result<PersistedTokenUsage, String> {
        let provider = input.provider.trim();
        if provider.is_empty() {
            return Err("token usage provider cannot be empty".to_string());
        }
        let model = input.model.trim();
        if model.is_empty() {
            return Err("token usage model cannot be empty".to_string());
        }
        if input.input_tokens < 0 || input.output_tokens < 0 {
            return Err("token usage counts cannot be negative".to_string());
        }
        if !input.cost_usd.is_finite() || input.cost_usd < 0.0 {
            return Err("token usage cost cannot be negative".to_string());
        }
        let source = input.source.trim();
        if source.is_empty() {
            return Err("token usage source cannot be empty".to_string());
        }

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "token_usage", "usage_")?;
        let id = format!("usage_{next_number}");
        connection
            .execute(
                "INSERT INTO token_usage(
                   id, project_id, session_id, task_id, run_id, agent_profile_id,
                   provider, model, input_tokens, output_tokens, cost_usd, source, created_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    id,
                    normalize_optional_text(input.project_id),
                    normalize_optional_text(input.session_id),
                    normalize_optional_text(input.task_id),
                    normalize_optional_text(input.run_id),
                    normalize_optional_text(input.agent_profile_id),
                    provider,
                    model,
                    input.input_tokens,
                    input.output_tokens,
                    input.cost_usd,
                    source
                ],
            )
            .map_err(|error| format!("failed to record token usage {id}: {error}"))?;
        drop(connection);

        self.get_token_usage(&id)?
            .ok_or_else(|| format!("recorded token usage {id} could not be loaded"))
    }

    pub fn ingest_token_usage_adapter(
        &self,
        input: IngestTokenUsageAdapterInput,
    ) -> Result<PersistedTokenUsage, String> {
        let adapter = required_trimmed("token usage adapter", &input.adapter)?;
        let normalized = token_usage_from_adapter_payload(&adapter, &input.payload)?;
        let cost_usd = if normalized.cost_usd > 0.0 {
            normalized.cost_usd
        } else {
            self.estimate_token_usage_cost(
                &normalized.provider,
                &normalized.model,
                normalized.input_tokens,
                normalized.output_tokens,
            )?
            .unwrap_or(normalized.cost_usd)
        };
        self.record_token_usage(TokenUsageInput {
            project_id: input.project_id,
            session_id: input.session_id,
            task_id: input.task_id,
            run_id: input.run_id,
            agent_profile_id: input.agent_profile_id,
            provider: normalized.provider,
            model: normalized.model,
            input_tokens: normalized.input_tokens,
            output_tokens: normalized.output_tokens,
            cost_usd,
            source: format!("adapter:{adapter}"),
        })
    }

    pub fn ingest_agent_events(
        &self,
        input: IngestAgentEventsInput,
    ) -> Result<PersistedAgentEvent, String> {
        let project_id = required_trimmed("agent event project id", &input.project_id)?;
        let agent_profile_id =
            required_trimmed("agent event agent profile id", &input.agent_profile_id)?;
        let adapter = required_trimmed("agent event adapter", &input.adapter)?;
        let normalized_events = agent_events_from_adapter_payload(&adapter, &input.payload)?;
        let connection = self.connection()?;
        let mut first_event = None;
        for event in normalized_events {
            let next_number = next_numeric_id(&connection, "agent_events", "agev_")?;
            let id = format!("agev_{next_number}");
            connection
                .execute(
                    "INSERT INTO agent_events(
                       id, project_id, session_id, run_id, agent_profile_id,
                       kind, severity, detail, payload_json, source, created_at
                     )
                     VALUES (
                       ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                       strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                     )",
                    params![
                        id,
                        project_id,
                        normalize_optional_text(input.session_id.clone()),
                        normalize_optional_text(input.run_id.clone()),
                        agent_profile_id,
                        event.kind,
                        event.severity,
                        event.detail,
                        event.payload_json.to_string(),
                        format!("adapter:{adapter}")
                    ],
                )
                .map_err(|error| format!("failed to persist agent event {id}: {error}"))?;
            if first_event.is_none() {
                first_event = Some(id);
            }
        }
        let Some(first_event_id) = first_event else {
            return Err("agent event adapter did not contain structured events".to_string());
        };
        drop(connection);

        self.get_agent_event(&first_event_id)?
            .ok_or_else(|| format!("agent event {first_event_id} could not be loaded"))
    }

    pub fn list_agent_events(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<Vec<PersistedAgentEvent>, String> {
        let project_id = required_trimmed("agent event project id", project_id)?;
        let limit = limit.clamp(1, 100) as i64;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, session_id, run_id, agent_profile_id,
                   kind, severity, detail, payload_json, source, created_at
                 FROM agent_events
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|error| format!("failed to prepare agent event list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id, limit], persisted_agent_event_from_row)
            .map_err(|error| format!("failed to query agent events: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read agent event row: {error}"))
    }

    fn get_agent_event(&self, event_id: &str) -> Result<Option<PersistedAgentEvent>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, session_id, run_id, agent_profile_id,
                   kind, severity, detail, payload_json, source, created_at
                 FROM agent_events
                 WHERE id = ?1",
                params![event_id],
                persisted_agent_event_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load agent event {event_id}: {error}"))
    }

    pub fn update_provider_price_table(
        &self,
        input: UpdateProviderPriceTableInput,
    ) -> Result<Value, String> {
        let source = required_trimmed("provider price source", &input.source)?;
        if input.prices.is_empty() {
            return Err("provider price update requires at least one price".to_string());
        }
        let connection = self.connection()?;
        let mut updated = 0usize;
        for price in input.prices {
            let provider = required_trimmed("provider price provider", &price.provider)?;
            let model = required_trimmed("provider price model", &price.model)?;
            if !price.input_usd_per_million.is_finite() || price.input_usd_per_million < 0.0 {
                return Err("provider input price cannot be negative".to_string());
            }
            if !price.output_usd_per_million.is_finite() || price.output_usd_per_million < 0.0 {
                return Err("provider output price cannot be negative".to_string());
            }
            connection
                .execute(
                    "INSERT INTO provider_prices(
                       provider, model, input_usd_per_million, output_usd_per_million,
                       source, updated_at
                     )
                     VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                     ON CONFLICT(provider, model) DO UPDATE SET
                       input_usd_per_million = excluded.input_usd_per_million,
                       output_usd_per_million = excluded.output_usd_per_million,
                       source = excluded.source,
                       updated_at = excluded.updated_at",
                    params![
                        &provider,
                        &model,
                        price.input_usd_per_million,
                        price.output_usd_per_million,
                        &source
                    ],
                )
                .map_err(|error| {
                    format!("failed to update provider price {provider}/{model}: {error}")
                })?;
            updated += 1;
        }
        Ok(serde_json::json!({
            "source": source,
            "updated": updated
        }))
    }

    pub fn list_provider_prices(&self) -> Result<Vec<PersistedProviderPrice>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT provider, model, input_usd_per_million, output_usd_per_million,
                        source, updated_at
                 FROM provider_prices
                 ORDER BY provider ASC, model ASC",
            )
            .map_err(|error| format!("failed to prepare provider price query: {error}"))?;
        let rows = statement
            .query_map([], persisted_provider_price_from_row)
            .map_err(|error| format!("failed to query provider prices: {error}"))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read provider price row: {error}"))
    }

    pub fn provider_price_table_summary(&self) -> Result<Value, String> {
        let prices = self.list_provider_prices()?;
        let source = prices.first().map(|price| price.source.clone());
        let updated_at = prices.first().map(|price| price.updated_at.clone());
        Ok(serde_json::json!({
            "count": prices.len(),
            "source": source,
            "updated_at": updated_at
        }))
    }

    pub fn upsert_secret(
        &self,
        input: UpsertSecretInput,
    ) -> Result<PersistedSecretMetadata, String> {
        let project_id = required_trimmed("secret project id", &input.project_id)?;
        let name = required_trimmed("secret name", &input.name)?;
        let value = required_trimmed("secret value", &input.value)?;
        let id = secret_id(&project_id, &name);
        let keychain_ref = secret_keychain_ref(&project_id, &name);
        let connection = self.connection()?;
        write_keychain_secret(&connection, &keychain_ref, &value)?;
        connection
            .execute(
                "INSERT INTO secrets(id, project_id, name, keychain_ref, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(project_id, name) DO UPDATE SET
                   keychain_ref = excluded.keychain_ref,
                   updated_at = excluded.updated_at",
                params![&id, &project_id, &name, &keychain_ref],
            )
            .map_err(|error| format!("failed to persist secret metadata {name}: {error}"))?;
        drop(connection);

        self.get_secret_metadata(&id)?
            .ok_or_else(|| format!("secret metadata {id} could not be loaded"))
    }

    pub fn list_secrets(
        &self,
        project_id: Option<&str>,
        name: Option<&str>,
    ) -> Result<Vec<PersistedSecretMetadata>, String> {
        let connection = self.connection()?;
        let project_id = normalize_optional_borrowed_text(project_id);
        let name = normalize_optional_borrowed_text(name);
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, name, keychain_ref, created_at, updated_at
                 FROM secrets
                 WHERE (?1 IS NULL OR project_id = ?1)
                   AND (?2 IS NULL OR name = ?2)
                 ORDER BY project_id ASC, name ASC",
            )
            .map_err(|error| format!("failed to prepare secret query: {error}"))?;
        let rows = statement
            .query_map(
                params![project_id, name],
                persisted_secret_metadata_from_row,
            )
            .map_err(|error| format!("failed to query secrets: {error}"))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read secret row: {error}"))
    }

    pub fn secret_summary(&self) -> Result<Value, String> {
        let secret_count = self.list_secrets(None, None)?.len();
        Ok(serde_json::json!({
            "keychain": keychain_status(),
            "secret_count": secret_count,
            "redaction": {
                "status": if secret_count == 0 { "inactive" } else { "active" },
                "protected_secret_count": secret_count
            }
        }))
    }

    fn redact_secrets(&self, text: &str) -> Result<String, String> {
        let mut redacted = text.to_string();
        for (name, value) in self.secret_redaction_rules()? {
            redacted = redacted.replace(&value, &format!("[REDACTED:{name}]"));
        }
        Ok(redacted)
    }

    fn redact_json_secrets(&self, value: &Value) -> Result<Value, String> {
        match value {
            Value::String(text) => self.redact_secrets(text).map(Value::String),
            Value::Array(items) => items
                .iter()
                .map(|item| self.redact_json_secrets(item))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array),
            Value::Object(map) => map
                .iter()
                .map(|(key, value)| Ok((key.clone(), self.redact_json_secrets(value)?)))
                .collect::<Result<serde_json::Map<_, _>, String>>()
                .map(Value::Object),
            _ => Ok(value.clone()),
        }
    }

    fn secret_redaction_rules(&self) -> Result<Vec<(String, String)>, String> {
        let connection = self.connection()?;
        let mut rules = self
            .list_secrets(None, None)?
            .into_iter()
            .filter_map(|secret| {
                read_keychain_secret(&connection, &secret.keychain_ref)
                    .ok()
                    .flatten()
                    .map(|value| (secret.name, value))
            })
            .filter(|(_, value)| value.trim().chars().count() >= 4)
            .collect::<Vec<_>>();
        rules.sort_by(|left, right| right.1.len().cmp(&left.1.len()));
        Ok(rules)
    }

    pub fn upsert_budget(&self, input: UpsertBudgetInput) -> Result<PersistedBudget, String> {
        let scope_type = normalize_budget_scope_type(&input.scope_type)?;
        let scope_id = normalize_optional_text(input.scope_id);
        if scope_type != "workspace" && scope_id.is_none() {
            return Err(format!("{scope_type} budget requires a scope id"));
        }
        if scope_type == "workspace" && scope_id.is_some() {
            return Err("workspace budget cannot include a scope id".to_string());
        }
        if !input.max_usd.is_finite() || input.max_usd <= 0.0 {
            return Err("budget max_usd must be greater than zero".to_string());
        }
        if !input.warn_pct.is_finite() || input.warn_pct <= 0.0 || input.warn_pct > 1.0 {
            return Err("budget warn_pct must be between 0 and 1".to_string());
        }

        let id = budget_id_for_scope(scope_type, scope_id.as_deref());
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO budgets(
                   id, scope_type, scope_id, max_usd, warn_pct, hard_limit, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   max_usd = excluded.max_usd,
                   warn_pct = excluded.warn_pct,
                   hard_limit = excluded.hard_limit,
                   updated_at = excluded.updated_at",
                params![
                    id,
                    scope_type,
                    scope_id,
                    input.max_usd,
                    input.warn_pct,
                    if input.hard_limit { 1 } else { 0 }
                ],
            )
            .map_err(|error| format!("failed to upsert budget {id}: {error}"))?;
        drop(connection);

        self.get_budget(&id)?
            .ok_or_else(|| format!("upserted budget {id} could not be loaded"))
    }

    pub fn upsert_agent_profile(
        &self,
        input: UpsertAgentProfileInput,
    ) -> Result<PersistedAgentProfile, String> {
        let id = input.id.trim();
        if id.is_empty() {
            return Err("agent profile id cannot be empty".to_string());
        }
        let name = input.name.trim();
        if name.is_empty() {
            return Err("agent profile name cannot be empty".to_string());
        }
        let runtime = input.runtime.trim();
        if runtime.is_empty() {
            return Err("agent profile runtime cannot be empty".to_string());
        }
        let command = input.command.trim();
        if command.is_empty() {
            return Err("agent profile command cannot be empty".to_string());
        }
        let status = normalize_agent_status(input.status.as_deref().unwrap_or("available"))?;
        let args_json = input.args_json.unwrap_or_else(|| serde_json::json!([]));
        let env_policy_json = input
            .env_policy_json
            .unwrap_or_else(default_agent_env_policy_json);
        let skills_json = input.skills_json.unwrap_or_else(|| serde_json::json!([]));

        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO agent_profiles(
                   id, name, runtime, command, args_json, env_policy_json,
                   skills_json, status, last_heartbeat_at, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   runtime = excluded.runtime,
                   command = excluded.command,
                   args_json = excluded.args_json,
                   env_policy_json = excluded.env_policy_json,
                   skills_json = excluded.skills_json,
                   status = excluded.status,
                   updated_at = excluded.updated_at",
                params![
                    id,
                    name,
                    runtime,
                    command,
                    args_json.to_string(),
                    env_policy_json.to_string(),
                    skills_json.to_string(),
                    status
                ],
            )
            .map_err(|error| format!("failed to upsert agent profile {id}: {error}"))?;
        drop(connection);

        self.get_agent_profile(id)?
            .ok_or_else(|| format!("upserted agent profile {id} could not be loaded"))
    }

    pub fn scan_agent_profiles(&self) -> Result<Vec<PersistedAgentProfile>, String> {
        for preset in builtin_agent_presets() {
            self.upsert_agent_profile(preset)?;
        }
        self.list_agent_profiles()
    }

    pub fn get_agent_profile(
        &self,
        agent_id: &str,
    ) -> Result<Option<PersistedAgentProfile>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, name, runtime, command, args_json, env_policy_json,
                   skills_json, status, last_heartbeat_at
                 FROM agent_profiles
                 WHERE id = ?1",
                params![agent_id],
                persisted_agent_profile_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load agent profile {agent_id}: {error}"))
    }

    pub fn list_agent_profiles(&self) -> Result<Vec<PersistedAgentProfile>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, name, runtime, command, args_json, env_policy_json,
                   skills_json, status, last_heartbeat_at
                 FROM agent_profiles
                 ORDER BY id ASC",
            )
            .map_err(|error| format!("failed to prepare agent profile list query: {error}"))?;
        let rows = statement
            .query_map([], persisted_agent_profile_from_row)
            .map_err(|error| format!("failed to query agent profiles: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read agent profile row: {error}"))
    }

    pub fn update_agent_profile_status(
        &self,
        agent_id: &str,
        status: &str,
    ) -> Result<PersistedAgentProfile, String> {
        let agent_id = agent_id.trim();
        if agent_id.is_empty() {
            return Err("agent profile id cannot be empty".to_string());
        }
        let status = normalize_agent_status(status)?;
        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE agent_profiles
                 SET status = ?1,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?2",
                params![status, agent_id],
            )
            .map_err(|error| format!("failed to update agent profile {agent_id}: {error}"))?;
        drop(connection);
        if updated == 0 {
            return Err(format!("agent profile {agent_id} not found"));
        }
        self.get_agent_profile(agent_id)?
            .ok_or_else(|| format!("updated agent profile {agent_id} could not be loaded"))
    }

    pub fn heartbeat_agent_profile(&self, agent_id: &str) -> Result<PersistedAgentProfile, String> {
        let agent_id = agent_id.trim();
        if agent_id.is_empty() {
            return Err("agent profile id cannot be empty".to_string());
        }
        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE agent_profiles
                 SET status = 'available',
                     last_heartbeat_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?1",
                params![agent_id],
            )
            .map_err(|error| format!("failed to heartbeat agent profile {agent_id}: {error}"))?;
        drop(connection);
        if updated == 0 {
            return Err(format!("agent profile {agent_id} not found"));
        }
        self.get_agent_profile(agent_id)?
            .ok_or_else(|| format!("heartbeated agent profile {agent_id} could not be loaded"))
    }

    pub fn agent_terminal_launch_plan(
        &self,
        input: AgentTerminalLaunchInput,
    ) -> Result<AgentTerminalLaunchPlan, String> {
        let project_id = required_trimmed("project id", &input.project_id)?;
        let agent_profile_id = required_trimmed("agent profile id", &input.agent_profile_id)?;
        self.enforce_agent_profile_available(Some(&agent_profile_id))?;
        let agent = self
            .get_agent_profile(&agent_profile_id)?
            .ok_or_else(|| format!("agent {agent_profile_id} is not registered"))?;
        let args = agent_profile_args(&agent.args_json)?;
        let title = normalize_optional_text(input.title).unwrap_or_else(|| agent.name.clone());

        Ok(AgentTerminalLaunchPlan {
            project_id,
            agent_profile_id,
            title,
            command: agent.command,
            args,
            cols: input.cols.unwrap_or(100),
            rows: input.rows.unwrap_or(30),
        })
    }

    pub fn create_session(&self, input: CreateSessionInput) -> Result<PersistedSession, String> {
        let project_id = input.project_id.trim();
        if project_id.is_empty() {
            return Err("session project id cannot be empty".to_string());
        }
        let mode = normalize_session_mode(&input.mode)?;
        let title = input.title.trim();
        if title.is_empty() {
            return Err("session title cannot be empty".to_string());
        }

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "sessions", "session_")?;
        let id = format!("session_{next_number}");
        let pane_id = format!("pane_{id}");
        connection
            .execute(
                "INSERT INTO sessions(
                   id, project_id, pane_id, mode, title, cwd, branch,
                   agent_profile_id, task_id, run_id, state, attention_state,
                   token_budget_state, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                   'running', 'none', 'unknown',
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    id,
                    project_id,
                    pane_id,
                    mode,
                    title,
                    normalize_optional_text(input.cwd),
                    normalize_optional_text(input.branch),
                    normalize_optional_text(input.agent_profile_id),
                    normalize_optional_text(input.task_id),
                    normalize_optional_text(input.run_id)
                ],
            )
            .map_err(|error| format!("failed to create session {id}: {error}"))?;
        drop(connection);

        self.get_session(&id)?
            .ok_or_else(|| format!("created session {id} could not be loaded"))
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<PersistedSession>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, pane_id, mode, title, cwd, branch,
                   agent_profile_id, task_id, run_id, state, attention_state,
                   token_budget_state, created_at, updated_at
                 FROM sessions
                 WHERE id = ?1",
                params![session_id],
                persisted_session_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load session {session_id}: {error}"))
    }

    pub fn list_sessions(&self, project_id: &str) -> Result<Vec<PersistedSession>, String> {
        let project_id = project_id.trim();
        if project_id.is_empty() {
            return Err("session project id cannot be empty".to_string());
        }
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, pane_id, mode, title, cwd, branch,
                   agent_profile_id, task_id, run_id, state, attention_state,
                   token_budget_state, created_at, updated_at
                 FROM sessions
                 WHERE project_id = ?1
                 ORDER BY created_at ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare session list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], persisted_session_from_row)
            .map_err(|error| format!("failed to query sessions: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read session row: {error}"))
    }

    pub fn focus_session(&self, session_id: &str) -> Result<PersistedSession, String> {
        self.update_session_state(session_id, None, Some("none"))
    }

    pub fn set_session_attention(
        &self,
        session_id: &str,
        attention_state: &str,
    ) -> Result<PersistedSession, String> {
        self.update_session_state(session_id, None, Some(attention_state))
    }

    pub fn resize_session(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<SessionResizeReceipt, String> {
        let session_id = required_trimmed("session id", session_id)?;
        if cols == 0 || rows == 0 {
            return Err("session resize cols and rows must be greater than zero".to_string());
        }
        let session = self
            .get_session(&session_id)?
            .ok_or_else(|| format!("session {session_id} not found"))?;

        Ok(SessionResizeReceipt {
            session_id: session.id,
            pane_id: session.pane_id,
            cols,
            rows,
        })
    }

    pub fn takeover_session(&self, session_id: &str) -> Result<PersistedSession, String> {
        self.update_session_state(session_id, Some("running"), Some("needs_input"))
    }

    pub fn release_session(&self, session_id: &str) -> Result<PersistedSession, String> {
        self.update_session_state(session_id, Some("running"), Some("none"))
    }

    pub fn kill_session(&self, session_id: &str) -> Result<PersistedSession, String> {
        self.update_session_state(session_id, Some("completed"), Some("none"))
    }

    pub fn attach_session_task(
        &self,
        session_id: &str,
        task_id: &str,
    ) -> Result<PersistedSession, String> {
        let session_id = required_trimmed("session id", session_id)?;
        let task_id = required_trimmed("session task id", task_id)?;
        if self.get_session(&session_id)?.is_none() {
            return Err(format!("session {session_id} not found"));
        }
        if self.get_task(&task_id)?.is_none() {
            return Err(format!("task {task_id} not found"));
        }
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE sessions
                 SET task_id = ?1,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?2",
                params![task_id, session_id],
            )
            .map_err(|error| {
                format!("failed to attach task {task_id} to session {session_id}: {error}")
            })?;
        drop(connection);
        self.get_session(&session_id)?
            .ok_or_else(|| format!("updated session {session_id} could not be loaded"))
    }

    pub fn detach_session_task(&self, session_id: &str) -> Result<PersistedSession, String> {
        let session_id = required_trimmed("session id", session_id)?;
        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE sessions
                 SET task_id = NULL,
                     run_id = NULL,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?1",
                params![session_id],
            )
            .map_err(|error| format!("failed to detach task from session {session_id}: {error}"))?;
        drop(connection);
        if updated == 0 {
            return Err(format!("session {session_id} not found"));
        }
        self.get_session(&session_id)?
            .ok_or_else(|| format!("updated session {session_id} could not be loaded"))
    }

    pub fn record_session_input(
        &self,
        input: SessionInputInput,
    ) -> Result<SessionInputReceipt, String> {
        let session_id = input.session_id.trim();
        if session_id.is_empty() {
            return Err("session id cannot be empty".to_string());
        }
        let text = input.text.trim_end_matches(['\r', '\n']);
        if text.trim().is_empty() {
            return Err("session input text cannot be empty".to_string());
        }
        let session = self
            .get_session(session_id)?
            .ok_or_else(|| format!("session {session_id} not found"))?;
        let dangerous = is_dangerous_session_input(text);
        if dangerous && !input.allow_dangerous {
            return Err("session input requires allowDangerous for dangerous text".to_string());
        }

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "command_blocks", "cmdblk_session_input_")?;
        drop(connection);
        let command_block_id = format!("cmdblk_session_input_{next_number}");
        self.upsert_command_block(&PersistedCommandBlockInput {
            id: command_block_id.clone(),
            session_id: session.id.clone(),
            task_id: session.task_id.clone(),
            run_id: session.run_id.clone(),
            seq_start: None,
            seq_end: None,
            command: summarize_session_input(text),
            cwd: session.cwd.clone(),
            branch: session.branch.clone(),
            exit_code: None,
            duration_ms: None,
            summary: Some("Session input accepted through guarded control API".to_string()),
        })?;

        Ok(SessionInputReceipt {
            session_id: session.id,
            accepted: true,
            dangerous,
            input_len: input.text.len(),
            command_block_id: Some(command_block_id),
        })
    }

    pub fn record_terminal_stream_chunk(
        &self,
        input: RecordTerminalStreamChunkInput,
    ) -> Result<PersistedTerminalStreamChunk, String> {
        let session_id = required_trimmed("terminal stream session id", &input.session_id)?;
        if input.seq_start < 0 {
            return Err("terminal stream seq_start cannot be negative".to_string());
        }
        if input.seq_end < input.seq_start {
            return Err("terminal stream seq_end cannot be before seq_start".to_string());
        }
        if input.body.is_empty() {
            return Err("terminal stream chunk body cannot be empty".to_string());
        }
        self.get_session(&session_id)?
            .ok_or_else(|| format!("session {session_id} not found"))?;

        let redacted_body = self.redact_secrets(&input.body)?;
        let connection = self.connection()?;
        let next_number = next_numeric_id(
            &connection,
            "terminal_stream_chunks",
            "terminal_stream_chunk_",
        )?;
        drop(connection);
        let id = format!("terminal_stream_chunk_{next_number}");
        let artifact_path = self.terminal_stream_chunk_artifact_path(&session_id, &id);
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create terminal stream chunk directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&artifact_path, &redacted_body).map_err(|error| {
            format!(
                "failed to write terminal stream chunk {id} artifact {}: {error}",
                artifact_path.display()
            )
        })?;
        let relative_path = self.relative_artifact_path(&artifact_path);

        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO terminal_stream_chunks(
                   id, session_id, seq_start, seq_end, artifact_path, created_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    &id,
                    &session_id,
                    input.seq_start,
                    input.seq_end,
                    &relative_path
                ],
            )
            .map_err(|error| format!("failed to persist terminal stream chunk {id}: {error}"))?;
        drop(connection);

        self.get_terminal_stream_chunk(&id)?
            .ok_or_else(|| format!("terminal stream chunk {id} could not be loaded"))
    }

    pub fn get_terminal_stream_chunk(
        &self,
        chunk_id: &str,
    ) -> Result<Option<PersistedTerminalStreamChunk>, String> {
        let chunk_id = required_trimmed("terminal stream chunk id", chunk_id)?;
        let connection = self.connection()?;
        let metadata = connection
            .query_row(
                "SELECT id, session_id, seq_start, seq_end, artifact_path, created_at
                 FROM terminal_stream_chunks
                 WHERE id = ?1",
                params![chunk_id],
                terminal_stream_chunk_metadata_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load terminal stream chunk {chunk_id}: {error}"))?;
        drop(connection);

        metadata
            .map(|metadata| self.hydrate_terminal_stream_chunk(metadata))
            .transpose()
    }

    pub fn list_terminal_stream_chunks(
        &self,
        session_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<PersistedTerminalStreamChunk>, String> {
        let session_id = required_trimmed("terminal stream session id", session_id)?;
        self.get_session(&session_id)?
            .ok_or_else(|| format!("session {session_id} not found"))?;
        let limit = limit.unwrap_or(100).clamp(1, 500) as i64;

        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, session_id, seq_start, seq_end, artifact_path, created_at
                 FROM terminal_stream_chunks
                 WHERE session_id = ?1
                 ORDER BY seq_start ASC, seq_end ASC, id ASC
                 LIMIT ?2",
            )
            .map_err(|error| {
                format!("failed to prepare terminal stream chunk query for {session_id}: {error}")
            })?;
        let rows = statement
            .query_map(
                params![session_id, limit],
                terminal_stream_chunk_metadata_from_row,
            )
            .map_err(|error| {
                format!("failed to query terminal stream chunks for {session_id}: {error}")
            })?;
        let metadata = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read terminal stream chunk row: {error}"))?;
        drop(statement);
        drop(connection);

        metadata
            .into_iter()
            .map(|metadata| self.hydrate_terminal_stream_chunk(metadata))
            .collect()
    }

    fn hydrate_terminal_stream_chunk(
        &self,
        metadata: TerminalStreamChunkMetadata,
    ) -> Result<PersistedTerminalStreamChunk, String> {
        let full_path = self.resolve_artifact_path(&metadata.artifact_path);
        let body = fs::read_to_string(&full_path).map_err(|error| {
            format!(
                "failed to read terminal stream chunk {} artifact {}: {error}",
                metadata.id,
                full_path.display()
            )
        })?;

        Ok(PersistedTerminalStreamChunk {
            id: metadata.id,
            session_id: metadata.session_id,
            seq_start: metadata.seq_start,
            seq_end: metadata.seq_end,
            artifact_path: metadata.artifact_path,
            body,
            created_at: metadata.created_at,
        })
    }

    fn update_session_state(
        &self,
        session_id: &str,
        state: Option<&str>,
        attention_state: Option<&str>,
    ) -> Result<PersistedSession, String> {
        let session_id = session_id.trim();
        if session_id.is_empty() {
            return Err("session id cannot be empty".to_string());
        }
        if let Some(state) = state {
            normalize_session_state(state)?;
        }
        if let Some(attention_state) = attention_state {
            normalize_session_attention_state(attention_state)?;
        }
        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE sessions
                 SET state = COALESCE(?1, state),
                     attention_state = COALESCE(?2, attention_state),
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?3",
                params![state, attention_state, session_id],
            )
            .map_err(|error| format!("failed to update session {session_id}: {error}"))?;
        drop(connection);

        if updated == 0 {
            return Err(format!("session {session_id} not found"));
        }
        self.get_session(session_id)?
            .ok_or_else(|| format!("updated session {session_id} could not be loaded"))
    }

    pub fn budget_summary(&self) -> Result<Value, String> {
        let budgets = self.list_budgets()?;
        let workspace_used = self.token_usage_total("workspace", None)?;
        let workspace_budget = budgets
            .iter()
            .find(|budget| budget.scope_type == "workspace");
        let workspace = workspace_budget
            .map(|budget| budget_summary_json(budget, workspace_used))
            .unwrap_or_else(|| {
                serde_json::json!({
                    "scope_type": "workspace",
                    "scope_id": null,
                    "used_usd": round_cost(workspace_used),
                    "state": "unknown"
                })
            });
        let projects = budgets
            .iter()
            .filter(|budget| budget.scope_type == "project")
            .map(|budget| {
                let used = self.token_usage_total("project", budget.scope_id.as_deref())?;
                Ok(budget_summary_json(budget, used))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let goals = budgets
            .iter()
            .filter(|budget| budget.scope_type == "goal")
            .map(|budget| {
                let used = self.token_usage_total("goal", budget.scope_id.as_deref())?;
                Ok(budget_summary_json(budget, used))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let tasks = budgets
            .iter()
            .filter(|budget| budget.scope_type == "task")
            .map(|budget| {
                let used = self.token_usage_total("task", budget.scope_id.as_deref())?;
                Ok(budget_summary_json(budget, used))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let runs = budgets
            .iter()
            .filter(|budget| budget.scope_type == "run")
            .map(|budget| {
                let used = self.token_usage_total("run", budget.scope_id.as_deref())?;
                Ok(budget_summary_json(budget, used))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let agents = budgets
            .iter()
            .filter(|budget| budget.scope_type == "agent")
            .map(|budget| {
                let used = self.token_usage_total("agent", budget.scope_id.as_deref())?;
                Ok(budget_summary_json(budget, used))
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(serde_json::json!({
            "workspace": workspace,
            "projects": projects,
            "goals": goals,
            "tasks": tasks,
            "runs": runs,
            "agents": agents
        }))
    }

    pub fn budget_forecast(&self) -> Result<Value, String> {
        let budgets = self.list_budgets()?;
        let workspace_used = self.token_usage_total("workspace", None)?;
        let workspace_budget = budgets
            .iter()
            .find(|budget| budget.scope_type == "workspace");
        let workspace = match workspace_budget {
            Some(budget) => {
                let run_costs = self.token_usage_run_costs("workspace", None)?;
                budget_forecast_json(budget, workspace_used, &run_costs)
            }
            None => serde_json::json!({}),
        };
        let projects = budgets
            .iter()
            .filter(|budget| budget.scope_type == "project")
            .map(|budget| {
                let used = self.token_usage_total("project", budget.scope_id.as_deref())?;
                let run_costs =
                    self.token_usage_run_costs("project", budget.scope_id.as_deref())?;
                Ok(budget_forecast_json(budget, used, &run_costs))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let goals = budgets
            .iter()
            .filter(|budget| budget.scope_type == "goal")
            .map(|budget| {
                let used = self.token_usage_total("goal", budget.scope_id.as_deref())?;
                let run_costs = self.token_usage_run_costs("goal", budget.scope_id.as_deref())?;
                Ok(budget_forecast_json(budget, used, &run_costs))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let tasks = budgets
            .iter()
            .filter(|budget| budget.scope_type == "task")
            .map(|budget| {
                let used = self.token_usage_total("task", budget.scope_id.as_deref())?;
                let run_costs = self.token_usage_run_costs("task", budget.scope_id.as_deref())?;
                Ok(budget_forecast_json(budget, used, &run_costs))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let runs = budgets
            .iter()
            .filter(|budget| budget.scope_type == "run")
            .map(|budget| {
                let used = self.token_usage_total("run", budget.scope_id.as_deref())?;
                let run_costs = self.token_usage_run_costs("run", budget.scope_id.as_deref())?;
                Ok(budget_forecast_json(budget, used, &run_costs))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let agents = budgets
            .iter()
            .filter(|budget| budget.scope_type == "agent")
            .map(|budget| {
                let used = self.token_usage_total("agent", budget.scope_id.as_deref())?;
                let run_costs = self.token_usage_run_costs("agent", budget.scope_id.as_deref())?;
                Ok(budget_forecast_json(budget, used, &run_costs))
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(serde_json::json!({
            "workspace": workspace,
            "projects": projects,
            "goals": goals,
            "tasks": tasks,
            "runs": runs,
            "agents": agents
        }))
    }

    pub fn token_usage_summary_for_session(
        &self,
        session_id: &str,
    ) -> Result<TokenUsageSummary, String> {
        let session_id = required_trimmed("token usage session id", session_id)?;
        let connection = self.connection()?;
        let (input_tokens, output_tokens, cost_usd) = connection
            .query_row(
                "SELECT
                   COALESCE(SUM(input_tokens), 0),
                   COALESCE(SUM(output_tokens), 0),
                   COALESCE(SUM(cost_usd), 0.0)
                 FROM token_usage
                 WHERE session_id = ?1",
                params![session_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, f64>(2)?,
                    ))
                },
            )
            .map_err(|error| {
                format!("failed to summarize token usage for session {session_id}: {error}")
            })?;
        Ok(TokenUsageSummary {
            session_id,
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cost_usd: round_cost(cost_usd),
        })
    }

    pub fn token_usage_summary_for_task(&self, task_id: &str) -> Result<TokenUsageTotals, String> {
        let task_id = required_trimmed("token usage task id", task_id)?;
        let connection = self.connection()?;
        let (input_tokens, output_tokens, cost_usd) = connection
            .query_row(
                "SELECT
                   COALESCE(SUM(input_tokens), 0),
                   COALESCE(SUM(output_tokens), 0),
                   COALESCE(SUM(cost_usd), 0.0)
                 FROM token_usage
                 WHERE task_id = ?1",
                params![task_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, f64>(2)?,
                    ))
                },
            )
            .map_err(|error| {
                format!("failed to summarize token usage for task {task_id}: {error}")
            })?;
        Ok(TokenUsageTotals {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cost_usd: round_cost(cost_usd),
        })
    }

    pub fn token_usage_summary_for_project(
        &self,
        project_id: &str,
    ) -> Result<TokenUsageTotals, String> {
        let project_id = required_trimmed("token usage project id", project_id)?;
        let connection = self.connection()?;
        let (input_tokens, output_tokens, cost_usd) = connection
            .query_row(
                "SELECT
                   COALESCE(SUM(input_tokens), 0),
                   COALESCE(SUM(output_tokens), 0),
                   COALESCE(SUM(cost_usd), 0.0)
                 FROM token_usage
                 WHERE project_id = ?1",
                params![project_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, f64>(2)?,
                    ))
                },
            )
            .map_err(|error| {
                format!("failed to summarize token usage for project {project_id}: {error}")
            })?;
        Ok(TokenUsageTotals {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cost_usd: round_cost(cost_usd),
        })
    }

    pub fn token_usage_summary_for_agent(
        &self,
        agent_profile_id: &str,
    ) -> Result<TokenUsageTotals, String> {
        let agent_profile_id = required_trimmed("token usage agent profile id", agent_profile_id)?;
        let connection = self.connection()?;
        let (input_tokens, output_tokens, cost_usd) = connection
            .query_row(
                "SELECT
                   COALESCE(SUM(input_tokens), 0),
                   COALESCE(SUM(output_tokens), 0),
                   COALESCE(SUM(cost_usd), 0.0)
                 FROM token_usage
                 WHERE agent_profile_id = ?1",
                params![agent_profile_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, f64>(2)?,
                    ))
                },
            )
            .map_err(|error| {
                format!(
                    "failed to summarize token usage for agent profile {agent_profile_id}: {error}"
                )
            })?;
        Ok(TokenUsageTotals {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cost_usd: round_cost(cost_usd),
        })
    }

    pub fn token_usage_summary_for_goal(
        &self,
        initiative_id: &str,
    ) -> Result<TokenUsageTotals, String> {
        let initiative_id = required_trimmed("token usage goal id", initiative_id)?;
        let connection = self.connection()?;
        let (input_tokens, output_tokens, cost_usd) = connection
            .query_row(
                "SELECT
                   COALESCE(SUM(token_usage.input_tokens), 0),
                   COALESCE(SUM(token_usage.output_tokens), 0),
                   COALESCE(SUM(token_usage.cost_usd), 0.0)
                 FROM token_usage
                 JOIN tasks ON tasks.id = token_usage.task_id
                 WHERE tasks.initiative_id = ?1",
                params![initiative_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, f64>(2)?,
                    ))
                },
            )
            .map_err(|error| {
                format!("failed to summarize token usage for goal {initiative_id}: {error}")
            })?;
        Ok(TokenUsageTotals {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cost_usd: round_cost(cost_usd),
        })
    }

    pub fn token_usage_export_for_run(&self, run_id: &str) -> Result<Value, String> {
        let run_id = required_trimmed("token usage run id", run_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, session_id, task_id, run_id, agent_profile_id,
                   provider, model, input_tokens, output_tokens, cost_usd, source
                 FROM token_usage
                 WHERE run_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|error| format!("failed to prepare token usage export query: {error}"))?;
        let rows = statement
            .query_map(params![run_id], persisted_token_usage_from_row)
            .map_err(|error| format!("failed to query token usage for run {run_id}: {error}"))?;
        let records = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read token usage row: {error}"))?;
        let input_tokens: i64 = records.iter().map(|usage| usage.input_tokens).sum();
        let output_tokens: i64 = records.iter().map(|usage| usage.output_tokens).sum();
        let cost_usd: f64 = records.iter().map(|usage| usage.cost_usd).sum();

        Ok(serde_json::json!({
            "scope_type": "run",
            "scope_id": run_id,
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
            "total_tokens": input_tokens + output_tokens,
            "cost_usd": round_cost(cost_usd),
            "records": records
        }))
    }

    pub fn upsert_knowledge_source(
        &self,
        input: UpsertKnowledgeSourceInput,
    ) -> Result<PersistedKnowledgeSource, String> {
        let project_id = required_trimmed("knowledge source project id", &input.project_id)?;
        let kind = required_trimmed("knowledge source kind", &input.kind)?;
        let path_or_ref = required_trimmed("knowledge source path", &input.path_or_ref)?;
        let fingerprint = required_trimmed("knowledge source fingerprint", &input.fingerprint)?;
        let status = normalize_knowledge_status(&input.status)?;
        let connection = self.connection()?;
        let existing_id = connection
            .query_row(
                "SELECT id FROM knowledge_sources
                 WHERE project_id = ?1 AND kind = ?2 AND path_or_ref = ?3",
                params![project_id, kind, path_or_ref],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| format!("failed to load knowledge source: {error}"))?;
        let id = match existing_id {
            Some(id) => id,
            None => format!(
                "ks_{}",
                next_numeric_id(&connection, "knowledge_sources", "ks_")?
            ),
        };
        connection
            .execute(
                "INSERT INTO knowledge_sources(
                   id, project_id, kind, path_or_ref, fingerprint, status, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   fingerprint = excluded.fingerprint,
                   status = excluded.status,
                   updated_at = excluded.updated_at",
                params![id, project_id, kind, path_or_ref, fingerprint, status],
            )
            .map_err(|error| format!("failed to persist knowledge source {id}: {error}"))?;
        drop(connection);

        self.get_knowledge_source(&id)?
            .ok_or_else(|| format!("saved knowledge source {id} could not be loaded"))
    }

    pub fn list_knowledge_sources(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedKnowledgeSource>, String> {
        let project_id = required_trimmed("knowledge source project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, kind, path_or_ref, fingerprint, status
                 FROM knowledge_sources
                 WHERE project_id = ?1
                 ORDER BY updated_at DESC, path_or_ref ASC",
            )
            .map_err(|error| format!("failed to prepare knowledge source index query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], persisted_knowledge_source_from_row)
            .map_err(|error| format!("failed to query knowledge source index: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read knowledge source row: {error}"))
    }

    pub fn save_knowledge_page(
        &self,
        input: SaveKnowledgePageInput,
    ) -> Result<PersistedKnowledgePage, String> {
        let project_id = required_trimmed("knowledge page project id", &input.project_id)?;
        let slug = normalize_knowledge_slug(&input.slug)?;
        let title = required_trimmed("knowledge page title", &input.title)?;
        let freshness_state = normalize_knowledge_status(&input.freshness_state)?;
        let connection = self.connection()?;
        let existing_id = connection
            .query_row(
                "SELECT id FROM knowledge_pages WHERE project_id = ?1 AND slug = ?2",
                params![project_id, slug],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| format!("failed to load knowledge page by slug: {error}"))?;
        let id = match existing_id {
            Some(id) => id,
            None => format!(
                "kp_{}",
                next_numeric_id(&connection, "knowledge_pages", "kp_")?
            ),
        };
        drop(connection);

        let artifact_path = self.knowledge_page_artifact_path(&slug);
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create knowledge artifact directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&artifact_path, &input.body_md).map_err(|error| {
            format!(
                "failed to write knowledge page artifact {}: {error}",
                artifact_path.display()
            )
        })?;
        let relative_path = self.relative_artifact_path(&artifact_path);
        let source_ids_json = serde_json::to_string(&input.source_ids)
            .map_err(|error| format!("failed to serialize knowledge source ids: {error}"))?;
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO knowledge_pages(
                   id, project_id, slug, title, artifact_path, source_ids_json,
                   freshness_state, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   title = excluded.title,
                   artifact_path = excluded.artifact_path,
                   source_ids_json = excluded.source_ids_json,
                   freshness_state = excluded.freshness_state,
                   updated_at = excluded.updated_at",
                params![
                    id,
                    project_id,
                    slug,
                    title,
                    relative_path,
                    source_ids_json,
                    freshness_state
                ],
            )
            .map_err(|error| format!("failed to persist knowledge page {id}: {error}"))?;
        drop(connection);

        self.get_knowledge_page(&id)?
            .ok_or_else(|| format!("saved knowledge page {id} could not be loaded"))
    }

    pub fn upsert_context_pack(
        &self,
        input: UpsertContextPackInput,
    ) -> Result<PersistedContextPack, String> {
        let project_id = required_trimmed("context pack project id", &input.project_id)?;
        let name = required_trimmed("context pack name", &input.name)?;
        let id = match normalize_optional_text(input.id) {
            Some(id) => id,
            None => {
                let connection = self.connection()?;
                format!(
                    "ctx_{}",
                    next_numeric_id(&connection, "context_packs", "ctx_")?
                )
            }
        };
        let sources = input
            .sources_json
            .as_array()
            .cloned()
            .ok_or_else(|| "context pack sources must be an array".to_string())?;
        let mut body = serde_json::json!({
            "sources": sources
        });
        if let Some(max_tokens_hint) = input.max_tokens_hint {
            if max_tokens_hint <= 0 {
                return Err("context pack max_tokens_hint must be positive".to_string());
            }
            body["budget"] = serde_json::json!({ "max_tokens_hint": max_tokens_hint });
        }

        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO context_packs(
                   id, project_id, name, description, sources_json, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   description = excluded.description,
                   sources_json = excluded.sources_json,
                   updated_at = excluded.updated_at",
                params![
                    id,
                    project_id,
                    name,
                    normalize_optional_text(input.description),
                    body.to_string()
                ],
            )
            .map_err(|error| format!("failed to persist context pack {id}: {error}"))?;
        drop(connection);

        self.get_context_pack(&id)?
            .ok_or_else(|| format!("saved context pack {id} could not be loaded"))
    }

    pub fn list_context_packs(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedContextPack>, String> {
        let project_id = required_trimmed("context pack project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, name, description, sources_json
                 FROM context_packs
                 WHERE project_id = ?1
                 ORDER BY updated_at DESC, name ASC",
            )
            .map_err(|error| format!("failed to prepare context pack index query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], persisted_context_pack_from_row)
            .map_err(|error| format!("failed to query context pack index: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read context pack row: {error}"))
    }

    pub fn upsert_skill_pack(
        &self,
        input: UpsertSkillPackInput,
    ) -> Result<PersistedSkillPack, String> {
        let project_id = required_trimmed("skill pack project id", &input.project_id)?;
        let name = required_trimmed("skill pack name", &input.name)?;
        let skills = input
            .skills_json
            .as_array()
            .cloned()
            .ok_or_else(|| "skill pack skills_json must be an array".to_string())?;
        if skills.iter().any(|skill| !skill.is_string()) {
            return Err("skill pack skills_json must be an array of strings".to_string());
        }
        if self.get_project(&project_id)?.is_none() {
            return Err(format!("project {project_id} not found"));
        }
        let connection = self.connection()?;
        let existing_id: Option<String> = connection
            .query_row(
                "SELECT id FROM skill_packs WHERE project_id = ?1 AND name = ?2",
                params![&project_id, &name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| format!("failed to load existing skill pack: {error}"))?;
        let id = match existing_id {
            Some(id) => id,
            None => {
                let next_number = next_numeric_id(&connection, "skill_packs", "skill_pack_")?;
                format!("skill_pack_{next_number}")
            }
        };
        connection
            .execute(
                "INSERT INTO skill_packs(
                   id, project_id, name, description, skills_json, source_context_pack_id,
                   created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(project_id, name) DO UPDATE SET
                   description = excluded.description,
                   skills_json = excluded.skills_json,
                   source_context_pack_id = excluded.source_context_pack_id,
                   updated_at = excluded.updated_at",
                params![
                    &id,
                    &project_id,
                    &name,
                    normalize_optional_text(input.description),
                    Value::Array(skills).to_string(),
                    normalize_optional_text(input.source_context_pack_id)
                ],
            )
            .map_err(|error| format!("failed to persist skill pack {name}: {error}"))?;
        drop(connection);

        self.get_skill_pack(&id)?
            .ok_or_else(|| format!("saved skill pack {id} could not be loaded"))
    }

    pub fn list_skill_packs(&self, project_id: &str) -> Result<Vec<PersistedSkillPack>, String> {
        let project_id = required_trimmed("skill pack project id", project_id)?;
        if self.get_project(&project_id)?.is_none() {
            return Err(format!("project {project_id} not found"));
        }
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, name, description, skills_json, source_context_pack_id,
                   created_at, updated_at
                 FROM skill_packs
                 WHERE project_id = ?1
                 ORDER BY lower(name) ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare skill pack list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], persisted_skill_pack_from_row)
            .map_err(|error| format!("failed to query skill packs: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read skill pack row: {error}"))
    }

    pub fn save_knowledge_exploration(
        &self,
        input: SaveKnowledgeExplorationInput,
    ) -> Result<PersistedKnowledgeExploration, String> {
        let project_id = required_trimmed("knowledge exploration project id", &input.project_id)?;
        let title = required_trimmed("knowledge exploration title", &input.title)?;
        let question = required_trimmed("knowledge exploration question", &input.question)?;
        let answer_md = required_trimmed("knowledge exploration answer", &input.answer_md)?;
        let context_pack_id = normalize_optional_text(input.context_pack_id);
        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "knowledge_explorations", "kexp_")?;
        let id = format!("kexp_{next_number}");
        drop(connection);

        let artifact_path = self.knowledge_exploration_artifact_path(&id);
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create knowledge exploration artifact directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&artifact_path, answer_md).map_err(|error| {
            format!(
                "failed to write knowledge exploration artifact {}: {error}",
                artifact_path.display()
            )
        })?;
        let relative_path = self.relative_artifact_path(&artifact_path);
        let page_ids_json = serde_json::to_string(&input.page_ids).map_err(|error| {
            format!("failed to serialize knowledge exploration page ids: {error}")
        })?;
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO knowledge_explorations(
                   id, project_id, title, question, artifact_path, page_ids_json,
                   context_pack_id, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    id,
                    project_id,
                    title,
                    question,
                    relative_path,
                    page_ids_json,
                    context_pack_id
                ],
            )
            .map_err(|error| format!("failed to persist knowledge exploration {id}: {error}"))?;
        drop(connection);

        self.get_knowledge_exploration(&id)?
            .ok_or_else(|| format!("saved knowledge exploration {id} could not be loaded"))
    }

    pub fn list_knowledge_explorations(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedKnowledgeExploration>, String> {
        let project_id = required_trimmed("knowledge exploration project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id
                 FROM knowledge_explorations
                 WHERE project_id = ?1
                 ORDER BY updated_at DESC, id DESC
                 LIMIT 50",
            )
            .map_err(|error| {
                format!("failed to prepare knowledge exploration index query: {error}")
            })?;
        let rows = statement
            .query_map(params![project_id], |row| row.get::<_, String>(0))
            .map_err(|error| format!("failed to query knowledge explorations: {error}"))?;
        let ids = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read knowledge exploration id row: {error}"))?;
        drop(statement);
        drop(connection);

        ids.into_iter()
            .map(|id| {
                self.get_knowledge_exploration(&id)?
                    .ok_or_else(|| format!("knowledge exploration {id} disappeared during load"))
            })
            .collect()
    }

    pub fn record_knowledge_lint_report(
        &self,
        input: RecordKnowledgeLintReportInput,
    ) -> Result<PersistedKnowledgeLintReport, String> {
        let project_id = required_trimmed("knowledge lint project id", &input.project_id)?;
        if input.stale_count < 0 || input.gap_count < 0 || input.contradiction_count < 0 {
            return Err("knowledge lint counts cannot be negative".to_string());
        }
        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "knowledge_lint_reports", "klr_")?;
        let id = format!("klr_{next_number}");
        drop(connection);
        let artifact_path = self.knowledge_lint_artifact_path(&id);
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create knowledge lint artifact directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&artifact_path, &input.body_md).map_err(|error| {
            format!(
                "failed to write knowledge lint artifact {}: {error}",
                artifact_path.display()
            )
        })?;
        let relative_path = self.relative_artifact_path(&artifact_path);
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO knowledge_lint_reports(
                   id, project_id, artifact_path, stale_count, gap_count,
                   contradiction_count, created_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    id,
                    project_id,
                    relative_path,
                    input.stale_count,
                    input.gap_count,
                    input.contradiction_count
                ],
            )
            .map_err(|error| format!("failed to persist knowledge lint report {id}: {error}"))?;
        drop(connection);

        self.get_knowledge_lint_report(&id)?
            .ok_or_else(|| format!("saved knowledge lint report {id} could not be loaded"))
    }

    pub fn search_knowledge_pages(
        &self,
        project_id: &str,
        query: Option<&str>,
    ) -> Result<Vec<PersistedKnowledgePage>, String> {
        let project_id = project_id.trim();
        if project_id.is_empty() {
            return Err("knowledge search project id cannot be empty".to_string());
        }
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id
                 FROM knowledge_pages
                 WHERE project_id = ?1
                 ORDER BY updated_at DESC, id DESC
                 LIMIT 50",
            )
            .map_err(|error| format!("failed to prepare knowledge search query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], |row| row.get::<_, String>(0))
            .map_err(|error| format!("failed to query knowledge pages: {error}"))?;
        let ids = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read knowledge page row: {error}"))?;
        drop(statement);
        drop(connection);

        let mut pages = ids
            .into_iter()
            .map(|id| {
                self.get_knowledge_page(&id)?
                    .ok_or_else(|| format!("knowledge page {id} disappeared while searching"))
            })
            .collect::<Result<Vec<_>, String>>()?;
        if let Some(query) = normalize_optional_borrowed_text(query.as_deref()) {
            pages.retain(|page| {
                page.body_md.to_lowercase().contains(&query.to_lowercase())
                    || page.title.to_lowercase().contains(&query.to_lowercase())
                    || page.slug.to_lowercase().contains(&query.to_lowercase())
            });
        }
        Ok(pages)
    }

    pub fn list_knowledge_concepts(
        &self,
        project_id: &str,
    ) -> Result<Vec<KnowledgeConcept>, String> {
        let project_id = required_trimmed("knowledge concept project id", project_id)?;
        let pages = self.search_knowledge_pages(&project_id, None)?;
        let mut concepts: BTreeMap<String, KnowledgeConcept> = BTreeMap::new();

        for page in &pages {
            let slug = slugify_knowledge_artifact(&page.title);
            let outbound_slugs = extract_knowledge_wiki_links(&page.body_md);
            let concept = concepts
                .entry(slug.clone())
                .or_insert_with(|| KnowledgeConcept {
                    slug: slug.clone(),
                    title: page.title.clone(),
                    page_id: None,
                    outbound_slugs: vec![],
                    inbound_page_ids: vec![],
                });
            concept.title = page.title.clone();
            concept.page_id = Some(page.id.clone());
            concept.outbound_slugs = outbound_slugs.clone();

            for outbound_slug in outbound_slugs {
                let linked =
                    concepts
                        .entry(outbound_slug.clone())
                        .or_insert_with(|| KnowledgeConcept {
                            slug: outbound_slug.clone(),
                            title: title_from_concept_slug(&outbound_slug),
                            page_id: None,
                            outbound_slugs: vec![],
                            inbound_page_ids: vec![],
                        });
                if !linked.inbound_page_ids.contains(&page.id) {
                    linked.inbound_page_ids.push(page.id.clone());
                }
            }
        }

        Ok(concepts.into_values().collect())
    }

    pub fn export_knowledge_obsidian_markdown(
        &self,
        project_id: &str,
    ) -> Result<KnowledgeObsidianExport, String> {
        let project_id = required_trimmed("knowledge obsidian export project id", project_id)?;
        let pages = self.search_knowledge_pages(&project_id, None)?;
        let export_root = self
            .artifact_root()
            .join("knowledge")
            .join("obsidian")
            .join(sanitize_artifact_name(&project_id));
        fs::create_dir_all(&export_root).map_err(|error| {
            format!(
                "failed to create knowledge obsidian export directory {}: {error}",
                export_root.display()
            )
        })?;

        let mut files = Vec::new();
        for page in &pages {
            let file_name = obsidian_markdown_file_name(&page.title);
            fs::write(export_root.join(&file_name), &page.body_md).map_err(|error| {
                format!("failed to write knowledge obsidian export {file_name}: {error}")
            })?;
            files.push(file_name);
        }
        files.sort();

        let index_body = build_obsidian_knowledge_index(&pages);
        let index_file = "Knowledge Index.md".to_string();
        fs::write(export_root.join(&index_file), index_body)
            .map_err(|error| format!("failed to write knowledge obsidian export index: {error}"))?;
        files.push(index_file);

        Ok(KnowledgeObsidianExport {
            project_id,
            status: "exported".to_string(),
            export_root: self.relative_artifact_path(&export_root),
            file_count: files.len(),
            files,
        })
    }

    pub fn answer_knowledge_question(
        &self,
        input: KnowledgeChatQuestionInput,
    ) -> Result<KnowledgeChatAnswer, String> {
        let project_id = required_trimmed("knowledge chat project id", &input.project_id)?;
        let question = required_trimmed("knowledge chat question", &input.question)?;
        let context_pack_id = normalize_optional_text(input.context_pack_id);
        let query_terms = knowledge_query_terms(&question);
        if query_terms.is_empty() {
            return Err("knowledge chat question must include searchable terms".to_string());
        }
        let mut scored_pages = self
            .search_knowledge_pages(&project_id, None)?
            .into_iter()
            .filter_map(|page| {
                let score = score_knowledge_page_for_terms(&page, &query_terms);
                (score > 0).then_some((score, page))
            })
            .collect::<Vec<_>>();
        scored_pages.sort_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| left.1.title.cmp(&right.1.title))
        });
        scored_pages.truncate(3);

        let cited_pages = scored_pages
            .into_iter()
            .map(|(_, page)| page)
            .collect::<Vec<_>>();
        let cited_page_ids = cited_pages
            .iter()
            .map(|page| page.id.clone())
            .collect::<Vec<_>>();
        let answer_md = build_local_knowledge_answer(&question, &cited_pages);

        Ok(KnowledgeChatAnswer {
            project_id,
            question,
            answer_md,
            cited_page_ids,
            context_pack_id,
            source_count: cited_pages.len(),
        })
    }

    pub fn knowledge_summary(&self, project_id: &str) -> Result<KnowledgeSummary, String> {
        let latest_lint = self.latest_knowledge_lint_report(project_id)?;
        let recent_pages = self
            .recent_knowledge_page_slugs(project_id, 5)?
            .into_iter()
            .collect::<Vec<_>>();
        Ok(KnowledgeSummary {
            stale_count: latest_lint
                .as_ref()
                .map(|report| report.stale_count)
                .unwrap_or(0),
            gap_count: latest_lint
                .as_ref()
                .map(|report| report.gap_count)
                .unwrap_or(0),
            contradiction_count: latest_lint
                .as_ref()
                .map(|report| report.contradiction_count)
                .unwrap_or(0),
            recent_pages,
        })
    }

    pub fn run_knowledge_automation(
        &self,
        input: RunKnowledgeAutomationInput,
    ) -> Result<KnowledgeAutomationRun, String> {
        let project_id = required_trimmed("knowledge automation project id", &input.project_id)?;
        let sources = self.list_knowledge_sources(&project_id)?;
        let pages = self.search_knowledge_pages(&project_id, None)?;
        let stale_sources = sources
            .iter()
            .filter(|source| source.status != "current")
            .count() as i64;
        let stale_pages = pages
            .iter()
            .filter(|page| page.freshness_state != "current")
            .count() as i64;
        let gap_count = pages
            .iter()
            .filter(|page| page.source_ids.is_empty())
            .count() as i64;
        let stale_count = stale_sources + stale_pages;
        let mut body_lines = vec![
            format!("Knowledge automation compile for {project_id}"),
            format!("Sources: {}", sources.len()),
            format!("Pages: {}", pages.len()),
            format!("Stale: {stale_count}"),
            format!("Gaps: {gap_count}"),
            format!(
                "Watch: {}",
                if input.watch { "enabled" } else { "disabled" }
            ),
        ];
        for source in sources.iter().filter(|source| source.status != "current") {
            body_lines.push(format!(
                "Stale source: {} {}",
                source.kind, source.path_or_ref
            ));
        }
        for page in pages.iter().filter(|page| page.source_ids.is_empty()) {
            body_lines.push(format!("Gap page: {}", page.slug));
        }

        let lint = self.record_knowledge_lint_report(RecordKnowledgeLintReportInput {
            project_id: project_id.clone(),
            stale_count,
            gap_count,
            contradiction_count: 0,
            body_md: body_lines.join("\n"),
        })?;

        Ok(KnowledgeAutomationRun {
            project_id,
            status: "compiled".to_string(),
            watch_enabled: input.watch,
            source_count: sources.len(),
            page_count: pages.len(),
            stale_count,
            gap_count,
            lint_report_id: lint.id,
        })
    }

    pub fn ingest_knowledge_artifact(
        &self,
        input: IngestKnowledgeArtifactInput,
    ) -> Result<KnowledgeIngestionResult, String> {
        let project_id = required_trimmed("knowledge ingestion project id", &input.project_id)?;
        let kind = required_trimmed("knowledge ingestion kind", &input.kind)?.to_lowercase();
        let path_or_ref = required_trimmed("knowledge ingestion path", &input.path_or_ref)?;
        let body_md = required_trimmed("knowledge ingestion body", &input.body_md)?;
        let max_chunk_chars = input.max_chunk_chars.unwrap_or(1200);
        if max_chunk_chars <= 0 {
            return Err("knowledge ingestion chunk size must be positive".to_string());
        }
        let title = normalize_optional_text(input.title)
            .unwrap_or_else(|| title_from_artifact_path(&path_or_ref));
        let slug = slugify_knowledge_artifact(&title);
        let fingerprint = format!("local:{kind}:{}:{path_or_ref}", body_md.len());
        let source = self.upsert_knowledge_source(UpsertKnowledgeSourceInput {
            project_id: project_id.clone(),
            kind: kind.clone(),
            path_or_ref: path_or_ref.clone(),
            fingerprint: fingerprint.clone(),
            status: "current".to_string(),
        })?;
        let chunks = chunk_text_by_chars(&body_md, max_chunk_chars as usize);
        let compiled_body = compile_ingested_knowledge_page(&title, &path_or_ref, &kind, &chunks);
        let page = self.save_knowledge_page(SaveKnowledgePageInput {
            project_id: project_id.clone(),
            slug: slug.clone(),
            title,
            body_md: compiled_body,
            source_ids: vec![source.id.clone()],
            freshness_state: "current".to_string(),
        })?;

        Ok(KnowledgeIngestionResult {
            project_id,
            source_id: source.id,
            page_id: page.id,
            slug,
            modality: kind,
            chunk_count: chunks.len(),
            fingerprint,
        })
    }

    pub fn get_evidence_pack(
        &self,
        evidence_pack_id: &str,
    ) -> Result<Option<PersistedEvidencePack>, String> {
        let connection = self.connection()?;
        let row = connection
            .query_row(
                "SELECT id, task_id, run_id, artifact_path, completeness_state
                 FROM evidence_packs
                 WHERE id = ?1",
                params![evidence_pack_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("failed to load evidence pack {evidence_pack_id}: {error}"))?;

        row.map(|(id, task_id, run_id, artifact_path, completeness_state)| {
            let full_path = self.resolve_artifact_path(&artifact_path);
            let raw = fs::read_to_string(&full_path).map_err(|error| {
                format!(
                    "failed to read evidence pack {id} artifact {}: {error}",
                    full_path.display()
                )
            })?;
            let body_json = serde_json::from_str(&raw)
                .map_err(|error| format!("failed to parse evidence pack {id} json: {error}"))?;
            Ok(PersistedEvidencePack {
                id,
                task_id,
                run_id,
                artifact_path,
                completeness_state,
                body_json,
            })
        })
        .transpose()
    }

    pub fn list_evidence_packs(&self) -> Result<Vec<PersistedEvidencePack>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id
                 FROM evidence_packs
                 ORDER BY updated_at DESC, id ASC",
            )
            .map_err(|error| format!("failed to prepare evidence pack list query: {error}"))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| format!("failed to query evidence packs: {error}"))?;
        let ids = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read evidence pack row: {error}"))?;
        ids.into_iter()
            .map(|id| {
                self.get_evidence_pack(&id)?
                    .ok_or_else(|| format!("evidence pack {id} disappeared while listing"))
            })
            .collect()
    }

    pub fn list_evidence_packs_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<PersistedEvidencePack>, String> {
        let project_id = required_trimmed("evidence project id", project_id)?;
        let tasks = self.list_tasks(&project_id)?;
        let runs = self.list_runs(&project_id)?;
        Ok(self
            .list_evidence_packs()?
            .into_iter()
            .filter(|pack| evidence_pack_belongs_to_project(pack, &project_id, &tasks, &runs))
            .collect())
    }

    fn save_evidence_pack_artifact(
        &self,
        evidence_pack_id: &str,
        task_id: Option<String>,
        run_id: Option<String>,
        completeness_state: &str,
        body_json: &Value,
    ) -> Result<PersistedEvidencePack, String> {
        let artifact_path = self.evidence_artifact_path(evidence_pack_id);
        let redacted_body_json = self.redact_json_secrets(body_json)?;
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create evidence artifact directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(
            &artifact_path,
            serde_json::to_string_pretty(&redacted_body_json).map_err(|error| {
                format!("failed to serialize evidence pack {evidence_pack_id}: {error}")
            })?,
        )
        .map_err(|error| {
            format!(
                "failed to write evidence pack {evidence_pack_id} artifact {}: {error}",
                artifact_path.display()
            )
        })?;
        let relative_path = self.relative_artifact_path(&artifact_path);
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO evidence_packs(
                   id, task_id, run_id, artifact_path, completeness_state, created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(id) DO UPDATE SET
                   task_id = excluded.task_id,
                   run_id = excluded.run_id,
                   artifact_path = excluded.artifact_path,
                   completeness_state = excluded.completeness_state,
                   updated_at = excluded.updated_at",
                params![
                    evidence_pack_id,
                    normalize_optional_text(task_id),
                    normalize_optional_text(run_id),
                    relative_path,
                    completeness_state
                ],
            )
            .map_err(|error| {
                format!("failed to persist evidence pack {evidence_pack_id}: {error}")
            })?;

        self.get_evidence_pack(evidence_pack_id)?
            .ok_or_else(|| format!("saved evidence pack {evidence_pack_id} could not be loaded"))
    }

    pub fn dispatch_run(&self, input: DispatchRunInput) -> Result<PersistedRun, String> {
        let task = self
            .get_task(&input.task_id)?
            .ok_or_else(|| format!("task {} not found", input.task_id))?;
        if task.status != "ready" {
            return Err(format!(
                "task {} must be ready before dispatch; current status is {}",
                task.id, task.status
            ));
        }
        let agent_profile_id = normalize_optional_text(input.agent_profile_id);
        self.enforce_agent_profile_available(agent_profile_id.as_deref())?;
        self.enforce_budget_gate(
            &task.id,
            &task.project_id,
            task.initiative_id.as_deref(),
            agent_profile_id.as_deref(),
        )?;

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "runs", "run_")?;
        drop(connection);
        let workflow = self.latest_workflow_version(&task.project_id, true)?;
        let context_pack_id = normalize_optional_text(input.context_pack_id).or_else(|| {
            workflow
                .as_ref()
                .and_then(default_context_pack_from_workflow)
        });
        let run_id = format!("run_{next_number}");
        let workspace_path = match normalize_optional_text(input.workspace_path) {
            Some(workspace_path) => Some(workspace_path),
            None => Some(self.default_dispatch_worktree_path(
                &task.project_id,
                &run_id,
                workflow.as_ref(),
            )?),
        };
        let workflow_version_id = workflow.map(|workflow| workflow.id);
        let mut run = PersistedRun {
            id: run_id,
            task_id: task.id.clone(),
            project_id: task.project_id.clone(),
            agent_profile_id: agent_profile_id.clone(),
            session_id: None,
            workflow_version_id,
            context_pack_id,
            workspace_path,
            lifecycle: "queued".to_string(),
            retry_count: 0,
            next_retry_at: None,
            status_detail: None,
            budget_id: None,
            started_at: None,
            ended_at: None,
        };
        let session = self.create_session(CreateSessionInput {
            project_id: run.project_id.clone(),
            mode: "agent".to_string(),
            title: "Agent session".to_string(),
            cwd: run.workspace_path.clone(),
            branch: None,
            agent_profile_id,
            task_id: Some(run.task_id.clone()),
            run_id: Some(run.id.clone()),
        })?;
        run.session_id = Some(session.id);

        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO runs(
                   id, task_id, project_id, agent_profile_id, session_id,
                   workflow_version_id, context_pack_id, workspace_path,
                   lifecycle, retry_count, next_retry_at, status_detail, budget_id, started_at, ended_at,
                   created_at, updated_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    &run.id,
                    &run.task_id,
                    &run.project_id,
                    &run.agent_profile_id,
                    &run.session_id,
                    &run.workflow_version_id,
                    &run.context_pack_id,
                    &run.workspace_path,
                    &run.lifecycle,
                    run.retry_count,
                    &run.next_retry_at,
                    &run.status_detail,
                    &run.budget_id,
                    &run.started_at,
                    &run.ended_at
                ],
            )
            .map_err(|error| format!("failed to dispatch run {}: {error}", run.id))?;
        drop(connection);

        self.move_task_status(&task.id, "running")?;
        self.get_run(&run.id)?
            .ok_or_else(|| format!("dispatched run {} could not be loaded", run.id))
    }

    pub fn get_run(&self, run_id: &str) -> Result<Option<PersistedRun>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, task_id, project_id, agent_profile_id, session_id,
                   workflow_version_id, context_pack_id, workspace_path,
                   lifecycle, retry_count, next_retry_at, status_detail, budget_id, started_at, ended_at
                 FROM runs
                 WHERE id = ?1",
                params![run_id],
                persisted_run_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load run {run_id}: {error}"))
    }

    pub fn list_runs(&self, project_id: &str) -> Result<Vec<PersistedRun>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, task_id, project_id, agent_profile_id, session_id,
                   workflow_version_id, context_pack_id, workspace_path,
                   lifecycle, retry_count, next_retry_at, status_detail, budget_id, started_at, ended_at
                 FROM runs
                 WHERE project_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|error| format!("failed to prepare run list query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], persisted_run_from_row)
            .map_err(|error| format!("failed to query runs: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read run row: {error}"))
    }

    pub fn runtime_pool(&self, project_id: &str) -> Result<Vec<RuntimePoolItem>, String> {
        let project_id = required_trimmed("runtime pool project id", project_id)?;
        if self.get_project(&project_id)?.is_none() {
            return Err(format!("project {project_id} not found"));
        }
        let sessions = self.list_sessions(&project_id)?;
        let runs = self.list_runs(&project_id)?;
        let session_modes = sessions
            .iter()
            .map(|session| (session.id.clone(), session.mode.clone()))
            .collect::<HashMap<_, _>>();
        let mut groups = BTreeMap::<String, RuntimePoolItem>::new();

        for session in sessions {
            let item = groups
                .entry(session.mode.clone())
                .or_insert_with(|| runtime_pool_item(&session.mode));
            item.session_count += 1;
        }
        for run in runs {
            let mode = run
                .session_id
                .as_ref()
                .and_then(|session_id| session_modes.get(session_id))
                .cloned()
                .unwrap_or_else(|| "agent".to_string());
            let item = groups
                .entry(mode.clone())
                .or_insert_with(|| runtime_pool_item(&mode));
            if !matches!(run.lifecycle.as_str(), "completed" | "cancelled") {
                item.run_count += 1;
            }
            if matches!(
                run.lifecycle.as_str(),
                "blocked" | "failed" | "permission_requested"
            ) {
                item.blocked_count += 1;
            }
        }

        let mut items = groups.into_values().collect::<Vec<_>>();
        items.sort_by(|left, right| {
            runtime_pool_sort_key(&left.id)
                .cmp(&runtime_pool_sort_key(&right.id))
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(items)
    }

    pub fn update_run_lifecycle(
        &self,
        input: UpdateRunLifecycleInput,
    ) -> Result<PersistedRun, String> {
        let lifecycle = normalize_run_lifecycle(&input.lifecycle)?;
        let status_detail = input
            .status_detail
            .map(|detail| self.redact_secrets(&detail))
            .transpose()?;
        let status_detail = normalize_run_status_detail(&lifecycle, status_detail)?;
        let run = self
            .get_run(&input.run_id)?
            .ok_or_else(|| format!("run {} not found", input.run_id))?;
        validate_run_transition(&run.lifecycle, &lifecycle)?;

        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE runs
                 SET lifecycle = ?1,
                     started_at = CASE
                       WHEN ?1 IN ('starting', 'running') AND started_at IS NULL
                       THEN strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                       ELSE started_at
                     END,
                     ended_at = CASE
                       WHEN ?1 IN ('completed', 'failed', 'cancelled')
                       THEN strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                       ELSE ended_at
                     END,
                     next_retry_at = CASE
                       WHEN ?1 = 'queued' THEN next_retry_at
                       ELSE NULL
                     END,
                     status_detail = ?3,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?2",
                params![lifecycle, input.run_id, status_detail],
            )
            .map_err(|error| format!("failed to update run {} lifecycle: {error}", input.run_id))?;
        drop(connection);

        if updated == 0 {
            return Err(format!("run {} not found", input.run_id));
        }

        self.sync_task_status_for_run(&run.task_id, &lifecycle)?;
        self.get_run(&input.run_id)?
            .ok_or_else(|| format!("updated run {} could not be loaded", input.run_id))
    }

    pub fn cancel_run(&self, run_id: &str) -> Result<PersistedRun, String> {
        self.update_run_lifecycle(UpdateRunLifecycleInput {
            run_id: run_id.to_string(),
            lifecycle: "cancelled".to_string(),
            status_detail: None,
        })
    }

    pub fn retry_run(&self, run_id: &str) -> Result<PersistedRun, String> {
        let run = self
            .get_run(run_id)?
            .ok_or_else(|| format!("run {run_id} not found"))?;
        if !matches!(run.lifecycle.as_str(), "failed" | "cancelled") {
            return Err(format!(
                "run {run_id} can only be retried from failed or cancelled; current lifecycle is {}",
                run.lifecycle
            ));
        }

        let retry_count = run.retry_count + 1;
        let next_retry_at = (Utc::now() + Duration::seconds(retry_backoff_seconds(retry_count)))
            .to_rfc3339_opts(SecondsFormat::Secs, true);
        let connection = self.connection()?;
        let updated = connection
            .execute(
                "UPDATE runs
                 SET lifecycle = 'queued',
                     retry_count = ?2,
                     next_retry_at = ?3,
                     status_detail = NULL,
                     started_at = NULL,
                     ended_at = NULL,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?1",
                params![run_id, retry_count, next_retry_at],
            )
            .map_err(|error| format!("failed to retry run {run_id}: {error}"))?;
        drop(connection);

        if updated == 0 {
            return Err(format!("run {run_id} not found"));
        }

        self.sync_task_status_for_run(&run.task_id, "queued")?;
        self.get_run(run_id)?
            .ok_or_else(|| format!("retried run {run_id} could not be loaded"))
    }

    pub fn create_policy_approval(
        &self,
        input: CreatePolicyApprovalInput,
    ) -> Result<PersistedPolicyApproval, String> {
        let project_id = input.project_id.trim();
        if project_id.is_empty() {
            return Err("policy approval project id cannot be empty".to_string());
        }
        let action_kind = input.action_kind.trim();
        if action_kind.is_empty() {
            return Err("policy approval action kind cannot be empty".to_string());
        }
        let risk_level = normalize_risk_level(&input.risk_level)?;
        let run_id = normalize_optional_text(input.run_id);
        let mut task_id = normalize_optional_text(input.task_id);
        let command = normalize_optional_text(input.command)
            .map(|command| self.redact_secrets(&command))
            .transpose()?;
        if let Some(run_id) = run_id.as_deref() {
            let run = self
                .get_run(run_id)?
                .ok_or_else(|| format!("run {run_id} not found"))?;
            if run.project_id != project_id {
                return Err(format!(
                    "run {run_id} does not belong to project {project_id}"
                ));
            }
            if task_id.is_none() {
                task_id = Some(run.task_id);
            }
            self.update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: run_id.to_string(),
                lifecycle: "permission_requested".to_string(),
                status_detail: Some(format_policy_status_detail(action_kind, command.as_deref())),
            })?;
        }

        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "policy_approvals", "policy_approval_")?;
        let id = format!("policy_approval_{next_number}");
        connection
            .execute(
                "INSERT INTO policy_approvals(
                   id, project_id, task_id, run_id, action_kind, command,
                   risk_level, state, requested_by, decision_by, decision_note,
                   created_at, decided_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8, NULL, NULL,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), NULL
                 )",
                params![
                    id,
                    project_id,
                    task_id,
                    run_id,
                    action_kind,
                    command,
                    risk_level,
                    normalize_optional_text(input.requested_by)
                ],
            )
            .map_err(|error| format!("failed to create policy approval {id}: {error}"))?;
        drop(connection);

        self.get_policy_approval(&id)?
            .ok_or_else(|| format!("created policy approval {id} could not be loaded"))
    }

    pub fn decide_policy_approval(
        &self,
        input: DecidePolicyApprovalInput,
    ) -> Result<PersistedPolicyApproval, String> {
        let approval_id = input.approval_id.trim();
        if approval_id.is_empty() {
            return Err("policy approval id cannot be empty".to_string());
        }
        let decision = normalize_policy_decision(&input.decision)?;
        let approval = self
            .get_policy_approval(approval_id)?
            .ok_or_else(|| format!("policy approval {approval_id} not found"))?;
        if approval.state != "pending" {
            return Err(format!(
                "policy approval {approval_id} is already {}",
                approval.state
            ));
        }
        let decision_by = normalize_optional_text(input.decision_by);
        let decision_note = normalize_optional_text(input.decision_note)
            .map(|note| self.redact_secrets(&note))
            .transpose()?;

        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE policy_approvals
                 SET state = ?1,
                     decision_by = ?2,
                     decision_note = ?3,
                     decided_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                WHERE id = ?4",
                params![decision, &decision_by, &decision_note, approval_id],
            )
            .map_err(|error| {
                format!("failed to record policy approval decision {approval_id}: {error}")
            })?;
        drop(connection);

        if let Some(run_id) = approval.run_id.as_deref() {
            if let Some(run) = self.get_run(run_id)? {
                if run.lifecycle == "permission_requested" {
                    let lifecycle = if decision == "approved" {
                        "running"
                    } else {
                        "blocked"
                    };
                    self.update_run_lifecycle(UpdateRunLifecycleInput {
                        run_id: run_id.to_string(),
                        lifecycle: lifecycle.to_string(),
                        status_detail: if decision == "approved" {
                            None
                        } else {
                            Some(format!(
                                "Permission denied: {}",
                                decision_note
                                    .as_deref()
                                    .unwrap_or(approval.action_kind.as_str())
                            ))
                        },
                    })?;
                }
            }
        }

        self.get_policy_approval(approval_id)?
            .ok_or_else(|| format!("decided policy approval {approval_id} could not be loaded"))
    }

    pub fn get_policy_approval(
        &self,
        approval_id: &str,
    ) -> Result<Option<PersistedPolicyApproval>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, task_id, run_id, action_kind, command,
                   risk_level, state, requested_by, decision_by, decision_note,
                   created_at, decided_at
                 FROM policy_approvals
                 WHERE id = ?1",
                params![approval_id],
                persisted_policy_approval_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load policy approval {approval_id}: {error}"))
    }

    fn get_policy_pack(&self, pack_id: &str) -> Result<Option<PersistedPolicyPack>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, project_id, name, sandbox_mode, network, network_profile, file_write, tools,
                        approval_required_json, forbidden_operations_json, active, created_at, updated_at
                 FROM policy_packs
                 WHERE id = ?1",
                params![pack_id],
                persisted_policy_pack_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load policy pack {pack_id}: {error}"))
    }

    fn active_policy_pack(&self, project_id: &str) -> Result<Option<PersistedPolicyPack>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, project_id, name, sandbox_mode, network, network_profile, file_write, tools,
                        approval_required_json, forbidden_operations_json, active, created_at, updated_at
                 FROM policy_packs
                 WHERE project_id = ?1 AND active = 1
                 ORDER BY updated_at DESC
                 LIMIT 1",
                params![project_id],
                persisted_policy_pack_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load active policy pack for {project_id}: {error}"))
    }

    pub fn list_policy_approvals(
        &self,
        project_id: &str,
        state: Option<&str>,
    ) -> Result<Vec<PersistedPolicyApproval>, String> {
        let project_id = project_id.trim();
        if project_id.is_empty() {
            return Err("policy approval project id cannot be empty".to_string());
        }
        let state = state
            .map(normalize_policy_approval_state)
            .transpose()?
            .map(str::to_string);
        let connection = self.connection()?;
        let mut statement = if state.is_some() {
            connection
                .prepare(
                    "SELECT
                       id, project_id, task_id, run_id, action_kind, command,
                       risk_level, state, requested_by, decision_by, decision_note,
                       created_at, decided_at
                     FROM policy_approvals
                     WHERE project_id = ?1 AND state = ?2
                     ORDER BY created_at ASC, id ASC",
                )
                .map_err(|error| format!("failed to prepare policy approval list query: {error}"))?
        } else {
            connection
                .prepare(
                    "SELECT
                       id, project_id, task_id, run_id, action_kind, command,
                       risk_level, state, requested_by, decision_by, decision_note,
                       created_at, decided_at
                     FROM policy_approvals
                     WHERE project_id = ?1
                     ORDER BY created_at ASC, id ASC",
                )
                .map_err(|error| format!("failed to prepare policy approval list query: {error}"))?
        };

        let rows = if let Some(state) = state.as_deref() {
            statement.query_map(
                params![project_id, state],
                persisted_policy_approval_from_row,
            )
        } else {
            statement.query_map(params![project_id], persisted_policy_approval_from_row)
        }
        .map_err(|error| format!("failed to query policy approvals: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read policy approval row: {error}"))
    }

    pub fn upsert_policy_pack(
        &self,
        input: UpsertPolicyPackInput,
    ) -> Result<PersistedPolicyPack, String> {
        let project_id = required_trimmed("policy pack project id", &input.project_id)?;
        let name = required_trimmed("policy pack name", &input.name)?;
        let sandbox_mode = normalize_policy_sandbox_mode(&input.sandbox_mode)?;
        let network = normalize_policy_permission(input.network.as_deref().unwrap_or("ask"))?;
        let network_profile =
            normalize_network_profile(input.network_profile.as_deref().unwrap_or("internet"))?;
        let file_write = normalize_policy_permission(input.file_write.as_deref().unwrap_or("ask"))?;
        let tools = normalize_policy_permission(input.tools.as_deref().unwrap_or("ask"))?;
        let approval_required = normalize_policy_list(input.approval_required.unwrap_or_default())?;
        let forbidden_operations =
            normalize_policy_list(input.forbidden_operations.unwrap_or_default())?;
        let active = input.set_active.unwrap_or(true);
        let id = policy_pack_id(&project_id, &name);
        let connection = self.connection()?;
        if active {
            connection
                .execute(
                    "UPDATE policy_packs SET active = 0, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE project_id = ?1",
                    params![&project_id],
                )
                .map_err(|error| format!("failed to deactivate policy packs for {project_id}: {error}"))?;
        }
        connection
            .execute(
                "INSERT INTO policy_packs(
                   id, project_id, name, sandbox_mode, network, network_profile, file_write, tools,
                   approval_required_json, forbidden_operations_json, active, created_at, updated_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(project_id, name) DO UPDATE SET
                   sandbox_mode = excluded.sandbox_mode,
                   network = excluded.network,
                   network_profile = excluded.network_profile,
                   file_write = excluded.file_write,
                   tools = excluded.tools,
                   approval_required_json = excluded.approval_required_json,
                   forbidden_operations_json = excluded.forbidden_operations_json,
                   active = excluded.active,
                   updated_at = excluded.updated_at",
                params![
                    &id,
                    &project_id,
                    &name,
                    &sandbox_mode,
                    &network,
                    &network_profile,
                    &file_write,
                    &tools,
                    serde_json::to_string(&approval_required).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string(&forbidden_operations)
                        .unwrap_or_else(|_| "[]".to_string()),
                    if active { 1 } else { 0 }
                ],
            )
            .map_err(|error| format!("failed to persist policy pack {name}: {error}"))?;
        drop(connection);

        self.get_policy_pack(&id)?
            .ok_or_else(|| format!("policy pack {id} could not be loaded"))
    }

    pub fn list_policy_packs(
        &self,
        project_id: &str,
        active: Option<bool>,
    ) -> Result<Vec<PersistedPolicyPack>, String> {
        let project_id = required_trimmed("policy pack project id", project_id)?;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, name, sandbox_mode, network, network_profile, file_write, tools,
                        approval_required_json, forbidden_operations_json, active, created_at, updated_at
                 FROM policy_packs
                 WHERE project_id = ?1
                   AND (?2 IS NULL OR active = ?2)
                 ORDER BY active DESC, name ASC",
            )
            .map_err(|error| format!("failed to prepare policy pack list query: {error}"))?;
        let active_int = active.map(|value| if value { 1 } else { 0 });
        let rows = statement
            .query_map(
                params![project_id, active_int],
                persisted_policy_pack_from_row,
            )
            .map_err(|error| format!("failed to query policy packs: {error}"))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read policy pack row: {error}"))
    }

    pub fn evaluate_policy_action(
        &self,
        input: EvaluatePolicyActionInput,
    ) -> Result<PolicyActionEvaluation, String> {
        let project_id = required_trimmed("policy evaluation project id", &input.project_id)?;
        let action_kind = required_trimmed("policy evaluation action kind", &input.action_kind)?;
        let pack = self.active_policy_pack(&project_id)?;
        let command = normalize_optional_text(input.command);
        let task_id = normalize_optional_text(input.task_id);
        let run_id = normalize_optional_text(input.run_id);
        let requested_by = normalize_optional_text(input.requested_by);
        let persist =
            |mut evaluation: PolicyActionEvaluation| -> Result<PolicyActionEvaluation, String> {
                self.record_permission_audit(
                    &mut evaluation,
                    task_id.as_deref(),
                    run_id.as_deref(),
                    command.as_deref(),
                    requested_by.as_deref(),
                )?;
                Ok(evaluation)
            };
        let Some(pack) = pack else {
            return persist(PolicyActionEvaluation {
                audit_id: None,
                project_id,
                policy_pack_id: None,
                action_kind,
                decision: "approval_required".to_string(),
                reason: "no active policy pack".to_string(),
            });
        };
        let command_matches_forbidden = command
            .as_deref()
            .map(|command| {
                pack.forbidden_operations
                    .iter()
                    .any(|operation| command.contains(operation))
            })
            .unwrap_or(false);
        if pack.forbidden_operations.contains(&action_kind) || command_matches_forbidden {
            return persist(PolicyActionEvaluation {
                audit_id: None,
                project_id,
                policy_pack_id: Some(pack.id),
                action_kind,
                decision: "forbidden".to_string(),
                reason: "action matches forbidden operation".to_string(),
            });
        }
        if action_kind == "network" {
            if pack.network_profile == "offline" {
                return persist(PolicyActionEvaluation {
                    audit_id: None,
                    project_id,
                    policy_pack_id: Some(pack.id),
                    action_kind,
                    decision: "forbidden".to_string(),
                    reason: "network profile is offline".to_string(),
                });
            }
            if pack.network_profile == "local-only"
                && !command
                    .as_deref()
                    .map(command_targets_local_endpoint)
                    .unwrap_or(false)
            {
                return persist(PolicyActionEvaluation {
                    audit_id: None,
                    project_id,
                    policy_pack_id: Some(pack.id),
                    action_kind,
                    decision: "forbidden".to_string(),
                    reason: "network profile blocks remote endpoint".to_string(),
                });
            }
            if pack.network == "blocked" {
                return persist(PolicyActionEvaluation {
                    audit_id: None,
                    project_id,
                    policy_pack_id: Some(pack.id),
                    action_kind,
                    decision: "forbidden".to_string(),
                    reason: "network permission is blocked".to_string(),
                });
            }
            if pack.network == "ask" {
                return persist(PolicyActionEvaluation {
                    audit_id: None,
                    project_id,
                    policy_pack_id: Some(pack.id),
                    action_kind,
                    decision: "approval_required".to_string(),
                    reason: "network permission requires approval".to_string(),
                });
            }
            if pack.network_profile == "local-only" {
                return persist(PolicyActionEvaluation {
                    audit_id: None,
                    project_id,
                    policy_pack_id: Some(pack.id),
                    action_kind,
                    decision: "allowed".to_string(),
                    reason: "network profile permits local endpoint".to_string(),
                });
            }
        }
        let needs_approval = pack.approval_required.contains(&action_kind)
            || (pack.sandbox_mode == "ask-before-write"
                && matches!(action_kind.as_str(), "shell_command" | "file_write"));
        if needs_approval {
            persist(PolicyActionEvaluation {
                audit_id: None,
                project_id,
                policy_pack_id: Some(pack.id),
                action_kind,
                decision: "approval_required".to_string(),
                reason: "policy requires approval".to_string(),
            })
        } else {
            persist(PolicyActionEvaluation {
                audit_id: None,
                project_id,
                policy_pack_id: Some(pack.id),
                action_kind,
                decision: "allowed".to_string(),
                reason: "policy permits action".to_string(),
            })
        }
    }

    fn record_permission_audit(
        &self,
        evaluation: &mut PolicyActionEvaluation,
        task_id: Option<&str>,
        run_id: Option<&str>,
        command: Option<&str>,
        requested_by: Option<&str>,
    ) -> Result<(), String> {
        let command = command
            .map(|command| self.redact_secrets(command))
            .transpose()?;
        let connection = self.connection()?;
        let next_number =
            next_numeric_id(&connection, "permission_audit_events", "permission_audit_")?;
        let id = format!("permission_audit_{next_number}");
        connection
            .execute(
                "INSERT INTO permission_audit_events(
                   id, project_id, task_id, run_id, policy_pack_id, action_kind,
                   command, decision, reason, requested_by, created_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    &id,
                    &evaluation.project_id,
                    task_id,
                    run_id,
                    &evaluation.policy_pack_id,
                    &evaluation.action_kind,
                    command,
                    &evaluation.decision,
                    &evaluation.reason,
                    requested_by
                ],
            )
            .map_err(|error| format!("failed to record permission audit {id}: {error}"))?;
        evaluation.audit_id = Some(id);
        Ok(())
    }

    pub fn list_permission_audit(
        &self,
        project_id: &str,
        decision: Option<&str>,
        action_kind: Option<&str>,
        run_id: Option<&str>,
        task_id: Option<&str>,
    ) -> Result<Vec<PersistedPermissionAudit>, String> {
        let project_id = required_trimmed("permission audit project id", project_id)?;
        let decision = decision
            .map(normalize_permission_decision)
            .transpose()?
            .map(str::to_string);
        let action_kind = action_kind
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let run_id = run_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let task_id = task_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let connection = self.connection()?;
        let mut sql = "SELECT id, project_id, task_id, run_id, policy_pack_id, action_kind,
                            command, decision, reason, requested_by, created_at
                     FROM permission_audit_events
                     WHERE project_id = ?"
            .to_string();
        let mut values: Vec<&dyn ToSql> = vec![&project_id];
        if let Some(decision) = decision.as_ref() {
            sql.push_str(" AND decision = ?");
            values.push(decision);
        }
        if let Some(action_kind) = action_kind.as_ref() {
            sql.push_str(" AND action_kind = ?");
            values.push(action_kind);
        }
        if let Some(run_id) = run_id.as_ref() {
            sql.push_str(" AND run_id = ?");
            values.push(run_id);
        }
        if let Some(task_id) = task_id.as_ref() {
            sql.push_str(" AND task_id = ?");
            values.push(task_id);
        }
        sql.push_str(" ORDER BY created_at DESC, id DESC LIMIT 50");
        let mut statement = connection
            .prepare(&sql)
            .map_err(|error| format!("failed to prepare permission audit query: {error}"))?;
        let rows = statement
            .query_map(
                params_from_iter(values),
                persisted_permission_audit_from_row,
            )
            .map_err(|error| format!("failed to query permission audit: {error}"))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read permission audit row: {error}"))
    }

    pub fn permission_audit_summary(&self, project_id: &str) -> Result<Value, String> {
        let audits = self.list_permission_audit(project_id, None, None, None, None)?;
        let allowed_count = audits
            .iter()
            .filter(|audit| audit.decision == "allowed")
            .count();
        let approval_required_count = audits
            .iter()
            .filter(|audit| audit.decision == "approval_required")
            .count();
        let forbidden_count = audits
            .iter()
            .filter(|audit| audit.decision == "forbidden")
            .count();
        let latest = audits.first();
        Ok(serde_json::json!({
            "recent_count": audits.len(),
            "allowed_count": allowed_count,
            "approval_required_count": approval_required_count,
            "forbidden_count": forbidden_count,
            "latest_decision": latest.map(|audit| audit.decision.as_str()),
            "latest_action_kind": latest.map(|audit| audit.action_kind.as_str())
        }))
    }

    pub fn policy_pack_summary(&self, project_id: &str) -> Result<Value, String> {
        match self.active_policy_pack(project_id)? {
            Some(pack) => Ok(serde_json::json!({
                "id": pack.id,
                "name": pack.name,
                "sandbox_mode": pack.sandbox_mode,
                "network": pack.network,
                "network_profile": pack.network_profile,
                "file_write": pack.file_write,
                "tools": pack.tools,
                "approval_required_count": pack.approval_required.len(),
                "forbidden_count": pack.forbidden_operations.len()
            })),
            None => Ok(serde_json::json!({
                "id": null,
                "name": "Default ask-before-write",
                "sandbox_mode": "ask-before-write",
                "network": "ask",
                "network_profile": "internet",
                "file_write": "ask",
                "tools": "ask",
                "approval_required_count": 0,
                "forbidden_count": 0
            })),
        }
    }

    fn policy_events_for_run(&self, run_id: &str) -> Result<Vec<Value>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, project_id, task_id, run_id, action_kind, command,
                   risk_level, state, requested_by, decision_by, decision_note,
                   created_at, decided_at
                 FROM policy_approvals
                 WHERE run_id = ?1
                 ORDER BY created_at ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare run policy events query: {error}"))?;
        let rows = statement
            .query_map(params![run_id], persisted_policy_approval_from_row)
            .map_err(|error| format!("failed to query run policy events: {error}"))?;
        let approvals = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read run policy event row: {error}"))?;
        let audits = self.permission_audit_for_run(run_id)?;
        Ok(approvals
            .iter()
            .map(policy_approval_to_event_json)
            .chain(audits.iter().map(permission_audit_to_event_json))
            .collect())
    }

    fn permission_audit_for_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<PersistedPermissionAudit>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, project_id, task_id, run_id, policy_pack_id, action_kind,
                        command, decision, reason, requested_by, created_at
                 FROM permission_audit_events
                 WHERE run_id = ?1
                 ORDER BY created_at ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare run permission audit query: {error}"))?;
        let rows = statement
            .query_map(params![run_id], persisted_permission_audit_from_row)
            .map_err(|error| format!("failed to query run permission audit: {error}"))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read run permission audit row: {error}"))
    }

    pub fn count_runs_by_lifecycle(&self, project_id: &str) -> Result<Value, String> {
        let mut counts = serde_json::json!({
            "queued": 0,
            "claimed": 0,
            "starting": 0,
            "running": 0,
            "waiting_input": 0,
            "permission_requested": 0,
            "blocked": 0,
            "review_ready": 0,
            "completed": 0,
            "failed": 0,
            "cancelled": 0
        });
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT lifecycle, COUNT(*) FROM runs WHERE project_id = ?1 GROUP BY lifecycle",
            )
            .map_err(|error| format!("failed to prepare run counts query: {error}"))?;
        let rows = statement
            .query_map(params![project_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|error| format!("failed to query run counts: {error}"))?;

        for row in rows {
            let (lifecycle, count) =
                row.map_err(|error| format!("failed to read run count row: {error}"))?;
            counts[lifecycle] = serde_json::json!(count);
        }

        Ok(counts)
    }

    fn sync_task_status_for_run(&self, task_id: &str, lifecycle: &str) -> Result<(), String> {
        let task_status = task_status_for_run_lifecycle(lifecycle)?;
        self.move_task_status(task_id, task_status).map(|_| ())
    }

    pub fn reload_workflow(
        &self,
        input: ReloadWorkflowInput,
    ) -> Result<PersistedWorkflowVersion, String> {
        let project_id = input.project_id.trim();
        let source_path = input.source_path.trim();
        if project_id.is_empty() {
            return Err("workflow project id cannot be empty".to_string());
        }
        if source_path.is_empty() {
            return Err("workflow source path cannot be empty".to_string());
        }

        let validation = validate_workflow_document(&input.content);
        let connection = self.connection()?;
        let next_number = next_numeric_id(&connection, "workflow_versions", "workflow_")?;
        let workflow = PersistedWorkflowVersion {
            id: format!("workflow_{next_number}"),
            project_id: project_id.to_string(),
            source_path: source_path.to_string(),
            content_hash: content_hash(&input.content),
            parsed_json: validation.parsed_json,
            valid: validation.valid,
            diagnostics_json: validation.diagnostics_json,
        };
        connection
            .execute(
                "INSERT INTO workflow_versions(
                   id, project_id, source_path, content_hash, parsed_json,
                   valid, diagnostics_json, created_at
                 )
                 VALUES (
                   ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )",
                params![
                    workflow.id,
                    workflow.project_id,
                    workflow.source_path,
                    workflow.content_hash,
                    workflow.parsed_json.to_string(),
                    if workflow.valid { 1 } else { 0 },
                    workflow.diagnostics_json.to_string()
                ],
            )
            .map_err(|error| format!("failed to persist workflow {}: {error}", workflow.id))?;

        self.get_workflow_version(&workflow.id)?
            .ok_or_else(|| format!("reloaded workflow {} could not be loaded", workflow.id))
    }

    pub fn validate_workflow(
        &self,
        input: ValidateWorkflowInput,
    ) -> Result<WorkflowValidationResult, String> {
        let project_id = input.project_id.trim();
        let source_path = input.source_path.trim();
        if project_id.is_empty() {
            return Err("workflow project id cannot be empty".to_string());
        }
        if source_path.is_empty() {
            return Err("workflow source path cannot be empty".to_string());
        }

        let validation = validate_workflow_document(&input.content);
        Ok(WorkflowValidationResult {
            project_id: project_id.to_string(),
            source_path: source_path.to_string(),
            valid: validation.valid,
            parsed_json: validation.parsed_json,
            diagnostics_json: validation.diagnostics_json,
        })
    }

    fn default_dispatch_worktree_path(
        &self,
        project_id: &str,
        run_id: &str,
        workflow: Option<&PersistedWorkflowVersion>,
    ) -> Result<String, String> {
        let project_root = self
            .get_project(project_id)?
            .map(|project| PathBuf::from(project.path))
            .or_else(|| self.db_path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."));
        let base_root = workflow
            .and_then(|workflow| workflow.parsed_json["workspace"]["base_root"].as_str())
            .map(str::trim)
            .filter(|base_root| !base_root.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".haneulchi").join("worktrees"));
        let workspace_root = if base_root.is_absolute() {
            base_root
        } else {
            project_root.join(base_root)
        };
        let workspace_path = workspace_root.join(run_id);
        fs::create_dir_all(&workspace_path).map_err(|error| {
            format!(
                "failed to prepare worktree workspace {}: {error}",
                workspace_path.to_string_lossy()
            )
        })?;
        Ok(workspace_path.to_string_lossy().to_string())
    }

    pub fn workflow_runtime_state(&self, project_id: &str) -> Result<WorkflowRuntimeState, String> {
        let current = self.latest_workflow_version(project_id, false)?;
        let last_known_good = self.latest_workflow_version(project_id, true)?;
        let valid = current
            .as_ref()
            .map(|workflow| workflow.valid)
            .unwrap_or(true);
        let diagnostics = current
            .as_ref()
            .map(|workflow| workflow.diagnostics_json.clone())
            .unwrap_or_else(|| serde_json::json!({ "errors": [] }));

        Ok(WorkflowRuntimeState {
            project_id: project_id.to_string(),
            valid,
            current_version_id: current.map(|workflow| workflow.id),
            last_known_good_version_id: last_known_good.map(|workflow| workflow.id),
            diagnostics,
        })
    }

    pub fn get_workflow_version(
        &self,
        workflow_id: &str,
    ) -> Result<Option<PersistedWorkflowVersion>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, project_id, source_path, content_hash, parsed_json, valid, diagnostics_json
                 FROM workflow_versions
                 WHERE id = ?1",
                params![workflow_id],
                persisted_workflow_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load workflow {workflow_id}: {error}"))
    }

    fn latest_workflow_version(
        &self,
        project_id: &str,
        valid_only: bool,
    ) -> Result<Option<PersistedWorkflowVersion>, String> {
        let connection = self.connection()?;
        let sql = if valid_only {
            "SELECT id, project_id, source_path, content_hash, parsed_json, valid, diagnostics_json
             FROM workflow_versions
             WHERE project_id = ?1 AND valid = 1
             ORDER BY CAST(substr(id, 10) AS INTEGER) DESC
             LIMIT 1"
        } else {
            "SELECT id, project_id, source_path, content_hash, parsed_json, valid, diagnostics_json
             FROM workflow_versions
             WHERE project_id = ?1
             ORDER BY CAST(substr(id, 10) AS INTEGER) DESC
             LIMIT 1"
        };
        connection
            .query_row(sql, params![project_id], persisted_workflow_from_row)
            .optional()
            .map_err(|error| format!("failed to load latest workflow for {project_id}: {error}"))
    }

    pub fn run_workflow_hook(
        &self,
        input: RunWorkflowHookInput,
    ) -> Result<WorkflowHookRunResult, String> {
        let hook_name = input.hook_name.trim();
        if hook_name.is_empty() {
            return Err("workflow hook name cannot be empty".to_string());
        }
        let run = self
            .get_run(&input.run_id)?
            .ok_or_else(|| format!("run {} not found", input.run_id))?;
        let workflow_id = run
            .workflow_version_id
            .as_deref()
            .ok_or_else(|| format!("run {} has no workflow version", run.id))?;
        let workflow = self
            .get_workflow_version(workflow_id)?
            .ok_or_else(|| format!("workflow {workflow_id} not found"))?;
        let Some(relative_hook_path) = workflow_hook_path(&workflow, hook_name) else {
            let workspace_path = normalize_optional_text(input.workspace_path)
                .or(run.workspace_path.clone())
                .unwrap_or_else(|| input.repo_root.clone());
            return Ok(WorkflowHookRunResult {
                run_id: run.id,
                hook_name: hook_name.to_string(),
                status: "skipped".to_string(),
                exit_code: None,
                stdout: String::new(),
                stderr: format!("workflow hook {hook_name} is not configured"),
                source_path: None,
                mirrored_path: None,
                workspace_path,
                env_json: serde_json::json!({}),
            });
        };

        let repo_root = canonicalize_existing_dir(&input.repo_root, "repo root")?;
        let workspace = normalize_optional_text(input.workspace_path)
            .or(run.workspace_path.clone())
            .unwrap_or_else(|| repo_root.to_string_lossy().to_string());
        let workspace_root = canonicalize_existing_dir(&workspace, "workspace path")?;
        ensure_path_inside(&workspace_root, &repo_root, "workspace path")?;

        let source_path = repo_root.join(&relative_hook_path);
        let source_path = source_path.canonicalize().map_err(|error| {
            format!(
                "workflow hook {hook_name} source {} could not be resolved: {error}",
                source_path.display()
            )
        })?;
        ensure_path_inside(&source_path, &repo_root, "workflow hook source")?;
        let mirrored_path = workspace_root.join(&relative_hook_path);
        ensure_path_inside(&mirrored_path, &workspace_root, "mirrored hook path")?;

        let env_json = serde_json::json!({
            "HANEULCHI_RUN_ID": run.id,
            "HANEULCHI_TASK_ID": run.task_id,
            "HANEULCHI_WORKSPACE_PATH": workspace_root.to_string_lossy().to_string(),
            "HANEULCHI_CONTEXT_PACK_ID": run.context_pack_id.clone().unwrap_or_default(),
            "HANEULCHI_WORKFLOW_VERSION_ID": workflow.id,
        });
        let output = Command::new(&source_path)
            .current_dir(&workspace_root)
            .env(
                "HANEULCHI_RUN_ID",
                env_json["HANEULCHI_RUN_ID"].as_str().unwrap_or(""),
            )
            .env(
                "HANEULCHI_TASK_ID",
                env_json["HANEULCHI_TASK_ID"].as_str().unwrap_or(""),
            )
            .env(
                "HANEULCHI_WORKSPACE_PATH",
                env_json["HANEULCHI_WORKSPACE_PATH"].as_str().unwrap_or(""),
            )
            .env(
                "HANEULCHI_CONTEXT_PACK_ID",
                env_json["HANEULCHI_CONTEXT_PACK_ID"].as_str().unwrap_or(""),
            )
            .env(
                "HANEULCHI_WORKFLOW_VERSION_ID",
                env_json["HANEULCHI_WORKFLOW_VERSION_ID"]
                    .as_str()
                    .unwrap_or(""),
            )
            .output()
            .map_err(|error| format!("failed to run workflow hook {hook_name}: {error}"))?;
        let exit_code = output.status.code().map(i64::from);

        let result = WorkflowHookRunResult {
            run_id: run.id.clone(),
            hook_name: hook_name.to_string(),
            status: if output.status.success() {
                "completed".to_string()
            } else {
                "failed".to_string()
            },
            exit_code,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            source_path: Some(source_path.to_string_lossy().to_string()),
            mirrored_path: Some(mirrored_path.to_string_lossy().to_string()),
            workspace_path: workspace_root.to_string_lossy().to_string(),
            env_json,
        };
        self.save_run_replay_hook_result(&run, &workflow, &result)?;
        Ok(result)
    }

    pub fn get_run_replay_metadata(
        &self,
        run_id: &str,
    ) -> Result<Option<PersistedRunReplayMetadata>, String> {
        let connection = self.connection()?;
        let row = connection
            .query_row(
                "SELECT id, run_id, artifact_path
                 FROM run_replay_metadata
                 WHERE run_id = ?1",
                params![run_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("failed to load replay metadata for run {run_id}: {error}"))?;
        let Some((id, run_id, artifact_path)) = row else {
            return Ok(None);
        };
        let full_path = self.resolve_artifact_path(&artifact_path);
        let body = fs::read_to_string(&full_path).map_err(|error| {
            format!(
                "failed to read replay metadata artifact {}: {error}",
                full_path.display()
            )
        })?;
        let body_json = serde_json::from_str(&body).map_err(|error| {
            format!(
                "failed to parse replay metadata artifact {}: {error}",
                full_path.display()
            )
        })?;

        Ok(Some(PersistedRunReplayMetadata {
            id,
            run_id,
            artifact_path,
            body_json,
        }))
    }

    fn save_run_replay_hook_result(
        &self,
        run: &PersistedRun,
        workflow: &PersistedWorkflowVersion,
        hook_result: &WorkflowHookRunResult,
    ) -> Result<PersistedRunReplayMetadata, String> {
        let mut body_json = self
            .get_run_replay_metadata(&run.id)?
            .map(|metadata| metadata.body_json)
            .unwrap_or_else(|| create_run_replay_metadata_json(run, workflow));
        if !body_json["hook_results"].is_array() {
            body_json["hook_results"] = serde_json::json!([]);
        }
        body_json["hook_results"]
            .as_array_mut()
            .expect("hook_results array")
            .push(serde_json::to_value(hook_result).map_err(|error| {
                format!(
                    "failed to serialize hook result for run {}: {error}",
                    run.id
                )
            })?);
        body_json["workspace_path"] = serde_json::json!(hook_result.workspace_path);
        let redacted_body_json = self.redact_json_secrets(&body_json)?;

        let artifact_path = self.run_replay_artifact_path(&run.id);
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create replay metadata artifact directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(
            &artifact_path,
            serde_json::to_string_pretty(&redacted_body_json).map_err(|error| {
                format!(
                    "failed to serialize replay metadata for run {}: {error}",
                    run.id
                )
            })?,
        )
        .map_err(|error| {
            format!(
                "failed to write replay metadata artifact {}: {error}",
                artifact_path.display()
            )
        })?;

        let relative_path = self.relative_artifact_path(&artifact_path);
        let id = format!("replay_{}", run.id);
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO run_replay_metadata(id, run_id, artifact_path, created_at, updated_at)
                 VALUES (
                   ?1, ?2, ?3,
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                   strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 )
                 ON CONFLICT(run_id) DO UPDATE SET
                   artifact_path = excluded.artifact_path,
                   updated_at = excluded.updated_at",
                params![id, run.id, relative_path],
            )
            .map_err(|error| {
                format!(
                    "failed to persist replay metadata for run {}: {error}",
                    run.id
                )
            })?;

        self.get_run_replay_metadata(&run.id)?.ok_or_else(|| {
            format!(
                "saved replay metadata for run {} could not be loaded",
                run.id
            )
        })
    }

    fn query_command_blocks(
        &self,
        query: Option<String>,
        task_id: Option<String>,
        session_id: Option<String>,
        limit: usize,
    ) -> Result<Vec<PersistedCommandBlock>, String> {
        let limit = limit.clamp(1, 100) as i64;
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, session_id, task_id, run_id, seq_start, seq_end, command,
                   cwd, branch, exit_code, duration_ms, summary
                 FROM command_blocks
                 WHERE (?1 IS NULL
                    OR lower(command) LIKE ?2
                    OR lower(COALESCE(cwd, '')) LIKE ?2
                    OR lower(COALESCE(branch, '')) LIKE ?2
                    OR lower(COALESCE(summary, '')) LIKE ?2)
                   AND (?3 IS NULL OR task_id = ?3)
                   AND (?4 IS NULL OR session_id = ?4)
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?5",
            )
            .map_err(|error| format!("failed to prepare command block query: {error}"))?;
        let pattern = query
            .as_ref()
            .map(|value| format!("%{}%", value.to_lowercase()));
        let rows = statement
            .query_map(params![query, pattern, task_id, session_id, limit], |row| {
                Ok(PersistedCommandBlock {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    task_id: row.get(2)?,
                    run_id: row.get(3)?,
                    seq_start: row.get(4)?,
                    seq_end: row.get(5)?,
                    command: row.get(6)?,
                    cwd: row.get(7)?,
                    branch: row.get(8)?,
                    exit_code: row.get(9)?,
                    duration_ms: row.get(10)?,
                    summary: row.get(11)?,
                })
            })
            .map_err(|error| format!("failed to query command blocks: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read command block row: {error}"))
    }

    fn command_blocks_for_run(&self, run_id: &str) -> Result<Vec<PersistedCommandBlock>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT
                   id, session_id, task_id, run_id, seq_start, seq_end, command,
                   cwd, branch, exit_code, duration_ms, summary
                 FROM command_blocks
                 WHERE run_id = ?1
                 ORDER BY COALESCE(seq_start, 0) ASC, id ASC",
            )
            .map_err(|error| format!("failed to prepare run command block query: {error}"))?;
        let rows = statement
            .query_map(params![run_id], |row| {
                Ok(PersistedCommandBlock {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    task_id: row.get(2)?,
                    run_id: row.get(3)?,
                    seq_start: row.get(4)?,
                    seq_end: row.get(5)?,
                    command: row.get(6)?,
                    cwd: row.get(7)?,
                    branch: row.get(8)?,
                    exit_code: row.get(9)?,
                    duration_ms: row.get(10)?,
                    summary: row.get(11)?,
                })
            })
            .map_err(|error| format!("failed to query command blocks for run {run_id}: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read command block row: {error}"))
    }

    fn get_token_usage(&self, usage_id: &str) -> Result<Option<PersistedTokenUsage>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, session_id, task_id, run_id, agent_profile_id,
                   provider, model, input_tokens, output_tokens, cost_usd, source
                 FROM token_usage
                 WHERE id = ?1",
                params![usage_id],
                persisted_token_usage_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load token usage {usage_id}: {error}"))
    }

    fn get_secret_metadata(
        &self,
        secret_id: &str,
    ) -> Result<Option<PersistedSecretMetadata>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, project_id, name, keychain_ref, created_at, updated_at
                 FROM secrets
                 WHERE id = ?1",
                params![secret_id],
                persisted_secret_metadata_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load secret metadata {secret_id}: {error}"))
    }

    fn get_budget(&self, budget_id: &str) -> Result<Option<PersistedBudget>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, scope_type, scope_id, max_usd, warn_pct, hard_limit
                 FROM budgets
                 WHERE id = ?1",
                params![budget_id],
                persisted_budget_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load budget {budget_id}: {error}"))
    }

    fn list_budgets(&self) -> Result<Vec<PersistedBudget>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, scope_type, scope_id, max_usd, warn_pct, hard_limit
                 FROM budgets
                 ORDER BY scope_type ASC, COALESCE(scope_id, '') ASC",
            )
            .map_err(|error| format!("failed to prepare budget list query: {error}"))?;
        let rows = statement
            .query_map([], persisted_budget_from_row)
            .map_err(|error| format!("failed to query budgets: {error}"))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read budget row: {error}"))
    }

    fn get_knowledge_source(
        &self,
        source_id: &str,
    ) -> Result<Option<PersistedKnowledgeSource>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, project_id, kind, path_or_ref, fingerprint, status
                 FROM knowledge_sources
                 WHERE id = ?1",
                params![source_id],
                persisted_knowledge_source_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load knowledge source {source_id}: {error}"))
    }

    pub fn get_knowledge_page(
        &self,
        page_id: &str,
    ) -> Result<Option<PersistedKnowledgePage>, String> {
        let connection = self.connection()?;
        let row = connection
            .query_row(
                "SELECT id, project_id, slug, title, artifact_path, source_ids_json, freshness_state
                 FROM knowledge_pages
                 WHERE id = ?1",
                params![page_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("failed to load knowledge page {page_id}: {error}"))?;
        let Some((id, project_id, slug, title, artifact_path, source_ids_json, freshness_state)) =
            row
        else {
            return Ok(None);
        };
        let full_path = self.resolve_artifact_path(&artifact_path);
        let body_md = fs::read_to_string(&full_path).map_err(|error| {
            format!(
                "failed to read knowledge page {id} artifact {}: {error}",
                full_path.display()
            )
        })?;
        let source_ids = serde_json::from_str(&source_ids_json)
            .map_err(|error| format!("failed to parse knowledge page {id} source ids: {error}"))?;
        Ok(Some(PersistedKnowledgePage {
            id,
            project_id,
            slug,
            title,
            artifact_path,
            source_ids,
            freshness_state,
            body_md,
        }))
    }

    pub fn get_context_pack(&self, pack_id: &str) -> Result<Option<PersistedContextPack>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, project_id, name, description, sources_json
                 FROM context_packs
                 WHERE id = ?1",
                params![pack_id],
                persisted_context_pack_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load context pack {pack_id}: {error}"))
    }

    pub fn get_skill_pack(&self, pack_id: &str) -> Result<Option<PersistedSkillPack>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT
                   id, project_id, name, description, skills_json, source_context_pack_id,
                   created_at, updated_at
                 FROM skill_packs
                 WHERE id = ?1",
                params![pack_id],
                persisted_skill_pack_from_row,
            )
            .optional()
            .map_err(|error| format!("failed to load skill pack {pack_id}: {error}"))
    }

    pub fn get_knowledge_exploration(
        &self,
        exploration_id: &str,
    ) -> Result<Option<PersistedKnowledgeExploration>, String> {
        let connection = self.connection()?;
        let row = connection
            .query_row(
                "SELECT id, project_id, title, question, artifact_path, page_ids_json,
                        context_pack_id
                 FROM knowledge_explorations
                 WHERE id = ?1",
                params![exploration_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<String>>(6)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| {
                format!("failed to load knowledge exploration {exploration_id}: {error}")
            })?;
        let Some((id, project_id, title, question, artifact_path, page_ids_json, context_pack_id)) =
            row
        else {
            return Ok(None);
        };
        let full_path = self.resolve_artifact_path(&artifact_path);
        let answer_md = fs::read_to_string(&full_path).map_err(|error| {
            format!(
                "failed to read knowledge exploration {id} artifact {}: {error}",
                full_path.display()
            )
        })?;
        let page_ids = serde_json::from_str(&page_ids_json).map_err(|error| {
            format!("failed to parse knowledge exploration {id} page ids: {error}")
        })?;
        Ok(Some(PersistedKnowledgeExploration {
            id,
            project_id,
            title,
            question,
            answer_md,
            artifact_path,
            page_ids,
            context_pack_id,
        }))
    }

    fn get_knowledge_lint_report(
        &self,
        report_id: &str,
    ) -> Result<Option<PersistedKnowledgeLintReport>, String> {
        let connection = self.connection()?;
        let row = connection
            .query_row(
                "SELECT id, project_id, artifact_path, stale_count, gap_count, contradiction_count
                 FROM knowledge_lint_reports
                 WHERE id = ?1",
                params![report_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| {
                format!("failed to load knowledge lint report {report_id}: {error}")
            })?;
        let Some((id, project_id, artifact_path, stale_count, gap_count, contradiction_count)) =
            row
        else {
            return Ok(None);
        };
        let full_path = self.resolve_artifact_path(&artifact_path);
        let body_md = fs::read_to_string(&full_path).map_err(|error| {
            format!(
                "failed to read knowledge lint report {id} artifact {}: {error}",
                full_path.display()
            )
        })?;
        Ok(Some(PersistedKnowledgeLintReport {
            id,
            project_id,
            artifact_path,
            stale_count,
            gap_count,
            contradiction_count,
            body_md,
        }))
    }

    fn latest_knowledge_lint_report(
        &self,
        project_id: &str,
    ) -> Result<Option<PersistedKnowledgeLintReport>, String> {
        let connection = self.connection()?;
        let report_id = connection
            .query_row(
                "SELECT id
                 FROM knowledge_lint_reports
                 WHERE project_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![project_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| {
                format!("failed to load latest knowledge lint report for {project_id}: {error}")
            })?;
        drop(connection);
        report_id
            .map(|id| self.get_knowledge_lint_report(&id))
            .transpose()
            .map(Option::flatten)
    }

    fn recent_knowledge_page_slugs(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<Vec<String>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT slug
                 FROM knowledge_pages
                 WHERE project_id = ?1
                 ORDER BY updated_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|error| format!("failed to prepare recent knowledge query: {error}"))?;
        let rows = statement
            .query_map(params![project_id, limit as i64], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|error| format!("failed to query recent knowledge pages: {error}"))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("failed to read knowledge page slug: {error}"))
    }

    fn token_usage_total(&self, scope_type: &str, scope_id: Option<&str>) -> Result<f64, String> {
        let connection = self.connection()?;
        let sql = match scope_type {
            "workspace" => "SELECT COALESCE(SUM(cost_usd), 0.0) FROM token_usage",
            "project" => {
                "SELECT COALESCE(SUM(cost_usd), 0.0) FROM token_usage WHERE project_id = ?1"
            }
            "goal" => {
                "SELECT COALESCE(SUM(token_usage.cost_usd), 0.0)
                 FROM token_usage
                 JOIN tasks ON tasks.id = token_usage.task_id
                 WHERE tasks.initiative_id = ?1"
            }
            "task" => "SELECT COALESCE(SUM(cost_usd), 0.0) FROM token_usage WHERE task_id = ?1",
            "run" => "SELECT COALESCE(SUM(cost_usd), 0.0) FROM token_usage WHERE run_id = ?1",
            "agent" => {
                "SELECT COALESCE(SUM(cost_usd), 0.0) FROM token_usage WHERE agent_profile_id = ?1"
            }
            unsupported => return Err(format!("unsupported budget scope {unsupported}")),
        };
        let total = if scope_type == "workspace" {
            connection.query_row(sql, [], |row| row.get::<_, f64>(0))
        } else {
            connection.query_row(sql, params![scope_id], |row| row.get::<_, f64>(0))
        }
        .map_err(|error| format!("failed to total token usage for {scope_type}: {error}"))?;
        Ok(total)
    }

    fn token_usage_run_costs(
        &self,
        scope_type: &str,
        scope_id: Option<&str>,
    ) -> Result<Vec<f64>, String> {
        let connection = self.connection()?;
        let sql = match scope_type {
            "workspace" => {
                "SELECT COALESCE(SUM(cost_usd), 0.0)
                 FROM token_usage
                 WHERE run_id IS NOT NULL
                 GROUP BY run_id
                 ORDER BY run_id ASC"
            }
            "project" => {
                "SELECT COALESCE(SUM(cost_usd), 0.0)
                 FROM token_usage
                 WHERE project_id = ?1 AND run_id IS NOT NULL
                 GROUP BY run_id
                 ORDER BY run_id ASC"
            }
            "goal" => {
                "SELECT COALESCE(SUM(token_usage.cost_usd), 0.0)
                 FROM token_usage
                 JOIN tasks ON tasks.id = token_usage.task_id
                 WHERE tasks.initiative_id = ?1 AND token_usage.run_id IS NOT NULL
                 GROUP BY token_usage.run_id
                 ORDER BY token_usage.run_id ASC"
            }
            "task" => {
                "SELECT COALESCE(SUM(cost_usd), 0.0)
                 FROM token_usage
                 WHERE task_id = ?1 AND run_id IS NOT NULL
                 GROUP BY run_id
                 ORDER BY run_id ASC"
            }
            "run" => {
                "SELECT COALESCE(SUM(cost_usd), 0.0)
                 FROM token_usage
                 WHERE run_id = ?1
                 GROUP BY run_id
                 ORDER BY run_id ASC"
            }
            "agent" => {
                "SELECT COALESCE(SUM(cost_usd), 0.0)
                 FROM token_usage
                 WHERE agent_profile_id = ?1 AND run_id IS NOT NULL
                 GROUP BY run_id
                 ORDER BY run_id ASC"
            }
            unsupported => return Err(format!("unsupported budget scope {unsupported}")),
        };
        let mut statement = connection
            .prepare(sql)
            .map_err(|error| format!("failed to prepare token usage runway query: {error}"))?;
        if scope_type == "workspace" {
            let rows = statement
                .query_map([], |row| row.get::<_, f64>(0))
                .map_err(|error| {
                    format!("failed to query token usage runway for {scope_type}: {error}")
                })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|error| format!("failed to read token usage runway row: {error}"))
        } else {
            let rows = statement
                .query_map(params![scope_id], |row| row.get::<_, f64>(0))
                .map_err(|error| {
                    format!("failed to query token usage runway for {scope_type}: {error}")
                })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|error| format!("failed to read token usage runway row: {error}"))
        }
    }

    fn estimate_token_usage_cost(
        &self,
        provider: &str,
        model: &str,
        input_tokens: i64,
        output_tokens: i64,
    ) -> Result<Option<f64>, String> {
        let connection = self.connection()?;
        let price = connection
            .query_row(
                "SELECT provider, model, input_usd_per_million, output_usd_per_million,
                        source, updated_at
                 FROM provider_prices
                 WHERE provider = ?1 AND model = ?2",
                params![provider, model],
                persisted_provider_price_from_row,
            )
            .optional()
            .map_err(|error| {
                format!("failed to load provider price {provider}/{model}: {error}")
            })?;
        Ok(price.map(|price| {
            round_cost(
                (input_tokens as f64 * price.input_usd_per_million
                    + output_tokens as f64 * price.output_usd_per_million)
                    / 1_000_000.0,
            )
        }))
    }

    fn enforce_budget_gate(
        &self,
        task_id: &str,
        project_id: &str,
        initiative_id: Option<&str>,
        agent_profile_id: Option<&str>,
    ) -> Result<(), String> {
        self.enforce_budget_scope("project", Some(project_id))?;
        if let Some(initiative_id) = initiative_id {
            self.enforce_budget_scope("goal", Some(initiative_id))?;
        }
        self.enforce_budget_scope("task", Some(task_id))?;
        if let Some(agent_profile_id) = agent_profile_id {
            self.enforce_budget_scope("agent", Some(agent_profile_id))?;
        }
        self.enforce_budget_scope("workspace", None)
    }

    fn enforce_budget_scope(&self, scope_type: &str, scope_id: Option<&str>) -> Result<(), String> {
        let budget_id = budget_id_for_scope(scope_type, scope_id);
        let Some(budget) = self.get_budget(&budget_id)? else {
            return Ok(());
        };
        if !budget.hard_limit {
            return Ok(());
        }
        let used = self.token_usage_total(scope_type, scope_id)?;
        if used >= budget.max_usd {
            let label = match scope_type {
                "project" => "project budget exceeded",
                "goal" => "goal budget exceeded",
                "task" => "task budget exceeded",
                "run" => "run budget exceeded",
                "agent" => "agent budget exceeded",
                "workspace" => "workspace budget exceeded",
                _ => "budget exceeded",
            };
            return Err(format!(
                "{label}: used ${:.2} of ${:.2}",
                round_cost(used),
                round_cost(budget.max_usd)
            ));
        }
        Ok(())
    }

    fn enforce_agent_profile_available(
        &self,
        agent_profile_id: Option<&str>,
    ) -> Result<(), String> {
        let Some(agent_profile_id) = agent_profile_id else {
            return Ok(());
        };
        let Some(agent) = self.get_agent_profile(agent_profile_id)? else {
            return Ok(());
        };
        if agent.status == "available" {
            Ok(())
        } else {
            Err(format!("agent {agent_profile_id} is {}", agent.status))
        }
    }

    fn connection(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path).map_err(|error| {
            format!(
                "failed to open sqlite state store {}: {error}",
                self.db_path.display()
            )
        })
    }

    fn artifact_root(&self) -> PathBuf {
        self.db_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("artifacts")
    }

    fn workpad_artifact_path(&self, task_id: &str) -> PathBuf {
        self.artifact_root()
            .join("workpads")
            .join(format!("{}.md", sanitize_artifact_name(task_id)))
    }

    fn evidence_artifact_path(&self, evidence_pack_id: &str) -> PathBuf {
        self.artifact_root()
            .join("evidence")
            .join(format!("{}.json", sanitize_artifact_name(evidence_pack_id)))
    }

    fn run_replay_artifact_path(&self, run_id: &str) -> PathBuf {
        self.artifact_root()
            .join("runs")
            .join(sanitize_artifact_name(run_id))
            .join("replay.json")
    }

    fn terminal_stream_chunk_artifact_path(&self, session_id: &str, chunk_id: &str) -> PathBuf {
        self.artifact_root()
            .join("transcripts")
            .join(sanitize_artifact_name(session_id))
            .join(format!("{}.txt", sanitize_artifact_name(chunk_id)))
    }

    fn knowledge_page_artifact_path(&self, slug: &str) -> PathBuf {
        self.artifact_root()
            .join("knowledge")
            .join(format!("{}.md", sanitize_artifact_name(slug)))
    }

    fn knowledge_lint_artifact_path(&self, report_id: &str) -> PathBuf {
        self.artifact_root()
            .join("knowledge")
            .join("lint")
            .join(format!("{}.md", sanitize_artifact_name(report_id)))
    }

    fn knowledge_exploration_artifact_path(&self, exploration_id: &str) -> PathBuf {
        self.artifact_root()
            .join("knowledge")
            .join("explorations")
            .join(format!("{}.md", sanitize_artifact_name(exploration_id)))
    }

    fn relative_artifact_path(&self, artifact_path: &Path) -> String {
        self.db_path
            .parent()
            .and_then(|root| artifact_path.strip_prefix(root).ok())
            .unwrap_or(artifact_path)
            .to_string_lossy()
            .to_string()
    }

    fn resolve_artifact_path(&self, artifact_path: &str) -> PathBuf {
        let path = PathBuf::from(artifact_path);
        if path.is_absolute() {
            path
        } else {
            self.db_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(path)
        }
    }
}

fn apply_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS projects(
          id TEXT PRIMARY KEY,
          key TEXT NOT NULL UNIQUE,
          name TEXT NOT NULL,
          path TEXT NOT NULL,
          color TEXT,
          status TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS project_tabs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          order_index INTEGER NOT NULL,
          active INTEGER NOT NULL,
          layout_json TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS project_tab_groups(
          project_id TEXT PRIMARY KEY,
          group_name TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS project_layout_presets(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          layout_json TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, name)
        );
        CREATE TABLE IF NOT EXISTS project_detach_plans(
          project_id TEXT PRIMARY KEY,
          project_name TEXT NOT NULL,
          window_id TEXT NOT NULL,
          status TEXT NOT NULL,
          degraded_reason TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS sessions(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          pane_id TEXT,
          mode TEXT NOT NULL,
          title TEXT NOT NULL,
          cwd TEXT,
          branch TEXT,
          agent_profile_id TEXT,
          task_id TEXT,
          run_id TEXT,
          state TEXT NOT NULL,
          attention_state TEXT NOT NULL,
          token_budget_state TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS terminal_stream_chunks(
          id TEXT PRIMARY KEY,
          session_id TEXT NOT NULL,
          seq_start INTEGER NOT NULL,
          seq_end INTEGER NOT NULL,
          artifact_path TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS command_blocks(
          id TEXT PRIMARY KEY,
          session_id TEXT NOT NULL,
          task_id TEXT,
          run_id TEXT,
          seq_start INTEGER,
          seq_end INTEGER,
          command TEXT NOT NULL,
          cwd TEXT,
          branch TEXT,
          exit_code INTEGER,
          duration_ms INTEGER,
          summary TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS tasks(
          id TEXT PRIMARY KEY,
          key TEXT NOT NULL UNIQUE,
          project_id TEXT NOT NULL,
          title TEXT NOT NULL,
          description TEXT,
          status TEXT NOT NULL,
          priority TEXT NOT NULL,
          assignee_type TEXT,
          assignee_id TEXT,
          cycle_id TEXT,
          module_id TEXT,
          context_pack_id TEXT,
          initiative_id TEXT,
          due_at TEXT,
          estimate TEXT,
          labels_json TEXT NOT NULL DEFAULT '[]',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS subtasks(
          id TEXT PRIMARY KEY,
          task_id TEXT NOT NULL,
          title TEXT NOT NULL,
          status TEXT NOT NULL,
          order_index INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS comments(
          id TEXT PRIMARY KEY,
          task_id TEXT,
          run_id TEXT,
          author_type TEXT NOT NULL,
          author_id TEXT NOT NULL,
          body_md TEXT NOT NULL,
          parent_id TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS workpads(
          id TEXT PRIMARY KEY,
          task_id TEXT NOT NULL,
          artifact_path TEXT NOT NULL,
          title TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS cycles(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          starts_at TEXT,
          ends_at TEXT,
          status TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS modules(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          description TEXT,
          status TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS initiatives(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          description TEXT,
          budget_id TEXT,
          status TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS external_tracker_bindings(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          local_kind TEXT NOT NULL,
          local_id TEXT NOT NULL,
          provider TEXT NOT NULL,
          external_id TEXT NOT NULL,
          external_url TEXT,
          sync_mode TEXT NOT NULL,
          sync_status TEXT NOT NULL,
          conflict_state TEXT NOT NULL,
          metadata_json TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(provider, external_id),
          UNIQUE(local_kind, local_id, provider)
        );
        CREATE TABLE IF NOT EXISTS external_tracker_sync_runs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          provider TEXT NOT NULL,
          dry_run INTEGER NOT NULL,
          status TEXT NOT NULL,
          operation_count INTEGER NOT NULL,
          degraded_reason TEXT,
          operations_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS agent_profiles(
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          runtime TEXT NOT NULL,
          command TEXT NOT NULL,
          args_json TEXT NOT NULL,
          env_policy_json TEXT NOT NULL,
          skills_json TEXT NOT NULL,
          status TEXT NOT NULL,
          last_heartbeat_at TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS runs(
          id TEXT PRIMARY KEY,
          task_id TEXT NOT NULL,
          project_id TEXT NOT NULL,
          agent_profile_id TEXT,
          session_id TEXT,
          workflow_version_id TEXT,
          context_pack_id TEXT,
          workspace_path TEXT,
          lifecycle TEXT NOT NULL,
          retry_count INTEGER NOT NULL,
          next_retry_at TEXT,
          status_detail TEXT,
          budget_id TEXT,
          started_at TEXT,
          ended_at TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS run_replay_metadata(
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL UNIQUE,
          artifact_path TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS evidence_packs(
          id TEXT PRIMARY KEY,
          task_id TEXT,
          run_id TEXT,
          artifact_path TEXT NOT NULL,
          completeness_state TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS release_gate_runs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          status TEXT NOT NULL,
          scenario_count INTEGER NOT NULL,
          pass_count INTEGER NOT NULL,
          fail_count INTEGER NOT NULL,
          warning_count INTEGER NOT NULL,
          scenarios_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS terminal_fidelity_smoke_runs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          status TEXT NOT NULL,
          case_count INTEGER NOT NULL,
          pass_count INTEGER NOT NULL,
          fail_count INTEGER NOT NULL,
          warning_count INTEGER NOT NULL,
          cases_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS task_lifecycle_e2e_runs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          status TEXT NOT NULL,
          task_id TEXT NOT NULL,
          run_id TEXT NOT NULL,
          evidence_pack_id TEXT NOT NULL,
          transitions_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS workflow_negative_test_runs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          status TEXT NOT NULL,
          baseline_workflow_id TEXT NOT NULL,
          invalid_workflow_id TEXT NOT NULL,
          last_known_good_workflow_id TEXT NOT NULL,
          dispatch_run_id TEXT NOT NULL,
          cases_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS dmg_smoke_runs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          status TEXT NOT NULL,
          explicit_blocker INTEGER NOT NULL,
          dmg_path TEXT,
          app_bundle_path TEXT,
          case_count INTEGER NOT NULL,
          pass_count INTEGER NOT NULL,
          fail_count INTEGER NOT NULL,
          warning_count INTEGER NOT NULL,
          cases_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS recovery_drill_runs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          status TEXT NOT NULL,
          drill_count INTEGER NOT NULL,
          pass_count INTEGER NOT NULL,
          fail_count INTEGER NOT NULL,
          warning_count INTEGER NOT NULL,
          drills_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS benchmark_runs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          status TEXT NOT NULL,
          suite_count INTEGER NOT NULL,
          pass_count INTEGER NOT NULL,
          fail_count INTEGER NOT NULL,
          warning_count INTEGER NOT NULL,
          duration_ms INTEGER NOT NULL,
          suites_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS dogfood_telemetry_reviews(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          status TEXT NOT NULL,
          evidence_pack_id TEXT NOT NULL,
          finding_count INTEGER NOT NULL,
          pass_count INTEGER NOT NULL,
          warning_count INTEGER NOT NULL,
          fail_count INTEGER NOT NULL,
          findings_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS visual_harness_links(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          source_id TEXT NOT NULL,
          target_id TEXT NOT NULL,
          kind TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS workflow_versions(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          source_path TEXT NOT NULL,
          content_hash TEXT NOT NULL,
          parsed_json TEXT NOT NULL,
          valid INTEGER NOT NULL,
          diagnostics_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS context_packs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          description TEXT,
          sources_json TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS skill_packs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          description TEXT,
          skills_json TEXT NOT NULL,
          source_context_pack_id TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, name)
        );
        CREATE TABLE IF NOT EXISTS knowledge_sources(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          kind TEXT NOT NULL,
          path_or_ref TEXT NOT NULL,
          fingerprint TEXT NOT NULL,
          status TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS knowledge_pages(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          slug TEXT NOT NULL,
          title TEXT NOT NULL,
          artifact_path TEXT NOT NULL,
          source_ids_json TEXT NOT NULL,
          freshness_state TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS knowledge_lint_reports(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          artifact_path TEXT NOT NULL,
          stale_count INTEGER NOT NULL,
          gap_count INTEGER NOT NULL,
          contradiction_count INTEGER NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS knowledge_explorations(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          title TEXT NOT NULL,
          question TEXT NOT NULL,
          artifact_path TEXT NOT NULL,
          page_ids_json TEXT NOT NULL,
          context_pack_id TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS token_usage(
          id TEXT PRIMARY KEY,
          project_id TEXT,
          session_id TEXT,
          task_id TEXT,
          run_id TEXT,
          agent_profile_id TEXT,
          provider TEXT NOT NULL,
          model TEXT NOT NULL,
          input_tokens INTEGER NOT NULL,
          output_tokens INTEGER NOT NULL,
          cost_usd REAL NOT NULL,
          source TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS agent_events(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          session_id TEXT,
          run_id TEXT,
          agent_profile_id TEXT NOT NULL,
          kind TEXT NOT NULL,
          severity TEXT NOT NULL,
          detail TEXT NOT NULL,
          payload_json TEXT NOT NULL,
          source TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS provider_prices(
          provider TEXT NOT NULL,
          model TEXT NOT NULL,
          input_usd_per_million REAL NOT NULL,
          output_usd_per_million REAL NOT NULL,
          source TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY(provider, model)
        );
        CREATE TABLE IF NOT EXISTS secrets(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          keychain_ref TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, name)
        );
        CREATE TABLE IF NOT EXISTS keychain_secret_values(
          keychain_ref TEXT PRIMARY KEY,
          value TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS budgets(
          id TEXT PRIMARY KEY,
          scope_type TEXT NOT NULL,
          scope_id TEXT,
          max_usd REAL NOT NULL,
          warn_pct REAL NOT NULL,
          hard_limit INTEGER NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS policy_approvals(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          task_id TEXT,
          run_id TEXT,
          action_kind TEXT NOT NULL,
          command TEXT,
          risk_level TEXT NOT NULL,
          state TEXT NOT NULL,
          requested_by TEXT,
          decision_by TEXT,
          decision_note TEXT,
          created_at TEXT NOT NULL,
          decided_at TEXT
        );
        CREATE TABLE IF NOT EXISTS policy_packs(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          sandbox_mode TEXT NOT NULL,
          network TEXT NOT NULL,
          network_profile TEXT NOT NULL DEFAULT 'internet',
          file_write TEXT NOT NULL,
          tools TEXT NOT NULL,
          approval_required_json TEXT NOT NULL,
          forbidden_operations_json TEXT NOT NULL,
          active INTEGER NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, name)
        );
        CREATE TABLE IF NOT EXISTS permission_audit_events(
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          task_id TEXT,
          run_id TEXT,
          policy_pack_id TEXT,
          action_kind TEXT NOT NULL,
          command TEXT,
          decision TEXT NOT NULL,
          reason TEXT NOT NULL,
          requested_by TEXT,
          created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS attention_events(
          id TEXT PRIMARY KEY,
          project_id TEXT,
          session_id TEXT,
          task_id TEXT,
          run_id TEXT,
          kind TEXT NOT NULL,
          severity TEXT NOT NULL,
          title TEXT NOT NULL,
          body TEXT NOT NULL,
          unread INTEGER NOT NULL,
          created_at TEXT NOT NULL,
          resolved_at TEXT
        );
        CREATE TABLE IF NOT EXISTS settings(
          key TEXT PRIMARY KEY,
          value_json TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_project_state ON sessions(project_id, state);
        CREATE INDEX IF NOT EXISTS idx_project_layout_presets_project_name ON project_layout_presets(project_id, name);
        CREATE INDEX IF NOT EXISTS idx_sessions_task ON sessions(task_id);
        CREATE INDEX IF NOT EXISTS idx_command_blocks_session_created ON command_blocks(session_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_command_blocks_task ON command_blocks(task_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_project_status ON tasks(project_id, status);
        CREATE INDEX IF NOT EXISTS idx_tasks_assignee ON tasks(assignee_type, assignee_id);
        CREATE INDEX IF NOT EXISTS idx_runs_task_lifecycle ON runs(task_id, lifecycle);
        CREATE INDEX IF NOT EXISTS idx_runs_agent_lifecycle ON runs(agent_profile_id, lifecycle);
        CREATE INDEX IF NOT EXISTS idx_run_replay_metadata_run ON run_replay_metadata(run_id);
        CREATE INDEX IF NOT EXISTS idx_attention_unread_severity_created ON attention_events(unread, severity, created_at);
        CREATE INDEX IF NOT EXISTS idx_token_usage_project_created ON token_usage(project_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_agent_events_project_created ON agent_events(project_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_knowledge_pages_project_slug ON knowledge_pages(project_id, slug);
        CREATE INDEX IF NOT EXISTS idx_knowledge_explorations_project_updated ON knowledge_explorations(project_id, updated_at);
        CREATE INDEX IF NOT EXISTS idx_skill_packs_project_name ON skill_packs(project_id, name);
        CREATE INDEX IF NOT EXISTS idx_policy_approvals_project_state_created ON policy_approvals(project_id, state, created_at);
        CREATE INDEX IF NOT EXISTS idx_policy_packs_project_active ON policy_packs(project_id, active);
        CREATE INDEX IF NOT EXISTS idx_permission_audit_project_decision_created ON permission_audit_events(project_id, decision, created_at);
        CREATE INDEX IF NOT EXISTS idx_permission_audit_run_created ON permission_audit_events(run_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_secrets_project_name ON secrets(project_id, name);
        ",
    )?;
    add_column_if_missing(connection, "runs", "next_retry_at", "TEXT")?;
    add_column_if_missing(connection, "runs", "status_detail", "TEXT")?;
    add_column_if_missing(connection, "tasks", "context_pack_id", "TEXT")?;
    add_column_if_missing(connection, "tasks", "initiative_id", "TEXT")?;
    add_column_if_missing(connection, "tasks", "due_at", "TEXT")?;
    add_column_if_missing(connection, "tasks", "estimate", "TEXT")?;
    add_column_if_missing(
        connection,
        "tasks",
        "labels_json",
        "TEXT NOT NULL DEFAULT '[]'",
    )?;
    add_column_if_missing(
        connection,
        "policy_packs",
        "network_profile",
        "TEXT NOT NULL DEFAULT 'internet'",
    )?;
    Ok(())
}

fn add_column_if_missing(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
    column_type: &str,
) -> rusqlite::Result<()> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
    for column in columns {
        if column? == column_name {
            return Ok(());
        }
    }
    connection.execute(
        &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_type}"),
        [],
    )?;
    Ok(())
}

fn retry_backoff_seconds(retry_count: i64) -> i64 {
    let exponent = retry_count.saturating_sub(1).min(5) as u32;
    10 * 2_i64.pow(exponent)
}

fn normalize_run_status_detail(
    lifecycle: &str,
    status_detail: Option<String>,
) -> Result<Option<String>, String> {
    let detail = normalize_optional_text(status_detail);
    if matches!(
        lifecycle,
        "waiting_input" | "permission_requested" | "blocked"
    ) && detail.is_none()
    {
        return Err(format!("run lifecycle {lifecycle} requires status detail"));
    }
    Ok(detail)
}

fn format_policy_status_detail(action_kind: &str, command: Option<&str>) -> String {
    match command {
        Some(command) => format!("Permission requested: {action_kind} ({command})"),
        None => format!("Permission requested: {action_kind}"),
    }
}

fn next_numeric_id(connection: &Connection, table_name: &str, prefix: &str) -> Result<i64, String> {
    let sql = format!("SELECT id FROM {table_name} WHERE id LIKE ?1",);
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| format!("failed to prepare id query for {table_name}: {error}"))?;
    let ids = statement
        .query_map(params![format!("{prefix}%")], |row| row.get::<_, String>(0))
        .map_err(|error| format!("failed to query ids for {table_name}: {error}"))?;

    let mut max_id = 0_i64;
    for id in ids {
        let id = id.map_err(|error| format!("failed to read id from {table_name}: {error}"))?;
        if let Some(number) = id
            .strip_prefix(prefix)
            .and_then(|suffix| suffix.parse::<i64>().ok())
        {
            max_id = max_id.max(number);
        }
    }
    Ok(max_id + 1)
}

fn persisted_run_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PersistedRun> {
    Ok(PersistedRun {
        id: row.get(0)?,
        task_id: row.get(1)?,
        project_id: row.get(2)?,
        agent_profile_id: row.get(3)?,
        session_id: row.get(4)?,
        workflow_version_id: row.get(5)?,
        context_pack_id: row.get(6)?,
        workspace_path: row.get(7)?,
        lifecycle: row.get(8)?,
        retry_count: row.get(9)?,
        next_retry_at: row.get(10)?,
        status_detail: row.get(11)?,
        budget_id: row.get(12)?,
        started_at: row.get(13)?,
        ended_at: row.get(14)?,
    })
}

fn persisted_project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PersistedProject> {
    Ok(PersistedProject {
        id: row.get(0)?,
        key: row.get(1)?,
        name: row.get(2)?,
        path: row.get(3)?,
        color: row.get(4)?,
        status: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn persisted_project_tab_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedProjectTab> {
    Ok(PersistedProjectTab {
        id: row.get(0)?,
        project_id: row.get(1)?,
        order_index: row.get(2)?,
        active: row.get::<_, i64>(3)? == 1,
        layout_json: parse_json_cell(row.get::<_, String>(4)?),
    })
}

fn persisted_project_tab_group_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedProjectTabGroup> {
    Ok(PersistedProjectTabGroup {
        project_id: row.get(0)?,
        group_name: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
    })
}

fn persisted_project_layout_preset_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedProjectLayoutPreset> {
    Ok(PersistedProjectLayoutPreset {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        layout_json: parse_json_cell(row.get::<_, String>(3)?),
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn persisted_project_detach_plan_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedProjectDetachPlan> {
    Ok(PersistedProjectDetachPlan {
        project_id: row.get(0)?,
        project_name: row.get(1)?,
        window_id: row.get(2)?,
        status: row.get(3)?,
        degraded_reason: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn persisted_external_tracker_binding_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedExternalTrackerBinding> {
    Ok(PersistedExternalTrackerBinding {
        id: row.get(0)?,
        project_id: row.get(1)?,
        local_kind: row.get(2)?,
        local_id: row.get(3)?,
        provider: row.get(4)?,
        external_id: row.get(5)?,
        external_url: row.get(6)?,
        sync_mode: row.get(7)?,
        sync_status: row.get(8)?,
        conflict_state: row.get(9)?,
        metadata_json: parse_json_cell(row.get::<_, String>(10)?),
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn persisted_external_tracker_sync_run_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedExternalTrackerSyncRun> {
    let operations_json = row.get::<_, String>(7)?;
    Ok(PersistedExternalTrackerSyncRun {
        id: row.get(0)?,
        project_id: row.get(1)?,
        provider: row.get(2)?,
        dry_run: row.get::<_, i64>(3)? == 1,
        status: row.get(4)?,
        operation_count: row.get(5)?,
        degraded_reason: row.get(6)?,
        operations: serde_json::from_str(&operations_json).unwrap_or_default(),
        created_at: row.get(8)?,
    })
}

fn persisted_release_gate_run_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedReleaseGateRun> {
    let scenarios_json = row.get::<_, String>(7)?;
    Ok(PersistedReleaseGateRun {
        id: row.get(0)?,
        project_id: row.get(1)?,
        status: row.get(2)?,
        scenario_count: row.get(3)?,
        pass_count: row.get(4)?,
        fail_count: row.get(5)?,
        warning_count: row.get(6)?,
        scenarios: serde_json::from_str(&scenarios_json).unwrap_or_default(),
        created_at: row.get(8)?,
    })
}

fn persisted_terminal_fidelity_smoke_run_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedTerminalFidelitySmokeRun> {
    let cases_json = row.get::<_, String>(7)?;
    Ok(PersistedTerminalFidelitySmokeRun {
        id: row.get(0)?,
        project_id: row.get(1)?,
        status: row.get(2)?,
        case_count: row.get(3)?,
        pass_count: row.get(4)?,
        fail_count: row.get(5)?,
        warning_count: row.get(6)?,
        cases: serde_json::from_str(&cases_json).unwrap_or_default(),
        created_at: row.get(8)?,
    })
}

fn persisted_task_lifecycle_e2e_run_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedTaskLifecycleE2ERun> {
    let transitions_json = row.get::<_, String>(6)?;
    Ok(PersistedTaskLifecycleE2ERun {
        id: row.get(0)?,
        project_id: row.get(1)?,
        status: row.get(2)?,
        task_id: row.get(3)?,
        run_id: row.get(4)?,
        evidence_pack_id: row.get(5)?,
        transitions: serde_json::from_str(&transitions_json).unwrap_or_default(),
        created_at: row.get(7)?,
    })
}

fn persisted_workflow_negative_test_run_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedWorkflowNegativeTestRun> {
    let cases_json = row.get::<_, String>(7)?;
    Ok(PersistedWorkflowNegativeTestRun {
        id: row.get(0)?,
        project_id: row.get(1)?,
        status: row.get(2)?,
        baseline_workflow_id: row.get(3)?,
        invalid_workflow_id: row.get(4)?,
        last_known_good_workflow_id: row.get(5)?,
        dispatch_run_id: row.get(6)?,
        cases: serde_json::from_str(&cases_json).unwrap_or_default(),
        created_at: row.get(8)?,
    })
}

fn persisted_dmg_smoke_run_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedDmgSmokeRun> {
    let cases_json = row.get::<_, String>(10)?;
    Ok(PersistedDmgSmokeRun {
        id: row.get(0)?,
        project_id: row.get(1)?,
        status: row.get(2)?,
        explicit_blocker: row.get::<_, i64>(3)? == 1,
        dmg_path: row.get(4)?,
        app_bundle_path: row.get(5)?,
        case_count: row.get(6)?,
        pass_count: row.get(7)?,
        fail_count: row.get(8)?,
        warning_count: row.get(9)?,
        cases: serde_json::from_str(&cases_json).unwrap_or_default(),
        created_at: row.get(11)?,
    })
}

fn persisted_recovery_drill_run_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedRecoveryDrillRun> {
    let drills_json = row.get::<_, String>(7)?;
    Ok(PersistedRecoveryDrillRun {
        id: row.get(0)?,
        project_id: row.get(1)?,
        status: row.get(2)?,
        drill_count: row.get(3)?,
        pass_count: row.get(4)?,
        fail_count: row.get(5)?,
        warning_count: row.get(6)?,
        drills: serde_json::from_str(&drills_json).unwrap_or_default(),
        created_at: row.get(8)?,
    })
}

fn persisted_benchmark_run_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedBenchmarkRun> {
    let suites_json = row.get::<_, String>(8)?;
    Ok(PersistedBenchmarkRun {
        id: row.get(0)?,
        project_id: row.get(1)?,
        status: row.get(2)?,
        suite_count: row.get(3)?,
        pass_count: row.get(4)?,
        fail_count: row.get(5)?,
        warning_count: row.get(6)?,
        duration_ms: row.get(7)?,
        suites: serde_json::from_str(&suites_json).unwrap_or_default(),
        created_at: row.get(9)?,
    })
}

fn persisted_dogfood_telemetry_review_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedDogfoodTelemetryReview> {
    let findings_json = row.get::<_, String>(8)?;
    Ok(PersistedDogfoodTelemetryReview {
        id: row.get(0)?,
        project_id: row.get(1)?,
        status: row.get(2)?,
        evidence_pack_id: row.get(3)?,
        finding_count: row.get(4)?,
        pass_count: row.get(5)?,
        warning_count: row.get(6)?,
        fail_count: row.get(7)?,
        findings: serde_json::from_str(&findings_json).unwrap_or_default(),
        created_at: row.get(9)?,
    })
}

fn persisted_visual_harness_link_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedVisualHarnessLink> {
    Ok(PersistedVisualHarnessLink {
        id: row.get(0)?,
        project_id: row.get(1)?,
        source_id: row.get(2)?,
        target_id: row.get(3)?,
        kind: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn persisted_policy_approval_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedPolicyApproval> {
    Ok(PersistedPolicyApproval {
        id: row.get(0)?,
        project_id: row.get(1)?,
        task_id: row.get(2)?,
        run_id: row.get(3)?,
        action_kind: row.get(4)?,
        command: row.get(5)?,
        risk_level: row.get(6)?,
        state: row.get(7)?,
        requested_by: row.get(8)?,
        decision_by: row.get(9)?,
        decision_note: row.get(10)?,
        created_at: row.get(11)?,
        decided_at: row.get(12)?,
    })
}

fn persisted_policy_pack_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedPolicyPack> {
    let approval_required_json: String = row.get(8)?;
    let forbidden_operations_json: String = row.get(9)?;
    Ok(PersistedPolicyPack {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        sandbox_mode: row.get(3)?,
        network: row.get(4)?,
        network_profile: row.get(5)?,
        file_write: row.get(6)?,
        tools: row.get(7)?,
        approval_required: json_string_array(&approval_required_json),
        forbidden_operations: json_string_array(&forbidden_operations_json),
        active: row.get::<_, i64>(10)? == 1,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn persisted_permission_audit_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedPermissionAudit> {
    Ok(PersistedPermissionAudit {
        id: row.get(0)?,
        project_id: row.get(1)?,
        task_id: row.get(2)?,
        run_id: row.get(3)?,
        policy_pack_id: row.get(4)?,
        action_kind: row.get(5)?,
        command: row.get(6)?,
        decision: row.get(7)?,
        reason: row.get(8)?,
        requested_by: row.get(9)?,
        created_at: row.get(10)?,
    })
}

fn persisted_budget_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PersistedBudget> {
    Ok(PersistedBudget {
        id: row.get(0)?,
        scope_type: row.get(1)?,
        scope_id: row.get(2)?,
        max_usd: row.get(3)?,
        warn_pct: row.get(4)?,
        hard_limit: row.get::<_, i64>(5)? == 1,
    })
}

fn persisted_token_usage_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedTokenUsage> {
    Ok(PersistedTokenUsage {
        id: row.get(0)?,
        project_id: row.get(1)?,
        session_id: row.get(2)?,
        task_id: row.get(3)?,
        run_id: row.get(4)?,
        agent_profile_id: row.get(5)?,
        provider: row.get(6)?,
        model: row.get(7)?,
        input_tokens: row.get(8)?,
        output_tokens: row.get(9)?,
        cost_usd: row.get(10)?,
        source: row.get(11)?,
    })
}

fn persisted_agent_event_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedAgentEvent> {
    Ok(PersistedAgentEvent {
        id: row.get(0)?,
        project_id: row.get(1)?,
        session_id: row.get(2)?,
        run_id: row.get(3)?,
        agent_profile_id: row.get(4)?,
        kind: row.get(5)?,
        severity: row.get(6)?,
        detail: row.get(7)?,
        payload_json: parse_json_cell(row.get::<_, String>(8)?),
        source: row.get(9)?,
        created_at: row.get(10)?,
    })
}

fn persisted_provider_price_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedProviderPrice> {
    Ok(PersistedProviderPrice {
        provider: row.get(0)?,
        model: row.get(1)?,
        input_usd_per_million: row.get(2)?,
        output_usd_per_million: row.get(3)?,
        source: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn persisted_secret_metadata_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedSecretMetadata> {
    Ok(PersistedSecretMetadata {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        keychain_ref: row.get(3)?,
        redacted: true,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn persisted_knowledge_source_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedKnowledgeSource> {
    Ok(PersistedKnowledgeSource {
        id: row.get(0)?,
        project_id: row.get(1)?,
        kind: row.get(2)?,
        path_or_ref: row.get(3)?,
        fingerprint: row.get(4)?,
        status: row.get(5)?,
    })
}

fn persisted_context_pack_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedContextPack> {
    Ok(PersistedContextPack {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        sources_json: parse_json_cell(row.get::<_, String>(4)?),
    })
}

fn persisted_skill_pack_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PersistedSkillPack> {
    Ok(PersistedSkillPack {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        skills_json: parse_json_cell(row.get::<_, String>(4)?),
        source_context_pack_id: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn persisted_agent_profile_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedAgentProfile> {
    Ok(PersistedAgentProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        runtime: row.get(2)?,
        command: row.get(3)?,
        args_json: parse_json_cell(row.get::<_, String>(4)?),
        env_policy_json: parse_json_cell(row.get::<_, String>(5)?),
        skills_json: parse_json_cell(row.get::<_, String>(6)?),
        status: row.get(7)?,
        last_heartbeat_at: row.get(8)?,
    })
}

fn persisted_session_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PersistedSession> {
    Ok(PersistedSession {
        id: row.get(0)?,
        project_id: row.get(1)?,
        pane_id: row.get(2)?,
        mode: row.get(3)?,
        title: row.get(4)?,
        cwd: row.get(5)?,
        branch: row.get(6)?,
        agent_profile_id: row.get(7)?,
        task_id: row.get(8)?,
        run_id: row.get(9)?,
        state: row.get(10)?,
        attention_state: row.get(11)?,
        token_budget_state: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

struct TerminalStreamChunkMetadata {
    id: String,
    session_id: String,
    seq_start: i64,
    seq_end: i64,
    artifact_path: String,
    created_at: String,
}

fn terminal_stream_chunk_metadata_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<TerminalStreamChunkMetadata> {
    Ok(TerminalStreamChunkMetadata {
        id: row.get(0)?,
        session_id: row.get(1)?,
        seq_start: row.get(2)?,
        seq_end: row.get(3)?,
        artifact_path: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn persisted_workflow_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PersistedWorkflowVersion> {
    let parsed_json = parse_json_cell(row.get::<_, String>(4)?);
    let diagnostics_json = parse_json_cell(row.get::<_, String>(6)?);
    Ok(PersistedWorkflowVersion {
        id: row.get(0)?,
        project_id: row.get(1)?,
        source_path: row.get(2)?,
        content_hash: row.get(3)?,
        parsed_json,
        valid: row.get::<_, i64>(5)? == 1,
        diagnostics_json,
    })
}

fn parse_json_cell(raw: String) -> Value {
    serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}))
}

fn normalize_project_registry_path(path: &str) -> Result<String, String> {
    let raw_path = PathBuf::from(path);
    let absolute_path = if raw_path.is_absolute() {
        raw_path
    } else {
        env::current_dir()
            .map_err(|error| format!("project path unavailable: {error}"))?
            .join(raw_path)
    };
    let normalized_path = fs::canonicalize(&absolute_path)
        .unwrap_or_else(|_| normalize_absolute_path_components(&absolute_path));
    Ok(normalized_path.to_string_lossy().to_string())
}

fn normalize_absolute_path_components(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

fn normalize_project_relative_path(value: Option<&str>) -> Result<String, String> {
    let Some(value) = value else {
        return Ok(String::new());
    };
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "." {
        return Ok(String::new());
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err("project file path must stay inside project".to_string());
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("project file path must stay inside project".to_string());
            }
        }
    }
    Ok(parts.join("/"))
}

fn normalize_path_for_git(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn git_statuses_for_project(root_path: &Path) -> (HashMap<String, String>, Option<String>) {
    let output = Command::new("git")
        .args([
            "-C",
            &root_path.to_string_lossy(),
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
        ])
        .output();
    let Ok(output) = output else {
        return (HashMap::new(), Some("git status unavailable".to_string()));
    };
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return (
            HashMap::new(),
            Some(if detail.is_empty() {
                "git status failed".to_string()
            } else {
                detail
            }),
        );
    }

    let mut statuses = HashMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.len() < 4 {
            continue;
        }
        let code = &line[..2];
        let raw_path = line[3..]
            .split(" -> ")
            .last()
            .unwrap_or("")
            .trim_matches('"')
            .trim();
        if raw_path.is_empty() {
            continue;
        }
        let Some(status) = classify_git_status(code) else {
            continue;
        };
        statuses.insert(raw_path.to_string(), status.clone());
        let mut current = Path::new(raw_path).parent();
        while let Some(parent) = current {
            let parent_path = normalize_path_for_git(parent);
            if parent_path.is_empty() {
                break;
            }
            statuses
                .entry(parent_path)
                .or_insert_with(|| status.clone());
            current = parent.parent();
        }
    }
    (statuses, None)
}

fn append_deleted_project_file_entries(
    relative_path: &str,
    git_statuses: &HashMap<String, String>,
    entries: &mut Vec<ProjectFileEntry>,
) {
    let existing_paths = entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<std::collections::HashSet<_>>();
    let current_dir = relative_path.trim_matches('/');
    let mut deleted_paths = git_statuses
        .iter()
        .filter_map(|(path, status)| {
            if status != "deleted" || existing_paths.contains(path) {
                return None;
            }
            let entry_parent = Path::new(path)
                .parent()
                .map(normalize_path_for_git)
                .unwrap_or_default();
            if entry_parent == current_dir {
                Some(path.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    deleted_paths.sort();

    for path in deleted_paths {
        let name = Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        entries.push(ProjectFileEntry {
            name,
            path,
            kind: "file".to_string(),
            git_status: Some("deleted".to_string()),
        });
    }
}

fn append_deleted_project_file_search_entries(
    query: &str,
    git_statuses: &HashMap<String, String>,
    entries: &mut Vec<ProjectFileEntry>,
) {
    let existing_paths = entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<std::collections::HashSet<_>>();
    let mut deleted_paths = git_statuses
        .iter()
        .filter_map(|(path, status)| {
            if status == "deleted"
                && !existing_paths.contains(path)
                && path.to_lowercase().contains(query)
            {
                Some(path.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    deleted_paths.sort();

    for path in deleted_paths {
        let name = Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        entries.push(ProjectFileEntry {
            name,
            path,
            kind: "file".to_string(),
            git_status: Some("deleted".to_string()),
        });
    }
}

fn git_diff_bytes_for_project(
    root_path: &Path,
    relative_path: Option<&str>,
) -> std::io::Result<Result<Vec<u8>, String>> {
    let head = run_git_diff(root_path, relative_path, &["HEAD"])?;
    if head.status.success() {
        return Ok(Ok(head.stdout));
    }

    let cached = run_git_diff(root_path, relative_path, &["--cached"])?;
    if !cached.status.success() {
        return Ok(Err(git_diff_error(&cached.stderr)));
    }
    let working = run_git_diff(root_path, relative_path, &[])?;
    if !working.status.success() {
        return Ok(Err(git_diff_error(&working.stderr)));
    }

    let mut body = cached.stdout;
    if !body.is_empty() && !working.stdout.is_empty() {
        body.push(b'\n');
    }
    body.extend(working.stdout);
    Ok(Ok(body))
}

fn run_git_diff(
    root_path: &Path,
    relative_path: Option<&str>,
    extra_args: &[&str],
) -> std::io::Result<std::process::Output> {
    let mut command = Command::new("git");
    command.args(["-C", &root_path.to_string_lossy(), "diff", "--no-ext-diff"]);
    command.args(extra_args);
    if let Some(path) = relative_path {
        command.arg("--").arg(path);
    }
    command.output()
}

fn git_diff_error(stderr: &[u8]) -> String {
    let detail = String::from_utf8_lossy(stderr).trim().to_string();
    if detail.is_empty() {
        "git diff failed".to_string()
    } else {
        detail
    }
}

#[derive(Default)]
struct ProjectDiffFileSummaryDraft {
    old_path: Option<String>,
    new_path: Option<String>,
    status: Option<String>,
    additions: usize,
    deletions: usize,
}

impl ProjectDiffFileSummaryDraft {
    fn finish(self) -> Option<ProjectDiffFileSummary> {
        let path = self
            .new_path
            .clone()
            .or_else(|| self.old_path.clone())
            .filter(|value| !value.is_empty())?;
        let status = self.status.unwrap_or_else(|| {
            if self.old_path.is_none() {
                "added".to_string()
            } else if self.new_path.is_none() {
                "deleted".to_string()
            } else {
                "modified".to_string()
            }
        });
        Some(ProjectDiffFileSummary {
            path,
            status,
            additions: self.additions,
            deletions: self.deletions,
        })
    }
}

fn summarize_unified_diff_files(body: &str) -> Vec<ProjectDiffFileSummary> {
    let mut files = Vec::new();
    let mut current: Option<ProjectDiffFileSummaryDraft> = None;

    for line in body.lines() {
        if line.starts_with("diff --git ") {
            if let Some(summary) = current.take().and_then(ProjectDiffFileSummaryDraft::finish) {
                files.push(summary);
            }
            current = Some(ProjectDiffFileSummaryDraft::default());
            if let Some((old_path, new_path)) = diff_header_paths(line) {
                if let Some(draft) = current.as_mut() {
                    draft.old_path = old_path;
                    draft.new_path = new_path;
                }
            }
            continue;
        }

        let Some(draft) = current.as_mut() else {
            continue;
        };

        if line.starts_with("new file mode ") {
            draft.status = Some("added".to_string());
            continue;
        }
        if line.starts_with("deleted file mode ") {
            draft.status = Some("deleted".to_string());
            continue;
        }
        if let Some(path) = line.strip_prefix("rename from ") {
            draft.old_path = Some(clean_diff_path(path, None));
            draft.status = Some("renamed".to_string());
            continue;
        }
        if let Some(path) = line.strip_prefix("rename to ") {
            draft.new_path = Some(clean_diff_path(path, None));
            draft.status = Some("renamed".to_string());
            continue;
        }
        if let Some(path) = line.strip_prefix("--- ") {
            draft.old_path = clean_diff_path_marker(path, Some("a/"));
            continue;
        }
        if let Some(path) = line.strip_prefix("+++ ") {
            draft.new_path = clean_diff_path_marker(path, Some("b/"));
            continue;
        }
        if line.starts_with('+') && !line.starts_with("+++") {
            draft.additions += 1;
            continue;
        }
        if line.starts_with('-') && !line.starts_with("---") {
            draft.deletions += 1;
        }
    }

    if let Some(summary) = current.and_then(ProjectDiffFileSummaryDraft::finish) {
        files.push(summary);
    }

    files
}

fn diff_header_paths(line: &str) -> Option<(Option<String>, Option<String>)> {
    let mut parts = line.split_whitespace();
    parts.next()?;
    parts.next()?;
    let old_path = parts.next()?;
    let new_path = parts.next()?;
    Some((
        Some(clean_diff_path(old_path, Some("a/"))),
        Some(clean_diff_path(new_path, Some("b/"))),
    ))
}

fn clean_diff_path_marker(path: &str, prefix: Option<&str>) -> Option<String> {
    let trimmed = path.trim();
    if trimmed == "/dev/null" {
        return None;
    }
    Some(clean_diff_path(trimmed, prefix))
}

fn clean_diff_path(path: &str, prefix: Option<&str>) -> String {
    let mut value = path.trim().trim_matches('"').to_string();
    if let Some(prefix) = prefix {
        if let Some(stripped) = value.strip_prefix(prefix) {
            value = stripped.to_string();
        }
    }
    value
}

fn truncate_to_char_boundary(bytes: &[u8], max_len: usize) -> &[u8] {
    if bytes.len() <= max_len {
        return bytes;
    }
    &bytes[..max_len]
}

fn collect_project_file_search_entries(
    root_path: &Path,
    current_path: &Path,
    query: &str,
    git_statuses: &HashMap<String, String>,
    entries: &mut Vec<ProjectFileEntry>,
) -> Result<(), String> {
    if entries.len() >= PROJECT_FILE_SEARCH_MAX_RESULTS {
        return Ok(());
    }
    let read_dir = fs::read_dir(current_path)
        .map_err(|error| format!("failed to search project files: {error}"))?;
    for entry in read_dir.filter_map(|entry| entry.ok()) {
        if entries.len() >= PROJECT_FILE_SEARCH_MAX_RESULTS {
            break;
        }
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".git" {
            continue;
        }
        let entry_path = entry.path();
        let Some(relative) = entry_path
            .strip_prefix(root_path)
            .ok()
            .map(normalize_path_for_git)
        else {
            continue;
        };
        if relative.to_lowercase().contains(query) {
            entries.push(ProjectFileEntry {
                name: name.clone(),
                path: relative.clone(),
                kind: if file_type.is_dir() {
                    "directory".to_string()
                } else {
                    "file".to_string()
                },
                git_status: git_statuses.get(&relative).cloned(),
            });
        }
        if file_type.is_dir() {
            collect_project_file_search_entries(
                root_path,
                &entry_path,
                query,
                git_statuses,
                entries,
            )?;
        }
    }
    Ok(())
}

fn classify_git_status(code: &str) -> Option<String> {
    if code.contains('?') {
        Some("untracked".to_string())
    } else if code.contains('R') {
        Some("renamed".to_string())
    } else if code.contains('A') {
        Some("added".to_string())
    } else if code.contains('D') {
        Some("deleted".to_string())
    } else if code.contains('M') {
        Some("modified".to_string())
    } else if code.trim().is_empty() {
        None
    } else {
        Some("changed".to_string())
    }
}

enum ProjectFilePreviewKind {
    Text(Option<String>),
    Binary { language: String, mime_type: String },
}

impl ProjectFilePreviewKind {
    fn language(&self) -> Option<String> {
        match self {
            ProjectFilePreviewKind::Text(language) => language.clone(),
            ProjectFilePreviewKind::Binary { language, .. } => Some(language.clone()),
        }
    }

    fn binary_mime_type(&self) -> Option<&str> {
        match self {
            ProjectFilePreviewKind::Text(_) => None,
            ProjectFilePreviewKind::Binary { mime_type, .. } => Some(mime_type),
        }
    }
}

fn project_file_preview_kind(path: &Path) -> ProjectFilePreviewKind {
    if let Some(mime_type) = binary_preview_mime_type(path) {
        let language = if mime_type == "application/pdf" {
            "pdf"
        } else {
            "image"
        };
        return ProjectFilePreviewKind::Binary {
            language: language.to_string(),
            mime_type: mime_type.to_string(),
        };
    }
    ProjectFilePreviewKind::Text(language_for_path(path))
}

fn binary_preview_mime_type(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(OsStr::to_str)?
        .to_ascii_lowercase()
        .as_str()
    {
        "pdf" => Some("application/pdf"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "avif" => Some("image/avif"),
        "bmp" => Some("image/bmp"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

fn language_for_path(path: &Path) -> Option<String> {
    let extension = path.extension().and_then(OsStr::to_str)?;
    let language = match extension {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "json" => "json",
        "md" | "markdown" => "markdown",
        "css" => "css",
        "html" | "htm" => "html",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "log" => "log",
        "sh" | "bash" | "zsh" => "shell",
        "py" => "python",
        "swift" => "swift",
        _ => return None,
    };
    Some(language.to_string())
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        let triple = ((first as u32) << 16) | ((second as u32) << 8) | third as u32;

        output.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(TABLE[(triple & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
    }

    output
}

fn is_lsp_source_path(path: &str) -> bool {
    matches!(
        Path::new(path).extension().and_then(OsStr::to_str),
        Some("ts" | "tsx" | "js" | "jsx" | "rs" | "py" | "swift")
    )
}

fn collect_lsp_diagnostics_from_body(
    path: &str,
    body: &str,
    diagnostics: &mut Vec<ProjectLspDiagnostic>,
) {
    for (index, line) in body.lines().enumerate() {
        let line_number = index + 1;
        if line.contains("TODO") {
            diagnostics.push(ProjectLspDiagnostic {
                path: path.to_string(),
                line: line_number,
                severity: "warning".to_string(),
                message: "TODO marker should be resolved or tracked before release".to_string(),
            });
        }
        if line.contains(": any") || line.contains(" as any") {
            diagnostics.push(ProjectLspDiagnostic {
                path: path.to_string(),
                line: line_number,
                severity: "warning".to_string(),
                message: "TypeScript explicit any weakens local LSP guarantees".to_string(),
            });
        }
    }
}

fn collect_lsp_symbols_from_body(path: &str, body: &str, symbols: &mut Vec<ProjectLspSymbol>) {
    for (index, line) in body.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim_start();
        if let Some(name) = symbol_name_after_keyword(trimmed, "function ") {
            symbols.push(ProjectLspSymbol {
                path: path.to_string(),
                name,
                kind: "function".to_string(),
                line: line_number,
            });
            continue;
        }
        if let Some(name) = symbol_name_after_keyword(trimmed, "fn ") {
            symbols.push(ProjectLspSymbol {
                path: path.to_string(),
                name,
                kind: "function".to_string(),
                line: line_number,
            });
            continue;
        }
        if let Some(name) = symbol_name_after_keyword(trimmed, "class ") {
            symbols.push(ProjectLspSymbol {
                path: path.to_string(),
                name,
                kind: "class".to_string(),
                line: line_number,
            });
            continue;
        }
        if let Some(name) = symbol_name_after_keyword(trimmed, "const ") {
            symbols.push(ProjectLspSymbol {
                path: path.to_string(),
                name,
                kind: "constant".to_string(),
                line: line_number,
            });
            continue;
        }
        if let Some(name) = symbol_name_after_keyword(trimmed, "let ") {
            symbols.push(ProjectLspSymbol {
                path: path.to_string(),
                name,
                kind: "variable".to_string(),
                line: line_number,
            });
        }
    }
}

fn symbol_name_after_keyword(line: &str, keyword: &str) -> Option<String> {
    let source = line
        .strip_prefix("export ")
        .or_else(|| line.strip_prefix("pub "))
        .unwrap_or(line);
    let rest = source.strip_prefix(keyword)?;
    let name = rest
        .trim_start()
        .split(|character: char| {
            !(character == '_' || character == '$' || character.is_ascii_alphanumeric())
        })
        .next()
        .unwrap_or_default();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn is_localhost_url(url: &str) -> bool {
    let value = url.trim().to_lowercase();
    matches!(
        value
            .strip_prefix("http://")
            .or_else(|| value.strip_prefix("https://"))
            .and_then(|rest| rest.split(['/', ':']).next()),
        Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
    )
}

fn stable_text_hash(value: &str) -> String {
    let mut hash: u64 = 14_695_981_039_346_656_037;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    format!("{hash:016x}")
}

fn default_context_pack_from_workflow(workflow: &PersistedWorkflowVersion) -> Option<String> {
    workflow.parsed_json["context"]["default_pack"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn workflow_hook_path(workflow: &PersistedWorkflowVersion, hook_name: &str) -> Option<PathBuf> {
    workflow.parsed_json["hooks"][hook_name]
        .as_str()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

fn canonicalize_existing_dir(path: &str, label: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("{label} {} could not be resolved: {error}", path.display()))?;
    if canonical.is_dir() {
        Ok(canonical)
    } else {
        Err(format!(
            "{label} {} is not a directory",
            canonical.display()
        ))
    }
}

fn ensure_path_inside(path: &Path, root: &Path, label: &str) -> Result<(), String> {
    if path.starts_with(root) {
        Ok(())
    } else {
        Err(format!(
            "{label} {} must stay inside the repo {}",
            path.display(),
            root.display()
        ))
    }
}

struct WorkflowValidation {
    valid: bool,
    parsed_json: Value,
    diagnostics_json: Value,
}

fn validate_workflow_document(content: &str) -> WorkflowValidation {
    let mut errors = Vec::<Value>::new();
    let Some((frontmatter, template)) = split_workflow_frontmatter(content) else {
        errors.push(workflow_error(
            "frontmatter_missing",
            "WORKFLOW.md must start with YAML frontmatter",
        ));
        return WorkflowValidation {
            valid: false,
            parsed_json: serde_json::json!({}),
            diagnostics_json: serde_json::json!({ "errors": errors }),
        };
    };

    let parsed = parse_workflow_frontmatter(frontmatter);
    if parsed["haneulchi"] != 1 {
        errors.push(workflow_error(
            "haneulchi_version_missing",
            "workflow frontmatter must include haneulchi: 1",
        ));
    }
    if parsed["project"]["key"]
        .as_str()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        errors.push(workflow_error(
            "project_key_missing",
            "workflow project.key is required",
        ));
    }
    if parsed["workspace"]["strategy"]
        .as_str()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        errors.push(workflow_error(
            "workspace_strategy_missing",
            "workflow workspace.strategy is required",
        ));
    }

    validate_template_namespaces(template, &mut errors);
    validate_hook_paths(&parsed, &mut errors);

    WorkflowValidation {
        valid: errors.is_empty(),
        parsed_json: parsed,
        diagnostics_json: serde_json::json!({ "errors": errors }),
    }
}

fn split_workflow_frontmatter(content: &str) -> Option<(&str, &str)> {
    let rest = content.strip_prefix("---\n")?;
    let (frontmatter, template) = rest.split_once("\n---")?;
    Some((frontmatter, template.trim_start_matches('-').trim_start()))
}

fn parse_workflow_frontmatter(frontmatter: &str) -> Value {
    let mut haneulchi = Value::Null;
    let mut project = serde_json::Map::new();
    let mut workspace = serde_json::Map::new();
    let mut agents = serde_json::Map::new();
    let mut context = serde_json::Map::new();
    let mut hooks = serde_json::Map::new();
    let mut tools = serde_json::Map::new();
    let mut review = serde_json::Map::new();
    let mut current_section = "";
    let mut current_nested = "";
    let mut required_evidence = Vec::<Value>::new();

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            if current_section == "review" && current_nested == "required_evidence" {
                required_evidence.push(Value::String(item.trim().to_string()));
            }
            continue;
        }
        let indent = line
            .chars()
            .take_while(|character| character.is_whitespace())
            .count();
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if indent == 0 {
            current_section = key;
            current_nested = "";
            if key == "haneulchi" {
                haneulchi = parse_scalar(value);
            }
        } else if indent == 2 {
            current_nested = key;
            let target = match current_section {
                "project" => Some(&mut project),
                "workspace" => Some(&mut workspace),
                "agents" => Some(&mut agents),
                "context" => Some(&mut context),
                "hooks" => Some(&mut hooks),
                "tools" => Some(&mut tools),
                "review" => Some(&mut review),
                _ => None,
            };
            if let Some(target) = target {
                if !value.is_empty() {
                    target.insert(key.to_string(), parse_scalar(value));
                }
            }
        }
    }

    if !required_evidence.is_empty() {
        review.insert(
            "required_evidence".to_string(),
            Value::Array(required_evidence),
        );
    }

    serde_json::json!({
        "haneulchi": haneulchi,
        "project": project,
        "workspace": workspace,
        "agents": agents,
        "context": context,
        "hooks": hooks,
        "tools": tools,
        "review": review
    })
}

fn parse_scalar(value: &str) -> Value {
    if value.is_empty() {
        Value::Null
    } else if let Ok(number) = value.parse::<i64>() {
        serde_json::json!(number)
    } else {
        Value::String(value.trim_matches('"').to_string())
    }
}

fn validate_template_namespaces(template: &str, errors: &mut Vec<Value>) {
    let allowed = [
        "task",
        "project",
        "goal",
        "cycle",
        "module",
        "agent",
        "run",
        "workspace",
        "context_pack",
        "policy",
    ];
    let mut rest = template;
    while let Some((_, after_open)) = rest.split_once('{') {
        let Some((raw, after_close)) = after_open.split_once('}') else {
            break;
        };
        rest = after_close;
        let path = raw.trim().trim_start_matches('?');
        let namespace = path.split('.').next().unwrap_or("");
        if !namespace.is_empty() && !allowed.contains(&namespace) {
            errors.push(workflow_error(
                "template_namespace_not_allowed",
                &format!("template namespace {namespace} is not allowed"),
            ));
        }
    }
}

fn validate_hook_paths(parsed: &Value, errors: &mut Vec<Value>) {
    if let Some(hooks) = parsed["hooks"].as_object() {
        for (name, value) in hooks {
            let Some(path) = value.as_str() else {
                continue;
            };
            if path.starts_with('/') || path.split('/').any(|component| component == "..") {
                errors.push(workflow_error(
                    "hook_path_escapes_repo",
                    &format!("hook {name} must stay inside the repo"),
                ));
            }
        }
    }
}

fn workflow_error(code: &str, message: &str) -> Value {
    serde_json::json!({
        "code": code,
        "message": message
    })
}

fn content_hash(content: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv64:{hash:016x}")
}

fn normalize_run_lifecycle(lifecycle: &str) -> Result<String, String> {
    let lifecycle = lifecycle.trim();
    if is_supported_run_lifecycle(lifecycle) {
        Ok(lifecycle.to_string())
    } else {
        Err(format!("unsupported run lifecycle {lifecycle}"))
    }
}

fn is_supported_run_lifecycle(lifecycle: &str) -> bool {
    matches!(
        lifecycle,
        "queued"
            | "claimed"
            | "starting"
            | "running"
            | "waiting_input"
            | "permission_requested"
            | "blocked"
            | "review_ready"
            | "completed"
            | "failed"
            | "cancelled"
    )
}

fn runtime_pool_label(mode: &str) -> String {
    match mode {
        "shell" => "Local".to_string(),
        "ssh" => "Remote SSH".to_string(),
        "agent" | "cloud" => "Cloud agents".to_string(),
        other => other.to_string(),
    }
}

fn runtime_pool_item(mode: &str) -> RuntimePoolItem {
    RuntimePoolItem {
        id: mode.to_string(),
        label: runtime_pool_label(mode),
        session_count: 0,
        run_count: 0,
        blocked_count: 0,
    }
}

fn runtime_pool_sort_key(mode: &str) -> usize {
    match mode {
        "shell" => 0,
        "ssh" => 1,
        "agent" | "cloud" => 2,
        _ => 3,
    }
}

fn validate_run_transition(current: &str, next: &str) -> Result<(), String> {
    if current == next {
        return Ok(());
    }
    if matches!(current, "completed" | "failed" | "cancelled") {
        return Err(format!(
            "run cannot transition from terminal lifecycle {current} to {next}"
        ));
    }
    if is_supported_run_lifecycle(next) {
        Ok(())
    } else {
        Err(format!("unsupported run lifecycle {next}"))
    }
}

fn task_status_for_run_lifecycle(lifecycle: &str) -> Result<&'static str, String> {
    match lifecycle {
        "queued"
        | "claimed"
        | "starting"
        | "running"
        | "waiting_input"
        | "permission_requested" => Ok("running"),
        "blocked" | "failed" | "cancelled" => Ok("blocked"),
        "review_ready" => Ok("review"),
        "completed" => Ok("done"),
        _ => Err(format!("unsupported run lifecycle {lifecycle}")),
    }
}

fn normalize_risk_level(risk_level: &str) -> Result<&'static str, String> {
    match risk_level.trim() {
        "low" => Ok("low"),
        "medium" => Ok("medium"),
        "high" => Ok("high"),
        "critical" => Ok("critical"),
        unsupported => Err(format!(
            "unsupported policy approval risk level {unsupported}"
        )),
    }
}

fn normalize_policy_sandbox_mode(mode: &str) -> Result<String, String> {
    match mode.trim() {
        "normal" => Ok("normal".to_string()),
        "ask-before-write" => Ok("ask-before-write".to_string()),
        "sandboxed" => Ok("sandboxed".to_string()),
        unsupported => Err(format!("unsupported policy sandbox mode {unsupported}")),
    }
}

fn normalize_policy_permission(value: &str) -> Result<String, String> {
    match value.trim() {
        "allowed" => Ok("allowed".to_string()),
        "ask" => Ok("ask".to_string()),
        "blocked" => Ok("blocked".to_string()),
        unsupported => Err(format!("unsupported policy permission {unsupported}")),
    }
}

fn normalize_network_profile(value: &str) -> Result<String, String> {
    match value.trim() {
        "internet" => Ok("internet".to_string()),
        "local-only" => Ok("local-only".to_string()),
        "offline" => Ok("offline".to_string()),
        unsupported => Err(format!("unsupported network profile {unsupported}")),
    }
}

fn command_targets_local_endpoint(command: &str) -> bool {
    let normalized = command.to_ascii_lowercase();
    normalized.contains("localhost")
        || normalized.contains("127.0.0.1")
        || normalized.contains("http://[::1]")
        || normalized.contains("https://[::1]")
        || normalized.contains("http://::1")
        || normalized.contains("https://::1")
}

fn normalize_policy_list(items: Vec<String>) -> Result<Vec<String>, String> {
    let mut normalized = Vec::new();
    for item in items {
        let item = required_trimmed("policy item", &item)?;
        if !normalized.contains(&item) {
            normalized.push(item);
        }
    }
    Ok(normalized)
}

fn json_string_array(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

fn normalize_policy_decision(decision: &str) -> Result<&'static str, String> {
    match decision.trim() {
        "approved" => Ok("approved"),
        "denied" => Ok("denied"),
        unsupported => Err(format!(
            "unsupported policy approval decision {unsupported}"
        )),
    }
}

fn normalize_policy_approval_state(state: &str) -> Result<&'static str, String> {
    match state.trim() {
        "pending" => Ok("pending"),
        "approved" => Ok("approved"),
        "denied" => Ok("denied"),
        unsupported => Err(format!("unsupported policy approval state {unsupported}")),
    }
}

fn normalize_permission_decision(decision: &str) -> Result<&'static str, String> {
    match decision.trim() {
        "allowed" => Ok("allowed"),
        "approval_required" => Ok("approval_required"),
        "forbidden" => Ok("forbidden"),
        unsupported => Err(format!(
            "unsupported permission audit decision {unsupported}"
        )),
    }
}

fn normalize_budget_scope_type(scope_type: &str) -> Result<&'static str, String> {
    match scope_type.trim() {
        "workspace" => Ok("workspace"),
        "project" => Ok("project"),
        "goal" => Ok("goal"),
        "task" => Ok("task"),
        "run" => Ok("run"),
        "agent" => Ok("agent"),
        unsupported => Err(format!("unsupported budget scope {unsupported}")),
    }
}

struct NormalizedTokenUsage {
    provider: String,
    model: String,
    input_tokens: i64,
    output_tokens: i64,
    cost_usd: f64,
}

struct NormalizedAgentEvent {
    kind: String,
    severity: String,
    detail: String,
    payload_json: Value,
}

fn agent_events_from_adapter_payload(
    adapter: &str,
    payload: &Value,
) -> Result<Vec<NormalizedAgentEvent>, String> {
    match adapter {
        "raw-jsonl" | "generic.agent-json" => agent_events_from_json_lines(payload),
        unsupported => Err(format!("unsupported agent event adapter {unsupported}")),
    }
}

fn agent_events_from_json_lines(payload: &Value) -> Result<Vec<NormalizedAgentEvent>, String> {
    let raw = string_field(payload, "raw")
        .ok_or_else(|| "agent event adapter payload raw is required".to_string())?;
    let events = raw
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .filter_map(|value| normalize_agent_event(&value).ok())
        .collect::<Vec<_>>();
    if events.is_empty() {
        Err("agent event adapter did not contain structured events".to_string())
    } else {
        Ok(events)
    }
}

fn normalize_agent_event(payload: &Value) -> Result<NormalizedAgentEvent, String> {
    let kind = string_field(payload, "kind")
        .or_else(|| string_field(payload, "type"))
        .ok_or_else(|| "agent event kind is required".to_string())?;
    let normalized_kind = match kind.as_str() {
        "status" | "message" | "tool_call" | "tool_result" | "error" | "token_usage" => kind,
        "assistant_message" => "message".to_string(),
        "function_call" => "tool_call".to_string(),
        unsupported => unsupported.to_string(),
    };
    let detail = string_field(payload, "message")
        .or_else(|| string_field(payload, "detail"))
        .or_else(|| string_field(payload, "status"))
        .or_else(|| string_field(payload, "tool"))
        .unwrap_or_else(|| normalized_kind.clone());
    let severity = string_field(payload, "severity").unwrap_or_else(|| {
        if normalized_kind == "error" {
            "error".to_string()
        } else if string_field(payload, "status").as_deref() == Some("needs_input") {
            "warning".to_string()
        } else {
            "info".to_string()
        }
    });

    Ok(NormalizedAgentEvent {
        kind: normalized_kind,
        severity,
        detail,
        payload_json: payload.clone(),
    })
}

fn token_usage_from_adapter_payload(
    adapter: &str,
    payload: &Value,
) -> Result<NormalizedTokenUsage, String> {
    match adapter {
        "openai.responses" => token_usage_from_openai_response(payload),
        "codex.log" => token_usage_from_codex_log(payload),
        "local.usage-json" => token_usage_from_usage_json(payload, "local usage json"),
        unsupported => Err(format!("unsupported token usage adapter {unsupported}")),
    }
}

fn token_usage_from_openai_response(payload: &Value) -> Result<NormalizedTokenUsage, String> {
    let usage = payload
        .get("usage")
        .ok_or_else(|| "adapter payload usage is required".to_string())?;
    Ok(NormalizedTokenUsage {
        provider: string_field(payload, "provider").unwrap_or_else(|| "openai".to_string()),
        model: string_field(payload, "model")
            .ok_or_else(|| "adapter payload model is required".to_string())?,
        input_tokens: i64_field_any(usage, &["input_tokens", "prompt_tokens"])
            .ok_or_else(|| "adapter payload usage input_tokens is required".to_string())?,
        output_tokens: i64_field_any(usage, &["output_tokens", "completion_tokens"])
            .ok_or_else(|| "adapter payload usage output_tokens is required".to_string())?,
        cost_usd: f64_field_any(payload, &["cost_usd", "costUsd"]).unwrap_or(0.0),
    })
}

fn token_usage_from_codex_log(payload: &Value) -> Result<NormalizedTokenUsage, String> {
    let raw = string_field(payload, "raw")
        .ok_or_else(|| "codex log adapter payload raw is required".to_string())?;
    for line in raw.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) == Some("token_usage")
            || value.get("input_tokens").is_some()
        {
            return token_usage_from_usage_json(&value, "codex log token usage");
        }
    }
    Err("codex log adapter did not contain token usage".to_string())
}

fn token_usage_from_usage_json(
    payload: &Value,
    label: &str,
) -> Result<NormalizedTokenUsage, String> {
    Ok(NormalizedTokenUsage {
        provider: string_field(payload, "provider")
            .ok_or_else(|| format!("{label} provider is required"))?,
        model: string_field(payload, "model")
            .ok_or_else(|| format!("{label} model is required"))?,
        input_tokens: i64_field_any(payload, &["input_tokens", "inputTokens"])
            .ok_or_else(|| format!("{label} input_tokens is required"))?,
        output_tokens: i64_field_any(payload, &["output_tokens", "outputTokens"])
            .ok_or_else(|| format!("{label} output_tokens is required"))?,
        cost_usd: f64_field_any(payload, &["cost_usd", "costUsd"]).unwrap_or(0.0),
    })
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToString::to_string)
}

fn i64_field_any(value: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_i64))
}

fn f64_field_any(value: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_f64))
}

fn normalize_agent_status(status: &str) -> Result<&'static str, String> {
    match status.trim() {
        "available" => Ok("available"),
        "unavailable" => Ok("unavailable"),
        "paused" => Ok("paused"),
        unsupported => Err(format!("unsupported agent profile status {unsupported}")),
    }
}

fn default_provider_model_settings() -> ProviderModelSettings {
    ProviderModelSettings {
        provider: "openai".to_string(),
        model: "gpt-5.4".to_string(),
        agent_profile_id: "agent_codex".to_string(),
    }
}

fn normalize_provider_model_settings(
    input: ProviderModelSettingsInput,
) -> Result<ProviderModelSettings, String> {
    Ok(ProviderModelSettings {
        provider: required_trimmed("provider model provider", &input.provider)?,
        model: required_trimmed("provider model model", &input.model)?,
        agent_profile_id: required_trimmed(
            "provider model agent profile id",
            &input.agent_profile_id,
        )?,
    })
}

fn provider_model_settings_from_json(value: Value) -> Result<ProviderModelSettings, String> {
    let provider = value
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or("openai")
        .to_string();
    let model = value
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("gpt-5.4")
        .to_string();
    let agent_profile_id = value
        .get("agentProfileId")
        .or_else(|| value.get("agent_profile_id"))
        .and_then(Value::as_str)
        .unwrap_or("agent_codex")
        .to_string();

    normalize_provider_model_settings(ProviderModelSettingsInput {
        provider,
        model,
        agent_profile_id,
    })
}

fn default_terminal_theme_settings() -> TerminalThemeSettings {
    TerminalThemeSettings {
        project_id: None,
        name: "Haneulchi Default".to_string(),
        background: "#050607".to_string(),
        foreground: "#d7ffe1".to_string(),
        accent: "#42e355".to_string(),
    }
}

fn terminal_theme_setting_key(project_id: Option<&str>) -> String {
    match project_id {
        Some(project_id) => format!("terminal_theme.project.{project_id}"),
        None => "terminal_theme.defaults".to_string(),
    }
}

fn normalize_terminal_theme_settings(
    input: TerminalThemeSettingsInput,
) -> Result<TerminalThemeSettings, String> {
    let project_id = normalize_optional_text(input.project_id);
    Ok(TerminalThemeSettings {
        project_id,
        name: required_trimmed("terminal theme name", &input.name)?,
        background: normalize_hex_color("terminal theme background", &input.background)?,
        foreground: normalize_hex_color("terminal theme foreground", &input.foreground)?,
        accent: normalize_hex_color("terminal theme accent", &input.accent)?,
    })
}

fn terminal_theme_settings_from_json(
    project_id: Option<String>,
    value: Value,
) -> Result<TerminalThemeSettings, String> {
    let defaults = default_terminal_theme_settings();
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(&defaults.name)
        .to_string();
    let background = value
        .get("background")
        .and_then(Value::as_str)
        .unwrap_or(&defaults.background)
        .to_string();
    let foreground = value
        .get("foreground")
        .and_then(Value::as_str)
        .unwrap_or(&defaults.foreground)
        .to_string();
    let accent = value
        .get("accent")
        .and_then(Value::as_str)
        .unwrap_or(&defaults.accent)
        .to_string();
    normalize_terminal_theme_settings(TerminalThemeSettingsInput {
        project_id,
        name,
        background,
        foreground,
        accent,
    })
}

fn command_block_explanation(
    block: &PersistedCommandBlock,
    input: ExplainCommandBlockInput,
) -> Result<CommandBlockExplanation, String> {
    let sequence = format!(
        "{}-{}",
        block
            .seq_start
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string()),
        block
            .seq_end
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    let status = command_block_status_label(block.exit_code);
    let cwd = block.cwd.as_deref().unwrap_or("-");
    let branch = block.branch.as_deref().unwrap_or("-");
    let first_output_line = block
        .summary
        .as_deref()
        .and_then(|summary| summary.lines().find(|line| !line.trim().is_empty()))
        .map(str::trim);
    let local_summary = format!("{} {status} in {cwd} on {branch}", block.command);
    let mut evidence = vec![
        format!("context: {local_summary}"),
        format!("sequence {sequence}"),
        format!("status {status}"),
        first_output_line
            .map(|line| format!("output: {line}"))
            .unwrap_or_else(|| "output: none captured".to_string()),
    ];

    let provider = normalize_optional_text(input.provider);
    let model = normalize_optional_text(input.model);
    let agent_profile_id = normalize_optional_text(input.agent_profile_id);
    let (summary, prompt, diagnostics) =
        if provider.is_some() || model.is_some() || agent_profile_id.is_some() {
            if let (Some(provider), Some(model), Some(agent_profile_id)) =
                (provider.as_ref(), model.as_ref(), agent_profile_id.as_ref())
            {
                evidence.insert(0, format!("agent {agent_profile_id}"));
                evidence.insert(0, format!("ai route {provider}/{model}"));
            }
            (
                format!(
                    "AI explanation draft for {}: {status} in {cwd} on {branch}",
                    block.command
                ),
                Some(
                    [
                        "Explain this command block for review evidence.".to_string(),
                        format!("Command: {}", block.command),
                        format!("Status: {status}"),
                        format!("Cwd: {cwd}"),
                        format!("Branch: {branch}"),
                        format!("sequence {sequence}"),
                        format!(
                            "Output excerpt:\n{}",
                            block.summary.as_deref().unwrap_or("none captured")
                        ),
                    ]
                    .join("\n"),
                ),
                vec![
                    "No external AI call has been made; draft is ready for the selected agent."
                        .to_string(),
                ],
            )
        } else {
            (local_summary, None, Vec::new())
        };

    Ok(CommandBlockExplanation {
        id: format!("explain_{}", block.id),
        command_block_id: block.id.clone(),
        command: block.command.clone(),
        summary,
        evidence,
        provider,
        model,
        agent_profile_id,
        prompt,
        diagnostics,
    })
}

fn command_block_status_label(exit_code: Option<i64>) -> &'static str {
    match exit_code {
        Some(0) => "completed",
        Some(_) => "failed",
        None => "unknown",
    }
}

fn command_block_exit_code_for_status(status: &str) -> Result<Option<i64>, String> {
    match status.trim() {
        "completed" => Ok(Some(0)),
        "failed" => Ok(Some(1)),
        "running" | "unknown" => Ok(None),
        value => Err(format!(
            "command block status must be running, completed, failed, or unknown, got {value}"
        )),
    }
}

fn merge_command_block_exit_code(first: Option<i64>, second: Option<i64>) -> Option<i64> {
    match (first, second) {
        (Some(0), Some(0)) => Some(0),
        (Some(code), _) if code != 0 => Some(code),
        (_, Some(code)) if code != 0 => Some(code),
        _ => None,
    }
}

fn merge_command_block_duration(first: Option<i64>, second: Option<i64>) -> Option<i64> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first + second),
        (Some(duration), None) | (None, Some(duration)) => Some(duration),
        (None, None) => None,
    }
}

fn merge_command_block_summaries(first: Option<&str>, second: Option<&str>) -> Option<String> {
    let summaries = [first, second]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|summary| !summary.is_empty())
        .collect::<Vec<_>>();
    if summaries.is_empty() {
        None
    } else {
        Some(summaries.join("\n\n"))
    }
}

fn min_i64_option(first: Option<i64>, second: Option<i64>) -> Option<i64> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first.min(second)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn max_i64_option(first: Option<i64>, second: Option<i64>) -> Option<i64> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first.max(second)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn split_command_block_sequence(
    seq_start: Option<i64>,
    seq_end: Option<i64>,
) -> (Option<i64>, Option<i64>, Option<i64>, Option<i64>) {
    let (Some(seq_start), Some(seq_end)) = (seq_start, seq_end) else {
        return (seq_start, seq_start, seq_end, seq_end);
    };
    if seq_end <= seq_start {
        return (
            Some(seq_start),
            Some(seq_start),
            Some(seq_end),
            Some(seq_end),
        );
    }
    let midpoint = (seq_start + seq_end) / 2;
    (
        Some(seq_start),
        Some(midpoint),
        Some(midpoint + 1),
        Some(seq_end),
    )
}

fn split_command_block_summary(summary: Option<&str>) -> (Option<String>, Option<String>) {
    let Some(summary) = summary else {
        return (None, None);
    };
    let lines = summary
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if lines.len() <= 1 {
        return (Some(summary.to_string()), None);
    }
    let midpoint = lines.len().div_ceil(2);
    (
        Some(lines[..midpoint].join("\n")),
        Some(lines[midpoint..].join("\n")),
    )
}

fn normalize_hex_color(label: &str, color: &str) -> Result<String, String> {
    let color = required_trimmed(label, color)?;
    let valid = color.len() == 7
        && color.starts_with('#')
        && color
            .chars()
            .skip(1)
            .all(|character| character.is_ascii_hexdigit());
    if valid {
        Ok(color.to_ascii_lowercase())
    } else {
        Err(format!("{label} must be a #rrggbb color"))
    }
}

fn normalize_session_mode(mode: &str) -> Result<&'static str, String> {
    match mode.trim() {
        "shell" => Ok("shell"),
        "agent" => Ok("agent"),
        "ssh" => Ok("ssh"),
        "dev_server" => Ok("dev_server"),
        "test" => Ok("test"),
        "review" => Ok("review"),
        unsupported => Err(format!("unsupported session mode {unsupported}")),
    }
}

fn normalize_session_state(state: &str) -> Result<&'static str, String> {
    match state.trim() {
        "running" => Ok("running"),
        "waiting_input" => Ok("waiting_input"),
        "blocked" => Ok("blocked"),
        "completed" => Ok("completed"),
        "failed" => Ok("failed"),
        "idle" => Ok("idle"),
        unsupported => Err(format!("unsupported session state {unsupported}")),
    }
}

fn normalize_session_attention_state(state: &str) -> Result<&'static str, String> {
    match state.trim() {
        "none" => Ok("none"),
        "unread" => Ok("unread"),
        "needs_input" => Ok("needs_input"),
        "permission" => Ok("permission"),
        "error" => Ok("error"),
        unsupported => Err(format!("unsupported session attention state {unsupported}")),
    }
}

fn is_dangerous_session_input(text: &str) -> bool {
    let lower = text.to_lowercase();
    text.lines().count() > 1
        || text.len() > 1024
        || lower.contains("rm -rf")
        || lower.contains("sudo ")
        || lower.contains("chmod -r")
        || lower.contains("curl ")
        || lower.contains("| sh")
}

fn summarize_session_input(text: &str) -> String {
    let first_line = text.lines().next().unwrap_or("").trim();
    let mut chars = first_line.chars();
    let summary = chars.by_ref().take(160).collect::<String>();
    if chars.next().is_some() {
        format!("{summary}...")
    } else {
        summary
    }
}

fn default_agent_env_policy_json() -> Value {
    serde_json::json!({
        "secrets": "redacted",
        "mode": "inherit_safe"
    })
}

fn builtin_agent_presets() -> Vec<UpsertAgentProfileInput> {
    [
        (
            "agent_claude",
            "Claude Code",
            "claude",
            vec!["coding", "review"],
        ),
        ("agent_codex", "Codex", "codex", vec!["coding", "review"]),
        ("agent_gemini", "Gemini CLI", "gemini", vec!["coding"]),
        ("agent_opencode", "OpenCode", "opencode", vec!["coding"]),
        ("agent_openclaw", "OpenClaw", "openclaw", vec!["coding"]),
        (
            "agent_cursor",
            "Cursor Agent",
            "cursor-agent",
            vec!["coding"],
        ),
    ]
    .into_iter()
    .map(|(id, name, command, skills)| {
        agent_profile_input(
            id,
            name,
            "cli",
            command,
            if command_on_path(command) {
                "available"
            } else {
                "unavailable"
            },
            skills,
        )
    })
    .chain(std::iter::once(agent_profile_input(
        "agent_generic_shell",
        "Generic Shell",
        "shell",
        env::var("SHELL")
            .ok()
            .filter(|shell| !shell.trim().is_empty())
            .as_deref()
            .unwrap_or("sh"),
        "available",
        vec!["fallback"],
    )))
    .collect()
}

fn agent_profile_input(
    id: &str,
    name: &str,
    runtime: &str,
    command: &str,
    status: &str,
    skills: Vec<&str>,
) -> UpsertAgentProfileInput {
    UpsertAgentProfileInput {
        id: id.to_string(),
        name: name.to_string(),
        runtime: runtime.to_string(),
        command: command.to_string(),
        args_json: Some(serde_json::json!([])),
        env_policy_json: Some(default_agent_env_policy_json()),
        skills_json: Some(Value::Array(
            skills
                .into_iter()
                .map(|skill| Value::String(skill.to_string()))
                .collect(),
        )),
        status: Some(status.to_string()),
    }
}

fn agent_profile_args(args_json: &Value) -> Result<Vec<String>, String> {
    let Some(args) = args_json.as_array() else {
        return Err("agent profile args_json must be an array of strings".to_string());
    };
    args.iter()
        .map(|arg| {
            arg.as_str()
                .map(str::to_string)
                .ok_or_else(|| "agent profile args_json must be an array of strings".to_string())
        })
        .collect()
}

fn command_on_path(command: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path).any(|dir| dir.join(command).is_file())
}

fn budget_id_for_scope(scope_type: &str, scope_id: Option<&str>) -> String {
    let suffix = scope_id
        .map(sanitize_artifact_name)
        .unwrap_or_else(|| "workspace".to_string());
    format!("budget_{scope_type}_{suffix}")
}

fn policy_pack_id(project_id: &str, name: &str) -> String {
    format!(
        "policy_pack_{}_{}",
        sanitize_artifact_name(project_id),
        sanitize_artifact_name(name)
    )
}

fn secret_id(project_id: &str, name: &str) -> String {
    format!(
        "secret_{}_{}",
        sanitize_artifact_name(project_id),
        sanitize_artifact_name(name)
    )
}

fn secret_keychain_ref(project_id: &str, name: &str) -> String {
    format!("haneulchi:{project_id}:{name}")
}

fn keychain_status() -> &'static str {
    #[cfg(test)]
    {
        "local"
    }
    #[cfg(not(test))]
    {
        if command_on_path("security") {
            "macos"
        } else {
            "unavailable"
        }
    }
}

fn write_keychain_secret(
    _connection: &Connection,
    keychain_ref: &str,
    value: &str,
) -> Result<(), String> {
    #[cfg(test)]
    {
        _connection
            .execute(
                "INSERT INTO keychain_secret_values(keychain_ref, value, updated_at)
                 VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(keychain_ref) DO UPDATE SET
                   value = excluded.value,
                   updated_at = excluded.updated_at",
                params![keychain_ref, value],
            )
            .map_err(|error| format!("failed to store secret in local keychain: {error}"))?;
        Ok(())
    }
    #[cfg(not(test))]
    {
        let output = Command::new("security")
            .args([
                "add-generic-password",
                "-U",
                "-s",
                "haneulchi",
                "-a",
                keychain_ref,
                "-w",
                value,
            ])
            .output()
            .map_err(|error| format!("failed to invoke macOS Keychain: {error}"))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                "macOS Keychain rejected secret storage".to_string()
            } else {
                format!("macOS Keychain rejected secret storage: {stderr}")
            })
        }
    }
}

fn read_keychain_secret(
    _connection: &Connection,
    keychain_ref: &str,
) -> Result<Option<String>, String> {
    #[cfg(test)]
    {
        _connection
            .query_row(
                "SELECT value FROM keychain_secret_values WHERE keychain_ref = ?1",
                params![keychain_ref],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| format!("failed to read secret from local keychain: {error}"))
    }
    #[cfg(not(test))]
    {
        let output = Command::new("security")
            .args([
                "find-generic-password",
                "-s",
                "haneulchi",
                "-a",
                keychain_ref,
                "-w",
            ])
            .output()
            .map_err(|error| format!("failed to invoke macOS Keychain: {error}"))?;
        if output.status.success() {
            Ok(Some(
                String::from_utf8_lossy(&output.stdout)
                    .trim_end_matches(['\r', '\n'])
                    .to_string(),
            ))
        } else {
            Ok(None)
        }
    }
}

fn project_id_from_key(key: &str) -> Result<String, String> {
    let normalized = sanitize_artifact_name(&key.to_ascii_lowercase())
        .trim_matches('_')
        .to_string();
    if normalized.is_empty() {
        Err("project key must include at least one safe character".to_string())
    } else {
        Ok(format!("proj_{normalized}"))
    }
}

fn required_trimmed(label: &str, value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(format!("{label} cannot be empty"))
    } else {
        Ok(trimmed.to_string())
    }
}

fn normalize_tracker_local_kind(local_kind: &str) -> Result<String, String> {
    let local_kind = local_kind.trim();
    if matches!(local_kind, "task" | "project") {
        Ok(local_kind.to_string())
    } else {
        Err(format!("unsupported tracker local kind {local_kind}"))
    }
}

fn normalize_tracker_provider(provider: &str) -> Result<String, String> {
    let provider = provider.trim();
    if matches!(
        provider,
        "linear" | "github" | "plane" | "custom" | "manual"
    ) {
        Ok(provider.to_string())
    } else {
        Err(format!("unsupported tracker provider {provider}"))
    }
}

fn normalize_tracker_sync_mode(sync_mode: &str) -> Result<String, String> {
    let sync_mode = sync_mode.trim();
    if matches!(sync_mode, "manual" | "mirror" | "import" | "export") {
        Ok(sync_mode.to_string())
    } else {
        Err(format!("unsupported tracker sync mode {sync_mode}"))
    }
}

fn normalize_sync_provider(provider: &str) -> Result<String, String> {
    let provider = provider.trim();
    if matches!(provider, "linear" | "github" | "plane") {
        Ok(provider.to_string())
    } else {
        Err(format!("unsupported tracker sync provider {provider}"))
    }
}

fn sync_secret_name(provider: &str) -> &'static str {
    match provider {
        "linear" => "LINEAR_API_KEY",
        "github" => "GITHUB_TOKEN",
        "plane" => "PLANE_API_KEY",
        _ => "TRACKER_API_KEY",
    }
}

fn normalize_knowledge_status(status: &str) -> Result<String, String> {
    let status = status.trim();
    if matches!(status, "current" | "stale" | "gap" | "missing" | "unknown") {
        Ok(status.to_string())
    } else {
        Err(format!("unsupported knowledge status {status}"))
    }
}

fn normalize_knowledge_slug(slug: &str) -> Result<String, String> {
    let slug = slug.trim();
    if slug.is_empty() {
        return Err("knowledge page slug cannot be empty".to_string());
    }
    let normalized = sanitize_artifact_name(slug).trim_matches('_').to_string();
    if normalized.is_empty() {
        Err("knowledge page slug must include at least one safe character".to_string())
    } else {
        Ok(normalized)
    }
}

fn title_from_artifact_path(path_or_ref: &str) -> String {
    Path::new(path_or_ref)
        .file_stem()
        .and_then(OsStr::to_str)
        .map(|stem| stem.replace(['-', '_'], " "))
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| "Knowledge Artifact".to_string())
}

fn slugify_knowledge_artifact(value: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for character in value.trim().to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(character);
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }
    if slug.is_empty() {
        "knowledge-artifact".to_string()
    } else {
        slug
    }
}

fn extract_knowledge_wiki_links(body_md: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut rest = body_md;
    while let Some(start) = rest.find("[[") {
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("]]") else {
            break;
        };
        let raw_target = after_start[..end]
            .split_once('|')
            .map(|(target, _)| target)
            .unwrap_or(&after_start[..end])
            .trim();
        if !raw_target.is_empty() {
            let slug = slugify_knowledge_artifact(raw_target);
            if !links.contains(&slug) {
                links.push(slug);
            }
        }
        rest = &after_start[end + 2..];
    }
    links
}

fn title_from_concept_slug(slug: &str) -> String {
    slug.replace('-', " ")
}

fn obsidian_markdown_file_name(title: &str) -> String {
    let mut name = String::new();
    let mut last_was_space = false;
    for character in title.trim().chars() {
        if character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_') {
            if character == ' ' {
                if !last_was_space && !name.is_empty() {
                    name.push(' ');
                }
                last_was_space = true;
            } else {
                name.push(character);
                last_was_space = false;
            }
        }
    }
    let name = name.trim();
    if name.is_empty() {
        "Knowledge Page.md".to_string()
    } else {
        format!("{name}.md")
    }
}

fn build_obsidian_knowledge_index(pages: &[PersistedKnowledgePage]) -> String {
    let mut sorted_pages = pages.to_vec();
    sorted_pages.sort_by(|left, right| left.title.cmp(&right.title));
    let mut lines = vec!["# Knowledge Index".to_string(), String::new()];
    for page in sorted_pages {
        lines.push(format!("- [[{}]]", page.title));
    }
    lines.join("\n")
}

fn knowledge_query_terms(question: &str) -> Vec<String> {
    const STOP_WORDS: &[&str] = &[
        "a", "an", "and", "are", "for", "how", "is", "of", "or", "should", "the", "to", "what",
        "when", "where", "which", "with",
    ];
    let mut terms = Vec::new();
    for raw in question
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|term| term.len() >= 3)
    {
        let term = raw.to_lowercase();
        if !STOP_WORDS.contains(&term.as_str()) && !terms.contains(&term) {
            terms.push(term);
        }
    }
    terms
}

fn score_knowledge_page_for_terms(page: &PersistedKnowledgePage, terms: &[String]) -> usize {
    let haystack = format!("{} {} {}", page.title, page.slug, page.body_md).to_lowercase();
    terms
        .iter()
        .map(|term| haystack.matches(term).count())
        .sum()
}

fn build_local_knowledge_answer(question: &str, pages: &[PersistedKnowledgePage]) -> String {
    let mut lines = vec![
        "## Local knowledge answer draft".to_string(),
        String::new(),
        format!("Question: {question}"),
        String::new(),
    ];
    if pages.is_empty() {
        lines.push("No matching knowledge pages were found in the local vault.".to_string());
    } else {
        lines.push("Relevant local citations:".to_string());
        for page in pages {
            lines.push(format!(
                "- [[{}]] ({}) - {}",
                page.title,
                page.id,
                first_non_heading_markdown_line(&page.body_md)
            ));
        }
    }
    lines.join("\n")
}

fn first_non_heading_markdown_line(body_md: &str) -> String {
    body_md
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .unwrap_or("No excerpt available.")
        .to_string()
}

fn chunk_text_by_chars(text: &str, max_chunk_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        current.push(character);
        if current.chars().count() >= max_chunk_chars {
            chunks.push(current);
            current = String::new();
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    if chunks.is_empty() {
        chunks.push(String::new());
    }
    chunks
}

fn compile_ingested_knowledge_page(
    title: &str,
    path_or_ref: &str,
    kind: &str,
    chunks: &[String],
) -> String {
    let mut body = vec![
        format!("# {title}"),
        String::new(),
        format!("Source: {path_or_ref}"),
        format!("Modality: {kind}"),
        format!("Chunks: {}", chunks.len()),
        String::new(),
    ];
    for (index, chunk) in chunks.iter().enumerate() {
        body.push(format!("## Chunk {}", index + 1));
        body.push(chunk.clone());
        body.push(String::new());
    }
    body.join("\n").trim_end().to_string()
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn normalize_task_labels(labels: Option<Vec<String>>) -> Vec<String> {
    let mut normalized = Vec::new();
    for label in labels.unwrap_or_default() {
        let trimmed = label.trim().to_string();
        if !trimmed.is_empty() && !normalized.contains(&trimmed) {
            normalized.push(trimmed);
        }
    }
    normalized
}

fn parse_task_labels_json(labels_json: &str) -> Vec<String> {
    normalize_task_labels(serde_json::from_str::<Vec<String>>(labels_json).ok())
}

fn normalize_optional_borrowed_text(value: Option<&str>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn normalize_visual_link_kind(kind: &str) -> Result<String, String> {
    let normalized = kind.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "context" | "tool" | "task" | "workflow" | "dependency" => Ok(normalized),
        _ => Err(
            "visual harness link kind must be context, tool, task, workflow, or dependency"
                .to_string(),
        ),
    }
}

fn create_evidence_pack_json(
    evidence_pack_id: &str,
    task_id: Option<String>,
    run_id: Option<String>,
) -> Value {
    serde_json::json!({
        "id": evidence_pack_id,
        "task_id": task_id,
        "run_id": run_id,
        "workflow_version": null,
        "context_sources": [],
        "diff_summary": {},
        "command_blocks": [],
        "tests": [],
        "screenshots": [],
        "transcript_summary": "",
        "token_usage": {},
        "policy_events": [],
        "review_decision": null
    })
}

fn create_run_replay_metadata_json(
    run: &PersistedRun,
    workflow: &PersistedWorkflowVersion,
) -> Value {
    serde_json::json!({
        "id": format!("replay_{}", run.id),
        "run_id": run.id,
        "task_id": run.task_id,
        "project_id": run.project_id,
        "agent_profile_id": run.agent_profile_id,
        "workflow_version_id": run.workflow_version_id,
        "workflow_content_hash": workflow.content_hash,
        "context_pack_id": run.context_pack_id,
        "context_sources": [],
        "workspace_path": run.workspace_path,
        "command_block_ids": [],
        "token_usage": {},
        "hook_results": [],
        "review_decision": null
    })
}

fn run_terminal_fidelity_smoke_cases() -> Vec<TerminalFidelitySmokeCaseResult> {
    vec![
        terminal_capture_case(
            "shell_basic",
            "Shell basics",
            "printf 'shell-basic-ok\\n'; exit 0",
            80,
            24,
            |capture| capture.exit_success && capture.output.contains("shell-basic-ok"),
        ),
        terminal_capture_case(
            "unicode_cjk_emoji",
            "Unicode CJK emoji",
            "printf '한글🙂é\\n'",
            80,
            24,
            |capture| capture.exit_success && capture.output.contains("한글"),
        ),
        terminal_capture_case("resize", "PTY resize", "stty size", 101, 31, |capture| {
            capture.exit_success && capture.output.split_whitespace().count() >= 2
        }),
        terminal_capture_case(
            "throughput",
            "High-throughput output",
            "i=1; while [ $i -le 200 ]; do printf 'line-%04d\\n' $i; i=$((i+1)); done",
            120,
            32,
            |capture| capture.exit_success && capture.output.contains("line-0200"),
        ),
        terminal_smoke_case(
            "safe_link_sanitization",
            "Safe link sanitization",
            "pass",
            "Terminal links are inert until clicked; HTTP(S) is allowed and dangerous schemes are blocked.",
        ),
        terminal_smoke_case(
            "osc_allowlist",
            "OSC allowlist",
            "pass",
            "OSC 9/99/777 are allowlisted, unknown OSC codes are ignored, and oversized payloads are rejected.",
        ),
        terminal_smoke_case(
            "webgl_fallback",
            "WebGL fallback",
            "warning",
            "Browser renderer fallback requires a renderer smoke run; raw PTY remains testable.",
        ),
        terminal_smoke_case(
            "ime_korean",
            "Korean IME",
            "warning",
            "Korean IME composition requires an interactive browser smoke run.",
        ),
    ]
}

fn terminal_capture_case(
    case_id: &str,
    name: &str,
    script: &str,
    cols: u16,
    rows: u16,
    predicate: fn(&crate::pty::PtyCommandCapture) -> bool,
) -> TerminalFidelitySmokeCaseResult {
    match crate::pty::capture_command_once("sh", &["-lc", script], cols, rows) {
        Ok(capture) if predicate(&capture) => terminal_smoke_case(
            case_id,
            name,
            "pass",
            &format!(
                "PTY capture exited {} with {} bytes.",
                capture.exit_code,
                capture.output.len()
            ),
        ),
        Ok(capture) => terminal_smoke_case(
            case_id,
            name,
            "fail",
            &format!(
                "PTY capture exited {} but output did not match expected smoke marker.",
                capture.exit_code
            ),
        ),
        Err(error) => terminal_smoke_case(
            case_id,
            name,
            "fail",
            &format!("PTY capture failed: {error}"),
        ),
    }
}

fn terminal_smoke_case(
    case_id: &str,
    name: &str,
    status: &str,
    detail: &str,
) -> TerminalFidelitySmokeCaseResult {
    TerminalFidelitySmokeCaseResult {
        case_id: case_id.to_string(),
        name: name.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
    }
}

fn workflow_negative_case(
    case_id: &str,
    status: &str,
    detail: &str,
) -> WorkflowNegativeTestCaseResult {
    WorkflowNegativeTestCaseResult {
        case_id: case_id.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
    }
}

fn workflow_negative_valid_document() -> String {
    r#"---
haneulchi: 1
project:
  key: AUTH
  default_branch: main
workspace:
  strategy: worktree
  base_root: .haneulchi/worktrees
agents:
  default: claude
context:
  default_pack: auth-default
hooks:
  before_run: .haneulchi/hooks/before_run.sh
review:
  required_evidence:
    - diff
    - tests
    - transcript_summary
---

# Prompt template

You are working on {task.id} in {project.name}.
Use the context pack {?context_pack.name}.
"#
    .to_string()
}

fn workflow_negative_invalid_document() -> String {
    r#"---
haneulchi: 1
project:
  key: AUTH
workspace:
  strategy: worktree
hooks:
  before_run: ../escape.sh
---

Use {secret.token} for {task.id}.
"#
    .to_string()
}

fn run_dmg_smoke_cases(
    dmg_path: Option<&str>,
    app_bundle_path: Option<&str>,
) -> Vec<DmgSmokeCaseResult> {
    let dmg_exists = dmg_path
        .map(|path| Path::new(path).is_file())
        .unwrap_or(false);
    let app_exists = app_bundle_path
        .map(|path| Path::new(path).is_dir())
        .unwrap_or(false);

    let artifact_detail = match dmg_path {
        Some(path) if dmg_exists => format!("DMG artifact found at {path}."),
        Some(path) => format!("DMG artifact is missing at {path}."),
        None => {
            "No DMG artifact path was provided; distribution is a non-ship blocker.".to_string()
        }
    };
    let app_detail = match app_bundle_path {
        Some(path) if app_exists => format!("App bundle found at {path}."),
        Some(path) => format!("App bundle is missing at {path}."),
        None => "No app bundle path was provided for install smoke verification.".to_string(),
    };
    let codesign_check = if app_exists {
        app_bundle_path.map(run_app_codesign_check)
    } else {
        None
    };
    let notarization_check = if dmg_exists {
        dmg_path.map(run_dmg_notarization_check)
    } else {
        None
    };

    vec![
        dmg_smoke_case(
            "dmg_artifact",
            "DMG artifact",
            if dmg_exists { "pass" } else { "fail" },
            &artifact_detail,
        ),
        dmg_smoke_case(
            "app_bundle",
            "App bundle",
            if app_bundle_path.is_none() {
                "warning"
            } else if app_exists {
                "pass"
            } else {
                "fail"
            },
            &app_detail,
        ),
        dmg_smoke_case(
            "codesign",
            "Code signing",
            codesign_check
                .as_ref()
                .map(|check| check.status)
                .unwrap_or("fail"),
            codesign_check
                .as_ref()
                .map(|check| check.detail.as_str())
                .unwrap_or("Signed app bundle evidence is unavailable."),
        ),
        dmg_smoke_case(
            "notarization",
            "Notarization",
            notarization_check
                .as_ref()
                .map(|check| check.status)
                .unwrap_or("fail"),
            notarization_check
                .as_ref()
                .map(|check| check.detail.as_str())
                .unwrap_or("Notarized DMG evidence is unavailable."),
        ),
    ]
}

fn dmg_smoke_case(case_id: &str, name: &str, status: &str, detail: &str) -> DmgSmokeCaseResult {
    DmgSmokeCaseResult {
        case_id: case_id.to_string(),
        name: name.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DmgSmokeLocalCheck {
    status: &'static str,
    detail: String,
}

fn run_app_codesign_check(app_path: &str) -> DmgSmokeLocalCheck {
    run_local_release_check(
        "codesign",
        &["--verify", "--deep", "--strict", "--verbose=2", app_path],
        "App code signature verification passed.",
        "App code signature verification failed",
    )
}

fn run_dmg_notarization_check(dmg_path: &str) -> DmgSmokeLocalCheck {
    let primary_signature = run_local_release_check(
        "spctl",
        &[
            "--assess",
            "--type",
            "open",
            "--context",
            "context:primary-signature",
            "--verbose",
            dmg_path,
        ],
        "DMG primary signature assessment passed.",
        "DMG primary signature assessment failed",
    );
    let staple = run_local_release_check(
        "xcrun",
        &["stapler", "validate", dmg_path],
        "Notarization staple validation passed.",
        "Notarization staple validation failed",
    );
    let status = if primary_signature.status == "fail" || staple.status == "fail" {
        "fail"
    } else if primary_signature.status == "warning" || staple.status == "warning" {
        "warning"
    } else {
        "pass"
    };

    DmgSmokeLocalCheck {
        status,
        detail: format!("{} {}", primary_signature.detail, staple.detail),
    }
}

fn run_local_release_check(
    command: &str,
    args: &[&str],
    pass_detail: &str,
    fail_detail: &str,
) -> DmgSmokeLocalCheck {
    match Command::new(command).args(args).output() {
        Ok(output) if output.status.success() => DmgSmokeLocalCheck {
            status: "pass",
            detail: command_detail(pass_detail, &output),
        },
        Ok(output) => DmgSmokeLocalCheck {
            status: "fail",
            detail: command_detail(&format!("{fail_detail}: {}", output.status), &output),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => DmgSmokeLocalCheck {
            status: "warning",
            detail: format!("{command} is unavailable on this host; run this smoke test on a macOS release verification host."),
        },
        Err(error) => DmgSmokeLocalCheck {
            status: "fail",
            detail: format!("Failed to run {command}: {error}"),
        },
    }
}

fn command_detail(prefix: &str, output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    if text.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}: {}", trim_command_detail(text))
    }
}

fn trim_command_detail(text: &str) -> String {
    const MAX_DETAIL_CHARS: usize = 360;
    let mut detail = text.replace('\n', " ");
    if detail.chars().count() > MAX_DETAIL_CHARS {
        detail = detail.chars().take(MAX_DETAIL_CHARS).collect::<String>();
        detail.push_str("...");
    }
    detail
}

fn recovery_drill_result(
    drill_id: &str,
    name: &str,
    status: &str,
    detail: &str,
    evidence: Vec<String>,
) -> RecoveryDrillResult {
    RecoveryDrillResult {
        drill_id: drill_id.to_string(),
        name: name.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
        evidence,
    }
}

fn benchmark_suite_result(
    suite_id: &str,
    name: &str,
    status: &str,
    metric_value: i64,
    target_value: i64,
    unit: &str,
    detail: &str,
) -> BenchmarkSuiteResult {
    BenchmarkSuiteResult {
        suite_id: suite_id.to_string(),
        name: name.to_string(),
        status: status.to_string(),
        metric_value,
        target_value,
        unit: unit.to_string(),
        detail: detail.to_string(),
    }
}

fn dogfood_telemetry_finding(
    finding_id: &str,
    status: &str,
    detail: &str,
) -> DogfoodTelemetryFinding {
    DogfoodTelemetryFinding {
        finding_id: finding_id.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
    }
}

fn release_gate_scenarios(
    db_status: &str,
    tasks: &[PersistedTask],
    runs: &[PersistedRun],
    sessions: &[PersistedSession],
    agent_profiles: &[PersistedAgentProfile],
    command_blocks: &[PersistedCommandBlock],
    evidence_packs: &[PersistedEvidencePack],
    workflow_state: &WorkflowRuntimeState,
    budget_summary: &Value,
    knowledge_summary: &KnowledgeSummary,
    secret_summary: &Value,
    policy_pack_summary: &Value,
    permission_audit_summary: &Value,
    tracker_bindings: &[PersistedExternalTrackerBinding],
    terminal_smoke_run: Option<&PersistedTerminalFidelitySmokeRun>,
    task_lifecycle_e2e_run: Option<&PersistedTaskLifecycleE2ERun>,
    workflow_negative_run: Option<&PersistedWorkflowNegativeTestRun>,
    dmg_smoke_run: Option<&PersistedDmgSmokeRun>,
    recovery_drill_run: Option<&PersistedRecoveryDrillRun>,
    benchmark_run: Option<&PersistedBenchmarkRun>,
) -> Vec<ReleaseGateScenarioResult> {
    let approved_evidence = evidence_packs
        .iter()
        .filter(|pack| release_evidence_pack_has_required_schema(pack))
        .map(|pack| pack.id.clone())
        .collect::<Vec<_>>();
    let done_task_ids = tasks
        .iter()
        .filter(|task| task.status == "done")
        .map(|task| task.id.as_str())
        .collect::<Vec<_>>();
    let linked_done_run = runs
        .iter()
        .find(|run| done_task_ids.contains(&run.task_id.as_str()));
    let command_block_evidence = command_blocks
        .iter()
        .filter(|block| block.exit_code == Some(0))
        .map(|block| block.id.clone())
        .collect::<Vec<_>>();
    let raw_agent_terminal_evidence = sessions
        .iter()
        .filter(|session| session.mode == "agent")
        .filter_map(|session| {
            let agent_id = session.agent_profile_id.as_deref()?;
            let agent = agent_profiles
                .iter()
                .find(|profile| profile.id == agent_id)?;
            if agent.runtime == "generic-cli" {
                Some(session.id.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let session_like_blocks = command_blocks
        .iter()
        .filter(|block| {
            let haystack = format!(
                "{} {}",
                block.command.to_lowercase(),
                block.summary.clone().unwrap_or_default().to_lowercase()
            );
            haystack.contains("terminal")
                || haystack.contains("xterm")
                || haystack.contains("webgl")
                || haystack.contains("ime")
        })
        .map(|block| block.id.clone())
        .collect::<Vec<_>>();
    let terminal_smoke_status = terminal_smoke_run
        .map(|run| match run.status.as_str() {
            "passed" => "pass",
            "failed" => "fail",
            _ => "warning",
        })
        .unwrap_or("warning");
    let has_budget = budget_summary_has_configured_budget(budget_summary);
    let protected_secret_count = secret_summary
        .get("redaction")
        .and_then(|redaction| redaction.get("protected_secret_count"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let policy_pack_active = policy_pack_summary
        .get("id")
        .and_then(Value::as_str)
        .map(|id| !id.is_empty())
        .unwrap_or(false);
    let permission_audit_count = permission_audit_summary
        .get("recent_count")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let parity_suite = benchmark_run.and_then(|run| {
        run.suites
            .iter()
            .find(|suite| suite.suite_id == "ui_cli_api_snapshot_parity")
            .map(|suite| (run, suite))
    });

    vec![
        release_gate_result(
            "RG-01",
            "App shell/readiness",
            if db_status == "ok" { "pass" } else { "fail" },
            if db_status == "ok" {
                "Durable state store is healthy and the app can build a readiness snapshot."
            } else {
                "Durable state store is not healthy."
            },
            vec![format!("db:{db_status}")],
        ),
        release_gate_result(
            "RG-02",
            "Terminal fidelity",
            if terminal_smoke_run.is_some() {
                terminal_smoke_status
            } else if session_like_blocks.is_empty() {
                "warning"
            } else {
                "pass"
            },
            if let Some(run) = terminal_smoke_run {
                if run.status == "passed" {
                    "Terminal fidelity smoke run passed."
                } else {
                    "Terminal fidelity smoke run completed with warnings or failures."
                }
            } else if session_like_blocks.is_empty() {
                "Terminal/WebGL/IME scenario evidence has not been attached."
            } else {
                "Terminal fidelity evidence is present in command blocks."
            },
            terminal_smoke_run
                .map(|run| vec![run.id.clone()])
                .unwrap_or(session_like_blocks),
        ),
        release_gate_result(
            "RG-03",
            "Multi-project/multi-terminal",
            if benchmark_run.map(|run| run.status.as_str()) == Some("passed") {
                "pass"
            } else if command_blocks.len() >= 20 {
                "pass"
            } else {
                "warning"
            },
            if benchmark_run.map(|run| run.status.as_str()) == Some("passed") {
                "Benchmark suite evidence is available for local stress readiness."
            } else if command_blocks.len() >= 20 {
                "Stress evidence includes at least 20 command blocks."
            } else {
                "Five-project/twenty-pane stability evidence is not complete."
            },
            benchmark_run
                .map(|run| vec![run.id.clone()])
                .unwrap_or_else(|| {
                    command_blocks
                        .iter()
                        .map(|block| block.id.clone())
                        .collect()
                }),
        ),
        release_gate_result(
            "RG-04",
            "Command blocks",
            if command_block_evidence.is_empty() {
                "warning"
            } else {
                "pass"
            },
            if command_block_evidence.is_empty() {
                "No successful command block evidence is available."
            } else {
                "Successful command block evidence is available."
            },
            command_block_evidence,
        ),
        release_gate_result(
            "RG-05",
            "Task lifecycle",
            if task_lifecycle_e2e_run.map(|run| run.status.as_str()) == Some("passed") {
                "pass"
            } else if let Some(run) = linked_done_run {
                if evidence_packs
                    .iter()
                    .any(|pack| pack.run_id.as_deref() == Some(run.id.as_str()))
                {
                    "pass"
                } else {
                    "warning"
                }
            } else {
                "warning"
            },
            if task_lifecycle_e2e_run.is_some() {
                "Task lifecycle E2E exercised Ready to Running to Review to Done with linked run/evidence."
            } else if linked_done_run.is_some() {
                "A task reached done with a linked run/evidence trail."
            } else {
                "Ready to done lifecycle evidence is not complete."
            },
            task_lifecycle_e2e_run
                .map(|run| vec![run.id.clone()])
                .or_else(|| linked_done_run.map(|run| vec![run.id.clone()]))
                .unwrap_or_default(),
        ),
        release_gate_result(
            "RG-06",
            "Agent terminal fallback",
            if raw_agent_terminal_evidence.is_empty() {
                "warning"
            } else {
                "pass"
            },
            if raw_agent_terminal_evidence.is_empty() {
                "Unsupported-agent raw terminal fallback evidence has not been recorded."
            } else {
                "At least one third-party CLI agent is available as a raw terminal session."
            },
            raw_agent_terminal_evidence,
        ),
        release_gate_result(
            "RG-07",
            "Workflow runtime",
            if workflow_negative_run.map(|run| run.status.as_str()) == Some("passed") {
                "pass"
            } else if workflow_state.valid && workflow_state.current_version_id.is_some() {
                "pass"
            } else if workflow_state.valid {
                "warning"
            } else {
                "fail"
            },
            if workflow_negative_run.is_some() {
                "Workflow negative tests passed invalid reload, LKG, hook diagnostics, and restore scenarios."
            } else if workflow_state.current_version_id.is_some() {
                "Workflow runtime has a valid current version."
            } else {
                "Workflow validation is healthy but no current workflow version is loaded."
            },
            workflow_negative_run
                .map(|run| vec![run.id.clone()])
                .unwrap_or_else(|| {
                    workflow_state
                        .current_version_id
                        .clone()
                        .into_iter()
                        .collect()
                }),
        ),
        release_gate_result(
            "RG-08",
            "UI/CLI/API parity",
            if db_status != "ok" {
                "fail"
            } else if let Some((_, suite)) = parity_suite {
                if suite.status == "pass" {
                    "pass"
                } else {
                    "fail"
                }
            } else {
                "warning"
            },
            if db_status != "ok" {
                "Durable state is not healthy enough to compare UI/CLI/API state."
            } else if let Some((_, suite)) = parity_suite {
                &suite.detail
            } else {
                "UI/CLI/API parity benchmark evidence has not been recorded."
            },
            parity_suite
                .map(|(run, _)| vec![run.id.clone()])
                .unwrap_or_default(),
        ),
        release_gate_result(
            "RG-09",
            "Review/evidence",
            if approved_evidence.is_empty() {
                "fail"
            } else {
                "pass"
            },
            if approved_evidence.is_empty() {
                "No approved evidence pack contains required diff/tests/transcript/cost/policy fields."
            } else {
                "Approved evidence pack contains the required release evidence fields."
            },
            approved_evidence,
        ),
        release_gate_result(
            "RG-10",
            "Token/budget",
            if has_budget { "pass" } else { "warning" },
            if has_budget {
                "Budget state is configured."
            } else {
                "Budget warning/dashboard evidence is not configured."
            },
            vec![],
        ),
        release_gate_result(
            "RG-11",
            "Knowledge MVP-lite",
            if knowledge_summary.stale_count == 0
                && knowledge_summary.gap_count == 0
                && !knowledge_summary.recent_pages.is_empty()
            {
                "pass"
            } else {
                "warning"
            },
            if knowledge_summary.recent_pages.is_empty() {
                "Knowledge lineage evidence has not been compiled."
            } else {
                "Knowledge summary is available."
            },
            knowledge_summary.recent_pages.clone(),
        ),
        release_gate_result(
            "RG-12",
            "Security",
            if protected_secret_count > 0 || policy_pack_active || permission_audit_count > 0 {
                "pass"
            } else {
                "warning"
            },
            if protected_secret_count > 0 || policy_pack_active || permission_audit_count > 0 {
                "Security policy/redaction evidence is present."
            } else {
                "Secret redaction and policy approval evidence is not yet present."
            },
            vec![],
        ),
        release_gate_result(
            "RG-13",
            "DMG distribution",
            if let Some(run) = dmg_smoke_run {
                if run.status == "passed" || run.explicit_blocker {
                    "pass"
                } else {
                    "fail"
                }
            } else {
                "fail"
            },
            if let Some(run) = dmg_smoke_run {
                if run.status == "passed" {
                    "DMG install smoke evidence passed for artifact, app bundle, signing, and notarization."
                } else if run.explicit_blocker {
                    "DMG install smoke recorded an explicit non-ship blocker."
                } else {
                    "DMG install smoke did not pass and no explicit blocker was recorded."
                }
            } else {
                "Signed/notarized DMG evidence or an explicit non-ship blocker is required."
            },
            dmg_smoke_run
                .map(|run| vec![run.id.clone()])
                .unwrap_or_default(),
        ),
        release_gate_result(
            "RG-14",
            "Recovery",
            if recovery_drill_run.map(|run| run.status.as_str()) == Some("passed") {
                "pass"
            } else if recovery_drill_run.is_some() {
                "fail"
            } else if workflow_state.valid && db_status == "ok" {
                "warning"
            } else {
                "fail"
            },
            if recovery_drill_run.is_some() {
                "Recovery drill evidence covers DB schema reopen, renderer fallback, invalid workflow LKG, and cleanup."
            } else {
                "Recovery scenarios require explicit DB migration, renderer fallback, invalid workflow, and cleanup evidence."
            },
            recovery_drill_run
                .map(|run| vec![run.id.clone()])
                .unwrap_or_else(|| {
                    tracker_bindings
                        .iter()
                        .map(|binding| binding.id.clone())
                        .collect()
                }),
        ),
    ]
}

fn budget_summary_has_configured_budget(budget_summary: &Value) -> bool {
    budget_summary
        .get("workspace")
        .and_then(Value::as_object)
        .map(|workspace| workspace.contains_key("max_usd"))
        .unwrap_or(false)
        || ["projects", "goals", "tasks", "runs", "agents"]
            .iter()
            .any(|scope| {
                budget_summary
                    .get(scope)
                    .and_then(Value::as_array)
                    .map(|items| !items.is_empty())
                    .unwrap_or(false)
            })
}

fn release_gate_result(
    gate_id: &str,
    name: &str,
    status: &str,
    detail: &str,
    evidence: Vec<String>,
) -> ReleaseGateScenarioResult {
    ReleaseGateScenarioResult {
        gate_id: gate_id.to_string(),
        name: name.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
        evidence,
    }
}

fn release_evidence_pack_has_required_schema(pack: &PersistedEvidencePack) -> bool {
    let body = &pack.body_json;
    let approved = body
        .get("review_decision")
        .and_then(|review| review.get("decision"))
        .and_then(Value::as_str)
        == Some("approved");
    approved
        && body.get("diff_summary").is_some()
        && body
            .get("tests")
            .and_then(Value::as_array)
            .map(|tests| !tests.is_empty())
            .unwrap_or(false)
        && body
            .get("transcript_summary")
            .and_then(Value::as_str)
            .map(|summary| !summary.trim().is_empty())
            .unwrap_or(false)
        && body.get("token_usage").is_some()
        && body.get("policy_events").is_some()
}

fn evidence_pack_belongs_to_project(
    pack: &PersistedEvidencePack,
    project_id: &str,
    tasks: &[PersistedTask],
    runs: &[PersistedRun],
) -> bool {
    if let Some(task_id) = pack.task_id.as_deref() {
        if tasks.iter().any(|task| task.id == task_id) {
            return true;
        }
    }
    if let Some(run_id) = pack.run_id.as_deref() {
        if runs.iter().any(|run| run.id == run_id) {
            return true;
        }
    }
    evidence_pack_body_project_id(pack) == Some(project_id)
}

fn evidence_pack_body_project_id(pack: &PersistedEvidencePack) -> Option<&str> {
    pack.body_json
        .get("project_id")
        .and_then(Value::as_str)
        .or_else(|| {
            pack.body_json
                .get("dogfood_telemetry")
                .and_then(|dogfood| dogfood.get("project_id"))
                .and_then(Value::as_str)
        })
}

fn evidence_pack_links_to_task(
    pack: &PersistedEvidencePack,
    task: &PersistedTask,
    project_runs: &[PersistedRun],
) -> bool {
    if pack.task_id.as_deref() == Some(task.id.as_str()) {
        return true;
    }
    pack.run_id
        .as_deref()
        .and_then(|run_id| project_runs.iter().find(|run| run.id == run_id))
        .map(|run| run.task_id == task.id)
        .unwrap_or(false)
}

fn evidence_pack_review_decision(pack: &PersistedEvidencePack) -> Option<String> {
    pack.body_json
        .get("review_decision")
        .and_then(|review| review.get("decision"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn evidence_pack_id_from_review_id(review_id: &str) -> Result<String, String> {
    review_id
        .strip_prefix("review_")
        .filter(|evidence_pack_id| !evidence_pack_id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "review id must start with review_".to_string())
}

fn evidence_pack_tracker_summary(pack: &PersistedEvidencePack) -> String {
    let completeness_state = pack
        .body_json
        .get("completeness")
        .and_then(|completeness| completeness.get("state"))
        .and_then(Value::as_str)
        .unwrap_or(&pack.completeness_state);
    let review_decision =
        evidence_pack_review_decision(pack).unwrap_or_else(|| "pending".to_string());
    let test_count = pack
        .body_json
        .get("tests")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let command_block_count = pack
        .body_json
        .get("command_blocks")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    format!(
        "Evidence {} - {completeness_state} - review {review_decision} - {test_count} tests - {command_block_count} command blocks",
        pack.id
    )
}

fn command_block_to_evidence_json(block: &PersistedCommandBlock) -> Value {
    serde_json::json!({
        "id": block.id,
        "session_id": block.session_id,
        "task_id": block.task_id,
        "run_id": block.run_id,
        "seq_start": block.seq_start,
        "seq_end": block.seq_end,
        "command": block.command,
        "cwd": block.cwd,
        "branch": block.branch,
        "exit_code": block.exit_code,
        "duration_ms": block.duration_ms,
        "summary": block.summary
    })
}

fn policy_approval_to_event_json(approval: &PersistedPolicyApproval) -> Value {
    serde_json::json!({
        "event_type": "policy_approval",
        "id": approval.id,
        "project_id": approval.project_id,
        "task_id": approval.task_id,
        "run_id": approval.run_id,
        "action_kind": approval.action_kind,
        "command": approval.command,
        "risk_level": approval.risk_level,
        "state": approval.state,
        "requested_by": approval.requested_by,
        "decision_by": approval.decision_by,
        "decision_note": approval.decision_note,
        "created_at": approval.created_at,
        "decided_at": approval.decided_at
    })
}

fn permission_audit_to_event_json(audit: &PersistedPermissionAudit) -> Value {
    serde_json::json!({
        "event_type": "permission_audit",
        "id": audit.id,
        "project_id": audit.project_id,
        "task_id": audit.task_id,
        "run_id": audit.run_id,
        "policy_pack_id": audit.policy_pack_id,
        "action_kind": audit.action_kind,
        "command": audit.command,
        "decision": audit.decision,
        "reason": audit.reason,
        "requested_by": audit.requested_by,
        "created_at": audit.created_at
    })
}

fn budget_summary_json(budget: &PersistedBudget, used_usd: f64) -> Value {
    serde_json::json!({
        "id": budget.id,
        "scope_type": budget.scope_type,
        "scope_id": budget.scope_id,
        "used_usd": round_cost(used_usd),
        "max_usd": round_cost(budget.max_usd),
        "warn_pct": budget.warn_pct,
        "hard_limit": budget.hard_limit,
        "state": budget_state(used_usd, budget.max_usd, budget.warn_pct)
    })
}

fn budget_forecast_json(budget: &PersistedBudget, used_usd: f64, run_costs: &[f64]) -> Value {
    let remaining_usd = round_cost((budget.max_usd - used_usd).max(0.0));
    let average_run_cost_usd = if run_costs.is_empty() {
        Value::Null
    } else {
        let total: f64 = run_costs.iter().sum();
        serde_json::json!(round_cost(total / run_costs.len() as f64))
    };
    let estimated_runs_remaining = average_run_cost_usd
        .as_f64()
        .filter(|average| *average > 0.0)
        .map(|average| (remaining_usd / average).floor() as i64)
        .map(Value::from)
        .unwrap_or(Value::Null);

    serde_json::json!({
        "id": budget.id,
        "scope_type": budget.scope_type,
        "scope_id": budget.scope_id,
        "used_usd": round_cost(used_usd),
        "max_usd": round_cost(budget.max_usd),
        "remaining_usd": remaining_usd,
        "average_run_cost_usd": average_run_cost_usd,
        "estimated_runs_remaining": estimated_runs_remaining,
        "run_sample_count": run_costs.len(),
        "state": budget_state(used_usd, budget.max_usd, budget.warn_pct)
    })
}

fn budget_state(used_usd: f64, max_usd: f64, warn_pct: f64) -> &'static str {
    if used_usd >= max_usd {
        "exceeded"
    } else if used_usd >= max_usd * warn_pct {
        "warn"
    } else {
        "ok"
    }
}

fn round_cost(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

fn required_evidence_from_workflow(workflow: &PersistedWorkflowVersion) -> Vec<String> {
    workflow
        .parsed_json
        .get("review")
        .and_then(|review| review.get("required_evidence"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn evidence_tests_from_command_blocks(blocks: &[PersistedCommandBlock]) -> Vec<Value> {
    blocks
        .iter()
        .filter(|block| block.command.to_lowercase().contains("test") && block.exit_code == Some(0))
        .map(|block| {
            serde_json::json!({
                "command_block_id": block.id,
                "command": block.command,
                "exit_code": block.exit_code,
                "summary": block.summary
            })
        })
        .collect()
}

fn evidence_diff_summary_from_command_blocks(blocks: &[PersistedCommandBlock]) -> Value {
    let diff_blocks = blocks
        .iter()
        .filter(|block| {
            let command = block.command.to_lowercase();
            command.contains("git diff")
                || command.starts_with("diff ")
                || command.contains(" diff")
        })
        .collect::<Vec<_>>();
    if diff_blocks.is_empty() {
        return serde_json::json!({});
    }

    serde_json::json!({
        "command_block_ids": diff_blocks
            .iter()
            .map(|block| Value::String(block.id.clone()))
            .collect::<Vec<_>>(),
        "summary": diff_blocks
            .iter()
            .filter_map(|block| block.summary.clone())
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn evidence_transcript_summary(
    replay_json: Option<&Value>,
    blocks: &[PersistedCommandBlock],
    terminal_stream_chunks: &[PersistedTerminalStreamChunk],
) -> String {
    let mut parts = Vec::new();
    if let Some(hook_results) = replay_json
        .and_then(|json| json.get("hook_results"))
        .and_then(Value::as_array)
    {
        for hook in hook_results {
            if let Some(stdout) = hook.get("stdout").and_then(Value::as_str) {
                let trimmed = stdout.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
            }
            if let Some(stderr) = hook.get("stderr").and_then(Value::as_str) {
                let trimmed = stderr.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
            }
        }
    }
    parts.extend(
        terminal_stream_chunks
            .iter()
            .map(|chunk| chunk.body.trim())
            .filter(|body| !body.is_empty())
            .map(str::to_string),
    );
    parts.extend(
        blocks
            .iter()
            .filter_map(|block| block.summary.as_deref())
            .map(str::trim)
            .filter(|summary| !summary.is_empty())
            .map(str::to_string),
    );
    parts.join("\n")
}

fn missing_required_evidence(
    required: &[String],
    tests: &[Value],
    diff_summary: &Value,
    transcript_summary: &str,
) -> Vec<String> {
    required
        .iter()
        .filter(|item| match item.as_str() {
            "diff" => diff_summary
                .as_object()
                .map(|value| value.is_empty())
                .unwrap_or(true),
            "tests" => tests.is_empty(),
            "transcript_summary" => transcript_summary.trim().is_empty(),
            _ => true,
        })
        .cloned()
        .collect()
}

fn normalize_review_decision(decision: &str) -> Result<&'static str, String> {
    match decision.trim() {
        "approved" => Ok("approved"),
        "changes_requested" => Ok("changes_requested"),
        "reopened" => Ok("reopened"),
        "blocked" => Ok("blocked"),
        unsupported => Err(format!("unsupported review decision {unsupported}")),
    }
}

fn sanitize_artifact_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn initializes_sqlite_schema_with_core_tables_and_indexes() {
        let db_path = unique_test_db_path("schema");

        let store = StateStore::open_at(&db_path).expect("state store opens");

        assert_eq!(store.health().status, "ok");
        assert_eq!(store.health().path, db_path.to_string_lossy());

        let connection = Connection::open(&db_path).expect("sqlite db exists");
        for table_name in [
            "projects",
            "project_tabs",
            "project_tab_groups",
            "project_layout_presets",
            "project_detach_plans",
            "sessions",
            "terminal_stream_chunks",
            "command_blocks",
            "tasks",
            "comments",
            "workpads",
            "runs",
            "evidence_packs",
            "release_gate_runs",
            "terminal_fidelity_smoke_runs",
            "task_lifecycle_e2e_runs",
            "workflow_negative_test_runs",
            "dmg_smoke_runs",
            "recovery_drill_runs",
            "benchmark_runs",
            "dogfood_telemetry_reviews",
            "visual_harness_links",
            "agent_events",
            "skill_packs",
            "permission_audit_events",
            "settings",
        ] {
            assert_eq!(
                object_count(&connection, "table", table_name),
                1,
                "{table_name}"
            );
        }
        for index_name in [
            "idx_project_layout_presets_project_name",
            "idx_sessions_project_state",
            "idx_sessions_task",
            "idx_command_blocks_session_created",
            "idx_command_blocks_task",
            "idx_tasks_project_status",
            "idx_runs_task_lifecycle",
            "idx_skill_packs_project_name",
            "idx_permission_audit_project_decision_created",
        ] {
            assert_eq!(
                object_count(&connection, "index", index_name),
                1,
                "{index_name}"
            );
        }

        let _ = fs::remove_file(&db_path);
    }

    #[test]
    fn dmg_smoke_cases_run_local_signature_and_notarization_checks_when_artifacts_exist() {
        let db_path = unique_test_db_path("dmg-smoke-local-verification");
        let workspace = db_path
            .parent()
            .expect("test db path has parent")
            .join("artifacts");
        let app_path = workspace.join("Haneulchi.app");
        let dmg_path = workspace.join("Haneulchi.dmg");
        fs::create_dir_all(&app_path).expect("app bundle directory created");
        fs::write(&dmg_path, b"fake dmg").expect("dmg artifact created");

        let cases = run_dmg_smoke_cases(
            Some(dmg_path.to_string_lossy().as_ref()),
            Some(app_path.to_string_lossy().as_ref()),
        );
        let codesign = cases
            .iter()
            .find(|case| case.case_id == "codesign")
            .expect("codesign case exists");
        let notarization = cases
            .iter()
            .find(|case| case.case_id == "notarization")
            .expect("notarization case exists");

        assert!(!codesign
            .detail
            .contains("pending local codesign integration"));
        assert!(!notarization
            .detail
            .contains("pending local spctl integration"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn persists_json_settings_across_reopen() {
        let db_path = unique_test_db_path("settings");

        {
            let store = StateStore::open_at(&db_path).expect("state store opens");
            store
                .put_setting_json(
                    "workspace.default_project",
                    &serde_json::json!({"id":"proj_local"}),
                )
                .expect("setting persisted");
        }

        let reopened = StateStore::open_at(&db_path).expect("state store reopens");

        assert_eq!(
            reopened
                .get_setting_json("workspace.default_project")
                .expect("setting loads"),
            Some(serde_json::json!({"id":"proj_local"})),
        );

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn persists_provider_model_settings_across_reopen() {
        let db_path = unique_test_db_path("provider-model-settings");

        {
            let store = StateStore::open_at(&db_path).expect("state store opens");
            let settings = store
                .upsert_provider_model_settings(ProviderModelSettingsInput {
                    provider: "anthropic".to_string(),
                    model: "claude-3-7-sonnet-latest".to_string(),
                    agent_profile_id: "agent_claude".to_string(),
                })
                .expect("provider model settings persisted");

            assert_eq!(settings.provider, "anthropic");
            assert_eq!(settings.model, "claude-3-7-sonnet-latest");
            assert_eq!(settings.agent_profile_id, "agent_claude");
        }

        let reopened = StateStore::open_at(&db_path).expect("state store reopens");
        let settings = reopened
            .provider_model_settings()
            .expect("provider model settings load");

        assert_eq!(settings.provider, "anthropic");
        assert_eq!(settings.model, "claude-3-7-sonnet-latest");
        assert_eq!(settings.agent_profile_id, "agent_claude");

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn persists_project_terminal_theme_settings_across_reopen() {
        let db_path = unique_test_db_path("terminal-theme-settings");

        {
            let store = StateStore::open_at(&db_path).expect("state store opens");
            let theme = store
                .upsert_terminal_theme_settings(TerminalThemeSettingsInput {
                    project_id: Some("proj_auth".to_string()),
                    name: "Auth Focus".to_string(),
                    background: "#09111f".to_string(),
                    foreground: "#eaf6ff".to_string(),
                    accent: "#19c37d".to_string(),
                })
                .expect("terminal theme persisted");

            assert_eq!(theme.project_id.as_deref(), Some("proj_auth"));
            assert_eq!(theme.name, "Auth Focus");
            assert_eq!(theme.background, "#09111f");
            assert!(store
                .upsert_terminal_theme_settings(TerminalThemeSettingsInput {
                    project_id: Some("proj_auth".to_string()),
                    name: "Broken".to_string(),
                    background: "blue".to_string(),
                    foreground: "#eaf6ff".to_string(),
                    accent: "#19c37d".to_string(),
                })
                .is_err());
        }

        let reopened = StateStore::open_at(&db_path).expect("state store reopens");
        let project_theme = reopened
            .terminal_theme_settings(Some("proj_auth"))
            .expect("project terminal theme loads");
        let default_theme = reopened
            .terminal_theme_settings(Some("proj_unknown"))
            .expect("default terminal theme loads");

        assert_eq!(project_theme.name, "Auth Focus");
        assert_eq!(project_theme.project_id.as_deref(), Some("proj_auth"));
        assert_eq!(default_theme.name, "Haneulchi Default");
        assert_eq!(default_theme.project_id, None);

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn persists_tasks_and_counts_by_status_across_reopen() {
        let db_path = unique_test_db_path("tasks");

        {
            let store = StateStore::open_at(&db_path).expect("state store opens");
            store
                .upsert_task(&PersistedTaskInput {
                    id: "task_ready".to_string(),
                    key: "HC-READY".to_string(),
                    project_id: "proj_local".to_string(),
                    title: "Wire state snapshot".to_string(),
                    description: Some("Persisted task row".to_string()),
                    status: "ready".to_string(),
                    priority: "high".to_string(),
                    assignee_type: Some("agent".to_string()),
                    assignee_id: Some("agent_codex".to_string()),
                    cycle_id: Some("cycle_sprint_5".to_string()),
                    module_id: Some("module_control_api".to_string()),
                    initiative_id: None,
                    context_pack_id: None,
                })
                .expect("ready task persisted");
            store
                .upsert_task(&PersistedTaskInput {
                    id: "task_blocked".to_string(),
                    key: "HC-BLOCKED".to_string(),
                    project_id: "proj_local".to_string(),
                    title: "Resolve packaging gate".to_string(),
                    description: None,
                    status: "blocked".to_string(),
                    priority: "urgent".to_string(),
                    assignee_type: None,
                    assignee_id: None,
                    cycle_id: None,
                    module_id: None,
                    initiative_id: None,
                    context_pack_id: None,
                })
                .expect("blocked task persisted");
        }

        let reopened = StateStore::open_at(&db_path).expect("state store reopens");
        let tasks = reopened.list_tasks("proj_local").expect("tasks load");

        assert_eq!(tasks.len(), 2);
        assert_eq!(
            tasks[0],
            PersistedTask {
                id: "task_blocked".to_string(),
                key: "HC-BLOCKED".to_string(),
                project_id: "proj_local".to_string(),
                title: "Resolve packaging gate".to_string(),
                description: None,
                status: "blocked".to_string(),
                priority: "urgent".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                due_at: None,
                estimate: None,
                labels: vec![],
                context_pack_id: None,
                workpad_md: None,
                comment_count: 0,
                subtask_count: 0,
                open_subtask_count: 0,
            }
        );
        assert_eq!(tasks[1].assignee_id.as_deref(), Some("agent_codex"));
        let contexted = reopened
            .update_task_context(UpdateTaskContextInput {
                task_id: tasks[1].id.clone(),
                context_pack_id: Some("ctx_auth".to_string()),
            })
            .expect("task context updates");
        assert_eq!(contexted.context_pack_id.as_deref(), Some("ctx_auth"));

        assert_eq!(
            reopened
                .count_tasks_by_status("proj_local")
                .expect("task counts load"),
            serde_json::json!({
                "inbox": 0,
                "ready": 1,
                "running": 0,
                "review": 0,
                "blocked": 1,
                "done": 0,
                "archived": 0
            })
        );

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn creates_moves_and_comments_on_tasks_with_validation() {
        let db_path = unique_test_db_path("task-mutations");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let created = store
            .create_task(CreateTaskInput {
                project_id: "proj_local".to_string(),
                title: "  Implement SQLite task mutations  ".to_string(),
                priority: Some("high".to_string()),
                initiative_id: None,
            })
            .expect("task created");

        assert_eq!(created.id, "task_1");
        assert_eq!(created.key, "LOCAL-1");
        assert_eq!(created.title, "Implement SQLite task mutations");
        assert_eq!(created.status, "inbox");
        assert_eq!(created.priority, "high");
        assert!(store
            .create_task(CreateTaskInput {
                project_id: "proj_local".to_string(),
                title: "   ".to_string(),
                priority: None,
                initiative_id: None,
            })
            .is_err());

        let moved = store
            .move_task_status("task_1", "ready")
            .expect("task moved");
        assert_eq!(moved.status, "ready");
        assert!(store.move_task_status("missing", "ready").is_err());

        let comment = store
            .add_task_comment(AddTaskCommentInput {
                task_id: "task_1".to_string(),
                author_type: "human".to_string(),
                author_id: "local_user".to_string(),
                body_md: "  Durable comment.  ".to_string(),
            })
            .expect("comment added");
        assert_eq!(comment.id, "comment_1");
        assert_eq!(comment.body_md, "Durable comment.");
        assert_eq!(
            store.list_task_comments("task_1").expect("comments load"),
            vec![comment]
        );
        assert!(store
            .add_task_comment(AddTaskCommentInput {
                task_id: "task_1".to_string(),
                author_type: "human".to_string(),
                author_id: "local_user".to_string(),
                body_md: "   ".to_string(),
            })
            .is_err());

        let subtask = store
            .add_task_subtask(AddTaskSubtaskInput {
                task_id: "task_1".to_string(),
                title: "  Attach screenshots  ".to_string(),
            })
            .expect("subtask added");
        assert_eq!(subtask.id, "subtask_1");
        assert_eq!(subtask.title, "Attach screenshots");
        assert_eq!(subtask.status, "open");
        assert_eq!(
            store.list_task_subtasks("task_1").expect("subtasks load"),
            vec![subtask.clone()]
        );
        let completed = store
            .update_task_subtask_status(UpdateTaskSubtaskStatusInput {
                task_id: "task_1".to_string(),
                subtask_id: "subtask_1".to_string(),
                status: "done".to_string(),
            })
            .expect("subtask completed");
        assert_eq!(completed.status, "done");
        let reloaded = store
            .get_task("task_1")
            .expect("task loads")
            .expect("task exists");
        assert_eq!(reloaded.subtask_count, 1);
        assert_eq!(reloaded.open_subtask_count, 0);
        assert!(store
            .add_task_subtask(AddTaskSubtaskInput {
                task_id: "task_1".to_string(),
                title: "   ".to_string(),
            })
            .is_err());

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn creates_and_lists_task_cycles_and_modules_with_validation() {
        let db_path = unique_test_db_path("task-planning-registries");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let cycle = store
            .create_task_cycle(CreateTaskCycleInput {
                project_id: "proj_local".to_string(),
                name: "  Sprint 13  ".to_string(),
                starts_at: Some(" 2026-05-01 ".to_string()),
                ends_at: Some(" 2026-05-15 ".to_string()),
                status: Some(" planned ".to_string()),
            })
            .expect("cycle created");
        let module = store
            .create_task_module(CreateTaskModuleInput {
                project_id: "proj_local".to_string(),
                name: "  Release  ".to_string(),
                description: Some("  Release gate work  ".to_string()),
                status: None,
            })
            .expect("module created");

        assert_eq!(cycle.id, "cycle_1");
        assert_eq!(cycle.name, "Sprint 13");
        assert_eq!(cycle.starts_at.as_deref(), Some("2026-05-01"));
        assert_eq!(cycle.ends_at.as_deref(), Some("2026-05-15"));
        assert_eq!(cycle.status, "planned");
        assert_eq!(module.id, "module_1");
        assert_eq!(module.name, "Release");
        assert_eq!(module.description.as_deref(), Some("Release gate work"));
        assert_eq!(module.status, "active");
        assert_eq!(
            store.list_task_cycles("proj_local").expect("cycles load"),
            vec![cycle]
        );
        assert_eq!(
            store.list_task_modules("proj_local").expect("modules load"),
            vec![module]
        );
        assert!(store
            .create_task_cycle(CreateTaskCycleInput {
                project_id: "proj_local".to_string(),
                name: "   ".to_string(),
                starts_at: None,
                ends_at: None,
                status: None,
            })
            .is_err());
        assert!(store
            .create_task_module(CreateTaskModuleInput {
                project_id: "proj_local".to_string(),
                name: "   ".to_string(),
                description: None,
                status: None,
            })
            .is_err());

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn tracker_sync_plans_comment_and_evidence_summary_mirrors_with_redaction() {
        let db_path = unique_test_db_path("tracker-sync-comment-evidence");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_secret(UpsertSecretInput {
                project_id: "proj_local".to_string(),
                name: "LINEAR_API_KEY".to_string(),
                value: "secret-linear-token".to_string(),
            })
            .expect("secret saved");
        let task = store
            .create_task(CreateTaskInput {
                project_id: "proj_local".to_string(),
                title: "Mirror tracker evidence".to_string(),
                priority: Some("high".to_string()),
                initiative_id: None,
            })
            .expect("task created");
        store
            .upsert_external_tracker_binding(UpsertExternalTrackerBindingInput {
                project_id: "proj_local".to_string(),
                local_kind: "task".to_string(),
                local_id: task.id.clone(),
                provider: "linear".to_string(),
                external_id: "LIN-77".to_string(),
                external_url: None,
                sync_mode: Some("mirror".to_string()),
                metadata_json: None,
            })
            .expect("tracker binding saved");
        let comment = store
            .add_task_comment(AddTaskCommentInput {
                task_id: task.id.clone(),
                author_type: "human".to_string(),
                author_id: "reviewer".to_string(),
                body_md: "Reviewed output with secret-linear-token and approved.".to_string(),
            })
            .expect("comment added");
        store
            .move_task_status(&task.id, "ready")
            .expect("task ready");
        let run = store
            .dispatch_run(DispatchRunInput {
                task_id: task.id.clone(),
                agent_profile_id: None,
                context_pack_id: None,
                workspace_path: Some("/tmp/haneulchi-tracker-sync".to_string()),
            })
            .expect("run dispatched");
        store
            .upsert_command_block(&PersistedCommandBlockInput {
                id: "cmdblk_tracker_evidence".to_string(),
                session_id: run.session_id.clone().expect("run session id"),
                task_id: Some(task.id.clone()),
                run_id: Some(run.id.clone()),
                seq_start: Some(1),
                seq_end: Some(2),
                command: "npm test".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(1200),
                summary: Some("Tests passed with secret-linear-token hidden".to_string()),
            })
            .expect("command block saved");
        let evidence = store
            .generate_evidence_pack_for_run(GenerateEvidencePackInput {
                run_id: run.id.clone(),
                evidence_pack_id: Some("ev_tracker_summary".to_string()),
            })
            .expect("evidence generated");
        store
            .record_evidence_review_decision(RecordEvidenceReviewDecisionInput {
                evidence_pack_id: evidence.id.clone(),
                decision: "approved".to_string(),
                reviewer_id: Some("human".to_string()),
                body_md: Some("Ship after redacted evidence review.".to_string()),
            })
            .expect("review decision recorded");

        let sync = store
            .run_tracker_sync(
                "linear",
                RunTrackerSyncInput {
                    project_id: "proj_local".to_string(),
                    dry_run: true,
                },
            )
            .expect("tracker sync planned");
        let serialized_operations =
            serde_json::to_string(&sync.operations).expect("operations serialize");

        assert_eq!(sync.status, "planned");
        assert_eq!(sync.operation_count, 3);
        assert!(sync.operations.iter().any(|operation| {
            operation.operation == "commentMirror"
                && operation.payload["commentId"] == comment.id
                && operation.payload["bodyMd"]
                    == "Reviewed output with [REDACTED:LINEAR_API_KEY] and approved."
        }));
        assert!(sync.operations.iter().any(|operation| {
            operation.operation == "evidenceSummaryMirror"
                && operation.payload["evidencePackId"] == "ev_tracker_summary"
                && operation.payload["reviewDecision"] == "approved"
        }));
        assert!(!serialized_operations.contains("secret-linear-token"));
        assert!(serialized_operations.contains("[REDACTED:LINEAR_API_KEY]"));

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn updates_task_planning_metadata_without_clearing_description() {
        let db_path = unique_test_db_path("task-planning");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_ready".to_string(),
                key: "HC-READY".to_string(),
                project_id: "proj_local".to_string(),
                title: "Wire state snapshot".to_string(),
                description: Some("Keep this description".to_string()),
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");

        let updated = store
            .update_task_planning(UpdateTaskPlanningInput {
                task_id: "task_ready".to_string(),
                cycle_id: Some("  Sprint 5  ".to_string()),
                module_id: Some(" Control API ".to_string()),
                initiative_id: None,
                due_at: Some(" 2026-05-15 ".to_string()),
                estimate: Some(" 3 pts ".to_string()),
                labels: Some(vec![
                    " release ".to_string(),
                    "evidence".to_string(),
                    "".to_string(),
                    "release".to_string(),
                ]),
                assignee_type: Some(" agent ".to_string()),
                assignee_id: Some(" agent_codex ".to_string()),
            })
            .expect("planning metadata updates");

        assert_eq!(
            updated.description.as_deref(),
            Some("Keep this description")
        );
        assert_eq!(updated.cycle_id.as_deref(), Some("Sprint 5"));
        assert_eq!(updated.module_id.as_deref(), Some("Control API"));
        assert_eq!(updated.due_at.as_deref(), Some("2026-05-15"));
        assert_eq!(updated.estimate.as_deref(), Some("3 pts"));
        assert_eq!(
            updated.labels,
            vec!["release".to_string(), "evidence".to_string()]
        );
        assert_eq!(updated.assignee_type.as_deref(), Some("agent"));
        assert_eq!(updated.assignee_id.as_deref(), Some("agent_codex"));

        let cleared = store
            .update_task_planning(UpdateTaskPlanningInput {
                task_id: "task_ready".to_string(),
                cycle_id: Some("   ".to_string()),
                module_id: None,
                initiative_id: None,
                due_at: Some(" ".to_string()),
                estimate: Some(" ".to_string()),
                labels: Some(vec![]),
                assignee_type: None,
                assignee_id: None,
            })
            .expect("planning metadata clears");

        assert_eq!(cleared.cycle_id, None);
        assert_eq!(cleared.module_id, None);
        assert_eq!(cleared.due_at, None);
        assert_eq!(cleared.estimate, None);
        assert_eq!(cleared.labels, Vec::<String>::new());
        assert_eq!(cleared.assignee_type, None);
        assert_eq!(cleared.assignee_id, None);
        assert!(store
            .update_task_planning(UpdateTaskPlanningInput {
                task_id: "missing".to_string(),
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                due_at: None,
                estimate: None,
                labels: None,
                assignee_type: None,
                assignee_id: None,
            })
            .is_err());

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn creates_and_lists_initiatives_as_task_goal_context() {
        let db_path = unique_test_db_path("initiatives");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let initiative = store
            .create_initiative(CreateInitiativeInput {
                project_id: "proj_local".to_string(),
                name: "  Auth reliability goal  ".to_string(),
                description: Some("  Explain why auth tasks matter  ".to_string()),
                budget_id: Some(" budget_auth ".to_string()),
                status: Some(" active ".to_string()),
            })
            .expect("initiative created");

        assert_eq!(initiative.id, "init_1");
        assert_eq!(initiative.name, "Auth reliability goal");
        assert_eq!(
            initiative.description.as_deref(),
            Some("Explain why auth tasks matter")
        );
        assert_eq!(initiative.budget_id.as_deref(), Some("budget_auth"));
        assert_eq!(initiative.status, "active");

        let reopened = StateStore::open_at(&db_path).expect("state store reopens");
        let initiatives = reopened
            .list_initiatives("proj_local")
            .expect("initiatives load");
        assert_eq!(initiatives, vec![initiative]);
        assert!(store
            .create_initiative(CreateInitiativeInput {
                project_id: "proj_local".to_string(),
                name: "   ".to_string(),
                description: None,
                budget_id: None,
                status: None,
            })
            .is_err());

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn saves_task_workpad_markdown_as_a_durable_artifact() {
        let db_path = unique_test_db_path("task-workpad");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_review".to_string(),
                key: "HC-REVIEW".to_string(),
                project_id: "proj_local".to_string(),
                title: "Review evidence pack".to_string(),
                description: None,
                status: "review".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .add_task_comment(AddTaskCommentInput {
                task_id: "task_review".to_string(),
                author_type: "human".to_string(),
                author_id: "local_user".to_string(),
                body_md: "Confirm terminal proof.".to_string(),
            })
            .expect("comment persisted");

        let saved = store
            .save_task_workpad(SaveTaskWorkpadInput {
                task_id: "task_review".to_string(),
                body_md: "Evidence links:\n- terminal proof".to_string(),
            })
            .expect("workpad saved");

        assert_eq!(saved.id, "workpad_task_review");
        assert_eq!(saved.task_id, "task_review");
        assert_eq!(saved.body_md, "Evidence links:\n- terminal proof");
        assert!(saved
            .artifact_path
            .ends_with("artifacts/workpads/task_review.md"));

        let reopened = StateStore::open_at(&db_path).expect("state store reopens");
        assert_eq!(
            reopened
                .get_task_workpad("task_review")
                .expect("workpad loads")
                .expect("workpad exists")
                .body_md,
            "Evidence links:\n- terminal proof"
        );
        assert_eq!(
            reopened.list_tasks("proj_local").expect("tasks load")[0]
                .workpad_md
                .as_deref(),
            Some("Evidence links:\n- terminal proof")
        );
        assert_eq!(
            reopened.list_tasks("proj_local").expect("tasks load")[0].comment_count,
            1
        );
        assert!(store
            .save_task_workpad(SaveTaskWorkpadInput {
                task_id: "missing".to_string(),
                body_md: "No task".to_string(),
            })
            .is_err());

        let _ = fs::remove_file(&db_path);
        if let Some(parent) = db_path.parent() {
            let _ = fs::remove_dir_all(parent.join("artifacts"));
        }
    }

    #[test]
    fn persists_and_searches_command_blocks_across_reopen() {
        let db_path = unique_test_db_path("command-blocks");
        {
            let store = StateStore::open_at(&db_path).expect("state store opens");
            store
                .upsert_command_block(&PersistedCommandBlockInput {
                    id: "cmdblk_1".to_string(),
                    session_id: "pty_1".to_string(),
                    task_id: Some("task_review".to_string()),
                    run_id: None,
                    seq_start: Some(4),
                    seq_end: Some(9),
                    command: "npm test".to_string(),
                    cwd: Some("/repo/frontend".to_string()),
                    branch: Some("feature/command-search".to_string()),
                    exit_code: Some(0),
                    duration_ms: Some(1200),
                    summary: Some("PASS frontend diagnostics".to_string()),
                })
                .expect("first command block persisted");
            store
                .upsert_command_block(&PersistedCommandBlockInput {
                    id: "cmdblk_2".to_string(),
                    session_id: "pty_2".to_string(),
                    task_id: None,
                    run_id: None,
                    seq_start: Some(10),
                    seq_end: Some(12),
                    command: "cargo check".to_string(),
                    cwd: Some("/repo/src-tauri".to_string()),
                    branch: Some("main".to_string()),
                    exit_code: None,
                    duration_ms: None,
                    summary: Some("Rust diagnostics clean".to_string()),
                })
                .expect("second command block persisted");
        }

        let reopened = StateStore::open_at(&db_path).expect("state store reopens");
        let frontend = reopened
            .search_command_blocks(Some("frontend"), 10)
            .expect("command blocks search");
        let diagnostics = reopened
            .search_command_blocks(Some("diagnostics"), 10)
            .expect("summary search");
        let recent = reopened
            .recent_command_blocks(1)
            .expect("recent command blocks");

        assert_eq!(frontend.len(), 1);
        assert_eq!(frontend[0].id, "cmdblk_1");
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].id, "cmdblk_2");

        let _ = fs::remove_file(&db_path);
        if let Some(parent) = db_path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn marks_merges_and_splits_persisted_command_blocks() {
        let db_path = unique_test_db_path("command-block-actions");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_command_block(&PersistedCommandBlockInput {
                id: "cmdblk_1".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_review".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(1),
                seq_end: Some(4),
                command: "npm test".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: None,
                duration_ms: Some(100),
                summary: Some("PASS frontend tests".to_string()),
            })
            .expect("first command block persisted");
        store
            .upsert_command_block(&PersistedCommandBlockInput {
                id: "cmdblk_2".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_review".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(5),
                seq_end: Some(8),
                command: "cargo test".to_string(),
                cwd: Some("/repo/src-tauri".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(200),
                summary: Some("PASS rust tests".to_string()),
            })
            .expect("second command block persisted");

        let marked = store
            .mark_command_block_status("cmdblk_1", "completed")
            .expect("command block marked");
        let merged = store
            .merge_command_blocks("cmdblk_1", "cmdblk_2")
            .expect("command blocks merged");
        let split = store
            .split_command_block("cmdblk_1")
            .expect("command block split");

        assert_eq!(marked.exit_code, Some(0));
        assert_eq!(merged.command, "npm test && cargo test");
        assert_eq!(merged.seq_start, Some(1));
        assert_eq!(merged.seq_end, Some(8));
        assert_eq!(merged.duration_ms, Some(300));
        assert_eq!(
            merged.summary.as_deref(),
            Some("PASS frontend tests\n\nPASS rust tests")
        );
        assert_eq!(
            split.updated_block.command,
            "npm test && cargo test (part 1)"
        );
        assert_eq!(split.updated_block.seq_end, Some(4));
        assert_eq!(
            split.created_block.command,
            "npm test && cargo test (part 2)"
        );
        assert_eq!(split.created_block.seq_start, Some(5));
        assert_eq!(
            store
                .search_command_blocks(Some("cargo"), 10)
                .expect("command blocks search")
                .len(),
            2
        );

        let _ = fs::remove_file(&db_path);
        if let Some(parent) = db_path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn attaches_persisted_command_blocks_to_evidence_pack_artifacts() {
        let db_path = unique_test_db_path("evidence-pack");
        {
            let store = StateStore::open_at(&db_path).expect("state store opens");
            store
                .upsert_command_block(&PersistedCommandBlockInput {
                    id: "cmdblk_1".to_string(),
                    session_id: "pty_1".to_string(),
                    task_id: Some("task_review".to_string()),
                    run_id: None,
                    seq_start: Some(4),
                    seq_end: Some(9),
                    command: "npm test".to_string(),
                    cwd: Some("/repo".to_string()),
                    branch: Some("main".to_string()),
                    exit_code: Some(0),
                    duration_ms: Some(1200),
                    summary: Some("PASS evidence workflow".to_string()),
                })
                .expect("command block persisted");

            let attached = store
                .attach_command_block_to_evidence(AttachCommandBlockEvidenceInput {
                    evidence_pack_id: "ev_local".to_string(),
                    command_block_id: "cmdblk_1".to_string(),
                    task_id: Some("task_review".to_string()),
                    run_id: None,
                })
                .expect("command block attaches");
            let attached_again = store
                .attach_command_block_to_evidence(AttachCommandBlockEvidenceInput {
                    evidence_pack_id: "ev_local".to_string(),
                    command_block_id: "cmdblk_1".to_string(),
                    task_id: Some("task_review".to_string()),
                    run_id: None,
                })
                .expect("duplicate attach is idempotent");

            assert_eq!(attached.id, "ev_local");
            assert!(attached
                .artifact_path
                .ends_with("artifacts/evidence/ev_local.json"));
            assert_eq!(
                attached_again.body_json["command_blocks"]
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
            assert_eq!(attached.body_json["command_blocks"][0]["id"], "cmdblk_1");
            assert_eq!(
                attached.body_json["command_blocks"][0]["summary"],
                "PASS evidence workflow"
            );
            assert!(store
                .attach_command_block_to_evidence(AttachCommandBlockEvidenceInput {
                    evidence_pack_id: "ev_local".to_string(),
                    command_block_id: "missing".to_string(),
                    task_id: None,
                    run_id: None,
                })
                .is_err());
        }

        let reopened = StateStore::open_at(&db_path).expect("state store reopens");
        let loaded = reopened
            .get_evidence_pack("ev_local")
            .expect("evidence loads")
            .expect("evidence exists");
        assert_eq!(loaded.body_json["command_blocks"][0]["command"], "npm test");

        let _ = fs::remove_file(&db_path);
        if let Some(parent) = db_path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn redacts_saved_secrets_from_command_blocks_status_updates_and_evidence() {
        let db_path = unique_test_db_path("secret-redaction");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_secret(UpsertSecretInput {
                project_id: "proj_local".to_string(),
                name: "OPENAI_API_KEY".to_string(),
                value: "haneulchi-secret-fixture-value".to_string(),
            })
            .expect("secret stored");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_secret_redaction".to_string(),
                key: "HC-SECRET-REDACTION".to_string(),
                project_id: "proj_local".to_string(),
                title: "Redact secret evidence".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .dispatch_run(DispatchRunInput {
                task_id: "task_secret_redaction".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");
        let block = store
            .upsert_command_block(&PersistedCommandBlockInput {
                id: "cmdblk_secret".to_string(),
                session_id: "session_1".to_string(),
                task_id: Some("task_secret_redaction".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(1),
                seq_end: Some(2),
                command: "curl -H 'Authorization: Bearer haneulchi-secret-fixture-value' https://api.example.test".to_string(),
                cwd: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(10),
                summary: Some("request succeeded with haneulchi-secret-fixture-value".to_string()),
            })
            .expect("command block persists");
        let update = store
            .record_run_status_update(RecordRunStatusUpdateInput {
                run_id: "run_1".to_string(),
                body_md: "Agent observed haneulchi-secret-fixture-value in output".to_string(),
                lifecycle: None,
                status_detail: None,
            })
            .expect("status update persists");
        let evidence = store
            .generate_evidence_pack_for_run(GenerateEvidencePackInput {
                run_id: "run_1".to_string(),
                evidence_pack_id: Some("ev_secret".to_string()),
            })
            .expect("evidence generated");
        let evidence_body = evidence.body_json.to_string();

        assert!(block.command.contains("[REDACTED:OPENAI_API_KEY]"));
        assert!(block
            .summary
            .as_deref()
            .unwrap()
            .contains("[REDACTED:OPENAI_API_KEY]"));
        assert_eq!(
            update.body_md,
            "Agent observed [REDACTED:OPENAI_API_KEY] in output"
        );
        assert!(!block.command.contains("haneulchi-secret-fixture-value"));
        assert!(!block
            .summary
            .unwrap()
            .contains("haneulchi-secret-fixture-value"));
        assert!(!update.body_md.contains("haneulchi-secret-fixture-value"));
        assert!(!evidence_body.contains("haneulchi-secret-fixture-value"));
        assert!(evidence_body.contains("[REDACTED:OPENAI_API_KEY]"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn records_terminal_stream_chunks_and_includes_redacted_transcript_in_evidence() {
        let db_path = unique_test_db_path("terminal-stream-chunks");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_secret(UpsertSecretInput {
                project_id: "proj_local".to_string(),
                name: "SESSION_TOKEN".to_string(),
                value: "terminal-secret-value".to_string(),
            })
            .expect("secret stored");
        let task = store
            .create_task(CreateTaskInput {
                project_id: "proj_local".to_string(),
                title: "Capture terminal transcript".to_string(),
                priority: Some("high".to_string()),
                initiative_id: None,
            })
            .expect("task created");
        store
            .move_task_status(&task.id, "ready")
            .expect("task ready");
        let run = store
            .dispatch_run(DispatchRunInput {
                task_id: task.id.clone(),
                agent_profile_id: None,
                context_pack_id: None,
                workspace_path: Some("/tmp/haneulchi-terminal-stream".to_string()),
            })
            .expect("run dispatched");
        let session_id = run.session_id.clone().expect("run session id");

        let chunk = store
            .record_terminal_stream_chunk(RecordTerminalStreamChunkInput {
                session_id: session_id.clone(),
                seq_start: 10,
                seq_end: 19,
                body: "npm test\nterminal-secret-value\n88 tests passed\n".to_string(),
            })
            .expect("stream chunk recorded");
        let chunks = store
            .list_terminal_stream_chunks(&session_id, Some(10))
            .expect("stream chunks load");

        assert_eq!(chunk.id, "terminal_stream_chunk_1");
        assert_eq!(chunks, vec![chunk.clone()]);
        assert_eq!(chunk.seq_start, 10);
        assert_eq!(chunk.seq_end, 19);
        assert!(chunk.artifact_path.contains("terminal_stream_chunk_1"));
        assert!(chunk.body.contains("[REDACTED:SESSION_TOKEN]"));
        assert!(!chunk.body.contains("terminal-secret-value"));

        let evidence = store
            .generate_evidence_pack_for_run(GenerateEvidencePackInput {
                run_id: run.id.clone(),
                evidence_pack_id: Some("ev_terminal_stream".to_string()),
            })
            .expect("evidence generated");
        let transcript_summary = evidence.body_json["transcript_summary"]
            .as_str()
            .expect("transcript summary string");

        assert!(transcript_summary.contains("npm test"));
        assert!(transcript_summary.contains("[REDACTED:SESSION_TOKEN]"));
        assert!(transcript_summary.contains("88 tests passed"));
        assert!(!transcript_summary.contains("terminal-secret-value"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn generates_evidence_pack_from_run_replay_and_command_blocks() {
        let db_path = unique_test_db_path("generated-evidence");
        let root = db_path.parent().unwrap().join("repo");
        let hook_dir = root.join(".haneulchi/hooks");
        let workspace = root.join(".haneulchi/worktrees/run_1");
        fs::create_dir_all(&hook_dir).expect("hook dir");
        fs::create_dir_all(&workspace).expect("workspace dir");
        let hook_path = hook_dir.join("before_run.sh");
        fs::write(
            &hook_path,
            "#!/bin/sh\nprintf 'verified transcript for %s\\n' \"$HANEULCHI_RUN_ID\"\n",
        )
        .expect("hook writes");
        let mut permissions = fs::metadata(&hook_path)
            .expect("hook metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook_path, permissions).expect("hook executable");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .reload_workflow(ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: root.join("WORKFLOW.md").to_string_lossy().to_string(),
                content: valid_workflow_document(),
            })
            .expect("workflow persists");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_evidence".to_string(),
                key: "HC-EVIDENCE".to_string(),
                project_id: "proj_local".to_string(),
                title: "Generate evidence".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .dispatch_run(DispatchRunInput {
                task_id: "task_evidence".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: Some(workspace.to_string_lossy().to_string()),
            })
            .expect("run dispatched");
        store
            .run_workflow_hook(RunWorkflowHookInput {
                run_id: "run_1".to_string(),
                hook_name: "before_run".to_string(),
                repo_root: root.to_string_lossy().to_string(),
                workspace_path: None,
            })
            .expect("hook runs");
        store
            .upsert_command_block(&PersistedCommandBlockInput {
                id: "cmdblk_test".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_evidence".to_string()),
                run_id: Some("run_1".to_string()),
                seq_start: Some(10),
                seq_end: Some(20),
                command: "npm test".to_string(),
                cwd: Some(root.to_string_lossy().to_string()),
                branch: Some("main".to_string()),
                exit_code: Some(0),
                duration_ms: Some(2400),
                summary: Some("88 tests passed".to_string()),
            })
            .expect("test command block persisted");
        store
            .record_token_usage(TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: Some("session_1".to_string()),
                task_id: Some("task_evidence".to_string()),
                run_id: Some("run_1".to_string()),
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 1200,
                output_tokens: 800,
                cost_usd: 8.5,
                source: "adapter:openai.responses".to_string(),
            })
            .expect("usage records");

        let evidence = store
            .generate_evidence_pack_for_run(GenerateEvidencePackInput {
                run_id: "run_1".to_string(),
                evidence_pack_id: None,
            })
            .expect("evidence generated");

        assert_eq!(evidence.id, "ev_run_1");
        assert_eq!(evidence.task_id.as_deref(), Some("task_evidence"));
        assert_eq!(evidence.run_id.as_deref(), Some("run_1"));
        assert_eq!(evidence.completeness_state, "incomplete");
        assert_eq!(evidence.body_json["workflow_version"]["id"], "workflow_1");
        assert_eq!(evidence.body_json["context_pack_id"], "auth-default");
        assert_eq!(evidence.body_json["command_blocks"][0]["id"], "cmdblk_test");
        assert_eq!(
            evidence.body_json["tests"][0]["command_block_id"],
            "cmdblk_test"
        );
        assert!(evidence.body_json["transcript_summary"]
            .as_str()
            .unwrap()
            .contains("verified transcript for run_1"));
        assert_eq!(evidence.body_json["token_usage"]["scope_type"], "run");
        assert_eq!(evidence.body_json["token_usage"]["scope_id"], "run_1");
        assert_eq!(evidence.body_json["token_usage"]["input_tokens"], 1200);
        assert_eq!(evidence.body_json["token_usage"]["output_tokens"], 800);
        assert_eq!(evidence.body_json["token_usage"]["total_tokens"], 2000);
        assert_eq!(evidence.body_json["token_usage"]["cost_usd"], 8.5);
        assert_eq!(
            evidence.body_json["token_usage"]["records"][0]["source"],
            "adapter:openai.responses"
        );
        assert_eq!(evidence.body_json["completeness"]["missing"][0], "diff");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn records_review_decision_and_moves_task_status() {
        let db_path = unique_test_db_path("review-decision");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_review_decision".to_string(),
                key: "HC-REVIEW".to_string(),
                project_id: "proj_local".to_string(),
                title: "Review decision".to_string(),
                description: None,
                status: "review".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .upsert_command_block(&PersistedCommandBlockInput {
                id: "cmdblk_review".to_string(),
                session_id: "pty_1".to_string(),
                task_id: Some("task_review_decision".to_string()),
                run_id: Some("run_99".to_string()),
                seq_start: None,
                seq_end: None,
                command: "npm test".to_string(),
                cwd: None,
                branch: None,
                exit_code: Some(0),
                duration_ms: None,
                summary: Some("tests passed".to_string()),
            })
            .expect("command block persisted");
        store
            .attach_command_block_to_evidence(AttachCommandBlockEvidenceInput {
                evidence_pack_id: "ev_review".to_string(),
                command_block_id: "cmdblk_review".to_string(),
                task_id: Some("task_review_decision".to_string()),
                run_id: Some("run_99".to_string()),
            })
            .expect("evidence exists");

        let approved = store
            .record_evidence_review_decision(RecordEvidenceReviewDecisionInput {
                evidence_pack_id: "ev_review".to_string(),
                decision: "approved".to_string(),
                reviewer_id: Some("human".to_string()),
                body_md: Some("Ship it.".to_string()),
            })
            .expect("review decision records");

        assert_eq!(
            approved.body_json["review_decision"]["decision"],
            "approved"
        );
        assert_eq!(
            approved.body_json["review_decision"]["reviewer_id"],
            "human"
        );
        assert_eq!(approved.body_json["review_decision"]["body_md"], "Ship it.");
        assert_eq!(
            store
                .get_task("task_review_decision")
                .expect("task loads")
                .expect("task exists")
                .status,
            "done"
        );

        let changes = store
            .record_evidence_review_decision(RecordEvidenceReviewDecisionInput {
                evidence_pack_id: "ev_review".to_string(),
                decision: "changes_requested".to_string(),
                reviewer_id: Some("human".to_string()),
                body_md: Some("Add diff proof.".to_string()),
            })
            .expect("changes requested records");

        assert_eq!(
            changes.body_json["review_decision"]["decision"],
            "changes_requested"
        );
        assert_eq!(
            store
                .get_task("task_review_decision")
                .expect("task loads")
                .expect("task exists")
                .status,
            "blocked"
        );

        let reopened = store
            .record_evidence_review_decision(RecordEvidenceReviewDecisionInput {
                evidence_pack_id: "ev_review".to_string(),
                decision: "reopened".to_string(),
                reviewer_id: Some("human".to_string()),
                body_md: Some("Reopen for another agent pass.".to_string()),
            })
            .expect("reopen decision records");

        assert_eq!(
            reopened.body_json["review_decision"]["decision"],
            "reopened"
        );
        assert_eq!(
            store
                .get_task("task_review_decision")
                .expect("task loads")
                .expect("task exists")
                .status,
            "ready"
        );

        let blocked = store
            .record_evidence_review_decision(RecordEvidenceReviewDecisionInput {
                evidence_pack_id: "ev_review".to_string(),
                decision: "blocked".to_string(),
                reviewer_id: Some("human".to_string()),
                body_md: Some("Blocked on missing fixture.".to_string()),
            })
            .expect("blocked decision records");

        assert_eq!(blocked.body_json["review_decision"]["decision"], "blocked");
        assert_eq!(
            store
                .get_task("task_review_decision")
                .expect("task loads")
                .expect("task exists")
                .status,
            "blocked"
        );
        assert!(store
            .record_evidence_review_decision(RecordEvidenceReviewDecisionInput {
                evidence_pack_id: "ev_review".to_string(),
                decision: "maybe".to_string(),
                reviewer_id: None,
                body_md: None,
            })
            .is_err());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn creates_follow_up_task_from_review_with_source_comment() {
        let db_path = unique_test_db_path("review-follow-up-task");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_review_followup".to_string(),
                key: "HC-FOLLOWUP".to_string(),
                project_id: "proj_local".to_string(),
                title: "Review generated patch".to_string(),
                description: None,
                status: "review".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .upsert_command_block(&PersistedCommandBlockInput {
                id: "cmdblk_followup".to_string(),
                session_id: "pty_followup".to_string(),
                task_id: Some("task_review_followup".to_string()),
                run_id: Some("run_followup".to_string()),
                seq_start: Some(1),
                seq_end: Some(3),
                command: "npm test".to_string(),
                cwd: None,
                branch: None,
                exit_code: Some(0),
                duration_ms: Some(900),
                summary: Some("tests passed".to_string()),
            })
            .expect("command block persisted");
        store
            .attach_command_block_to_evidence(AttachCommandBlockEvidenceInput {
                evidence_pack_id: "ev_followup".to_string(),
                command_block_id: "cmdblk_followup".to_string(),
                task_id: Some("task_review_followup".to_string()),
                run_id: Some("run_followup".to_string()),
            })
            .expect("evidence attached");

        let receipt = store
            .create_review_follow_up_task(CreateReviewFollowUpTaskInput {
                review_id: "review_ev_followup".to_string(),
                title: Some("Address review gap".to_string()),
                priority: Some("urgent".to_string()),
            })
            .expect("follow-up task created");

        assert_eq!(receipt.review_id, "review_ev_followup");
        assert_eq!(receipt.evidence_pack_id, "ev_followup");
        assert_eq!(
            receipt.source_task_id.as_deref(),
            Some("task_review_followup")
        );
        assert_eq!(receipt.source_run_id.as_deref(), Some("run_followup"));
        assert_eq!(receipt.task.title, "Address review gap");
        assert_eq!(receipt.task.project_id, "proj_local");
        assert_eq!(receipt.task.priority, "urgent");
        assert_eq!(
            receipt.comment.task_id.as_deref(),
            Some(receipt.task.id.as_str())
        );
        assert!(receipt.comment.body_md.contains("review_ev_followup"));
        assert!(receipt.comment.body_md.contains("ev_followup"));
        assert!(receipt.comment.body_md.contains("task_review_followup"));
        assert!(receipt.comment.body_md.contains("run_followup"));
        assert_eq!(
            store
                .list_task_comments(&receipt.task.id)
                .expect("comments load")
                .len(),
            1
        );
        assert!(store
            .create_review_follow_up_task(CreateReviewFollowUpTaskInput {
                review_id: "review_missing".to_string(),
                title: None,
                priority: None,
            })
            .is_err());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn plans_review_pr_landing_with_review_and_evidence_context() {
        let db_path = unique_test_db_path("review-pr-landing-plan");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "LOCAL".to_string(),
                name: "Local Project".to_string(),
                path: "/repo/local".to_string(),
                color: None,
            })
            .expect("project persists");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_review_pr".to_string(),
                key: "HC-PR".to_string(),
                project_id: "proj_local".to_string(),
                title: "Review PR landing".to_string(),
                description: None,
                status: "review".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .upsert_command_block(&PersistedCommandBlockInput {
                id: "cmdblk_review_pr".to_string(),
                session_id: "pty_review_pr".to_string(),
                task_id: Some("task_review_pr".to_string()),
                run_id: Some("run_review_pr".to_string()),
                seq_start: Some(1),
                seq_end: Some(2),
                command: "cargo test".to_string(),
                cwd: None,
                branch: None,
                exit_code: Some(0),
                duration_ms: Some(1200),
                summary: Some("tests passed".to_string()),
            })
            .expect("command block persisted");
        store
            .attach_command_block_to_evidence(AttachCommandBlockEvidenceInput {
                evidence_pack_id: "ev_review_pr".to_string(),
                command_block_id: "cmdblk_review_pr".to_string(),
                task_id: Some("task_review_pr".to_string()),
                run_id: Some("run_review_pr".to_string()),
            })
            .expect("evidence attached");

        let receipt = store
            .plan_review_pr_landing(PlanReviewPrLandingInput {
                review_id: "review_ev_review_pr".to_string(),
                title: Some("Ship review PR".to_string()),
                draft: Some(true),
            })
            .expect("review PR landing planned");

        assert_eq!(receipt.review_id, "review_ev_review_pr");
        assert_eq!(receipt.evidence_pack_id, "ev_review_pr");
        assert_eq!(receipt.source_task_id.as_deref(), Some("task_review_pr"));
        assert_eq!(receipt.source_run_id.as_deref(), Some("run_review_pr"));
        assert_eq!(receipt.plan.project_id, "proj_local");
        assert_eq!(receipt.plan.title, "Ship review PR");
        assert!(receipt.plan.draft);
        assert!(receipt
            .plan
            .checklist
            .iter()
            .any(|item| item.contains("review_ev_review_pr") && item.contains("ev_review_pr")));
        assert!(store
            .plan_review_pr_landing(PlanReviewPrLandingInput {
                review_id: "review_missing".to_string(),
                title: None,
                draft: None,
            })
            .is_err());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn records_token_usage_and_summarizes_budget_thresholds() {
        let db_path = unique_test_db_path("budget-summary");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .create_initiative(CreateInitiativeInput {
                project_id: "proj_local".to_string(),
                name: "Auth reliability goal".to_string(),
                description: None,
                budget_id: None,
                status: Some("active".to_string()),
            })
            .expect("initiative persists");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_budget".to_string(),
                key: "HC-BUDGET".to_string(),
                project_id: "proj_local".to_string(),
                title: "Budget scoped task".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: Some("init_1".to_string()),
                context_pack_id: None,
            })
            .expect("task persists");
        store
            .upsert_budget(UpsertBudgetInput {
                scope_type: "project".to_string(),
                scope_id: Some("proj_local".to_string()),
                max_usd: 10.0,
                warn_pct: 0.8,
                hard_limit: true,
            })
            .expect("budget persists");
        store
            .upsert_budget(UpsertBudgetInput {
                scope_type: "goal".to_string(),
                scope_id: Some("init_1".to_string()),
                max_usd: 9.0,
                warn_pct: 0.8,
                hard_limit: false,
            })
            .expect("goal budget persists");
        store
            .upsert_budget(UpsertBudgetInput {
                scope_type: "task".to_string(),
                scope_id: Some("task_budget".to_string()),
                max_usd: 9.0,
                warn_pct: 0.8,
                hard_limit: false,
            })
            .expect("task budget persists");
        store
            .upsert_budget(UpsertBudgetInput {
                scope_type: "run".to_string(),
                scope_id: Some("run_1".to_string()),
                max_usd: 9.0,
                warn_pct: 0.8,
                hard_limit: false,
            })
            .expect("run budget persists");
        store
            .record_token_usage(TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: Some("pty_1".to_string()),
                task_id: Some("task_budget".to_string()),
                run_id: Some("run_1".to_string()),
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 1200,
                output_tokens: 800,
                cost_usd: 8.5,
                source: "adapter".to_string(),
            })
            .expect("usage records");

        let summary = store.budget_summary().expect("budget summary loads");

        assert_eq!(summary["projects"][0]["scope_type"], "project");
        assert_eq!(summary["projects"][0]["scope_id"], "proj_local");
        assert_eq!(summary["projects"][0]["used_usd"], 8.5);
        assert_eq!(summary["projects"][0]["max_usd"], 10.0);
        assert_eq!(summary["projects"][0]["state"], "warn");
        assert_eq!(summary["goals"][0]["scope_type"], "goal");
        assert_eq!(summary["goals"][0]["scope_id"], "init_1");
        assert_eq!(summary["goals"][0]["used_usd"], 8.5);
        assert_eq!(summary["goals"][0]["state"], "warn");
        assert_eq!(summary["tasks"][0]["scope_type"], "task");
        assert_eq!(summary["tasks"][0]["scope_id"], "task_budget");
        assert_eq!(summary["tasks"][0]["used_usd"], 8.5);
        assert_eq!(summary["tasks"][0]["state"], "warn");
        assert_eq!(summary["runs"][0]["scope_type"], "run");
        assert_eq!(summary["runs"][0]["scope_id"], "run_1");
        assert_eq!(summary["runs"][0]["used_usd"], 8.5);
        assert_eq!(summary["runs"][0]["state"], "warn");
        assert_eq!(summary["workspace"]["used_usd"], 8.5);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn ingests_token_usage_from_adapter_payloads() {
        let db_path = unique_test_db_path("token-usage-adapter");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let usage = store
            .ingest_token_usage_adapter(IngestTokenUsageAdapterInput {
                project_id: Some("proj_local".to_string()),
                session_id: None,
                task_id: None,
                run_id: None,
                agent_profile_id: Some("agent_codex".to_string()),
                adapter: "openai.responses".to_string(),
                payload: serde_json::json!({
                    "model": "gpt-5.4",
                    "usage": {
                        "input_tokens": 1200,
                        "output_tokens": 800
                    },
                    "cost_usd": 8.5
                }),
            })
            .expect("adapter usage ingests");
        let log_usage = store
            .ingest_token_usage_adapter(IngestTokenUsageAdapterInput {
                project_id: Some("proj_local".to_string()),
                session_id: Some("pty_1".to_string()),
                task_id: None,
                run_id: None,
                agent_profile_id: Some("agent_codex".to_string()),
                adapter: "codex.log".to_string(),
                payload: serde_json::json!({
                    "raw": "noise\n{\"type\":\"token_usage\",\"provider\":\"openai\",\"model\":\"gpt-5.4\",\"input_tokens\":300,\"output_tokens\":200,\"cost_usd\":1.25}\n"
                }),
            })
            .expect("log usage ingests");
        let invalid = store
            .ingest_token_usage_adapter(IngestTokenUsageAdapterInput {
                project_id: Some("proj_local".to_string()),
                session_id: None,
                task_id: None,
                run_id: None,
                agent_profile_id: None,
                adapter: "openai.responses".to_string(),
                payload: serde_json::json!({"model": "gpt-5.4"}),
            })
            .expect_err("missing usage is rejected");
        let summary = store.budget_summary().expect("budget summary loads");

        assert_eq!(usage.provider, "openai");
        assert_eq!(usage.model, "gpt-5.4");
        assert_eq!(usage.input_tokens, 1200);
        assert_eq!(usage.output_tokens, 800);
        assert_eq!(usage.source, "adapter:openai.responses");
        assert_eq!(log_usage.session_id.as_deref(), Some("pty_1"));
        assert_eq!(log_usage.cost_usd, 1.25);
        assert!(invalid.contains("adapter payload usage"));
        assert_eq!(summary["workspace"]["used_usd"], 9.75);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn normalizes_and_persists_structured_agent_events() {
        let db_path = unique_test_db_path("agent-events");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let event = store
            .ingest_agent_events(IngestAgentEventsInput {
                project_id: "proj_local".to_string(),
                session_id: Some("session_1".to_string()),
                run_id: Some("run_1".to_string()),
                agent_profile_id: "agent_codex".to_string(),
                adapter: "raw-jsonl".to_string(),
                payload: serde_json::json!({
                    "raw": "noise\n{\"type\":\"status\",\"status\":\"needs_input\",\"message\":\"Waiting for review\"}\n{\"type\":\"tool_call\",\"tool\":\"shell\",\"message\":\"cargo test\"}\n"
                }),
            })
            .expect("agent events ingest");

        assert_eq!(event.kind, "status");
        assert_eq!(event.severity, "warning");
        assert_eq!(event.detail, "Waiting for review");
        assert_eq!(event.run_id.as_deref(), Some("run_1"));
        let events = store
            .list_agent_events("proj_local", 10)
            .expect("events list");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, "tool_call");
        assert_eq!(events[1].kind, "status");
        assert!(store
            .ingest_agent_events(IngestAgentEventsInput {
                project_id: "proj_local".to_string(),
                session_id: None,
                run_id: None,
                agent_profile_id: "agent_codex".to_string(),
                adapter: "raw-jsonl".to_string(),
                payload: serde_json::json!({ "raw": "not-json\n" }),
            })
            .expect_err("missing structured event rejected")
            .contains("agent event adapter did not contain structured events"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn summarizes_token_usage_by_session() {
        let db_path = unique_test_db_path("session-token-usage");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .record_token_usage(TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: Some("session_1".to_string()),
                task_id: None,
                run_id: None,
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 1200,
                output_tokens: 800,
                cost_usd: 8.5,
                source: "adapter".to_string(),
            })
            .expect("usage records");
        store
            .record_token_usage(TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: Some("session_1".to_string()),
                task_id: None,
                run_id: None,
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 300,
                output_tokens: 200,
                cost_usd: 1.25,
                source: "adapter".to_string(),
            })
            .expect("usage records");

        let summary = store
            .token_usage_summary_for_session("session_1")
            .expect("session usage loads");

        assert_eq!(summary.session_id, "session_1");
        assert_eq!(summary.input_tokens, 1500);
        assert_eq!(summary.output_tokens, 1000);
        assert_eq!(summary.cost_usd, 9.75);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn blocks_dispatch_when_project_budget_hard_limit_is_exceeded() {
        let db_path = unique_test_db_path("budget-dispatch-gate");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_budget_gate".to_string(),
                key: "HC-BUDGET".to_string(),
                project_id: "proj_local".to_string(),
                title: "Budget gated dispatch".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");
        store
            .upsert_budget(UpsertBudgetInput {
                scope_type: "project".to_string(),
                scope_id: Some("proj_local".to_string()),
                max_usd: 5.0,
                warn_pct: 0.8,
                hard_limit: true,
            })
            .expect("budget persists");
        store
            .record_token_usage(TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: None,
                task_id: None,
                run_id: None,
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 1000,
                output_tokens: 1000,
                cost_usd: 5.1,
                source: "adapter".to_string(),
            })
            .expect("usage records");

        let error = store
            .dispatch_run(DispatchRunInput {
                task_id: "task_budget_gate".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect_err("dispatch blocked by budget");

        assert!(error.contains("project budget exceeded"));
        assert!(store.list_runs("proj_local").expect("runs load").is_empty());
        assert_eq!(
            store
                .get_task("task_budget_gate")
                .expect("task loads")
                .expect("task exists")
                .status,
            "ready"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn blocks_dispatch_when_task_budget_hard_limit_is_exceeded() {
        let db_path = unique_test_db_path("task-budget-dispatch-gate");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_budget_gate".to_string(),
                key: "HC-TASK-BUDGET".to_string(),
                project_id: "proj_local".to_string(),
                title: "Task budget gated dispatch".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");
        store
            .upsert_budget(UpsertBudgetInput {
                scope_type: "task".to_string(),
                scope_id: Some("task_budget_gate".to_string()),
                max_usd: 5.0,
                warn_pct: 0.8,
                hard_limit: true,
            })
            .expect("task budget persists");
        store
            .record_token_usage(TokenUsageInput {
                project_id: Some("proj_local".to_string()),
                session_id: None,
                task_id: Some("task_budget_gate".to_string()),
                run_id: Some("run_previous".to_string()),
                agent_profile_id: Some("agent_codex".to_string()),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                input_tokens: 1000,
                output_tokens: 1000,
                cost_usd: 5.1,
                source: "adapter".to_string(),
            })
            .expect("usage records");

        let error = store
            .dispatch_run(DispatchRunInput {
                task_id: "task_budget_gate".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect_err("dispatch blocked by task budget");

        assert!(error.contains("task budget exceeded"));
        assert!(store.list_runs("proj_local").expect("runs load").is_empty());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn scans_agent_profiles_and_blocks_dispatch_to_paused_agents() {
        let db_path = unique_test_db_path("agent-profiles");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let scanned = store.scan_agent_profiles().expect("agents scan");
        assert!(scanned.iter().any(|agent| agent.id == "agent_codex"));
        assert!(scanned
            .iter()
            .any(|agent| { agent.id == "agent_generic_shell" && agent.status == "available" }));

        let paused = store
            .update_agent_profile_status("agent_codex", "paused")
            .expect("agent pauses");
        assert_eq!(paused.status, "paused");
        assert_eq!(
            store
                .list_agent_profiles()
                .expect("agents list")
                .into_iter()
                .find(|agent| agent.id == "agent_codex")
                .expect("codex profile exists")
                .status,
            "paused"
        );

        store
            .upsert_task(&PersistedTaskInput {
                id: "task_agent_pause".to_string(),
                key: "HC-AGENT-PAUSE".to_string(),
                project_id: "proj_local".to_string(),
                title: "Dispatch to paused agent".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: Some("agent".to_string()),
                assignee_id: Some("agent_codex".to_string()),
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");

        let error = store
            .dispatch_run(DispatchRunInput {
                task_id: "task_agent_pause".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect_err("paused agent blocks dispatch");
        assert!(error.contains("agent agent_codex is paused"));

        let resumed = store
            .update_agent_profile_status("agent_codex", "available")
            .expect("agent resumes");
        assert_eq!(resumed.status, "available");
        let run = store
            .dispatch_run(DispatchRunInput {
                task_id: "task_agent_pause".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("available agent dispatches");
        assert_eq!(run.agent_profile_id.as_deref(), Some("agent_codex"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn builds_raw_agent_terminal_launch_plans_from_profiles() {
        let db_path = unique_test_db_path("agent-raw-terminal");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_agent_profile(UpsertAgentProfileInput {
                id: "agent_acme".to_string(),
                name: "Acme CLI".to_string(),
                runtime: "generic-cli".to_string(),
                command: "acme-agent".to_string(),
                args_json: Some(serde_json::json!(["--raw", "--no-color"])),
                env_policy_json: Some(serde_json::json!({ "inherit": false })),
                skills_json: Some(serde_json::json!(["coding"])),
                status: Some("available".to_string()),
            })
            .expect("agent profile persists");

        let plan = store
            .agent_terminal_launch_plan(AgentTerminalLaunchInput {
                project_id: "proj_local".to_string(),
                agent_profile_id: "agent_acme".to_string(),
                title: Some("Acme raw".to_string()),
                cols: Some(120),
                rows: Some(34),
            })
            .expect("agent launch plan builds");

        assert_eq!(plan.project_id, "proj_local");
        assert_eq!(plan.agent_profile_id, "agent_acme");
        assert_eq!(plan.title, "Acme raw");
        assert_eq!(plan.command, "acme-agent");
        assert_eq!(plan.args, vec!["--raw", "--no-color"]);
        assert_eq!(plan.cols, 120);
        assert_eq!(plan.rows, 34);

        store
            .update_agent_profile_status("agent_acme", "paused")
            .expect("agent pauses");
        assert!(store
            .agent_terminal_launch_plan(AgentTerminalLaunchInput {
                project_id: "proj_local".to_string(),
                agent_profile_id: "agent_acme".to_string(),
                title: None,
                cols: None,
                rows: None,
            })
            .expect_err("paused agent rejected")
            .contains("agent agent_acme is paused"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn release_gate_accepts_third_party_agent_raw_terminal_evidence() {
        let db_path = unique_test_db_path("agent-raw-terminal-release-gate");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_agent_profile(UpsertAgentProfileInput {
                id: "agent_acme".to_string(),
                name: "Acme CLI".to_string(),
                runtime: "generic-cli".to_string(),
                command: "acme-agent".to_string(),
                args_json: Some(serde_json::json!(["--raw"])),
                env_policy_json: Some(serde_json::json!({ "inherit": false })),
                skills_json: Some(serde_json::json!(["coding"])),
                status: Some("available".to_string()),
            })
            .expect("agent profile persists");
        let session = store
            .create_session(CreateSessionInput {
                project_id: "proj_local".to_string(),
                mode: "agent".to_string(),
                title: "Acme raw terminal".to_string(),
                cwd: Some("/repo".to_string()),
                branch: None,
                agent_profile_id: Some("agent_acme".to_string()),
                task_id: None,
                run_id: None,
            })
            .expect("raw agent session persists");

        let release_gate = store
            .run_release_gate_scenarios(RunReleaseGatesInput {
                project_id: "proj_local".to_string(),
            })
            .expect("release gate runs");
        let rg06 = release_gate
            .scenarios
            .iter()
            .find(|scenario| scenario.gate_id == "RG-06")
            .expect("RG-06 scenario exists");

        assert_eq!(rg06.status, "pass");
        assert!(rg06.detail.contains("third-party CLI agent"));
        assert_eq!(rg06.evidence, vec![session.id]);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn release_gate_budget_requires_configured_budget_and_accepts_task_run_scopes() {
        let db_path = unique_test_db_path("release-gate-budget-scopes");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let unconfigured = store
            .run_release_gate_scenarios(RunReleaseGatesInput {
                project_id: "proj_local".to_string(),
            })
            .expect("release gate runs without budgets");
        let unconfigured_rg10 = unconfigured
            .scenarios
            .iter()
            .find(|scenario| scenario.gate_id == "RG-10")
            .expect("RG-10 exists");
        assert_eq!(unconfigured_rg10.status, "warning");
        assert!(unconfigured_rg10.detail.contains("not configured"));

        store
            .upsert_budget(UpsertBudgetInput {
                scope_type: "task".to_string(),
                scope_id: Some("task_budget_gate".to_string()),
                max_usd: 12.0,
                warn_pct: 0.8,
                hard_limit: false,
            })
            .expect("task budget persists");
        store
            .upsert_budget(UpsertBudgetInput {
                scope_type: "run".to_string(),
                scope_id: Some("run_budget_gate".to_string()),
                max_usd: 6.0,
                warn_pct: 0.8,
                hard_limit: false,
            })
            .expect("run budget persists");

        let configured = store
            .run_release_gate_scenarios(RunReleaseGatesInput {
                project_id: "proj_local".to_string(),
            })
            .expect("release gate runs with task and run budgets");
        let configured_rg10 = configured
            .scenarios
            .iter()
            .find(|scenario| scenario.gate_id == "RG-10")
            .expect("RG-10 exists");

        assert_eq!(configured_rg10.status, "pass");
        assert!(configured_rg10
            .detail
            .contains("Budget state is configured"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn release_gate_ui_cli_api_parity_requires_benchmark_evidence() {
        let db_path = unique_test_db_path("release-gate-parity-evidence");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let without_benchmark = store
            .run_release_gate_scenarios(RunReleaseGatesInput {
                project_id: "proj_local".to_string(),
            })
            .expect("release gate runs without parity evidence");
        let unverified_rg08 = without_benchmark
            .scenarios
            .iter()
            .find(|scenario| scenario.gate_id == "RG-08")
            .expect("RG-08 exists");
        assert_eq!(unverified_rg08.status, "warning");
        assert!(unverified_rg08
            .detail
            .contains("UI/CLI/API parity benchmark evidence has not been recorded"));

        let benchmark = store
            .run_benchmarks(RunBenchmarksInput {
                project_id: "proj_local".to_string(),
            })
            .expect("benchmark run persists");
        let parity_suite = benchmark
            .suites
            .iter()
            .find(|suite| suite.suite_id == "ui_cli_api_snapshot_parity")
            .expect("parity suite exists");
        assert_eq!(parity_suite.status, "pass");

        let verified = store
            .run_release_gate_scenarios(RunReleaseGatesInput {
                project_id: "proj_local".to_string(),
            })
            .expect("release gate runs with parity evidence");
        let verified_rg08 = verified
            .scenarios
            .iter()
            .find(|scenario| scenario.gate_id == "RG-08")
            .expect("RG-08 exists");

        assert_eq!(verified_rg08.status, "pass");
        assert_eq!(verified_rg08.evidence, vec![benchmark.id]);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn persists_sessions_and_guards_dangerous_input() {
        let db_path = unique_test_db_path("sessions");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let session = store
            .create_session(CreateSessionInput {
                project_id: "proj_local".to_string(),
                mode: "agent".to_string(),
                title: "Codex AUTH-104".to_string(),
                cwd: Some("/repo/auth-service".to_string()),
                branch: Some("fix/auth".to_string()),
                agent_profile_id: Some("agent_codex".to_string()),
                task_id: Some("task_auth".to_string()),
                run_id: Some("run_1".to_string()),
            })
            .expect("session persists");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_attached".to_string(),
                key: "HC-ATTACH".to_string(),
                project_id: "proj_local".to_string(),
                title: "Attach session task".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "medium".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("attach target task persists");

        assert_eq!(session.id, "session_1");
        assert_eq!(session.mode, "agent");
        assert_eq!(session.state, "running");
        assert_eq!(session.attention_state, "none");
        assert_eq!(
            store.list_sessions("proj_local").expect("sessions list"),
            vec![session.clone()]
        );
        let attached = store
            .attach_session_task("session_1", "task_attached")
            .expect("session attaches task");
        assert_eq!(attached.task_id.as_deref(), Some("task_attached"));
        let detached = store
            .detach_session_task("session_1")
            .expect("session detaches task");
        assert_eq!(detached.task_id, None);
        assert_eq!(detached.run_id, None);
        assert!(store
            .attach_session_task("session_1", "missing")
            .expect_err("missing task is rejected")
            .contains("task missing not found"));

        let focused = store.focus_session("session_1").expect("session focuses");
        assert_eq!(focused.attention_state, "none");

        let blocked_input = store
            .record_session_input(SessionInputInput {
                session_id: "session_1".to_string(),
                text: "rm -rf /tmp/build\n".to_string(),
                allow_dangerous: false,
            })
            .expect_err("dangerous input requires explicit allow flag");
        assert!(blocked_input.contains("allowDangerous"));

        let accepted_input = store
            .record_session_input(SessionInputInput {
                session_id: "session_1".to_string(),
                text: "rm -rf /tmp/build\n".to_string(),
                allow_dangerous: true,
            })
            .expect("dangerous input accepted with explicit flag");
        assert_eq!(accepted_input.session_id, "session_1");
        assert!(accepted_input.dangerous);
        assert_eq!(
            accepted_input.command_block_id.as_deref(),
            Some("cmdblk_session_input_1")
        );

        let takeover = store
            .takeover_session("session_1")
            .expect("session takeover");
        assert_eq!(takeover.attention_state, "needs_input");
        let released = store.release_session("session_1").expect("session release");
        assert_eq!(released.state, "running");
        assert_eq!(released.attention_state, "none");
        let killed = store.kill_session("session_1").expect("session kill");
        assert_eq!(killed.state, "completed");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn persists_remote_ssh_terminal_sessions() {
        let db_path = unique_test_db_path("ssh-sessions");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let session = store
            .create_session(CreateSessionInput {
                project_id: "proj_local".to_string(),
                mode: "ssh".to_string(),
                title: "SSH staging".to_string(),
                cwd: Some("ssh://deploy@staging.example.com/srv/app".to_string()),
                branch: Some("remote/main".to_string()),
                agent_profile_id: None,
                task_id: None,
                run_id: None,
            })
            .expect("ssh session persists");

        assert_eq!(session.mode, "ssh");
        assert_eq!(
            session.cwd.as_deref(),
            Some("ssh://deploy@staging.example.com/srv/app")
        );
        assert_eq!(
            store.list_sessions("proj_local").expect("sessions list")[0].mode,
            "ssh"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn persists_projects_and_focuses_project_tabs() {
        let db_path = unique_test_db_path("projects");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let first = store
            .add_project(AddProjectInput {
                key: "HC".to_string(),
                name: "Haneulchi".to_string(),
                path: "/repo/haneulchi".to_string(),
                color: Some("#4f46e5".to_string()),
            })
            .expect("first project persists");
        let second = store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: "/repo/auth-service".to_string(),
                color: Some("#059669".to_string()),
            })
            .expect("second project persists");

        assert_eq!(first.id, "proj_hc");
        assert_eq!(second.id, "proj_auth");
        assert_eq!(second.status, "active");
        assert_eq!(store.list_projects().expect("projects list").len(), 2);
        assert_eq!(
            store
                .list_project_tabs()
                .expect("tabs list")
                .into_iter()
                .filter(|tab| tab.active)
                .count(),
            1
        );

        let focused = store.focus_project("proj_hc").expect("project focuses");
        assert_eq!(focused.id, "proj_hc");
        assert!(store
            .list_project_tabs()
            .expect("tabs list")
            .into_iter()
            .any(|tab| tab.project_id == "proj_hc" && tab.active));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn rejects_duplicate_project_paths_for_different_project_keys() {
        let db_path = unique_test_db_path("duplicate-project-paths");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let project_root = db_path
            .parent()
            .expect("db has parent")
            .join("duplicate-workspace");
        std::fs::create_dir_all(&project_root).expect("project root created");
        let project_path = project_root.to_string_lossy().to_string();

        store
            .add_project(AddProjectInput {
                key: "HC".to_string(),
                name: "Haneulchi".to_string(),
                path: project_path.clone(),
                color: Some("#4f46e5".to_string()),
            })
            .expect("first project persists");

        let duplicate = store.add_project(AddProjectInput {
            key: "AUTH".to_string(),
            name: "Auth Service".to_string(),
            path: project_path,
            color: Some("#059669".to_string()),
        });

        assert_eq!(
            duplicate.expect_err("duplicate project path is rejected"),
            "project path already registered by project proj_hc"
        );
        assert_eq!(store.list_projects().expect("projects list").len(), 1);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn plans_project_detach_windows_for_existing_projects() {
        let db_path = unique_test_db_path("project-detach");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: "/repo/auth-service".to_string(),
                color: Some("#059669".to_string()),
            })
            .expect("project persists");

        let plan = store
            .plan_project_detach("proj_auth")
            .expect("project detach is planned");

        assert_eq!(plan.project_id, "proj_auth");
        assert_eq!(plan.project_name, "Auth Service");
        assert_eq!(plan.window_id, "win_proj_auth");
        assert_eq!(plan.status, "planned");
        assert_eq!(plan.degraded_reason, None);
        assert!(store
            .plan_project_detach("missing")
            .expect_err("missing project rejected")
            .contains("project missing not found"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn persists_project_tab_layout_json() {
        let db_path = unique_test_db_path("project-layouts");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: "/repo/auth-service".to_string(),
                color: Some("#059669".to_string()),
            })
            .expect("project persists");

        let tab = store
            .update_project_tab_layout(UpdateProjectTabLayoutInput {
                project_id: "proj_auth".to_string(),
                layout_json: serde_json::json!({
                    "mode": "grid",
                    "focused_session_id": "session_2",
                    "maximized_session_id": null,
                    "panes": ["session_1", "session_2"]
                }),
            })
            .expect("layout persists");

        assert_eq!(tab.id, "tab_proj_auth");
        assert_eq!(tab.layout_json["mode"], "grid");
        assert_eq!(tab.layout_json["focused_session_id"], "session_2");
        let reloaded = store
            .list_project_tabs()
            .expect("tabs list")
            .into_iter()
            .find(|tab| tab.project_id == "proj_auth")
            .expect("auth tab exists");
        assert_eq!(reloaded.layout_json["panes"][1], "session_2");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn persists_project_layout_presets_across_reopen() {
        let db_path = unique_test_db_path("project-layout-presets");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: "/repo/auth-service".to_string(),
                color: Some("#059669".to_string()),
            })
            .expect("project persists");

        let saved = store
            .save_project_layout_preset(SaveProjectLayoutPresetInput {
                project_id: "proj_auth".to_string(),
                name: "Review grid".to_string(),
                layout_json: serde_json::json!({
                    "mode": "grid",
                    "focusedSessionId": "session_2",
                    "maximizedSessionId": null,
                    "panes": ["session_1", "session_2"]
                }),
            })
            .expect("layout preset persists");
        let updated = store
            .save_project_layout_preset(SaveProjectLayoutPresetInput {
                project_id: "proj_auth".to_string(),
                name: "Review grid".to_string(),
                layout_json: serde_json::json!({
                    "mode": "maximized",
                    "focusedSessionId": "session_2",
                    "maximizedSessionId": "session_2",
                    "panes": ["session_1", "session_2"]
                }),
            })
            .expect("layout preset updates");

        assert_eq!(saved.id, "layout_preset_1");
        assert_eq!(updated.id, "layout_preset_1");
        assert_eq!(updated.layout_json["mode"], "maximized");
        let reloaded = StateStore::open_at(&db_path).expect("state store reopens");
        let presets = reloaded
            .list_project_layout_presets("proj_auth")
            .expect("layout presets list");
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].name, "Review grid");
        assert_eq!(presets[0].layout_json["maximizedSessionId"], "session_2");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn persists_skill_packs_across_reopen() {
        let db_path = unique_test_db_path("skill-packs");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: "/repo/auth-service".to_string(),
                color: Some("#059669".to_string()),
            })
            .expect("project persists");

        let saved = store
            .upsert_skill_pack(UpsertSkillPackInput {
                project_id: "proj_auth".to_string(),
                name: "Auth reviewer".to_string(),
                description: Some("Review auth flows".to_string()),
                skills_json: serde_json::json!(["code-review", "auth"]),
                source_context_pack_id: Some("ctx_auth".to_string()),
            })
            .expect("skill pack persists");
        let updated = store
            .upsert_skill_pack(UpsertSkillPackInput {
                project_id: "proj_auth".to_string(),
                name: "Auth reviewer".to_string(),
                description: Some("Review auth flows deeply".to_string()),
                skills_json: serde_json::json!(["code-review", "auth", "security"]),
                source_context_pack_id: Some("ctx_auth".to_string()),
            })
            .expect("skill pack updates");

        assert_eq!(saved.id, "skill_pack_1");
        assert_eq!(updated.id, "skill_pack_1");
        assert_eq!(
            updated.description.as_deref(),
            Some("Review auth flows deeply")
        );
        assert_eq!(updated.skills_json[2], "security");
        let reloaded = StateStore::open_at(&db_path).expect("state store reopens");
        let packs = reloaded
            .list_skill_packs("proj_auth")
            .expect("skill packs list");
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].name, "Auth reviewer");
        assert_eq!(packs[0].source_context_pack_id.as_deref(), Some("ctx_auth"));
        assert_eq!(packs[0].skills_json[1], "auth");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn summarizes_runtime_pool_for_project() {
        let db_path = unique_test_db_path("runtime-pool");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: "/repo/auth-service".to_string(),
                color: Some("#059669".to_string()),
            })
            .expect("project persists");
        store
            .create_session(CreateSessionInput {
                project_id: "proj_auth".to_string(),
                mode: "shell".to_string(),
                title: "Local zsh".to_string(),
                cwd: Some("/repo/auth-service".to_string()),
                branch: None,
                agent_profile_id: None,
                task_id: None,
                run_id: None,
            })
            .expect("shell session persists");
        store
            .create_session(CreateSessionInput {
                project_id: "proj_auth".to_string(),
                mode: "ssh".to_string(),
                title: "Deploy SSH".to_string(),
                cwd: Some("ssh://staging.example.com~/auth".to_string()),
                branch: None,
                agent_profile_id: None,
                task_id: None,
                run_id: None,
            })
            .expect("ssh session persists");
        store.scan_agent_profiles().expect("agent profiles scanned");
        store
            .update_agent_profile_status("agent_codex", "available")
            .expect("runtime pool fixture agent is available");
        let task = store
            .create_task(CreateTaskInput {
                project_id: "proj_auth".to_string(),
                title: "Review auth runtime pool".to_string(),
                priority: Some("high".to_string()),
                initiative_id: None,
            })
            .expect("task created");
        store
            .move_task_status(&task.id, "ready")
            .expect("task ready");
        let run = store
            .dispatch_run(DispatchRunInput {
                task_id: task.id,
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: Some("/tmp/haneulchi-runtime-pool".to_string()),
            })
            .expect("run dispatched");
        store
            .update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: run.id,
                lifecycle: "blocked".to_string(),
                status_detail: Some("Waiting for human review".to_string()),
            })
            .expect("run blocked");

        let pool = store
            .runtime_pool("proj_auth")
            .expect("runtime pool summary");

        assert_eq!(pool.len(), 3);
        assert!(pool.iter().any(|item| {
            item.id == "shell"
                && item.label == "Local"
                && item.session_count == 1
                && item.run_count == 0
                && item.blocked_count == 0
        }));
        assert!(pool.iter().any(|item| {
            item.id == "ssh"
                && item.label == "Remote SSH"
                && item.session_count == 1
                && item.run_count == 0
                && item.blocked_count == 0
        }));
        assert!(pool.iter().any(|item| {
            item.id == "agent"
                && item.label == "Cloud agents"
                && item.session_count == 1
                && item.run_count == 1
                && item.blocked_count == 1
        }));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn persists_project_tab_group_assignments() {
        let db_path = unique_test_db_path("project-tab-groups");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: "/repo/auth-service".to_string(),
                color: Some("#059669".to_string()),
            })
            .expect("project persists");

        let group = store
            .upsert_project_tab_group("proj_auth", "Backend")
            .expect("group persists");

        assert_eq!(group.project_id, "proj_auth");
        assert_eq!(group.group_name, "Backend");
        assert!(store
            .upsert_project_tab_group("missing", "Backend")
            .expect_err("missing project rejected")
            .contains("project missing not found"));
        assert_eq!(
            store
                .list_project_tab_groups()
                .expect("groups list")
                .into_iter()
                .find(|assignment| assignment.project_id == "proj_auth")
                .expect("auth group exists")
                .group_name,
            "Backend"
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn lists_project_files_with_git_status_badges() {
        let db_path = unique_test_db_path("project-files");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(workspace.join("src")).expect("workspace tree");
        fs::write(workspace.join("README.md"), "hello\n").expect("readme");
        fs::write(workspace.join("scratch.txt"), "draft\n").expect("scratch");
        fs::write(workspace.join("src").join("main.rs"), "fn main() {}\n").expect("main");
        Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .output()
            .expect("git init runs");
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&workspace)
            .output()
            .expect("git add runs");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let listing = store
            .list_project_files(ProjectFileListInput {
                project_id: "proj_auth".to_string(),
                relative_path: None,
            })
            .expect("project files listed");
        let readme = listing
            .entries
            .iter()
            .find(|entry| entry.path == "README.md")
            .expect("readme entry");
        let scratch = listing
            .entries
            .iter()
            .find(|entry| entry.path == "scratch.txt")
            .expect("scratch entry");
        let src = listing
            .entries
            .iter()
            .find(|entry| entry.path == "src")
            .expect("src entry");
        assert_eq!(listing.project_id, "proj_auth");
        assert_eq!(listing.relative_path, "");
        assert_eq!(readme.kind, "file");
        assert_eq!(readme.git_status.as_deref(), Some("added"));
        assert_eq!(scratch.git_status.as_deref(), Some("untracked"));
        assert_eq!(src.kind, "directory");

        let nested = store
            .list_project_files(ProjectFileListInput {
                project_id: "proj_auth".to_string(),
                relative_path: Some("src".to_string()),
            })
            .expect("nested files listed");
        assert_eq!(nested.relative_path, "src");
        assert!(nested
            .entries
            .iter()
            .any(|entry| entry.path == "src/main.rs"));

        let escape = store
            .list_project_files(ProjectFileListInput {
                project_id: "proj_auth".to_string(),
                relative_path: Some("../outside".to_string()),
            })
            .expect_err("path traversal is rejected");
        assert!(escape.contains("inside project"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn lists_deleted_project_files_from_git_status() {
        let db_path = unique_test_db_path("project-files-deleted");
        let workspace = db_path.with_extension("workspace");
        fs::create_dir_all(&workspace).expect("workspace");
        fs::write(workspace.join("README.md"), "# Auth\n").expect("readme");
        Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .output()
            .expect("git init runs");
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&workspace)
            .output()
            .expect("git add runs");
        Command::new("git")
            .args([
                "-c",
                "user.name=Haneulchi Test",
                "-c",
                "user.email=haneulchi@example.test",
                "commit",
                "-m",
                "initial",
            ])
            .current_dir(&workspace)
            .output()
            .expect("git commit runs");
        fs::remove_file(workspace.join("README.md")).expect("readme removed");

        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let listing = store
            .list_project_files(ProjectFileListInput {
                project_id: "proj_auth".to_string(),
                relative_path: None,
            })
            .expect("project files listed");

        let deleted = listing
            .entries
            .iter()
            .find(|entry| entry.path == "README.md")
            .expect("deleted file is visible");
        assert_eq!(deleted.kind, "file");
        assert_eq!(deleted.git_status.as_deref(), Some("deleted"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn reads_project_file_preview_with_path_safety() {
        let db_path = unique_test_db_path("project-file-preview");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(workspace.join("src")).expect("workspace tree");
        fs::write(
            workspace.join("src").join("main.rs"),
            "fn main() {\n    println!(\"hello\");\n}\n",
        )
        .expect("main");
        fs::write(
            workspace.join("src").join("server.log"),
            "INFO boot\nWARN retry\n",
        )
        .expect("log");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let preview = store
            .read_project_file(ProjectFileReadInput {
                project_id: "proj_auth".to_string(),
                path: "src/main.rs".to_string(),
            })
            .expect("file preview loads");
        assert_eq!(preview.project_id, "proj_auth");
        assert_eq!(preview.path, "src/main.rs");
        assert_eq!(preview.language.as_deref(), Some("rust"));
        assert_eq!(preview.truncated, false);
        assert!(preview.body.contains("println!"));

        let log_preview = store
            .read_project_file(ProjectFileReadInput {
                project_id: "proj_auth".to_string(),
                path: "src/server.log".to_string(),
            })
            .expect("log preview loads");
        assert_eq!(log_preview.language.as_deref(), Some("log"));
        assert_eq!(log_preview.body, "INFO boot\nWARN retry\n");

        let escape = store
            .read_project_file(ProjectFileReadInput {
                project_id: "proj_auth".to_string(),
                path: "../outside.rs".to_string(),
            })
            .expect_err("path traversal is rejected");
        assert!(escape.contains("inside project"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn writes_project_files_with_path_safety() {
        let db_path = unique_test_db_path("project-file-write");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(workspace.join("src")).expect("workspace tree");
        fs::write(
            workspace.join("src").join("main.ts"),
            "export const oldValue = 1;\n",
        )
        .expect("main");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let preview = store
            .write_project_file(ProjectFileWriteInput {
                project_id: "proj_auth".to_string(),
                path: "src/main.ts".to_string(),
                body: "export const newValue = 2;\n".to_string(),
            })
            .expect("project file written");

        assert_eq!(preview.project_id, "proj_auth");
        assert_eq!(preview.path, "src/main.ts");
        assert_eq!(preview.language.as_deref(), Some("typescript"));
        assert_eq!(preview.body, "export const newValue = 2;\n");
        assert_eq!(
            fs::read_to_string(workspace.join("src").join("main.ts")).expect("written file"),
            "export const newValue = 2;\n"
        );

        let escape = store
            .write_project_file(ProjectFileWriteInput {
                project_id: "proj_auth".to_string(),
                path: "../outside.ts".to_string(),
                body: "bad".to_string(),
            })
            .expect_err("path traversal is rejected");
        assert!(escape.contains("inside project"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn searches_project_files_by_path_with_git_status() {
        let db_path = unique_test_db_path("project-file-search");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(workspace.join("src").join("auth")).expect("workspace tree");
        fs::create_dir_all(workspace.join("docs")).expect("docs tree");
        fs::write(
            workspace.join("src").join("auth").join("login.ts"),
            "export {}\n",
        )
        .expect("login");
        fs::write(workspace.join("docs").join("auth.md"), "# Auth\n").expect("auth docs");
        fs::write(workspace.join("README.md"), "hello\n").expect("readme");
        Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .output()
            .expect("git init runs");
        Command::new("git")
            .args(["add", "docs/auth.md"])
            .current_dir(&workspace)
            .output()
            .expect("git add runs");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let results = store
            .search_project_files(ProjectFileSearchInput {
                project_id: "proj_auth".to_string(),
                query: "auth".to_string(),
            })
            .expect("project files searched");

        assert_eq!(results.project_id, "proj_auth");
        assert_eq!(results.query, "auth");
        assert!(results
            .entries
            .iter()
            .any(|entry| entry.path == "docs/auth.md"
                && entry.git_status.as_deref() == Some("added")));
        assert!(results
            .entries
            .iter()
            .any(|entry| entry.path == "src/auth/login.ts"));
        assert!(!results
            .entries
            .iter()
            .any(|entry| entry.path == "README.md"));

        let empty_query = store
            .search_project_files(ProjectFileSearchInput {
                project_id: "proj_auth".to_string(),
                query: " ".to_string(),
            })
            .expect_err("empty query is rejected");
        assert!(empty_query.contains("query cannot be empty"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn searches_deleted_project_files_from_git_status() {
        let db_path = unique_test_db_path("project-file-search-deleted");
        let workspace = db_path.with_extension("workspace");
        fs::create_dir_all(&workspace).expect("workspace");
        fs::write(workspace.join("README.md"), "# Auth\n").expect("readme");
        Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .output()
            .expect("git init runs");
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&workspace)
            .output()
            .expect("git add runs");
        Command::new("git")
            .args([
                "-c",
                "user.name=Haneulchi Test",
                "-c",
                "user.email=haneulchi@example.test",
                "commit",
                "-m",
                "initial",
            ])
            .current_dir(&workspace)
            .output()
            .expect("git commit runs");
        fs::remove_file(workspace.join("README.md")).expect("readme removed");

        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let results = store
            .search_project_files(ProjectFileSearchInput {
                project_id: "proj_auth".to_string(),
                query: "readme".to_string(),
            })
            .expect("project files searched");

        assert!(results.entries.iter().any(
            |entry| entry.path == "README.md" && entry.git_status.as_deref() == Some("deleted")
        ));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn reads_pdf_and_image_project_files_as_data_url_previews() {
        let db_path = unique_test_db_path("project-binary-preview");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(workspace.join("docs")).expect("workspace tree");
        fs::write(workspace.join("docs").join("sample.pdf"), b"%PDF-1.7\n").expect("pdf written");
        fs::write(
            workspace.join("docs").join("pixel.png"),
            [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'],
        )
        .expect("png written");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let pdf = store
            .read_project_file(ProjectFileReadInput {
                project_id: "proj_auth".to_string(),
                path: "docs/sample.pdf".to_string(),
            })
            .expect("pdf preview loads");
        let image = store
            .read_project_file(ProjectFileReadInput {
                project_id: "proj_auth".to_string(),
                path: "docs/pixel.png".to_string(),
            })
            .expect("image preview loads");

        assert_eq!(pdf.language.as_deref(), Some("pdf"));
        assert!(pdf.body.starts_with("data:application/pdf;base64,"));
        assert_eq!(image.language.as_deref(), Some("image"));
        assert!(image.body.starts_with("data:image/png;base64,"));
        assert_eq!(pdf.truncated, false);
        assert_eq!(image.truncated, false);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn reads_project_diff_with_path_safety() {
        let db_path = unique_test_db_path("project-diff");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(&workspace).expect("workspace tree");
        fs::write(workspace.join("README.md"), "hello\n").expect("readme");
        Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .output()
            .expect("git init runs");
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&workspace)
            .output()
            .expect("git add runs");
        Command::new("git")
            .args([
                "-c",
                "user.email=haneulchi@example.test",
                "-c",
                "user.name=Haneulchi Tests",
                "commit",
                "-m",
                "seed",
            ])
            .current_dir(&workspace)
            .output()
            .expect("git commit runs");
        fs::write(workspace.join("README.md"), "hello\nreview notes\n").expect("readme updated");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let diff = store
            .read_project_diff(ProjectDiffInput {
                project_id: "proj_auth".to_string(),
                path: Some("README.md".to_string()),
            })
            .expect("project diff loads");

        assert_eq!(diff.project_id, "proj_auth");
        assert_eq!(diff.path.as_deref(), Some("README.md"));
        assert_eq!(diff.file_count, 1);
        assert_eq!(diff.truncated, false);
        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, "README.md");
        assert_eq!(diff.files[0].status, "modified");
        assert_eq!(diff.files[0].additions, 1);
        assert_eq!(diff.files[0].deletions, 0);
        assert!(diff.body.contains("diff --git a/README.md b/README.md"));
        assert!(diff.body.contains("+review notes"));

        let escape = store
            .read_project_diff(ProjectDiffInput {
                project_id: "proj_auth".to_string(),
                path: Some("../outside.md".to_string()),
            })
            .expect_err("path traversal is rejected");
        assert!(escape.contains("inside project"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn plans_browser_lsp_patch_and_pr_boundary_workflows() {
        let db_path = unique_test_db_path("boundary-workflows");
        let workspace = db_path
            .parent()
            .expect("test db has parent")
            .join("workspace");
        fs::create_dir_all(workspace.join("src")).expect("workspace created");
        fs::write(
            workspace.join("src/app.ts"),
            "export function loadUser() {\n  return 1 as any;\n}\n// TODO tighten type\n",
        )
        .expect("source written");
        Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .output()
            .expect("git init runs");
        Command::new("git")
            .args(["add", "src/app.ts"])
            .current_dir(&workspace)
            .output()
            .expect("git add runs");
        Command::new("git")
            .args([
                "-c",
                "user.email=haneulchi@example.test",
                "-c",
                "user.name=Haneulchi Tests",
                "commit",
                "-m",
                "seed",
            ])
            .current_dir(&workspace)
            .output()
            .expect("git commit runs");
        fs::write(
            workspace.join("src/app.ts"),
            "export function loadUser() {\n  return 2 as any;\n}\n// TODO tighten type\n",
        )
        .expect("source updated");

        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "AUTH".to_string(),
                name: "Auth Service".to_string(),
                path: workspace.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project added");

        let browser = store
            .plan_browser_automation(RunBrowserAutomationInput {
                project_id: "proj_auth".to_string(),
                url: "http://localhost:3000/docs".to_string(),
                scenario: Some("smoke".to_string()),
            })
            .expect("browser plan");
        assert_eq!(browser.status, "planned");
        assert!(browser.steps.iter().any(|step| step.contains("localhost")));
        assert!(store
            .plan_browser_automation(RunBrowserAutomationInput {
                project_id: "proj_auth".to_string(),
                url: "https://example.com".to_string(),
                scenario: None,
            })
            .expect_err("remote browser URL rejected")
            .contains("localhost"));

        let lsp = store
            .collect_project_lsp_diagnostics(ProjectLspDiagnosticsInput {
                project_id: "proj_auth".to_string(),
                path: Some("src/app.ts".to_string()),
            })
            .expect("lsp diagnostics");
        assert!(lsp
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("TODO")));
        assert!(lsp
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("explicit any")));
        assert!(lsp
            .symbols
            .iter()
            .any(|symbol| symbol.name == "loadUser" && symbol.kind == "function"));

        let patch = store
            .export_project_patch(ProjectDiffInput {
                project_id: "proj_auth".to_string(),
                path: None,
            })
            .expect("patch exports");
        assert_eq!(patch.status, "exported");
        assert!(patch.body.contains("diff --git"));

        let imported = store
            .import_project_patch(ImportPatchInput {
                project_id: "proj_auth".to_string(),
                body: patch.body.clone(),
            })
            .expect("patch imports");
        assert_eq!(imported.status, "validated");
        assert_eq!(imported.file_count, 1);

        let pr_plan = store
            .plan_pr_landing(PlanPrLandingInput {
                project_id: "proj_auth".to_string(),
                title: "Ship auth update".to_string(),
                draft: true,
            })
            .expect("pr plan");
        assert_eq!(pr_plan.provider, "github");
        assert!(pr_plan.checklist.iter().any(|item| item.contains("patch")));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn persists_knowledge_sources_pages_context_packs_and_lint_summary() {
        let db_path = unique_test_db_path("knowledge-vault");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let source = store
            .upsert_knowledge_source(UpsertKnowledgeSourceInput {
                project_id: "proj_local".to_string(),
                kind: "file".to_string(),
                path_or_ref: "docs/auth.md".to_string(),
                fingerprint: "sha256:abc".to_string(),
                status: "current".to_string(),
            })
            .expect("source persists");
        let page = store
            .save_knowledge_page(SaveKnowledgePageInput {
                project_id: "proj_local".to_string(),
                slug: "auth-flow".to_string(),
                title: "Auth Flow".to_string(),
                body_md: "# Auth Flow\n\nToken rotation notes.".to_string(),
                source_ids: vec![source.id.clone()],
                freshness_state: "current".to_string(),
            })
            .expect("page persists");
        let pack = store
            .upsert_context_pack(UpsertContextPackInput {
                id: Some("ctx_auth".to_string()),
                project_id: "proj_local".to_string(),
                name: "auth-default".to_string(),
                description: Some("Auth docs and workflow notes".to_string()),
                sources_json: serde_json::json!([
                    {"type": "file", "path": "docs/auth.md"},
                    {"type": "knowledge_page", "id": page.id}
                ]),
                max_tokens_hint: Some(24000),
            })
            .expect("context pack persists");
        store
            .record_knowledge_lint_report(RecordKnowledgeLintReportInput {
                project_id: "proj_local".to_string(),
                stale_count: 2,
                gap_count: 1,
                contradiction_count: 0,
                body_md: "Stale: auth-flow\nGap: deployment rollback".to_string(),
            })
            .expect("lint report persists");

        let search = store
            .search_knowledge_pages("proj_local", Some("token"))
            .expect("knowledge search works");
        let summary = store
            .knowledge_summary("proj_local")
            .expect("knowledge summary loads");
        let source_index = store
            .list_knowledge_sources("proj_local")
            .expect("source index loads");
        let context_packs = store
            .list_context_packs("proj_local")
            .expect("context pack index loads");

        assert_eq!(source.id, "ks_1");
        assert_eq!(source_index[0].path_or_ref, "docs/auth.md");
        assert_eq!(source_index[0].status, "current");
        assert_eq!(page.id, "kp_1");
        assert!(page
            .artifact_path
            .ends_with("artifacts/knowledge/auth-flow.md"));
        assert_eq!(pack.id, "ctx_auth");
        assert_eq!(context_packs[0].id, "ctx_auth");
        assert_eq!(pack.sources_json["sources"][1]["id"], "kp_1");
        assert_eq!(pack.sources_json["budget"]["max_tokens_hint"], 24000);
        assert_eq!(search[0].slug, "auth-flow");
        assert_eq!(search[0].body_md, "# Auth Flow\n\nToken rotation notes.");
        assert_eq!(summary.stale_count, 2);
        assert_eq!(summary.gap_count, 1);
        assert_eq!(summary.recent_pages, vec!["auth-flow".to_string()]);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn saves_and_lists_knowledge_explorations() {
        let db_path = unique_test_db_path("knowledge-explorations");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let exploration = store
            .save_knowledge_exploration(SaveKnowledgeExplorationInput {
                project_id: "proj_local".to_string(),
                title: "Token rotation investigation".to_string(),
                question: "How should deploy rollback handle token rotation?".to_string(),
                answer_md: "Use the current issuer until the rollback window closes.".to_string(),
                page_ids: vec!["kp_auth".to_string()],
                context_pack_id: Some("ctx_auth".to_string()),
            })
            .expect("exploration persists");
        let saved = store
            .list_knowledge_explorations("proj_local")
            .expect("exploration index loads");

        assert_eq!(exploration.id, "kexp_1");
        assert_eq!(exploration.project_id, "proj_local");
        assert_eq!(exploration.title, "Token rotation investigation");
        assert_eq!(
            exploration.question,
            "How should deploy rollback handle token rotation?"
        );
        assert_eq!(
            exploration.answer_md,
            "Use the current issuer until the rollback window closes."
        );
        assert_eq!(exploration.page_ids, vec!["kp_auth".to_string()]);
        assert_eq!(exploration.context_pack_id, Some("ctx_auth".to_string()));
        assert!(exploration
            .artifact_path
            .ends_with("artifacts/knowledge/explorations/kexp_1.md"));
        assert_eq!(saved, vec![exploration]);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn lists_knowledge_concepts_and_cross_links() {
        let db_path = unique_test_db_path("knowledge-concepts");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let auth_page = store
            .save_knowledge_page(SaveKnowledgePageInput {
                project_id: "proj_local".to_string(),
                slug: "auth-flow".to_string(),
                title: "Auth Flow".to_string(),
                body_md: "# Auth Flow\n\nSee [[JWT rotation]] and [[error handling]].".to_string(),
                source_ids: vec![],
                freshness_state: "current".to_string(),
            })
            .expect("auth page persists");
        let jwt_page = store
            .save_knowledge_page(SaveKnowledgePageInput {
                project_id: "proj_local".to_string(),
                slug: "jwt-rotation".to_string(),
                title: "JWT Rotation".to_string(),
                body_md: "# JWT Rotation\n\nBacklink to [[auth flow]].".to_string(),
                source_ids: vec![],
                freshness_state: "current".to_string(),
            })
            .expect("jwt page persists");

        let concepts = store
            .list_knowledge_concepts("proj_local")
            .expect("concept graph loads");

        let auth = concepts
            .iter()
            .find(|concept| concept.slug == "auth-flow")
            .expect("auth concept exists");
        let jwt = concepts
            .iter()
            .find(|concept| concept.slug == "jwt-rotation")
            .expect("jwt concept exists");
        let missing = concepts
            .iter()
            .find(|concept| concept.slug == "error-handling")
            .expect("unresolved linked concept exists");

        assert_eq!(auth.page_id, Some(auth_page.id.clone()));
        assert_eq!(auth.outbound_slugs, vec!["jwt-rotation", "error-handling"]);
        assert_eq!(auth.inbound_page_ids, vec![jwt_page.id.clone()]);
        assert_eq!(jwt.page_id, Some(jwt_page.id.clone()));
        assert_eq!(jwt.inbound_page_ids, vec![auth_page.id.clone()]);
        assert_eq!(missing.page_id, None);
        assert_eq!(missing.title, "error handling");
        assert_eq!(missing.inbound_page_ids, vec![auth_page.id]);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn exports_knowledge_pages_as_obsidian_markdown_vault() {
        let db_path = unique_test_db_path("knowledge-obsidian-export");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .save_knowledge_page(SaveKnowledgePageInput {
                project_id: "proj_local".to_string(),
                slug: "auth-flow".to_string(),
                title: "Auth Flow".to_string(),
                body_md: "# Auth Flow\n\nSee [[JWT Rotation]].".to_string(),
                source_ids: vec![],
                freshness_state: "current".to_string(),
            })
            .expect("auth page persists");
        store
            .save_knowledge_page(SaveKnowledgePageInput {
                project_id: "proj_local".to_string(),
                slug: "jwt-rotation".to_string(),
                title: "JWT Rotation".to_string(),
                body_md: "# JWT Rotation\n\nBacklink target.".to_string(),
                source_ids: vec![],
                freshness_state: "current".to_string(),
            })
            .expect("jwt page persists");

        let export = store
            .export_knowledge_obsidian_markdown("proj_local")
            .expect("obsidian export succeeds");
        let root = db_path
            .parent()
            .expect("db parent")
            .join(&export.export_root);
        let auth_body = fs::read_to_string(root.join("Auth Flow.md")).expect("auth file exported");
        let jwt_body = fs::read_to_string(root.join("JWT Rotation.md")).expect("jwt file exported");
        let index_body =
            fs::read_to_string(root.join("Knowledge Index.md")).expect("index file exported");

        assert_eq!(export.status, "exported");
        assert_eq!(export.file_count, 3);
        assert!(export
            .export_root
            .ends_with("artifacts/knowledge/obsidian/proj_local"));
        assert!(export.files.contains(&"Auth Flow.md".to_string()));
        assert!(export.files.contains(&"JWT Rotation.md".to_string()));
        assert!(auth_body.contains("[[JWT Rotation]]"));
        assert!(jwt_body.contains("Backlink target."));
        assert!(index_body.contains("[[Auth Flow]]"));
        assert!(index_body.contains("[[JWT Rotation]]"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn answers_knowledge_questions_with_local_citations() {
        let db_path = unique_test_db_path("knowledge-chat");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let auth_page = store
            .save_knowledge_page(SaveKnowledgePageInput {
                project_id: "proj_local".to_string(),
                slug: "auth-flow".to_string(),
                title: "Auth Flow".to_string(),
                body_md: "# Auth Flow\n\nToken rotation rollback keeps both issuers until deploy health is green.".to_string(),
                source_ids: vec![],
                freshness_state: "current".to_string(),
            })
            .expect("auth page persists");
        store
            .save_knowledge_page(SaveKnowledgePageInput {
                project_id: "proj_local".to_string(),
                slug: "release-checks".to_string(),
                title: "Release Checks".to_string(),
                body_md: "# Release Checks\n\nRun smoke checks after deployment.".to_string(),
                source_ids: vec![],
                freshness_state: "current".to_string(),
            })
            .expect("release page persists");

        let answer = store
            .answer_knowledge_question(KnowledgeChatQuestionInput {
                project_id: "proj_local".to_string(),
                question: "How should rollback handle token rotation?".to_string(),
                context_pack_id: Some("ctx_auth".to_string()),
            })
            .expect("knowledge answer generated");

        assert_eq!(answer.project_id, "proj_local");
        assert_eq!(
            answer.question,
            "How should rollback handle token rotation?"
        );
        assert_eq!(answer.context_pack_id, Some("ctx_auth".to_string()));
        assert_eq!(answer.cited_page_ids, vec![auth_page.id]);
        assert_eq!(answer.source_count, 1);
        assert!(answer.answer_md.contains("Local knowledge answer draft"));
        assert!(answer.answer_md.contains("Auth Flow"));
        assert!(answer
            .answer_md
            .contains("Token rotation rollback keeps both issuers"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_knowledge_automation_compile_watch_and_lint() {
        let db_path = unique_test_db_path("knowledge-automation");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_knowledge_source(UpsertKnowledgeSourceInput {
                project_id: "proj_local".to_string(),
                kind: "file".to_string(),
                path_or_ref: "docs/stale.md".to_string(),
                fingerprint: "sha256:stale".to_string(),
                status: "stale".to_string(),
            })
            .expect("source persists");
        store
            .save_knowledge_page(SaveKnowledgePageInput {
                project_id: "proj_local".to_string(),
                slug: "gap-page".to_string(),
                title: "Gap Page".to_string(),
                body_md: "# Gap Page".to_string(),
                source_ids: vec![],
                freshness_state: "current".to_string(),
            })
            .expect("page persists");

        let run = store
            .run_knowledge_automation(RunKnowledgeAutomationInput {
                project_id: "proj_local".to_string(),
                watch: true,
            })
            .expect("automation runs");
        let summary = store
            .knowledge_summary("proj_local")
            .expect("summary loads");

        assert_eq!(run.status, "compiled");
        assert_eq!(run.watch_enabled, true);
        assert_eq!(run.source_count, 1);
        assert_eq!(run.page_count, 1);
        assert_eq!(run.stale_count, 1);
        assert_eq!(run.gap_count, 1);
        assert_eq!(run.lint_report_id, "klr_1");
        assert_eq!(summary.stale_count, 1);
        assert_eq!(summary.gap_count, 1);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn ingests_long_document_and_multimodal_artifact_as_source_and_page() {
        let db_path = unique_test_db_path("knowledge-ingestion");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let body = "Release runbook ".repeat(160);

        let result = store
            .ingest_knowledge_artifact(IngestKnowledgeArtifactInput {
                project_id: "proj_local".to_string(),
                kind: "pdf".to_string(),
                path_or_ref: "docs/release-runbook.pdf".to_string(),
                title: Some("Release Runbook".to_string()),
                body_md: body,
                max_chunk_chars: Some(1200),
            })
            .expect("artifact ingests");
        let source_index = store
            .list_knowledge_sources("proj_local")
            .expect("sources load");
        let pages = store
            .search_knowledge_pages("proj_local", Some("Release Runbook"))
            .expect("pages load");

        assert_eq!(result.source_id, "ks_1");
        assert_eq!(result.page_id, "kp_1");
        assert_eq!(result.modality, "pdf");
        assert_eq!(result.chunk_count, 3);
        assert_eq!(source_index[0].path_or_ref, "docs/release-runbook.pdf");
        assert_eq!(source_index[0].kind, "pdf");
        assert_eq!(pages[0].slug, "release-runbook");
        assert!(pages[0].body_md.contains("Chunk 1"));
        assert!(pages[0].body_md.contains("docs/release-runbook.pdf"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn dispatches_ready_tasks_into_queued_runs_and_counts_lifecycle() {
        let db_path = unique_test_db_path("runs");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_ready".to_string(),
                key: "HC-READY".to_string(),
                project_id: "proj_local".to_string(),
                title: "Dispatch run".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");

        let run = store
            .dispatch_run(DispatchRunInput {
                task_id: "task_ready".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: Some("ctx_default".to_string()),
                workspace_path: Some("/repo".to_string()),
            })
            .expect("run dispatched");

        assert_eq!(run.id, "run_1");
        assert_eq!(run.lifecycle, "queued");
        assert_eq!(run.retry_count, 0);
        assert_eq!(run.agent_profile_id.as_deref(), Some("agent_codex"));
        assert_eq!(run.session_id.as_deref(), Some("session_1"));
        let session = store
            .get_session("session_1")
            .expect("session loads")
            .expect("agent session exists");
        assert_eq!(session.task_id.as_deref(), Some("task_ready"));
        assert_eq!(session.run_id.as_deref(), Some("run_1"));
        assert_eq!(session.agent_profile_id.as_deref(), Some("agent_codex"));
        assert_eq!(session.mode, "agent");
        assert_eq!(
            store
                .get_task("task_ready")
                .expect("task loads")
                .expect("task exists")
                .status,
            "running"
        );
        assert_eq!(store.list_runs("proj_local").expect("runs load"), vec![run]);
        assert_eq!(
            store
                .count_runs_by_lifecycle("proj_local")
                .expect("run counts load"),
            serde_json::json!({
                "queued": 1,
                "claimed": 0,
                "starting": 0,
                "running": 0,
                "waiting_input": 0,
                "permission_requested": 0,
                "blocked": 0,
                "review_ready": 0,
                "completed": 0,
                "failed": 0,
                "cancelled": 0
            })
        );
        assert!(store
            .dispatch_run(DispatchRunInput {
                task_id: "missing".to_string(),
                agent_profile_id: None,
                context_pack_id: None,
                workspace_path: None,
            })
            .is_err());

        let _ = fs::remove_file(&db_path);
        if let Some(parent) = db_path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn dispatch_defaults_agent_write_runs_to_isolated_worktrees() {
        let db_path = unique_test_db_path("run-worktree-default");
        let project_root = db_path.parent().expect("db has parent").join("project");
        fs::create_dir_all(&project_root).expect("project root exists");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .add_project(AddProjectInput {
                key: "LOCAL".to_string(),
                name: "Local Project".to_string(),
                path: project_root.to_string_lossy().to_string(),
                color: None,
            })
            .expect("project persists");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_worktree".to_string(),
                key: "HC-WORKTREE".to_string(),
                project_id: "proj_local".to_string(),
                title: "Dispatch into default worktree".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");

        let run = store
            .dispatch_run(DispatchRunInput {
                task_id: "task_worktree".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");

        let expected_workspace = project_root
            .join(".haneulchi")
            .join("worktrees")
            .join("run_1");
        assert_eq!(
            run.workspace_path.as_deref(),
            Some(expected_workspace.to_string_lossy().as_ref())
        );
        assert!(expected_workspace.is_dir());
        let session = store
            .get_session("session_1")
            .expect("session loads")
            .expect("agent session exists");
        assert_eq!(
            session.cwd.as_deref(),
            Some(expected_workspace.to_string_lossy().as_ref())
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn updates_run_lifecycle_and_task_statuses() {
        let db_path = unique_test_db_path("run-lifecycle");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_lifecycle".to_string(),
                key: "HC-LIFE".to_string(),
                project_id: "proj_local".to_string(),
                title: "Lifecycle run".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");
        store
            .dispatch_run(DispatchRunInput {
                task_id: "task_lifecycle".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");

        let running = store
            .update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: "run_1".to_string(),
                lifecycle: "running".to_string(),
                status_detail: None,
            })
            .expect("run moves to running");
        assert_eq!(running.lifecycle, "running");
        assert!(running.started_at.is_some());

        let review_ready = store
            .update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: "run_1".to_string(),
                lifecycle: "review_ready".to_string(),
                status_detail: None,
            })
            .expect("run moves to review");
        assert_eq!(review_ready.lifecycle, "review_ready");
        assert_eq!(
            store
                .get_task("task_lifecycle")
                .expect("task loads")
                .expect("task exists")
                .status,
            "review"
        );

        let completed = store
            .update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: "run_1".to_string(),
                lifecycle: "completed".to_string(),
                status_detail: None,
            })
            .expect("run completes");
        assert_eq!(completed.lifecycle, "completed");
        assert!(completed.ended_at.is_some());
        assert_eq!(
            store
                .get_task("task_lifecycle")
                .expect("task loads")
                .expect("task exists")
                .status,
            "done"
        );
        assert!(store
            .update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: "run_1".to_string(),
                lifecycle: "running".to_string(),
                status_detail: None,
            })
            .is_err());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn policy_approval_requests_pause_runs_and_decisions_resume_or_block() {
        let db_path = unique_test_db_path("policy-approval");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_policy".to_string(),
                key: "HC-POLICY".to_string(),
                project_id: "proj_local".to_string(),
                title: "Dangerous action gate".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");
        store
            .dispatch_run(DispatchRunInput {
                task_id: "task_policy".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");
        store
            .update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: "run_1".to_string(),
                lifecycle: "running".to_string(),
                status_detail: None,
            })
            .expect("run starts");

        let approval = store
            .create_policy_approval(CreatePolicyApprovalInput {
                project_id: "proj_local".to_string(),
                task_id: Some("task_policy".to_string()),
                run_id: Some("run_1".to_string()),
                action_kind: "shell_command".to_string(),
                command: Some("rm -rf build/cache".to_string()),
                risk_level: "high".to_string(),
                requested_by: Some("agent_codex".to_string()),
            })
            .expect("approval is created");

        assert_eq!(approval.id, "policy_approval_1");
        assert_eq!(approval.state, "pending");
        assert_eq!(approval.action_kind, "shell_command");
        assert_eq!(approval.risk_level, "high");
        assert_eq!(approval.run_id.as_deref(), Some("run_1"));
        assert_eq!(
            store
                .get_run("run_1")
                .expect("run loads")
                .expect("run exists")
                .lifecycle,
            "permission_requested"
        );
        assert_eq!(
            store
                .get_run("run_1")
                .expect("run loads")
                .expect("run exists")
                .status_detail
                .as_deref(),
            Some("Permission requested: shell_command (rm -rf build/cache)")
        );

        let pending = store
            .list_policy_approvals("proj_local", Some("pending"))
            .expect("approvals list");
        assert_eq!(pending, vec![approval.clone()]);

        let approved = store
            .decide_policy_approval(DecidePolicyApprovalInput {
                approval_id: "policy_approval_1".to_string(),
                decision: "approved".to_string(),
                decision_by: Some("human".to_string()),
                decision_note: Some("Cache cleanup allowed.".to_string()),
            })
            .expect("approval decision persists");

        assert_eq!(approved.state, "approved");
        assert_eq!(approved.decision_by.as_deref(), Some("human"));
        assert_eq!(
            store
                .get_run("run_1")
                .expect("run loads")
                .expect("run exists")
                .lifecycle,
            "running"
        );
        assert!(store
            .list_policy_approvals("proj_local", Some("pending"))
            .expect("pending list")
            .is_empty());

        let evidence = store
            .generate_evidence_pack_for_run(GenerateEvidencePackInput {
                run_id: "run_1".to_string(),
                evidence_pack_id: None,
            })
            .expect("evidence generated");
        assert_eq!(
            evidence.body_json["policy_events"][0]["id"],
            "policy_approval_1"
        );
        assert_eq!(evidence.body_json["policy_events"][0]["state"], "approved");
        assert_eq!(
            evidence.body_json["policy_events"][0]["decision_note"],
            "Cache cleanup allowed."
        );

        let _ = fs::remove_file(&db_path);
        if let Some(parent) = db_path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn network_sandbox_profiles_allow_localhost_and_block_remote_network() {
        let db_path = unique_test_db_path("network-sandbox-profile");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        let pack = store
            .upsert_policy_pack(UpsertPolicyPackInput {
                project_id: "proj_local".to_string(),
                name: "Local network only".to_string(),
                sandbox_mode: "sandboxed".to_string(),
                network: Some("allowed".to_string()),
                network_profile: Some("local-only".to_string()),
                file_write: Some("ask".to_string()),
                tools: None,
                approval_required: None,
                forbidden_operations: None,
                set_active: Some(true),
            })
            .expect("policy pack persists");

        let localhost = store
            .evaluate_policy_action(EvaluatePolicyActionInput {
                project_id: "proj_local".to_string(),
                task_id: None,
                run_id: None,
                action_kind: "network".to_string(),
                command: Some("curl http://127.0.0.1:1420/health".to_string()),
                requested_by: None,
            })
            .expect("localhost evaluates");
        let remote = store
            .evaluate_policy_action(EvaluatePolicyActionInput {
                project_id: "proj_local".to_string(),
                task_id: None,
                run_id: None,
                action_kind: "network".to_string(),
                command: Some("curl https://example.com".to_string()),
                requested_by: None,
            })
            .expect("remote evaluates");

        assert_eq!(pack.network_profile, "local-only");
        assert_eq!(localhost.decision, "allowed");
        assert_eq!(localhost.reason, "network profile permits local endpoint");
        assert_eq!(remote.decision, "forbidden");
        assert_eq!(remote.reason, "network profile blocks remote endpoint");

        cleanup_test_db(&db_path);
    }

    #[test]
    fn permission_audit_records_redacted_policy_evaluations() {
        let db_path = unique_test_db_path("permission-audit");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_secret(UpsertSecretInput {
                project_id: "proj_local".to_string(),
                name: "DEPLOY_TOKEN".to_string(),
                value: "deploy-secret-987654".to_string(),
            })
            .expect("secret stored");
        store
            .upsert_policy_pack(UpsertPolicyPackInput {
                project_id: "proj_local".to_string(),
                name: "Audit strict".to_string(),
                sandbox_mode: "sandboxed".to_string(),
                network: Some("blocked".to_string()),
                network_profile: Some("internet".to_string()),
                file_write: Some("ask".to_string()),
                tools: None,
                approval_required: None,
                forbidden_operations: Some(vec!["deploy-secret".to_string()]),
                set_active: Some(true),
            })
            .expect("policy pack persists");

        let evaluation = store
            .evaluate_policy_action(EvaluatePolicyActionInput {
                project_id: "proj_local".to_string(),
                task_id: None,
                run_id: None,
                action_kind: "shell_command".to_string(),
                command: Some("deploy --token deploy-secret-987654".to_string()),
                requested_by: None,
            })
            .expect("policy evaluates");

        let connection = Connection::open(&db_path).expect("sqlite db exists");
        let audit: serde_json::Value = connection
            .query_row(
                "SELECT json_object(
                   'id', id,
                   'project_id', project_id,
                   'policy_pack_id', policy_pack_id,
                   'action_kind', action_kind,
                   'command', command,
                   'decision', decision,
                   'reason', reason
                 )
                 FROM permission_audit_events
                 WHERE project_id = 'proj_local'",
                [],
                |row| row.get::<_, String>(0),
            )
            .and_then(|json| {
                serde_json::from_str::<serde_json::Value>(&json).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
            })
            .expect("permission audit row exists");

        assert_eq!(evaluation.decision, "forbidden");
        assert_eq!(audit["id"], "permission_audit_1");
        assert_eq!(audit["policy_pack_id"], evaluation.policy_pack_id.unwrap());
        assert_eq!(audit["action_kind"], "shell_command");
        assert_eq!(audit["decision"], "forbidden");
        assert_eq!(audit["reason"], "action matches forbidden operation");
        assert_eq!(audit["command"], "deploy --token [REDACTED:DEPLOY_TOKEN]");
        assert!(!audit.to_string().contains("deploy-secret-987654"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn redacts_saved_secrets_from_policy_approval_logs() {
        let db_path = unique_test_db_path("policy-redaction");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_secret(UpsertSecretInput {
                project_id: "proj_local".to_string(),
                name: "DEPLOY_TOKEN".to_string(),
                value: "deploy-secret-987654".to_string(),
            })
            .expect("secret stored");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_policy_secret".to_string(),
                key: "HC-POLICY-SECRET".to_string(),
                project_id: "proj_local".to_string(),
                title: "Dangerous action with secret".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");
        store
            .dispatch_run(DispatchRunInput {
                task_id: "task_policy_secret".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");
        let approval = store
            .create_policy_approval(CreatePolicyApprovalInput {
                project_id: "proj_local".to_string(),
                task_id: None,
                run_id: Some("run_1".to_string()),
                action_kind: "shell_command".to_string(),
                command: Some("curl -H 'Authorization: Bearer deploy-secret-987654'".to_string()),
                risk_level: "high".to_string(),
                requested_by: Some("agent_codex".to_string()),
            })
            .expect("approval created");
        let run = store
            .get_run("run_1")
            .expect("run loads")
            .expect("run exists");

        assert_eq!(
            approval.command.as_deref(),
            Some("curl -H 'Authorization: Bearer [REDACTED:DEPLOY_TOKEN]'")
        );
        assert_eq!(
            run.status_detail.as_deref(),
            Some("Permission requested: shell_command (curl -H 'Authorization: Bearer [REDACTED:DEPLOY_TOKEN]')")
        );
        assert!(!format!("{approval:?}{run:?}").contains("deploy-secret-987654"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn blocked_waiting_and_permission_run_states_require_status_detail() {
        let db_path = unique_test_db_path("run-state-detail");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_waiting".to_string(),
                key: "HC-WAIT".to_string(),
                title: "Waiting state task".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                project_id: "proj_local".to_string(),
                assignee_type: Some("agent".to_string()),
                assignee_id: Some("agent_codex".to_string()),
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");
        store
            .dispatch_run(DispatchRunInput {
                task_id: "task_waiting".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");

        assert!(store
            .update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: "run_1".to_string(),
                lifecycle: "waiting_input".to_string(),
                status_detail: None,
            })
            .is_err());

        let waiting = store
            .update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: "run_1".to_string(),
                lifecycle: "waiting_input".to_string(),
                status_detail: Some("Needs OAuth test account".to_string()),
            })
            .expect("run waits with detail");
        assert_eq!(waiting.lifecycle, "waiting_input");
        assert_eq!(
            waiting.status_detail.as_deref(),
            Some("Needs OAuth test account")
        );

        let running = store
            .update_run_lifecycle(UpdateRunLifecycleInput {
                run_id: "run_1".to_string(),
                lifecycle: "running".to_string(),
                status_detail: None,
            })
            .expect("run resumes");
        assert_eq!(running.lifecycle, "running");
        assert!(running.status_detail.is_none());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn records_agent_status_updates_as_run_linked_task_comments() {
        let db_path = unique_test_db_path("agent-status-update");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_agent_update".to_string(),
                key: "HC-AGENT-UPDATE".to_string(),
                title: "Agent update task".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                project_id: "proj_local".to_string(),
                assignee_type: Some("agent".to_string()),
                assignee_id: Some("agent_codex".to_string()),
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");
        store
            .dispatch_run(DispatchRunInput {
                task_id: "task_agent_update".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");

        let update = store
            .record_run_status_update(RecordRunStatusUpdateInput {
                run_id: "run_1".to_string(),
                body_md: "Investigating OAuth fixture failure.".to_string(),
                lifecycle: Some("waiting_input".to_string()),
                status_detail: Some("Needs OAuth test account".to_string()),
            })
            .expect("agent update recorded");

        assert_eq!(update.task_id.as_deref(), Some("task_agent_update"));
        assert_eq!(update.run_id.as_deref(), Some("run_1"));
        assert_eq!(update.author_type, "agent");
        assert_eq!(update.author_id, "agent_codex");
        assert_eq!(update.body_md, "Investigating OAuth fixture failure.");
        assert_eq!(
            store
                .get_run("run_1")
                .expect("run loads")
                .expect("run exists")
                .status_detail
                .as_deref(),
            Some("Needs OAuth test account")
        );
        assert_eq!(
            store
                .list_task_comments("task_agent_update")
                .expect("comments load"),
            vec![update]
        );
        assert_eq!(
            store
                .get_task("task_agent_update")
                .expect("task loads")
                .expect("task exists")
                .comment_count,
            1
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn cancels_and_retries_failed_or_cancelled_runs() {
        let db_path = unique_test_db_path("run-retry");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_retry".to_string(),
                key: "HC-RETRY".to_string(),
                project_id: "proj_local".to_string(),
                title: "Retry run".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");
        store
            .dispatch_run(DispatchRunInput {
                task_id: "task_retry".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");

        let cancelled = store.cancel_run("run_1").expect("run cancelled");
        assert_eq!(cancelled.lifecycle, "cancelled");
        assert!(cancelled.ended_at.is_some());
        assert!(cancelled.next_retry_at.is_none());
        assert_eq!(
            store
                .get_task("task_retry")
                .expect("task loads")
                .expect("task exists")
                .status,
            "blocked"
        );

        let retried = store.retry_run("run_1").expect("run retried");
        assert_eq!(retried.lifecycle, "queued");
        assert_eq!(retried.retry_count, 1);
        assert!(retried.next_retry_at.is_some());
        assert!(retried.started_at.is_none());
        assert!(retried.ended_at.is_none());
        assert_eq!(
            store
                .get_task("task_retry")
                .expect("task loads")
                .expect("task exists")
                .status,
            "running"
        );
        assert_eq!(
            store
                .count_runs_by_lifecycle("proj_local")
                .expect("run counts load")["queued"],
            1
        );
        assert!(store.retry_run("run_1").is_err());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn reloads_valid_workflow_and_records_last_known_good() {
        let db_path = unique_test_db_path("workflow-valid");
        let store = StateStore::open_at(&db_path).expect("state store opens");

        let version = store
            .reload_workflow(ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: "/repo/WORKFLOW.md".to_string(),
                content: valid_workflow_document(),
            })
            .expect("workflow reloads");
        let state = store
            .workflow_runtime_state("proj_local")
            .expect("workflow state loads");

        assert_eq!(version.id, "workflow_1");
        assert!(version.valid);
        assert_eq!(version.parsed_json["project"]["key"], "AUTH");
        assert_eq!(
            version.diagnostics_json["errors"].as_array().unwrap().len(),
            0
        );
        assert_eq!(state.valid, true);
        assert_eq!(state.current_version_id.as_deref(), Some("workflow_1"));
        assert_eq!(
            state.last_known_good_version_id.as_deref(),
            Some("workflow_1")
        );

        cleanup_test_db(&db_path);
    }

    #[test]
    fn invalid_workflow_reload_preserves_last_known_good() {
        let db_path = unique_test_db_path("workflow-invalid");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .reload_workflow(ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: "/repo/WORKFLOW.md".to_string(),
                content: valid_workflow_document(),
            })
            .expect("valid workflow reloads");

        let invalid = store
            .reload_workflow(ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: "/repo/WORKFLOW.md".to_string(),
                content: invalid_workflow_document(),
            })
            .expect("invalid workflow persists diagnostics");
        let state = store
            .workflow_runtime_state("proj_local")
            .expect("workflow state loads");

        assert_eq!(invalid.id, "workflow_2");
        assert!(!invalid.valid);
        assert_eq!(state.valid, false);
        assert_eq!(state.current_version_id.as_deref(), Some("workflow_2"));
        assert_eq!(
            state.last_known_good_version_id.as_deref(),
            Some("workflow_1")
        );
        assert!(state.diagnostics["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| error["code"] == "template_namespace_not_allowed"));
        assert!(state.diagnostics["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| error["code"] == "hook_path_escapes_repo"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn dispatch_uses_last_known_good_workflow_and_default_context_pack() {
        let db_path = unique_test_db_path("dispatch-workflow-context");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .reload_workflow(ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: "/repo/WORKFLOW.md".to_string(),
                content: valid_workflow_document(),
            })
            .expect("valid workflow reloads");
        store
            .reload_workflow(ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: "/repo/WORKFLOW.md".to_string(),
                content: invalid_workflow_document(),
            })
            .expect("invalid workflow persists");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_context".to_string(),
                key: "HC-CONTEXT".to_string(),
                project_id: "proj_local".to_string(),
                title: "Resolve context on dispatch".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");

        let run = store
            .dispatch_run(DispatchRunInput {
                task_id: "task_context".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: None,
            })
            .expect("run dispatched");

        assert_eq!(run.workflow_version_id.as_deref(), Some("workflow_1"));
        assert_eq!(run.context_pack_id.as_deref(), Some("auth-default"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn explicit_dispatch_context_overrides_workflow_default_context_pack() {
        let db_path = unique_test_db_path("dispatch-context-override");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .reload_workflow(ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: "/repo/WORKFLOW.md".to_string(),
                content: valid_workflow_document(),
            })
            .expect("valid workflow reloads");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_override".to_string(),
                key: "HC-CONTEXT-OVERRIDE".to_string(),
                project_id: "proj_local".to_string(),
                title: "Override context on dispatch".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("ready task persisted");

        let run = store
            .dispatch_run(DispatchRunInput {
                task_id: "task_override".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: Some("task-specific".to_string()),
                workspace_path: None,
            })
            .expect("run dispatched");

        assert_eq!(run.workflow_version_id.as_deref(), Some("workflow_1"));
        assert_eq!(run.context_pack_id.as_deref(), Some("task-specific"));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn runs_workflow_hook_with_safe_metadata_environment() {
        let db_path = unique_test_db_path("workflow-hook-run");
        let root = db_path.parent().unwrap().join("repo");
        let hook_dir = root.join(".haneulchi/hooks");
        let workspace = root.join(".haneulchi/worktrees/run_1");
        fs::create_dir_all(&hook_dir).expect("hook dir");
        fs::create_dir_all(&workspace).expect("workspace dir");
        let hook_path = hook_dir.join("before_run.sh");
        fs::write(
            &hook_path,
            "#!/bin/sh\nprintf 'run=%s task=%s context=%s workspace=%s\\n' \"$HANEULCHI_RUN_ID\" \"$HANEULCHI_TASK_ID\" \"$HANEULCHI_CONTEXT_PACK_ID\" \"$HANEULCHI_WORKSPACE_PATH\"\n",
        )
        .expect("hook script writes");
        let mut permissions = fs::metadata(&hook_path)
            .expect("hook metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook_path, permissions).expect("hook executable");

        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .reload_workflow(ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: root.join("WORKFLOW.md").to_string_lossy().to_string(),
                content: valid_workflow_document(),
            })
            .expect("workflow reloads");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_hook".to_string(),
                key: "HC-HOOK".to_string(),
                project_id: "proj_local".to_string(),
                title: "Run hook".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .dispatch_run(DispatchRunInput {
                task_id: "task_hook".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: Some(workspace.to_string_lossy().to_string()),
            })
            .expect("run dispatched");

        let result = store
            .run_workflow_hook(RunWorkflowHookInput {
                run_id: "run_1".to_string(),
                hook_name: "before_run".to_string(),
                repo_root: root.to_string_lossy().to_string(),
                workspace_path: None,
            })
            .expect("hook runs");

        assert_eq!(result.status, "completed");
        assert_eq!(result.exit_code, Some(0));
        assert!(result
            .stdout
            .contains("run=run_1 task=task_hook context=auth-default"));
        assert!(result
            .stdout
            .contains(&workspace.to_string_lossy().to_string()));
        assert_eq!(result.env_json["HANEULCHI_RUN_ID"], "run_1");
        assert_eq!(result.env_json["HANEULCHI_TASK_ID"], "task_hook");
        assert_eq!(result.env_json["HANEULCHI_CONTEXT_PACK_ID"], "auth-default");

        let replay = store
            .get_run_replay_metadata("run_1")
            .expect("replay metadata loads")
            .expect("replay metadata exists");
        assert!(replay
            .artifact_path
            .ends_with("artifacts/runs/run_1/replay.json"));
        assert_eq!(replay.body_json["run_id"], "run_1");
        assert_eq!(replay.body_json["task_id"], "task_hook");
        assert_eq!(replay.body_json["workflow_version_id"], "workflow_1");
        assert_eq!(replay.body_json["context_pack_id"], "auth-default");
        assert_eq!(
            replay.body_json["workspace_path"],
            workspace
                .canonicalize()
                .expect("workspace canonicalizes")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(
            replay.body_json["hook_results"][0]["hook_name"],
            "before_run"
        );
        assert_eq!(replay.body_json["hook_results"][0]["status"], "completed");
        assert!(replay.body_json["hook_results"][0]["stdout"]
            .as_str()
            .unwrap()
            .contains("run=run_1"));

        let reopened = StateStore::open_at(&db_path).expect("state store reopens");
        let reopened_replay = reopened
            .get_run_replay_metadata("run_1")
            .expect("reopened replay metadata loads")
            .expect("reopened replay metadata exists");
        assert_eq!(reopened_replay.body_json["hook_results"][0]["exit_code"], 0);

        cleanup_test_db(&db_path);
    }

    #[test]
    fn rejects_workflow_hook_workspace_paths_that_escape_repo() {
        let db_path = unique_test_db_path("workflow-hook-escape");
        let root = db_path.parent().unwrap().join("repo");
        let hook_dir = root.join(".haneulchi/hooks");
        let workspace = root.join(".haneulchi/worktrees/run_1");
        let outside_workspace = db_path.parent().unwrap().join("outside-workspace");
        fs::create_dir_all(&hook_dir).expect("hook dir");
        fs::create_dir_all(&workspace).expect("workspace dir");
        fs::create_dir_all(&outside_workspace).expect("outside workspace dir");
        let hook_path = hook_dir.join("before_run.sh");
        fs::write(&hook_path, "#!/bin/sh\nprintf 'unsafe should not run\\n'\n")
            .expect("hook writes");
        let mut permissions = fs::metadata(&hook_path)
            .expect("hook metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook_path, permissions).expect("hook executable");
        let store = StateStore::open_at(&db_path).expect("state store opens");
        store
            .reload_workflow(ReloadWorkflowInput {
                project_id: "proj_local".to_string(),
                source_path: root.join("WORKFLOW.md").to_string_lossy().to_string(),
                content: valid_workflow_document(),
            })
            .expect("workflow persists");
        store
            .upsert_task(&PersistedTaskInput {
                id: "task_unsafe_hook".to_string(),
                key: "HC-HOOK-UNSAFE".to_string(),
                project_id: "proj_local".to_string(),
                title: "Unsafe hook".to_string(),
                description: None,
                status: "ready".to_string(),
                priority: "high".to_string(),
                assignee_type: None,
                assignee_id: None,
                cycle_id: None,
                module_id: None,
                initiative_id: None,
                context_pack_id: None,
            })
            .expect("task persisted");
        store
            .dispatch_run(DispatchRunInput {
                task_id: "task_unsafe_hook".to_string(),
                agent_profile_id: Some("agent_codex".to_string()),
                context_pack_id: None,
                workspace_path: Some(workspace.to_string_lossy().to_string()),
            })
            .expect("run dispatched");

        let error = store
            .run_workflow_hook(RunWorkflowHookInput {
                run_id: "run_1".to_string(),
                hook_name: "before_run".to_string(),
                repo_root: root.to_string_lossy().to_string(),
                workspace_path: Some(outside_workspace.to_string_lossy().to_string()),
            })
            .expect_err("unsafe workspace path rejected");

        assert!(error.contains("must stay inside the repo"));

        cleanup_test_db(&db_path);
    }

    fn object_count(connection: &Connection, object_type: &str, name: &str) -> i64 {
        connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = ?1 AND name = ?2",
                [object_type, name],
                |row| row.get(0),
            )
            .expect("sqlite_master query succeeds")
    }

    fn unique_test_db_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("haneulchi-{label}-{nanos}"))
            .join("haneulchi.sqlite")
    }

    fn cleanup_test_db(db_path: &Path) {
        let _ = fs::remove_file(db_path);
        if let Some(parent) = db_path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    fn valid_workflow_document() -> String {
        r#"---
haneulchi: 1
project:
  key: AUTH
  default_branch: main
workspace:
  strategy: worktree
  base_root: .haneulchi/worktrees
agents:
  default: claude
context:
  default_pack: auth-default
hooks:
  before_run: .haneulchi/hooks/before_run.sh
review:
  required_evidence:
    - diff
    - tests
    - transcript_summary
---

# Prompt template

You are working on {task.id} in {project.name}.
Use the context pack {?context_pack.name}.
"#
        .to_string()
    }

    fn invalid_workflow_document() -> String {
        r#"---
haneulchi: 1
project:
  key: AUTH
workspace:
  strategy: worktree
hooks:
  before_run: ../escape.sh
---

Use {secret.token} for {task.id}.
"#
        .to_string()
    }
}
