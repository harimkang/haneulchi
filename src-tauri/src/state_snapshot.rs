use crate::{pty::TerminalPtySnapshot, readiness::ReadinessSnapshot};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateSnapshot {
    pub snapshot_id: String,
    pub generated_at: String,
    pub app: StateSnapshotApp,
    pub projects: Vec<StateProject>,
    pub project_tabs: Vec<StateProjectTab>,
    pub sessions: Vec<StateSession>,
    pub command_blocks: StateCommandBlocks,
    pub tasks: StateCollection,
    pub initiatives: Vec<StateInitiative>,
    pub runs: StateRunCollection,
    pub agents: Vec<StateAgent>,
    pub reviews: Vec<StateReview>,
    pub attention: Vec<StateAttentionItem>,
    pub provider_model: StateProviderModel,
    pub budgets: StateBudgets,
    pub security: StateSecurity,
    pub workflow: StateWorkflow,
    pub workflow_negative: StateWorkflowNegative,
    pub knowledge: StateKnowledge,
    pub task_lifecycle: StateTaskLifecycle,
    pub terminal_fidelity: StateTerminalFidelity,
    pub release_gates: StateReleaseGates,
    pub distribution: StateDistribution,
    pub recovery: StateRecovery,
    pub benchmarks: StateBenchmarks,
    pub dogfood: StateDogfood,
    pub visual_harness: StateVisualHarness,
    pub tracker: StateTracker,
    pub health: StateHealth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateSnapshotApp {
    pub version: String,
    pub renderer: String,
    pub update_state: String,
    pub terminal_theme: StateTerminalTheme,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateTerminalTheme {
    pub project_id: Option<String>,
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub accent: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateProject {
    pub id: String,
    pub name: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<StateSessionTokenUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateProjectTab {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    pub label: String,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_json: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateSession {
    pub id: String,
    pub project_id: String,
    pub pane_id: String,
    pub mode: String,
    pub title: String,
    pub cwd: String,
    pub branch: String,
    pub agent_profile_id: Option<String>,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub state: String,
    pub attention_state: String,
    pub token_budget_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<StateSessionTokenUsage>,
    pub ports: Vec<u16>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateSessionTokenUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateCommandBlocks {
    pub recent: Vec<StateCommandBlockSummary>,
    pub unread_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateCommandBlockSummary {
    pub id: String,
    pub session_id: String,
    pub command: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateCollection {
    pub items: Vec<StateTaskSummary>,
    pub counts_by_status: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateTaskSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub project_id: String,
    pub assignee_type: Option<String>,
    pub assignee_id: Option<String>,
    pub cycle_id: Option<String>,
    pub module_id: Option<String>,
    pub initiative_id: Option<String>,
    pub due_at: Option<String>,
    pub estimate: Option<String>,
    pub labels: Vec<String>,
    pub context_pack_id: Option<String>,
    pub comment_count: i64,
    pub has_workpad: bool,
    pub subtask_count: i64,
    pub open_subtask_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<StateSessionTokenUsage>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateInitiative {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub budget_id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<StateSessionTokenUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateRunCollection {
    pub items: Vec<StateRunSummary>,
    pub counts_by_lifecycle: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateRunSummary {
    pub id: String,
    pub task_id: String,
    pub project_id: String,
    pub agent_profile_id: Option<String>,
    pub workflow_version_id: Option<String>,
    pub lifecycle: String,
    pub retry_count: i64,
    pub next_retry_at: Option<String>,
    pub status_detail: Option<String>,
    pub context_pack_id: Option<String>,
    pub workspace_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateAgent {
    pub id: String,
    pub label: String,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<StateSessionTokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_event_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_event_detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attention_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attention_severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_heartbeat_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateProviderModel {
    pub provider: String,
    pub model: String,
    pub agent_profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateReview {
    pub id: String,
    pub state: String,
    pub evidence_pack_id: String,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub completeness_state: String,
    pub diff_summary: serde_json::Value,
    pub token_usage: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateAttentionItem {
    pub id: String,
    pub label: String,
    pub severity: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateBudgets {
    pub workspace: serde_json::Value,
    pub projects: Vec<serde_json::Value>,
    pub goals: Vec<serde_json::Value>,
    pub tasks: Vec<serde_json::Value>,
    pub runs: Vec<serde_json::Value>,
    pub agents: Vec<serde_json::Value>,
    pub forecasts: serde_json::Value,
    pub price_table: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateSecurity {
    pub keychain: String,
    pub secret_count: usize,
    pub redaction: serde_json::Value,
    pub permission_audit: serde_json::Value,
    pub diagnostics: serde_json::Value,
    pub policy_pack: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateWorkflow {
    pub valid: bool,
    pub invalid_projects: Vec<String>,
    pub current_version_id: Option<String>,
    pub last_known_good_version_id: Option<String>,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateWorkflowNegative {
    pub last_run_id: Option<String>,
    pub last_status: String,
    pub last_baseline_workflow_id: Option<String>,
    pub last_invalid_workflow_id: Option<String>,
    pub last_known_good_workflow_id: Option<String>,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateKnowledge {
    pub stale_count: usize,
    pub gap_count: usize,
    pub recent_pages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateTaskLifecycle {
    pub last_run_id: Option<String>,
    pub last_status: String,
    pub last_task_id: Option<String>,
    pub last_agent_run_id: Option<String>,
    pub last_evidence_pack_id: Option<String>,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateTerminalFidelity {
    pub last_run_id: Option<String>,
    pub last_status: String,
    pub last_pass_count: i64,
    pub last_fail_count: i64,
    pub last_warning_count: i64,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateReleaseGates {
    pub last_run_id: Option<String>,
    pub last_status: String,
    pub last_pass_count: i64,
    pub last_fail_count: i64,
    pub last_warning_count: i64,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateDistribution {
    pub last_dmg_smoke_run_id: Option<String>,
    pub last_status: String,
    pub explicit_blocker: bool,
    pub last_pass_count: i64,
    pub last_fail_count: i64,
    pub last_warning_count: i64,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateRecovery {
    pub last_run_id: Option<String>,
    pub last_status: String,
    pub last_pass_count: i64,
    pub last_fail_count: i64,
    pub last_warning_count: i64,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateBenchmarks {
    pub last_run_id: Option<String>,
    pub last_status: String,
    pub last_pass_count: i64,
    pub last_fail_count: i64,
    pub last_warning_count: i64,
    pub suites: Vec<StateBenchmarkSuite>,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateBenchmarkSuite {
    pub suite_id: String,
    pub name: String,
    pub status: String,
    pub metric_value: i64,
    pub target_value: i64,
    pub unit: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateDogfood {
    pub last_review_id: Option<String>,
    pub last_status: String,
    pub last_evidence_pack_id: Option<String>,
    pub last_pass_count: i64,
    pub last_warning_count: i64,
    pub last_fail_count: i64,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateVisualHarness {
    pub nodes: Vec<StateVisualNode>,
    pub edges: Vec<StateVisualEdge>,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateVisualNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateVisualEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateTracker {
    pub binding_count: usize,
    pub bindings: Vec<StateTrackerBinding>,
    pub diagnostics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateTrackerBinding {
    pub id: String,
    pub local_kind: String,
    pub local_id: String,
    pub provider: String,
    pub external_id: String,
    pub external_url: Option<String>,
    pub sync_mode: String,
    pub sync_status: String,
    pub conflict_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StateHealth {
    pub db: String,
    pub pty: String,
    pub api: String,
}

pub fn current_timestamp_utc() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn build_state_snapshot_with_db_state(
    version: &str,
    generated_at: &str,
    readiness: ReadinessSnapshot,
    pty: TerminalPtySnapshot,
    db_health: &str,
    tasks: Vec<StateTaskSummary>,
    task_counts_by_status: serde_json::Value,
) -> StateSnapshot {
    let attention = attention_from_readiness(&readiness, db_health);
    let sessions = pty
        .sessions
        .into_iter()
        .map(|session| StateSession {
            id: session.id.clone(),
            project_id: "proj_local".to_string(),
            pane_id: session.id,
            mode: "shell".to_string(),
            title: session.title,
            cwd: String::new(),
            branch: String::new(),
            agent_profile_id: None,
            task_id: None,
            run_id: None,
            state: "running".to_string(),
            attention_state: "none".to_string(),
            token_budget_state: "unknown".to_string(),
            token_usage: None,
            ports: vec![],
            created_at: generated_at.to_string(),
            updated_at: generated_at.to_string(),
        })
        .collect();

    let mut snapshot = StateSnapshot {
        snapshot_id: String::new(),
        generated_at: generated_at.to_string(),
        app: StateSnapshotApp {
            version: version.to_string(),
            renderer: "xterm-webgl".to_string(),
            update_state: "current".to_string(),
            terminal_theme: StateTerminalTheme {
                project_id: None,
                name: "Haneulchi Default".to_string(),
                background: "#050607".to_string(),
                foreground: "#d7ffe1".to_string(),
                accent: "#42e355".to_string(),
            },
        },
        projects: vec![StateProject {
            id: "proj_local".to_string(),
            name: "haneulchi".to_string(),
            state: "active".to_string(),
            token_usage: None,
        }],
        project_tabs: vec![
            StateProjectTab {
                id: "terminal-deck".to_string(),
                project_id: None,
                label: "Terminal Deck".to_string(),
                active: true,
                layout_json: None,
                group_name: None,
            },
            StateProjectTab {
                id: "control-tower".to_string(),
                project_id: None,
                label: "Control Tower".to_string(),
                active: false,
                layout_json: None,
                group_name: None,
            },
        ],
        sessions,
        command_blocks: StateCommandBlocks {
            recent: vec![],
            unread_count: 0,
        },
        tasks: StateCollection {
            items: tasks,
            counts_by_status: task_counts_by_status,
        },
        initiatives: vec![],
        runs: StateRunCollection {
            items: vec![],
            counts_by_lifecycle: serde_json::json!({}),
        },
        agents: vec![],
        reviews: vec![],
        attention,
        provider_model: StateProviderModel {
            provider: "openai".to_string(),
            model: "gpt-5.4".to_string(),
            agent_profile_id: "agent_codex".to_string(),
        },
        budgets: StateBudgets {
            workspace: serde_json::json!({}),
            projects: vec![],
            goals: vec![],
            tasks: vec![],
            runs: vec![],
            agents: vec![],
            forecasts: serde_json::json!({}),
            price_table: serde_json::json!({}),
        },
        security: StateSecurity {
            keychain: "unknown".to_string(),
            secret_count: 0,
            redaction: serde_json::json!({
                "status": "inactive",
                "protected_secret_count": 0
            }),
            permission_audit: serde_json::json!({
                "recent_count": 0,
                "allowed_count": 0,
                "approval_required_count": 0,
                "forbidden_count": 0,
                "latest_decision": null,
                "latest_action_kind": null
            }),
            diagnostics: serde_json::json!({
                "status": "warning",
                "pending_policy_approvals": 0,
                "checks": []
            }),
            policy_pack: serde_json::json!({}),
        },
        workflow: StateWorkflow {
            valid: true,
            invalid_projects: vec![],
            current_version_id: None,
            last_known_good_version_id: None,
            diagnostics: serde_json::json!({ "errors": [] }),
        },
        workflow_negative: StateWorkflowNegative {
            last_run_id: None,
            last_status: "not_run".to_string(),
            last_baseline_workflow_id: None,
            last_invalid_workflow_id: None,
            last_known_good_workflow_id: None,
            diagnostics: serde_json::json!({
                "status": "not_run",
                "case_count": 0
            }),
        },
        knowledge: StateKnowledge {
            stale_count: 0,
            gap_count: 0,
            recent_pages: vec![],
        },
        task_lifecycle: StateTaskLifecycle {
            last_run_id: None,
            last_status: "not_run".to_string(),
            last_task_id: None,
            last_agent_run_id: None,
            last_evidence_pack_id: None,
            diagnostics: serde_json::json!({
                "status": "not_run",
                "transition_count": 0
            }),
        },
        terminal_fidelity: StateTerminalFidelity {
            last_run_id: None,
            last_status: "not_run".to_string(),
            last_pass_count: 0,
            last_fail_count: 0,
            last_warning_count: 0,
            diagnostics: serde_json::json!({
                "status": "not_run",
                "case_count": 0
            }),
        },
        release_gates: StateReleaseGates {
            last_run_id: None,
            last_status: "not_run".to_string(),
            last_pass_count: 0,
            last_fail_count: 0,
            last_warning_count: 0,
            diagnostics: serde_json::json!({
                "status": "not_run",
                "scenario_count": 0
            }),
        },
        distribution: StateDistribution {
            last_dmg_smoke_run_id: None,
            last_status: "not_run".to_string(),
            explicit_blocker: false,
            last_pass_count: 0,
            last_fail_count: 0,
            last_warning_count: 0,
            diagnostics: serde_json::json!({
                "status": "not_run",
                "case_count": 0
            }),
        },
        recovery: StateRecovery {
            last_run_id: None,
            last_status: "not_run".to_string(),
            last_pass_count: 0,
            last_fail_count: 0,
            last_warning_count: 0,
            diagnostics: serde_json::json!({
                "status": "not_run",
                "drill_count": 0
            }),
        },
        benchmarks: StateBenchmarks {
            last_run_id: None,
            last_status: "not_run".to_string(),
            last_pass_count: 0,
            last_fail_count: 0,
            last_warning_count: 0,
            suites: vec![],
            diagnostics: serde_json::json!({
                "status": "not_run",
                "suite_count": 0
            }),
        },
        dogfood: StateDogfood {
            last_review_id: None,
            last_status: "not_run".to_string(),
            last_evidence_pack_id: None,
            last_pass_count: 0,
            last_warning_count: 0,
            last_fail_count: 0,
            diagnostics: serde_json::json!({
                "status": "not_run",
                "finding_count": 0
            }),
        },
        visual_harness: StateVisualHarness {
            nodes: vec![],
            edges: vec![],
            diagnostics: serde_json::json!({
                "status": "empty",
                "node_count": 0,
                "edge_count": 0
            }),
        },
        tracker: StateTracker {
            binding_count: 0,
            bindings: vec![],
            diagnostics: serde_json::json!({
                "status": "unconfigured",
                "pending_count": 0,
                "conflict_count": 0
            }),
        },
        health: StateHealth {
            db: db_health.to_string(),
            pty: "ok".to_string(),
            api: "ok".to_string(),
        },
    };
    assign_snapshot_id(&mut snapshot);
    snapshot
}

fn attention_from_readiness(
    readiness: &ReadinessSnapshot,
    db_health: &str,
) -> Vec<StateAttentionItem> {
    let mut attention = Vec::new();

    if readiness.summary.missing > 0 || readiness.summary.warning > 0 {
        attention.push(StateAttentionItem {
            id: "readiness-warnings".to_string(),
            label: "Readiness checks need attention".to_string(),
            severity: "warning".to_string(),
            detail: format!(
                "{} warnings, {} missing",
                readiness.summary.warning, readiness.summary.missing
            ),
        });
    }

    if db_health != "ok" {
        attention.push(StateAttentionItem {
            id: "db-not-configured".to_string(),
            label: "Database persistence not configured".to_string(),
            severity: "warning".to_string(),
            detail: "Snapshot is live from Tauri state; durable SQLite state is still pending"
                .to_string(),
        });
    }

    attention
}

pub fn assign_snapshot_id(snapshot: &mut StateSnapshot) {
    let generated_at = snapshot.generated_at.clone();
    let mut canonical = serde_json::to_value(&snapshot).unwrap_or_else(|_| serde_json::json!({}));
    if let Some(object) = canonical.as_object_mut() {
        object.insert("snapshot_id".to_string(), serde_json::Value::Null);
        object.insert("generated_at".to_string(), serde_json::Value::Null);
        if let Some(sessions) = object
            .get_mut("sessions")
            .and_then(serde_json::Value::as_array_mut)
        {
            for session in sessions {
                if let Some(session) = session.as_object_mut() {
                    if session
                        .get("created_at")
                        .and_then(serde_json::Value::as_str)
                        == Some(generated_at.as_str())
                    {
                        session.insert("created_at".to_string(), serde_json::Value::Null);
                    }
                    if session
                        .get("updated_at")
                        .and_then(serde_json::Value::as_str)
                        == Some(generated_at.as_str())
                    {
                        session.insert("updated_at".to_string(), serde_json::Value::Null);
                    }
                }
            }
        }
    }
    let canonical = serde_json::to_string(&canonical).unwrap_or_default();
    snapshot.snapshot_id = format!("snap_{:016x}", stable_hash64(&canonical));
}

fn stable_hash64(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::{build_state_snapshot_with_db_state, StateTaskSummary};
    use crate::{
        pty::{TerminalPtySession, TerminalPtySnapshot},
        readiness::{summarize_checks, ReadinessCheck, ReadinessSnapshot, ReadinessStatus},
    };

    #[test]
    fn builds_a_spec_shaped_state_snapshot_from_readiness_and_pty_state() {
        let readiness = ReadinessSnapshot {
            checks: vec![ReadinessCheck::new(
                "shell",
                "Login shell",
                "zsh detected",
                ReadinessStatus::Ready,
            )],
            summary: summarize_checks(&[ReadinessCheck::new(
                "shell",
                "Login shell",
                "zsh detected",
                ReadinessStatus::Ready,
            )]),
        };
        let pty = TerminalPtySnapshot {
            total: 1,
            sessions: vec![TerminalPtySession {
                id: "pty_1".to_string(),
                title: "shell".to_string(),
                command: "zsh".to_string(),
                cols: 120,
                rows: 32,
            }],
        };

        let snapshot = build_state_snapshot_with_db_state(
            "0.1.0",
            "2026-04-30T01:00:00Z",
            readiness,
            pty,
            "degraded",
            vec![],
            serde_json::json!({}),
        );

        assert!(snapshot.snapshot_id.starts_with("snap_"));
        assert_eq!(snapshot.generated_at, "2026-04-30T01:00:00Z");
        assert_eq!(snapshot.app.version, "0.1.0");
        assert_eq!(snapshot.sessions.len(), 1);
        assert_eq!(snapshot.sessions[0].id, "pty_1");
        assert_eq!(snapshot.sessions[0].mode, "shell");
        assert_eq!(snapshot.health.pty, "ok");
        assert_eq!(snapshot.health.db, "degraded");
        assert_eq!(snapshot.command_blocks.unread_count, 0);
        assert_eq!(snapshot.workflow.valid, true);
    }

    #[test]
    fn snapshot_id_is_stable_for_identical_state_and_changes_with_state() {
        let readiness = ReadinessSnapshot {
            checks: vec![],
            summary: summarize_checks(&[]),
        };
        let pty = TerminalPtySnapshot {
            total: 1,
            sessions: vec![TerminalPtySession {
                id: "pty_1".to_string(),
                title: "shell".to_string(),
                command: "zsh".to_string(),
                cols: 120,
                rows: 32,
            }],
        };
        let task = StateTaskSummary {
            id: "task_1".to_string(),
            title: "Implement snapshot parity".to_string(),
            status: "ready".to_string(),
            priority: "high".to_string(),
            project_id: "proj_local".to_string(),
            assignee_type: None,
            assignee_id: None,
            cycle_id: None,
            module_id: None,
            initiative_id: None,
            due_at: None,
            estimate: None,
            labels: vec![],
            context_pack_id: None,
            comment_count: 0,
            has_workpad: false,
            subtask_count: 0,
            open_subtask_count: 0,
            token_usage: None,
        };

        let first = build_state_snapshot_with_db_state(
            "0.1.0",
            "2026-04-30T01:00:00Z",
            readiness.clone(),
            pty.clone(),
            "ok",
            vec![task.clone()],
            serde_json::json!({"ready": 1}),
        );
        let second = build_state_snapshot_with_db_state(
            "0.1.0",
            "2026-04-30T01:01:00Z",
            readiness.clone(),
            pty.clone(),
            "ok",
            vec![task.clone()],
            serde_json::json!({"ready": 1}),
        );
        let changed = build_state_snapshot_with_db_state(
            "0.1.0",
            "2026-04-30T01:01:00Z",
            readiness,
            pty,
            "ok",
            vec![StateTaskSummary {
                status: "running".to_string(),
                ..task
            }],
            serde_json::json!({"running": 1}),
        );

        assert_eq!(first.snapshot_id, second.snapshot_id);
        assert_ne!(first.generated_at, second.generated_at);
        assert_ne!(first.snapshot_id, changed.snapshot_id);
    }

    #[test]
    fn records_degraded_attention_when_database_persistence_is_not_configured() {
        let snapshot = build_state_snapshot_with_db_state(
            "0.1.0",
            "2026-04-30T01:00:00Z",
            ReadinessSnapshot {
                checks: vec![],
                summary: summarize_checks(&[]),
            },
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "degraded",
            vec![],
            serde_json::json!({}),
        );

        assert!(snapshot
            .attention
            .iter()
            .any(|item| item.id == "db-not-configured" && item.severity == "warning"));
        assert_eq!(snapshot.health.db, "degraded");
        assert_eq!(snapshot.health.api, "ok");
    }

    #[test]
    fn records_ok_database_health_without_database_warning_when_persistence_is_configured() {
        let snapshot = build_state_snapshot_with_db_state(
            "0.1.0",
            "2026-04-30T01:00:00Z",
            ReadinessSnapshot {
                checks: vec![],
                summary: summarize_checks(&[]),
            },
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "ok",
            vec![],
            serde_json::json!({}),
        );

        assert_eq!(snapshot.health.db, "ok");
        assert!(!snapshot
            .attention
            .iter()
            .any(|item| item.id == "db-not-configured"));
    }

    #[test]
    fn includes_persisted_task_summaries_and_counts() {
        let snapshot = build_state_snapshot_with_db_state(
            "0.1.0",
            "2026-04-30T01:00:00Z",
            ReadinessSnapshot {
                checks: vec![],
                summary: summarize_checks(&[]),
            },
            TerminalPtySnapshot {
                total: 0,
                sessions: vec![],
            },
            "ok",
            vec![StateTaskSummary {
                id: "task_ready".to_string(),
                title: "Wire state snapshot".to_string(),
                status: "ready".to_string(),
                priority: "high".to_string(),
                project_id: "proj_local".to_string(),
                assignee_type: Some("agent".to_string()),
                assignee_id: Some("agent_codex".to_string()),
                cycle_id: Some("cycle_sprint_5".to_string()),
                module_id: Some("module_control_api".to_string()),
                initiative_id: Some("init_auth".to_string()),
                due_at: Some("2026-05-15".to_string()),
                estimate: Some("3 pts".to_string()),
                labels: vec!["release".to_string(), "evidence".to_string()],
                subtask_count: 2,
                open_subtask_count: 1,
                context_pack_id: Some("ctx_default".to_string()),
                comment_count: 2,
                has_workpad: true,
                token_usage: None,
            }],
            serde_json::json!({"ready": 1, "blocked": 0}),
        );

        assert_eq!(snapshot.tasks.items.len(), 1);
        assert_eq!(snapshot.tasks.items[0].id, "task_ready");
        assert_eq!(
            snapshot.tasks.items[0].assignee_id.as_deref(),
            Some("agent_codex")
        );
        assert_eq!(snapshot.tasks.items[0].comment_count, 2);
        assert_eq!(snapshot.tasks.items[0].has_workpad, true);
        assert_eq!(
            snapshot.tasks.items[0].due_at.as_deref(),
            Some("2026-05-15")
        );
        assert_eq!(snapshot.tasks.items[0].estimate.as_deref(), Some("3 pts"));
        assert_eq!(
            snapshot.tasks.items[0].labels,
            vec!["release".to_string(), "evidence".to_string()]
        );
        assert_eq!(snapshot.tasks.items[0].subtask_count, 2);
        assert_eq!(snapshot.tasks.items[0].open_subtask_count, 1);
        assert_eq!(snapshot.tasks.counts_by_status["ready"], 1);
    }
}
