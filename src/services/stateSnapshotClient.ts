import { invoke } from "@tauri-apps/api/core";
import { HC_DEFAULT_TERMINAL_THEME } from "../design/haneulchiDesignTokens";
import type { CommandBlock } from "../domain/commandBlocks";
import { listTasks, type HaneulchiTask, type TaskState } from "../domain/tasks";
import type { ProjectTabLayout } from "./projectApiClient";

export interface StateSnapshotApp {
  version: string;
  renderer: string;
  update_state: string;
  terminal_theme?: {
    project_id?: string | null;
    name: string;
    background: string;
    foreground: string;
    accent: string;
  };
}

export interface StateProject {
  id: string;
  name: string;
  state: string;
  token_usage?: StateTokenUsageSummary;
}

export interface StateProjectTab {
  id: string;
  project_id?: string;
  label: string;
  active: boolean;
  layout_json?: ProjectTabLayout;
  group_name?: string;
}

export interface StateSession {
  id: string;
  project_id?: string;
  pane_id?: string;
  mode: string;
  title: string;
  cwd?: string;
  branch?: string;
  agent_profile_id?: string | null;
  task_id?: string | null;
  state: string;
  attention_state?: string;
  token_budget_state?: string;
  token_usage?: StateTokenUsageSummary;
  ports?: number[];
  run_id?: string | null;
  created_at?: string;
  updated_at?: string;
}

export interface StateTokenUsageSummary {
  input_tokens: number;
  output_tokens: number;
  total_tokens?: number;
  cost_usd: number;
}

export interface StateAttentionItem {
  id: string;
  label: string;
  severity: string;
  detail: string;
}

export interface StateCommandBlockSummary {
  id: string;
  session_id: string;
  command: string;
  status: string;
  seq_start?: number;
  seq_end?: number;
}

export interface StateTaskSummary {
  id: string;
  title: string;
  status: string;
  priority: string;
  project_id: string;
  assignee?: string;
  assignee_type?: string;
  assignee_id?: string;
  cycle_id?: string;
  module_id?: string;
  initiative_id?: string;
  due_at?: string;
  estimate?: string;
  labels?: string[];
  comment_count?: number;
  has_workpad?: boolean;
  subtask_count?: number;
  open_subtask_count?: number;
  initiative?: string;
  cycle?: string;
  module?: string;
  dueDate?: string;
  context_pack_id?: string;
  token_usage?: StateTokenUsageSummary;
}

export interface StateInitiative {
  id: string;
  project_id: string;
  name: string;
  description?: string | null;
  budget_id?: string | null;
  status: string;
  token_usage?: StateTokenUsageSummary;
}

export interface StateRunSummary {
  id: string;
  task_id: string;
  project_id: string;
  agent_profile_id?: string;
  session_id?: string;
  workflow_version_id?: string;
  lifecycle: string;
  retry_count: number;
  next_retry_at?: string;
  status_detail?: string;
  context_pack_id?: string;
  workspace_path?: string;
}

export interface StateReview {
  id: string;
  state: string;
  evidence_pack_id: string;
  task_id?: string | null;
  run_id?: string | null;
  completeness_state: string;
  diff_summary?: {
    summary?: string;
    [key: string]: unknown;
  };
  token_usage?: {
    scope_type?: string;
    scope_id?: string;
    input_tokens?: number;
    output_tokens?: number;
    total_tokens?: number;
    cost_usd?: number;
    records?: unknown[];
  };
}

export interface StateAgent {
  id: string;
  label: string;
  available: boolean;
  token_usage?: StateTokenUsageSummary;
  latest_event_kind?: string;
  latest_event_detail?: string;
  attention_state?: string;
  attention_severity?: string;
  notification_count?: number;
  last_heartbeat_at?: string | null;
}

export interface StateProviderModel {
  provider: string;
  model: string;
  agent_profile_id: string;
}

export interface StateBudgetSummary {
  id?: string;
  scope_type?: string;
  scope_id?: string | null;
  used_usd?: number;
  max_usd?: number;
  warn_pct?: number;
  hard_limit?: boolean;
  state?: string;
}

export interface StateBudgetForecast {
  id?: string;
  scope_type?: string;
  scope_id?: string | null;
  used_usd?: number;
  max_usd?: number;
  remaining_usd?: number;
  average_run_cost_usd?: number | null;
  estimated_runs_remaining?: number | null;
  run_sample_count?: number;
  state?: string;
}

export interface StateProviderPriceTableSummary {
  count?: number;
  source?: string | null;
  updated_at?: string | null;
}

export interface StateWorkflowDiagnostic {
  code: string;
  message: string;
}

export interface StateTrackerBinding {
  id: string;
  local_kind: "task" | "project";
  local_id: string;
  provider: "linear" | "github" | "plane" | "custom" | "manual";
  external_id: string;
  external_url?: string | null;
  sync_mode: "manual" | "mirror" | "import" | "export";
  sync_status: string;
  conflict_state: string;
}

export interface StateReleaseGates {
  last_run_id?: string | null;
  last_status: string;
  last_pass_count: number;
  last_fail_count: number;
  last_warning_count: number;
  diagnostics?: {
    status?: string;
    scenario_count?: number;
    created_at?: string;
  };
}

export interface StateDistribution {
  last_dmg_smoke_run_id?: string | null;
  last_status: string;
  explicit_blocker: boolean;
  last_pass_count: number;
  last_fail_count: number;
  last_warning_count: number;
  diagnostics?: {
    status?: string;
    case_count?: number;
    created_at?: string;
  };
}

export interface StateRecovery {
  last_run_id?: string | null;
  last_status: string;
  last_pass_count: number;
  last_fail_count: number;
  last_warning_count: number;
  diagnostics?: {
    status?: string;
    drill_count?: number;
    created_at?: string;
  };
}

export interface StateBenchmarkSuite {
  suite_id: string;
  name: string;
  status: string;
  metric_value: number;
  target_value: number;
  unit: string;
  detail: string;
}

export interface StateBenchmarks {
  last_run_id?: string | null;
  last_status: string;
  last_pass_count: number;
  last_fail_count: number;
  last_warning_count: number;
  suites: StateBenchmarkSuite[];
  diagnostics?: {
    status?: string;
    suite_count?: number;
    duration_ms?: number;
    created_at?: string;
  };
}

export interface StateDogfood {
  last_review_id?: string | null;
  last_status: string;
  last_evidence_pack_id?: string | null;
  last_pass_count: number;
  last_warning_count: number;
  last_fail_count: number;
  diagnostics?: {
    status?: string;
    finding_count?: number;
    created_at?: string;
  };
}

export interface StateVisualNode {
  id: string;
  label: string;
  kind: string;
  status: string;
}

export interface StateVisualEdge {
  id: string;
  source_id: string;
  target_id: string;
  kind: string;
  status: string;
}

export interface StateVisualHarness {
  nodes: StateVisualNode[];
  edges: StateVisualEdge[];
  diagnostics?: {
    status?: string;
    node_count?: number;
    edge_count?: number;
  };
}

export interface StateTerminalFidelity {
  last_run_id?: string | null;
  last_status: string;
  last_pass_count: number;
  last_fail_count: number;
  last_warning_count: number;
  diagnostics?: {
    status?: string;
    case_count?: number;
    created_at?: string;
  };
}

export interface StateTaskLifecycle {
  last_run_id?: string | null;
  last_status: string;
  last_task_id?: string | null;
  last_agent_run_id?: string | null;
  last_evidence_pack_id?: string | null;
  diagnostics?: {
    status?: string;
    transition_count?: number;
    created_at?: string;
  };
}

export interface StateWorkflowNegative {
  last_run_id?: string | null;
  last_status: string;
  last_baseline_workflow_id?: string | null;
  last_invalid_workflow_id?: string | null;
  last_known_good_workflow_id?: string | null;
  diagnostics?: {
    status?: string;
    case_count?: number;
    dispatch_run_id?: string;
    created_at?: string;
  };
}

export interface StateSnapshot {
  snapshot_id: string;
  generated_at: string;
  app: StateSnapshotApp;
  projects: StateProject[];
  project_tabs: StateProjectTab[];
  sessions: StateSession[];
  command_blocks: {
    recent: StateCommandBlockSummary[];
    unread_count: number;
  };
  tasks: {
    items: StateTaskSummary[];
    counts_by_status: Record<string, number>;
  };
  initiatives?: StateInitiative[];
  runs: {
    items: StateRunSummary[];
    counts_by_lifecycle: Record<string, number>;
  };
  agents: StateAgent[];
  reviews: StateReview[];
  attention: StateAttentionItem[];
  provider_model?: StateProviderModel;
  budgets: {
    workspace: StateBudgetSummary;
    projects: StateBudgetSummary[];
    goals?: StateBudgetSummary[];
    tasks?: StateBudgetSummary[];
    runs?: StateBudgetSummary[];
    agents: StateBudgetSummary[];
    forecasts?: {
      workspace?: StateBudgetForecast;
      projects?: StateBudgetForecast[];
      goals?: StateBudgetForecast[];
      tasks?: StateBudgetForecast[];
      runs?: StateBudgetForecast[];
      agents?: StateBudgetForecast[];
    };
    price_table?: StateProviderPriceTableSummary;
  };
  security?: {
    keychain: string;
    secret_count: number;
    redaction?: {
      status?: string;
      protected_secret_count?: number;
    };
    permission_audit?: {
      recent_count?: number;
      allowed_count?: number;
      approval_required_count?: number;
      forbidden_count?: number;
      latest_decision?: string | null;
      latest_action_kind?: string | null;
    };
    diagnostics?: {
      status?: string;
      pending_policy_approvals?: number;
      checks?: Array<{
        id?: string;
        label?: string;
        status?: string;
        detail?: string;
      }>;
    };
    policy_pack?: {
      id?: string | null;
      name?: string;
      sandbox_mode?: string;
      network?: string;
      network_profile?: string;
      file_write?: string;
      tools?: string;
      approval_required_count?: number;
      forbidden_count?: number;
    };
  };
  workflow: {
    valid: boolean;
    invalid_projects: unknown[];
    current_version_id?: string | null;
    last_known_good_version_id?: string | null;
    diagnostics: {
      errors: StateWorkflowDiagnostic[];
    };
  };
  workflow_negative?: StateWorkflowNegative;
  knowledge: {
    stale_count: number;
    gap_count: number;
    recent_pages: string[];
  };
  task_lifecycle?: StateTaskLifecycle;
  terminal_fidelity?: StateTerminalFidelity;
  release_gates?: StateReleaseGates;
  distribution?: StateDistribution;
  recovery?: StateRecovery;
  benchmarks?: StateBenchmarks;
  dogfood?: StateDogfood;
  visual_harness?: StateVisualHarness;
  tracker?: {
    binding_count: number;
    bindings: StateTrackerBinding[];
    diagnostics: {
      status?: string;
      pending_count?: number;
      conflict_count?: number;
      linear?: {
        last_run_id?: string | null;
        last_status?: string;
        last_operation_count?: number;
        degraded_reason?: string | null;
      };
      github?: {
        last_run_id?: string | null;
        last_status?: string;
        last_operation_count?: number;
        degraded_reason?: string | null;
      };
      plane?: {
        last_run_id?: string | null;
        last_status?: string;
        last_operation_count?: number;
        degraded_reason?: string | null;
      };
    };
  };
  health: {
    db: string;
    pty: string;
    api: string;
  };
}

export const fallbackStateSnapshot: StateSnapshot = {
  snapshot_id: "snap_fallback",
  generated_at: "",
  app: {
    version: "0.1.0",
    renderer: "xterm-webgl",
    update_state: "unknown",
    terminal_theme: {
      project_id: null,
      ...HC_DEFAULT_TERMINAL_THEME,
    },
  },
  projects: [],
  project_tabs: [],
  sessions: [],
  command_blocks: {
    recent: [],
    unread_count: 0,
  },
  tasks: {
    items: [],
    counts_by_status: {},
  },
  initiatives: [],
  runs: {
    items: [],
    counts_by_lifecycle: {},
  },
  agents: [],
  reviews: [],
  attention: [
    {
      id: "state-api-unavailable",
      label: "State snapshot command unavailable",
      severity: "warning",
      detail: "Using frontend fallback until Tauri control-plane state responds",
    },
  ],
  provider_model: {
    provider: "openai",
    model: "gpt-5.4",
    agent_profile_id: "agent_codex",
  },
  budgets: {
    workspace: {},
    projects: [],
    goals: [],
    tasks: [],
    runs: [],
    agents: [],
    forecasts: {},
    price_table: {},
  },
  security: {
    keychain: "unknown",
    secret_count: 0,
    redaction: { status: "inactive", protected_secret_count: 0 },
    permission_audit: {
      recent_count: 0,
      allowed_count: 0,
      approval_required_count: 0,
      forbidden_count: 0,
      latest_decision: null,
      latest_action_kind: null,
    },
    diagnostics: { status: "warning", pending_policy_approvals: 0, checks: [] },
    policy_pack: {},
  },
  workflow: {
    valid: true,
    invalid_projects: [],
    current_version_id: null,
    last_known_good_version_id: null,
    diagnostics: { errors: [] },
  },
  workflow_negative: {
    last_run_id: null,
    last_status: "not_run",
    last_baseline_workflow_id: null,
    last_invalid_workflow_id: null,
    last_known_good_workflow_id: null,
    diagnostics: {
      status: "not_run",
      case_count: 0,
    },
  },
  knowledge: {
    stale_count: 0,
    gap_count: 0,
    recent_pages: [],
  },
  task_lifecycle: {
    last_run_id: null,
    last_status: "not_run",
    last_task_id: null,
    last_agent_run_id: null,
    last_evidence_pack_id: null,
    diagnostics: {
      status: "not_run",
      transition_count: 0,
    },
  },
  terminal_fidelity: {
    last_run_id: null,
    last_status: "not_run",
    last_pass_count: 0,
    last_fail_count: 0,
    last_warning_count: 0,
    diagnostics: {
      status: "not_run",
      case_count: 0,
    },
  },
  release_gates: {
    last_run_id: null,
    last_status: "not_run",
    last_pass_count: 0,
    last_fail_count: 0,
    last_warning_count: 0,
    diagnostics: {
      status: "not_run",
      scenario_count: 0,
    },
  },
  distribution: {
    last_dmg_smoke_run_id: null,
    last_status: "not_run",
    explicit_blocker: false,
    last_pass_count: 0,
    last_fail_count: 0,
    last_warning_count: 0,
    diagnostics: {
      status: "not_run",
      case_count: 0,
    },
  },
  recovery: {
    last_run_id: null,
    last_status: "not_run",
    last_pass_count: 0,
    last_fail_count: 0,
    last_warning_count: 0,
    diagnostics: {
      status: "not_run",
      drill_count: 0,
    },
  },
  benchmarks: {
    last_run_id: null,
    last_status: "not_run",
    last_pass_count: 0,
    last_fail_count: 0,
    last_warning_count: 0,
    suites: [],
    diagnostics: {
      status: "not_run",
      suite_count: 0,
    },
  },
  dogfood: {
    last_review_id: null,
    last_status: "not_run",
    last_evidence_pack_id: null,
    last_pass_count: 0,
    last_warning_count: 0,
    last_fail_count: 0,
    diagnostics: {
      status: "not_run",
      finding_count: 0,
    },
  },
  visual_harness: {
    nodes: [],
    edges: [],
    diagnostics: {
      status: "empty",
      node_count: 0,
      edge_count: 0,
    },
  },
  tracker: {
    binding_count: 0,
    bindings: [],
    diagnostics: {
      status: "unconfigured",
      pending_count: 0,
      conflict_count: 0,
      linear: {
        last_run_id: null,
        last_status: "not_run",
        last_operation_count: 0,
        degraded_reason: null,
      },
      github: {
        last_run_id: null,
        last_status: "not_run",
        last_operation_count: 0,
        degraded_reason: null,
      },
      plane: {
        last_run_id: null,
        last_status: "not_run",
        last_operation_count: 0,
        degraded_reason: null,
      },
    },
  },
  health: {
    db: "unknown",
    pty: "unknown",
    api: "degraded",
  },
};

export function getStateSnapshot(projectId?: string): Promise<StateSnapshot> {
  if (projectId) {
    return invoke<StateSnapshot>("get_state_snapshot", { projectId });
  }
  return invoke<StateSnapshot>("get_state_snapshot");
}

export function mergeCommandBlocksIntoStateSnapshot(snapshot: StateSnapshot, blocks: CommandBlock[]): StateSnapshot {
  const nativeRecent = snapshot.command_blocks.recent ?? [];
  const nativeIds = new Set(nativeRecent.map((block) => block.id));
  const localRecent = blocks.slice(-10).map((block) => ({
    id: block.id,
    session_id: block.sessionId,
    command: block.command,
    status: block.status,
    seq_start: block.seqStart,
    seq_end: block.seqEnd,
  }));
  const mergedById = new Map<string, StateCommandBlockSummary>();

  [...nativeRecent, ...localRecent].forEach((block) => {
    mergedById.set(block.id, block);
  });

  const recent = Array.from(mergedById.values()).slice(-10);
  const localNewCount = localRecent.filter((block) => !nativeIds.has(block.id)).length;

  return {
    ...snapshot,
    command_blocks: {
      recent,
      unread_count: Math.max(snapshot.command_blocks.unread_count, nativeRecent.length) + localNewCount,
    },
  };
}

export function mergeTasksIntoStateSnapshot(snapshot: StateSnapshot, taskState: TaskState): StateSnapshot {
  const localItems = listTasks(taskState).map(toStateTaskSummary);
  const localItemIds = new Set(localItems.map((item) => item.id));
  const items = [
    ...snapshot.tasks.items.filter((item) => !localItemIds.has(item.id)),
    ...localItems,
  ];
  const counts = items.reduce<Record<string, number>>(
    (nextCounts, item) => {
      nextCounts[item.status] = (nextCounts[item.status] ?? 0) + 1;
      return nextCounts;
    },
    { inbox: 0, ready: 0, running: 0, review: 0, blocked: 0, done: 0 },
  );

  return {
    ...snapshot,
    tasks: {
      items,
      counts_by_status: counts,
    },
  };
}

function toStateTaskSummary(task: HaneulchiTask): StateTaskSummary {
  return {
    id: task.id,
    title: task.title,
    status: task.status,
    priority: task.priority,
    project_id: task.projectId,
    assignee: task.assignee,
    comment_count: task.comments?.length ?? 0,
    has_workpad: (task.workpad ?? "").trim().length > 0,
    subtask_count: task.subtasks?.length ?? 0,
    open_subtask_count: task.subtasks?.filter((subtask) => subtask.status !== "done").length ?? 0,
    cycle: task.cycle,
    module: task.module,
    initiative: task.initiative,
    initiative_id: task.initiative,
    due_at: task.dueDate,
    estimate: task.estimate,
    labels: task.labels,
    context_pack_id: task.contextPackId,
  };
}
